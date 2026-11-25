import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import path from "node:path";
import vm from "node:vm";

/** Runs public/sw.js against a fake service-worker scope and returns what it registered. */
function load() {
  const listeners: Record<string, (e: unknown) => void> = {};
  const shown: { title: string; options: Record<string, unknown> }[] = [];
  const opened: string[] = [];
  const focused: string[] = [];
  const windows: {
    focus: () => Promise<void>;
    navigate: (u: string) => void;
  }[] = [];
  const scope = {
    addEventListener: (type: string, fn: (e: unknown) => void) =>
      (listeners[type] = fn),
    registration: {
      showNotification: (title: string, options: Record<string, unknown>) => (
        shown.push({ title, options }),
        Promise.resolve()
      ),
    },
    clients: {
      matchAll: () => Promise.resolve(windows),
      openWindow: (u: string) => (opened.push(u), Promise.resolve()),
      claim: () => Promise.resolve(),
    },
    skipWaiting: () => {},
    location: { origin: "http://localhost" },
  };
  const code = readFileSync(
    path.join(import.meta.dir, "../public/sw.js"),
    "utf8",
  );
  vm.runInNewContext(code, {
    self: scope,
    caches: {},
    fetch: () => {},
    URL,
    Headers,
    Response,
    Request,
    crypto,
    TextEncoder,
    setTimeout,
    Promise,
    Date,
    Number,
    String,
    Array,
    Error,
  });
  return { listeners, shown, opened, focused, windows };
}

const event = (extra: Record<string, unknown>) => {
  const waits: Promise<unknown>[] = [];
  return {
    e: { ...extra, waitUntil: (p: Promise<unknown>) => waits.push(p) },
    done: () => Promise.all(waits),
  };
};

describe("notifications in the service worker", () => {
  test("a push shows the server's words and nothing else", async () => {
    const sw = load();
    const { e, done } = event({
      data: {
        json: () => ({ title: "joy", body: "Today: 2 bills.", url: "/money" }),
      },
    });
    sw.listeners.push!(e);
    await done();
    expect(sw.shown).toHaveLength(1);
    expect(sw.shown[0]?.title).toBe("joy");
    expect(sw.shown[0]?.options.body).toBe("Today: 2 bills.");
    expect(sw.shown[0]?.options.data).toEqual({ url: "/money" });
  });

  test("a push without readable data still shows something harmless", async () => {
    const sw = load();
    const { e, done } = event({
      data: {
        json: () => {
          throw new Error("not json");
        },
      },
    });
    sw.listeners.push!(e);
    await done();
    expect(sw.shown[0]?.title).toBe("joy");
    expect(sw.shown[0]?.options.body).toBe("");
  });

  test("tapping it opens the app when it is closed", async () => {
    const sw = load();
    let closed = false;
    const { e, done } = event({
      notification: { close: () => (closed = true), data: { url: "/money" } },
    });
    sw.listeners.notificationclick!(e);
    await done();
    expect(closed).toBe(true);
    expect(sw.opened).toEqual(["/money"]);
  });

  test("tapping it brings an open window forward and sends it to the page", async () => {
    const sw = load();
    const visited: string[] = [];
    let focused = false;
    sw.windows.push({
      focus: () => ((focused = true), Promise.resolve()),
      navigate: (u) => visited.push(u),
    });
    const { e, done } = event({
      notification: { close: () => {}, data: { url: "/" } },
    });
    sw.listeners.notificationclick!(e);
    await done();
    expect(focused).toBe(true);
    expect(visited).toEqual(["/"]);
    expect(sw.opened).toEqual([]);
  });
});
