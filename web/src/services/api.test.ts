import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { ingestApi } from "./api";

const originalFetch = globalThis.fetch;

beforeEach(() => {
  localStorage.clear();
  localStorage.setItem("token", "v4.local.old-token");
});

afterEach(() => {
  globalThis.fetch = originalFetch;
  localStorage.clear();
});

describe("multipart statement upload", () => {
  test("uses the shared session renewal behavior without setting a multipart boundary", async () => {
    let request: RequestInit | undefined;
    globalThis.fetch = (async (_url: RequestInfo | URL, init?: RequestInit) => {
      request = init;
      return new Response(JSON.stringify({ imported: 1 }), {
        status: 200,
        headers: { "content-type": "application/json", "x-refreshed-token": "v4.local.renewed" },
      });
    }) as typeof fetch;

    await ingestApi.uploadStatement(new File(["date,amount\n"], "statement.csv"));
    expect(new Headers(request?.headers).get("authorization")).toBe("Bearer v4.local.old-token");
    expect(new Headers(request?.headers).has("content-type")).toBe(false);
    expect(localStorage.getItem("token")).toBe("v4.local.renewed");
  });
});
