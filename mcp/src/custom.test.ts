import { expect, test } from "bun:test";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { JoyClient, writable } from "./client.ts";
import { buildServer } from "./server.ts";
import { DRESS_PROMPT } from "./custom.ts";

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

test("only a slip's dress path is writable by id", () => {
  expect(writable("/api/dates/slips/42/dress")).toBe(true);
  expect(writable("/api/dates/slips/42")).toBe(false);
  expect(writable("/api/dates/slips/42/delete")).toBe(false);
  expect(writable("/api/dates/slips/x/dress")).toBe(false);
  expect(writable("/api/dates/slips/42/dress/../delete")).toBe(false);
});

async function connect(calls: { m: string; p: string; k?: string; b?: any }[]) {
  const joy = new JoyClient({
    baseUrl: "http://x", token: "t",
    fetch: (async (u: string | URL, init: RequestInit) => {
      const h = init.headers as Record<string, string>;
      calls.push({ m: init.method!, p: new URL(String(u)).pathname, k: h["idempotency-key"], b: init.body ? JSON.parse(init.body as string) : undefined });
      return init.method === "GET" ? json([{ slip_id: 7, title: "The day we met", dressed: false }]) : json({ ok: true, filled: ["notes"], kept: [], in_jar: false, also: [] });
    }) as any,
  });
  const [a, b] = InMemoryTransport.createLinkedPair();
  const server = buildServer(joy);
  await server.connect(a);
  const client = new Client({ name: "t", version: "0" });
  await client.connect(b);
  return client;
}

test("dates_custom reads the list; dates_dress_custom posts only what was given, once", async () => {
  const calls: { m: string; p: string; k?: string; b?: any }[] = [];
  const c = await connect(calls);
  const list = await c.callTool({ name: "dates_custom", arguments: {} });
  expect((list.content as any)[0].text).toContain("The day we met");

  const args = { slip_id: 7, notes: "Coffee, then the sea.", energy: "low", in_jar: false };
  await c.callTool({ name: "dates_dress_custom", arguments: args });
  await c.callTool({ name: "dates_dress_custom", arguments: args });
  const posts = calls.filter((x) => x.m === "POST");
  expect(posts.map((x) => x.p)).toEqual(["/api/dates/slips/7/dress", "/api/dates/slips/7/dress"]);
  expect(posts[0].b).toEqual({ notes: "Coffee, then the sea.", energy: "low", in_jar: false });
  expect(posts[0].k).toBe(posts[1].k);
  expect(posts[0].k!.startsWith("mcp-dress:7:")).toBe(true);
});

test("the prompt names its tools and keeps titles fixed", () => {
  for (const n of ["dates_custom", "dates_dress_custom", "dates_slips", "tools/keepsakes"]) expect(DRESS_PROMPT).toContain(n);
  expect(DRESS_PROMPT).toContain("never suggest renaming");
});
