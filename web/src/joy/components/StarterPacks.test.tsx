import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { StarterPacks } from "./StarterPacks";

let calls: { url: string; body?: unknown }[];
let packs: unknown[];
const realFetch = globalThis.fetch;

const view = () =>
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <StarterPacks />
    </QueryClientProvider>,
  );

beforeEach(() => {
  calls = [];
  packs = [
    { key: "home", label: "At home", blurb: "Cooking and games.", total: 40, added: 0 },
    { key: "mumbai", label: "Mumbai", blurb: "Places in Mumbai.", total: 250, added: 250 },
  ];
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({ url: String(url), body: init?.body ? JSON.parse(String(init.body)) : undefined });
    if (String(url).endsWith("/packs/add")) return new Response(JSON.stringify({ added: 40 }), { status: 200 });
    return new Response(JSON.stringify(packs), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("StarterPacks", () => {
  test("offers each pack with what it holds; one already added says so", async () => {
    view();
    expect(await screen.findByText("At home", { selector: "strong" })).toBeTruthy();
    expect(screen.getByText(/40 slips/)).toBeTruthy();
    expect(screen.getByRole("button", { name: /add at home/i })).toBeTruthy();
    expect(screen.getByText(/added/i, { selector: ".pack-done" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /add mumbai/i })).toBeNull();
  });

  test("adding a pack asks the server and says how many came in", async () => {
    view();
    fireEvent.click(await screen.findByRole("button", { name: /add at home/i }));
    await waitFor(() => expect(calls.some((c) => c.url.endsWith("/packs/add") && (c.body as { pack: string }).pack === "home")).toBe(true));
    expect(await screen.findByText(/40 slips added/i)).toBeTruthy();
  });
});
