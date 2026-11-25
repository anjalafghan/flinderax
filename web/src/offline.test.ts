import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { enqueue, flush, loadQueue, sendOrQueue } from "./offline";

type Call = { url: string; key: string | null; body: unknown };
let calls: Call[];
let behave: (call: Call) => Response | "offline";
const realFetch = globalThis.fetch;

beforeEach(() => {
  localStorage.clear();
  localStorage.setItem("token", "s_abc");
  calls = [];
  behave = () => new Response(JSON.stringify({ ok: true }), { status: 200 });
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    const headers = new Headers(init?.headers);
    const call = {
      url: String(url),
      key: headers.get("idempotency-key"),
      body: init?.body ? JSON.parse(String(init.body)) : undefined,
    };
    calls.push(call);
    const r = behave(call);
    if (r === "offline") throw new TypeError("Failed to fetch");
    return r;
  }) as unknown as typeof fetch;
});
afterEach(() => {
  globalThis.fetch = realFetch;
  localStorage.clear();
});

describe("writes that survive having no signal", () => {
  test("online: it goes straight through with a key and nothing is queued", async () => {
    const r = await sendOrQueue("/api/x", { a: 1 }, "Thing");
    expect(r.queued).toBe(false);
    expect(r.response?.status).toBe(200);
    expect(calls[0]?.key).toBeTruthy();
    expect(loadQueue()).toHaveLength(0);
  });

  test("offline: it is kept on the phone with its key", async () => {
    behave = () => "offline";
    const r = await sendOrQueue("/api/x", { a: 1 }, "Thing");
    expect(r.queued).toBe(true);
    const q = loadQueue();
    expect(q).toHaveLength(1);
    expect(q[0]).toMatchObject({
      url: "/api/x",
      body: { a: 1 },
      label: "Thing",
      key: calls[0]?.key,
    });
  });

  test("a refusal from the server is returned, not queued", async () => {
    behave = () => new Response("nope", { status: 400 });
    const r = await sendOrQueue("/api/x", {}, "Thing");
    expect(r.queued).toBe(false);
    expect(r.response?.status).toBe(400);
    expect(loadQueue()).toHaveLength(0);
  });

  test("when the signal is back, saved items are sent in order, once each, with the key they were made with", async () => {
    enqueue({ key: "k1", url: "/api/a", body: { n: 1 }, label: "First" });
    enqueue({ key: "k2", url: "/api/b", body: { n: 2 }, label: "Second" });
    const out = await flush();
    expect(out).toEqual({ sent: 2, dropped: [], left: 0 });
    expect(calls.map((c) => [c.url, c.key])).toEqual([
      ["/api/a", "k1"],
      ["/api/b", "k2"],
    ]);
    expect(loadQueue()).toHaveLength(0);
    expect((await flush()).sent).toBe(0);
  });

  test("still offline: nothing is lost and it stops at the first failure", async () => {
    enqueue({ key: "k1", url: "/api/a", body: {}, label: "First" });
    enqueue({ key: "k2", url: "/api/b", body: {}, label: "Second" });
    behave = () => "offline";
    expect(await flush()).toEqual({ sent: 0, dropped: [], left: 2 });
    expect(calls).toHaveLength(1);
  });

  test("an item the server refuses is dropped with its name so it cannot block the rest", async () => {
    enqueue({ key: "k1", url: "/api/a", body: {}, label: "Bad one" });
    enqueue({ key: "k2", url: "/api/b", body: {}, label: "Good one" });
    behave = (c) =>
      c.url === "/api/a"
        ? new Response("no", { status: 422 })
        : new Response("{}", { status: 200 });
    expect(await flush()).toEqual({ sent: 1, dropped: ["Bad one"], left: 0 });
  });

  test("a server error is kept to try again later", async () => {
    enqueue({ key: "k1", url: "/api/a", body: {}, label: "First" });
    behave = () => new Response("oops", { status: 503 });
    expect(await flush()).toEqual({ sent: 0, dropped: [], left: 1 });
  });

  test("an expired session retains queued writes until reauthentication", async () => {
    enqueue({ key: "k1", url: "/api/a", body: {}, label: "First" });
    behave = () => new Response("expired", { status: 401 });
    expect(await flush()).toEqual({ sent: 0, dropped: [], left: 1 });
    expect(loadQueue()).toHaveLength(1);
  });

  test("one signed-in account cannot flush another session's queued writes", async () => {
    enqueue({ key: "a1", url: "/api/a", body: {}, label: "A" });
    localStorage.setItem("token", "s_user_b");
    enqueue({ key: "b1", url: "/api/b", body: {}, label: "B" });
    expect(await flush()).toEqual({ sent: 1, dropped: [], left: 0 });
    expect(calls.map((call) => call.url)).toEqual(["/api/b"]);
    localStorage.setItem("token", "s_abc");
    expect(await flush()).toEqual({ sent: 1, dropped: [], left: 0 });
    expect(calls.map((call) => call.url)).toEqual(["/api/b", "/api/a"]);
  });
});
