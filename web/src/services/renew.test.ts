import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { adoptRenewedToken } from "./renew";

beforeEach(() => localStorage.clear());
afterEach(() => localStorage.clear());

const res = (headers: Record<string, string>) => new Response("{}", { headers });

describe("a renewed session", () => {
  test("the new token replaces the old one", () => {
    localStorage.setItem("token", "old");
    adoptRenewedToken(res({ "x-refreshed-token": "new", "x-token-expires-at": "1900000000" }));
    expect(localStorage.getItem("token")).toBe("new");
  });

  test("a response without one changes nothing", () => {
    localStorage.setItem("token", "old");
    adoptRenewedToken(res({}));
    expect(localStorage.getItem("token")).toBe("old");
  });

  test("nobody signed in, nothing to renew", () => {
    adoptRenewedToken(res({ "x-refreshed-token": "new" }));
    expect(localStorage.getItem("token")).toBeNull();
  });
});
