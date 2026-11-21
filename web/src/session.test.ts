import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { clearSession, offlineCacheName, sweepOfflineCopies } from "./session";

const g = globalThis as Record<string, unknown>;
let cacheNames: Set<string>;
const realCaches = g.caches;
const keys = () => Object.keys(localStorage).sort();

beforeEach(() => {
  localStorage.clear();
  localStorage.setItem("token", "tok-a");
  localStorage.setItem("expires_at", "123");
  localStorage.setItem("flinderax-whatif-v1", "[]");
  localStorage.setItem("joy-whatif-v1", "[]");
  localStorage.setItem("vite-ui-theme", "light");
  cacheNames = new Set(["joy-shell-v1", "fx-data-aaaa", "joy-data-bbbb"]);
  g.caches = {
    keys: async () => [...cacheNames],
    delete: async (k: string) => cacheNames.delete(k),
  };
});

afterEach(() => {
  localStorage.clear();
  g.caches = realCaches;
});

describe("session cleanup", () => {
  test("clearSession removes personal keys and saved copies, keeps theme and shell", async () => {
    clearSession();
    await new Promise((r) => setTimeout(r, 0));
    expect(keys()).toEqual(["vite-ui-theme"]);
    expect([...cacheNames]).toEqual(["joy-shell-v1"]);
  });

  test("sweep keeps only the current token's cache", async () => {
    const mine = await offlineCacheName("tok-a");
    cacheNames.add(mine);
    await sweepOfflineCopies();
    expect([...cacheNames].sort()).toEqual(["joy-shell-v1", mine].sort());
  });

  test("sweep with no token removes every saved copy", async () => {
    localStorage.removeItem("token");
    await sweepOfflineCopies();
    expect([...cacheNames]).toEqual(["joy-shell-v1"]);
  });
});

import { authHeadersFor, isCookieSession, loginToKeep, newSessionId } from "./session";

describe("cookie sessions", () => {
  test("the page keeps a harmless id, never the token, and sends it with the request marker", () => {
    const id = newSessionId();
    expect(id.startsWith("s_") && id.length >= 20).toBe(true);
    expect(newSessionId()).not.toBe(id);
    expect(isCookieSession(id)).toBe(true);
    expect(authHeadersFor(id)).toEqual({ "X-Session-Id": id, "X-Joy": "1" });
  });

  test("a cookie sign-in keeps an id, not the token; an older answer keeps the token", () => {
    const kept = loginToKeep({ access_token: "v4.local.SECRET", expires_at: 99, cookie: true });
    expect(kept.token.startsWith("s_")).toBe(true);
    expect(kept.token).not.toContain("SECRET");
    expect(loginToKeep({ access_token: "v4.local.abc", expires_at: 99 })).toEqual({ token: "v4.local.abc", expiresAt: 99 });
  });

  test("an older bearer login keeps working until it ends", () => {
    expect(isCookieSession("v4.local.abc")).toBe(false);
    expect(authHeadersFor("v4.local.abc")).toEqual({ Authorization: "Bearer v4.local.abc" });
    expect(authHeadersFor(null)).toEqual({});
  });

  test("saved offline copies are named the way the service worker names them", async () => {
    const id = newSessionId();
    const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(`sid:${id}`));
    const hex = [...new Uint8Array(digest)].slice(0, 8).map((b) => b.toString(16).padStart(2, "0")).join("");
    expect(await offlineCacheName(id)).toBe(`joy-data-${hex}`);
  });
});
