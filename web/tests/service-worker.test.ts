import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import { describe, expect, test } from "bun:test";

function loadKeptApi(cache: Cache) {
  const source = readFileSync(new URL("../public/sw.js", import.meta.url), "utf8");
  const context = {
    self: { addEventListener() {} },
    caches: {
      open: async () => cache,
      delete: async () => true,
      keys: async () => [],
    },
    crypto,
    TextEncoder,
    Headers,
    Request,
    Response,
    URL,
    Date,
    setTimeout,
    fetch: async () => new Response("live response", { status: 200 }),
  };
  runInNewContext(`${source}\nglobalThis.__keptApi = keptApi;`, context);
  return (context as typeof context & { __keptApi: (request: Request) => Promise<Response> }).__keptApi;
}

describe("service worker kept API cache", () => {
  test("a cache write failure preserves the successful live response", async () => {
    const cache = {
      match: async () => new Response("stale response", { status: 200, headers: { "x-joy-saved-at": String(Date.now()) } }),
      put: async () => { throw new Error("quota exceeded"); },
    } as unknown as Cache;
    const keptApi = loadKeptApi(cache);
    const request = new Request("https://joy.test/api/dashboard/summary", {
      headers: { authorization: "Bearer session" },
    });
    const response = await keptApi(request);
    expect(response.status).toBe(200);
    expect(await response.text()).toBe("live response");
  });
});
