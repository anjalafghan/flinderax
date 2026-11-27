import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiTokensCard } from "./ApiTokensCard";

const realFetch = globalThis.fetch;
let tokens: Array<Record<string, unknown>>;
const calls: Array<{ url: string; method: string; body?: unknown }> = [];

beforeEach(() => {
  calls.length = 0;
  tokens = [];
  globalThis.fetch = (async (url: string | URL | Request, init?: RequestInit) => {
    const path = String(url);
    const method = init?.method ?? "GET";
    const body = init?.body ? JSON.parse(String(init.body)) : undefined;
    calls.push({ url: path, method, body });
    if (path === "/api/tokens" && method === "POST") {
      tokens.push({ id: "tok1", name: body.name, scopes: body.scopes, created_at: 1, last_used_at: null, expires_at: null, revoked_at: null });
      return new Response(JSON.stringify({ ...tokens[0], token: "joy_one_time_secret" }), { status: 201 });
    }
    if (path === "/api/tokens" && method === "GET") return new Response(JSON.stringify(tokens), { status: 200 });
    if (path === "/api/tokens/audit") return new Response("[]", { status: 200 });
    if (path === "/api/tokens/tok1/revoke" && method === "POST") {
      tokens[0]!.revoked_at = 2;
      return new Response(null, { status: 204 });
    }
    return new Response("Not found", { status: 404 });
  }) as typeof fetch;
});

afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("ApiTokensCard", () => {
  test("creates a token and only displays the raw value in the one-time notice", async () => {
    render(<ApiTokensCard />);
    fireEvent.change(screen.getByLabelText(/token name/i), { target: { value: "Claude daily brief" } });
    fireEvent.click(screen.getByRole("button", { name: /create token/i }));
    expect(await screen.findByText("joy_one_time_secret")).toBeTruthy();
    expect(screen.getByText(/only be shown once/i)).toBeTruthy();
    expect(calls.find((call) => call.method === "POST")?.body).toMatchObject({
      name: "Claude daily brief",
      scopes: ["money:read", "dates:read"],
    });
  });

  test("revokes a listed token", async () => {
    tokens = [{ id: "tok1", name: "Claude", scopes: ["money:read"], created_at: 1, last_used_at: null, expires_at: null, revoked_at: null }];
    render(<ApiTokensCard />);
    fireEvent.click(await screen.findByRole("button", { name: /revoke/i }));
    await waitFor(() => expect(screen.getByText(/Claude · Revoked/)).toBeTruthy());
    expect(calls.some((call) => call.url === "/api/tokens/tok1/revoke" && call.method === "POST")).toBe(true);
  });
});
