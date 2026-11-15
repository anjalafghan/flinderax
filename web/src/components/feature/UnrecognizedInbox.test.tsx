import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { UnrecognizedInbox } from "./UnrecognizedInbox";

let calls: { url: string; method?: string; body?: unknown }[];
let items: { alert_id: string; text: string; source: string; received_at: string }[];
const realFetch = globalThis.fetch;

const view = () =>
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <UnrecognizedInbox />
    </QueryClientProvider>,
  );

beforeEach(() => {
  calls = [];
  items = [{ alert_id: "a1", text: "A/c XX9 settled seven hundred at SOME SHOP", source: "sms", received_at: "2026-11-14 10:00:00" }];
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({ url: String(url), method: init?.method, body: init?.body ? JSON.parse(String(init.body)) : undefined });
    if (String(url).includes("/dismiss")) {
      items = [];
      return new Response(JSON.stringify({ status: true }), { status: 200 });
    }
    return new Response(JSON.stringify({ count: items.length, items }), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("UnrecognizedInbox", () => {
  test("shows the count and the text of what couldn't be read", async () => {
    view();
    expect(await screen.findByText(/1 message we couldn't read/i)).toBeTruthy();
    expect(screen.getByText(/SOME SHOP/)).toBeTruthy();
  });

  test("dismissing one clears it", async () => {
    view();
    fireEvent.click(await screen.findByRole("button", { name: /dismiss/i }));
    await waitFor(() => expect(calls.some((c) => c.url.includes("/dismiss") && (c.body as { alert_id: string }).alert_id === "a1")).toBe(true));
    await waitFor(() => expect(screen.queryByText(/SOME SHOP/)).toBeNull());
  });

  test("renders nothing when the inbox is empty", async () => {
    items = [];
    const { container } = view();
    await waitFor(() => expect(calls.length).toBeGreaterThan(0));
    expect(container.textContent).toBe("");
  });
});
