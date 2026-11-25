import { expect, test } from "bun:test";
import { JoyClient, configFromEnv } from "./client.ts";
import { tools } from "./tools.ts";

const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), { status });

test("only GET, only /api/", async () => {
  const seen: string[] = [];
  const c = new JoyClient({
    baseUrl: "http://x",
    token: "t",
    fetch: (async (u: URL, init: RequestInit) => {
      seen.push(`${init.method} ${u.pathname}`);
      return json({ ok: 1 });
    }) as any,
  });
  await c.get("/api/dashboard/summary");
  expect(seen).toEqual(["GET /api/dashboard/summary"]);
  expect(c.get("/healthz")).rejects.toThrow();
});

test("signs in, and again after a 401", async () => {
  let logins = 0;
  const c = new JoyClient({
    baseUrl: "http://x",
    username: "u",
    password: "p",
    fetch: (async (u: URL, init: RequestInit) => {
      if (String(u).endsWith("/login")) {
        logins++;
        return json({ access_token: `tok${logins}` });
      }
      const auth = (init.headers as Record<string, string>).authorization;
      return auth === "Bearer tok2" ? json({ ok: true }) : json({}, 401);
    }) as any,
  });
  expect(await c.get("/api/plan/forecast")).toEqual({ ok: true });
  expect(logins).toBe(2);
});

test("a 404 stays a plain not-found", () => {
  const c = new JoyClient({
    baseUrl: "http://x",
    token: "t",
    fetch: (async () => json({}, 404)) as any,
  });
  expect(c.get("/api/dates/slips")).rejects.toThrow("Not found.");
});

test("config needs credentials", () => {
  expect(() => configFromEnv({})).toThrow();
  expect(configFromEnv({ JOY_TOKEN: "t" }).baseUrl).toBe("https://joy.fly.dev");
});

test("tool names are unique and paths stay under /api/", () => {
  expect(new Set(tools.map((t) => t.name)).size).toBe(tools.length);
  for (const t of tools)
    expect(t.path({ year: 2026, month: 1 }).startsWith("/api/")).toBe(true);
});

test("saveAnalysis posts once to /api/analyses with an idempotency key", async () => {
  const calls: { m: string; p: string; k: string; b: any }[] = [];
  const c = new JoyClient({
    baseUrl: "http://x",
    token: "t",
    fetch: (async (u: string, init: RequestInit) => {
      const h = init.headers as Record<string, string>;
      calls.push({
        m: init.method!,
        p: new URL(u).pathname,
        k: h["idempotency-key"],
        b: JSON.parse(init.body as string),
      });
      return json({ id: "a1" }, 201);
    }) as any,
  });
  expect(await c.saveAnalysis("T", "B", "key-1")).toEqual({ id: "a1" });
  expect(calls).toEqual([
    {
      m: "POST",
      p: "/api/analyses",
      k: "key-1",
      b: { title: "T", body: "B", source: "claude" },
    },
  ]);
});

test("the prompt names the tools it needs", async () => {
  const { analysePrompt } = await import("./server.ts");
  const p = analysePrompt("eating out");
  for (const n of [
    "analyses_list",
    "money_export_all",
    "save_analysis",
    "eating out",
  ])
    expect(p).toContain(n);
});

test("the payee categorize path is writable, other ingest writes are not", async () => {
  const c = new JoyClient({
    baseUrl: "http://x",
    token: "t",
    fetch: (async () => json({ updated: 2 })) as any,
  });
  expect(
    await c.post(
      "/api/ingest/payees/categorize",
      { payee: "Swad", category: "food" },
      "k",
    ),
  ).toEqual({ updated: 2 });
  expect(c.post("/api/ingest/transactions/update", {}, "k")).rejects.toThrow(
    "cannot be written",
  );
  expect(c.post("/api/ingest/transactions/delete", {}, "k")).rejects.toThrow(
    "cannot be written",
  );
});
