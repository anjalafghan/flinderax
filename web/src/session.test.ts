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
  localStorage.setItem("vite-ui-theme", "light");
  cacheNames = new Set(["fx-shell-v1", "fx-data-aaaa", "fx-data-bbbb"]);
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
    expect([...cacheNames]).toEqual(["fx-shell-v1"]);
  });

  test("sweep keeps only the current token's cache", async () => {
    const mine = await offlineCacheName("tok-a");
    cacheNames.add(mine);
    await sweepOfflineCopies();
    expect([...cacheNames].sort()).toEqual(["fx-shell-v1", mine].sort());
  });

  test("sweep with no token removes every saved copy", async () => {
    localStorage.removeItem("token");
    await sweepOfflineCopies();
    expect([...cacheNames]).toEqual(["fx-shell-v1"]);
  });
});
