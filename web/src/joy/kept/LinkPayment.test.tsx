import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { LinkPayment } from "./LinkPayment";

let calls: { url: string; body?: unknown }[];
let candidates: unknown[];
const realFetch = globalThis.fetch;

beforeEach(() => {
  calls = [];
  candidates = [
    { txn_id: "t1", merchant: "Saltt dinner", amount_paise: 123050, day: "2026-11-15" },
    { txn_id: "t2", merchant: "Chai", amount_paise: 9000, day: "2026-11-15" },
  ];
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({ url: String(url), body: init?.body ? JSON.parse(String(init.body)) : undefined });
    if (String(url).endsWith("/candidates")) return new Response(JSON.stringify({ candidates }), { status: 200 });
    return new Response(JSON.stringify({ ok: true, to: "/joy/kept#k5" }), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("LinkPayment", () => {
  test("opens to your debits around the day, with their amounts, and links the one you tap", async () => {
    let done = 0;
    render(<LinkPayment historyId={5} linked={null} onDone={() => done++} />);
    fireEvent.click(screen.getByText(/which payment was this/i));
    expect(await screen.findByText(/Saltt dinner · ₹1,230\.50/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /link saltt dinner/i }));
    await waitFor(() => expect(done).toBe(1));
    expect(calls.find((c) => c.url.endsWith("/link"))?.body).toEqual({ txn_id: "t1" });
  });

  test("says when there is nothing to link", async () => {
    candidates = [];
    render(<LinkPayment historyId={5} linked={null} onDone={() => {}} />);
    fireEvent.click(screen.getByText(/which payment was this/i));
    expect(await screen.findByText(/no payments of yours around that day/i)).toBeTruthy();
  });

  test("a linked payment shows what it was and can be unlinked", async () => {
    let done = 0;
    render(<LinkPayment historyId={5} linked={{ txn_id: "t1", merchant: "Saltt dinner", amount_paise: 123050, day: "2026-11-15" }} onDone={() => done++} />);
    expect(screen.getByText(/paid with Saltt dinner/i)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /unlink/i }));
    await waitFor(() => expect(done).toBe(1));
    expect(calls[0]?.url).toContain("/history/5/unlink");
  });
});
