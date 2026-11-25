import { expect, test } from "bun:test";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { JoyClient } from "./client.ts";
import { buildServer } from "./server.ts";
import { maskDigits, rupeesToPaise, txnKey } from "./inbox.ts";

const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), { status });

test("masking hides account, card and phone numbers but keeps amounts and dates", () => {
  const s =
    "Rs 12,500.00 debited from A/c 50100123456789 on 21-11-2026 via card 4111 1111 1111 1234, call 9876543210";
  const m = maskDigits(s);
  expect(m).toContain("Rs 12,500.00");
  expect(m).toContain("21-11-2026");
  expect(m).toContain("A/c XX6789");
  expect(m).toContain("card XX1234");
  expect(m).toContain("call XX3210");
  expect(m).not.toMatch(/\d{8,}/);
});

test("rupees become paise exactly, the same as the server's parser", () => {
  expect(rupeesToPaise("450")).toBe(45000);
  expect(rupeesToPaise("₹1,250.5")).toBe(125050);
  expect(rupeesToPaise("0.07")).toBe(7);
  expect(rupeesToPaise(" 1 00 000.99 ")).toBe(10000099);
  for (const bad of ["", "-5", "1.234", "1.2.3", "abc", "0", "1e3"])
    expect(() => rupeesToPaise(bad)).toThrow();
});

test("keys: an alert keys on its id, hand-typed rows on their fields", async () => {
  const t = {
    occurred_on: "2026-11-20",
    direction: "debit" as const,
    amount_rupees: "450",
    merchant: "Chai",
  };
  expect(await txnKey({ ...t, alert_id: "al1" }, 45000)).toBe("mcp-txn:a:al1");
  const a = await txnKey(t, 45000);
  expect(await txnKey({ ...t, merchant: " chai " }, 45000)).toBe(a);
  expect(await txnKey({ ...t, occurrence: 2 }, 45000)).not.toBe(a);
  expect(a.length).toBeLessThanOrEqual(100);
});

test("the client writes only the allowed paths, always with a key", () => {
  const c = new JoyClient({
    baseUrl: "http://x",
    token: "t",
    fetch: (async () => json({})) as any,
  });
  expect(c.post("/api/ingest/transactions/delete", {}, "k")).rejects.toThrow(
    "cannot be written",
  );
  expect(c.post("/api/ingest/transactions/create", {}, "")).rejects.toThrow(
    "idempotency key",
  );
});

async function connect(
  fetchImpl: (path: string, init: RequestInit) => Response,
) {
  const calls: { m: string; p: string; k?: string; b?: any }[] = [];
  const joy = new JoyClient({
    baseUrl: "http://x",
    token: "t",
    fetch: (async (u: string | URL, init: RequestInit) => {
      const p = new URL(String(u)).pathname;
      const h = init.headers as Record<string, string>;
      calls.push({
        m: init.method!,
        p,
        k: h["idempotency-key"],
        b: init.body ? JSON.parse(init.body as string) : undefined,
      });
      return fetchImpl(p, init);
    }) as any,
  });
  const [a, b] = InMemoryTransport.createLinkedPair();
  await buildServer(joy).connect(a);
  const mcp = new Client({ name: "t", version: "0" });
  await mcp.connect(b);
  return { mcp, calls };
}

test("money_unrecognized returns masked texts", async () => {
  const { mcp } = await connect(() =>
    json({
      count: 1,
      items: [
        {
          alert_id: "al1",
          text: "Sent Rs.300 from 123456789012 to X",
          source: "sms",
          received_at: "2026-11-20 10:00:00",
        },
      ],
    }),
  );
  const r = (await mcp.callTool({
    name: "money_unrecognized",
    arguments: {},
  })) as any;
  expect(r.content[0].text).toContain("XX9012");
  expect(r.content[0].text).not.toContain("123456789012");
});

test("add_transaction posts paise with the alert key, then clears the alert", async () => {
  const { mcp, calls } = await connect((p) =>
    p.endsWith("/create")
      ? json({ txn_id: "tx1", status: true })
      : json({ status: true }),
  );
  const r = (await mcp.callTool({
    name: "money_add_transaction",
    arguments: {
      occurred_on: "2026-11-20",
      direction: "debit",
      amount_rupees: "1,250.50",
      merchant: "Zomato",
      category: "food",
      alert_id: "al1",
    },
  })) as any;
  expect(r.isError).toBeFalsy();
  expect(calls).toEqual([
    {
      m: "POST",
      p: "/api/ingest/transactions/create",
      k: "mcp-txn:a:al1",
      b: {
        occurred_on: "2026-11-20",
        direction: "debit",
        amount_paise: 125050,
        merchant: "Zomato",
        category: "food",
      },
    },
    {
      m: "POST",
      p: "/api/ingest/unrecognized/dismiss",
      k: "mcp-dismiss:al1",
      b: { alert_id: "al1" },
    },
  ]);
});

test("a failed create leaves the alert in the inbox", async () => {
  const { mcp, calls } = await connect(() =>
    json({ error: "unknown category" }, 400),
  );
  const r = (await mcp.callTool({
    name: "money_add_transaction",
    arguments: {
      occurred_on: "2026-11-20",
      direction: "debit",
      amount_rupees: "50",
      alert_id: "al1",
    },
  })) as any;
  expect(r.isError).toBe(true);
  expect(r.content[0].text).toBe("unknown category");
  expect(calls.map((c) => c.p)).toEqual(["/api/ingest/transactions/create"]);
});

test("a bad amount never reaches the server", async () => {
  const { mcp, calls } = await connect(() => json({}));
  const r = (await mcp.callTool({
    name: "money_add_transaction",
    arguments: {
      occurred_on: "2026-11-20",
      direction: "debit",
      amount_rupees: "12.345",
    },
  })) as any;
  expect(r.isError).toBe(true);
  expect(calls).toEqual([]);
});
