import { afterEach, describe, expect, test } from "bun:test";
import { storageGet, storageRemove, storageSet } from "./storage";

const real = Object.getOwnPropertyDescriptor(globalThis, "localStorage");

afterEach(() => {
  if (real) Object.defineProperty(globalThis, "localStorage", real);
  else delete (globalThis as { localStorage?: unknown }).localStorage;
});

function blocked() {
  const boom = () => {
    throw new Error("blocked");
  };
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: { getItem: boom, setItem: boom, removeItem: boom },
  });
}

describe("storage helpers", () => {
  test("round-trip when storage works", () => {
    const mem = new Map<string, string>();
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      value: {
        getItem: (k: string) => mem.get(k) ?? null,
        setItem: (k: string, v: string) => void mem.set(k, v),
        removeItem: (k: string) => void mem.delete(k),
      },
    });
    expect(storageGet("a")).toBeNull();
    expect(storageSet("a", "1")).toBe(true);
    expect(storageGet("a")).toBe("1");
    storageRemove("a");
    expect(storageGet("a")).toBeNull();
  });

  test("never throws when storage is blocked", () => {
    blocked();
    expect(storageGet("a")).toBeNull();
    expect(storageSet("a", "1")).toBe(false);
    expect(() => storageRemove("a")).not.toThrow();
  });
});
