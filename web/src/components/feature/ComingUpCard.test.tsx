import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { ComingUpCard } from "./ComingUpCard";

let items: unknown[];
const realFetch = globalThis.fetch;

const view = () =>
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <MemoryRouter>
        <ComingUpCard />
      </MemoryRouter>
    </QueryClientProvider>,
  );

beforeEach(() => {
  items = [];
  globalThis.fetch = (async () => new Response(JSON.stringify({ items }), { status: 200 })) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("ComingUpCard", () => {
  test("lists what is coming, soonest first, each with when and where to go", async () => {
    items = [
      { on: "2026-11-16", kind: "bill", title: "Rent", detail: "₹25,000", link: "/money", days_away: 1, when: "Tomorrow" },
      { on: "2026-11-18", kind: "birthday", title: "Asha: birthday", detail: "", link: "/people/p1", days_away: 3, when: "In 3 days" },
    ];
    view();
    expect(await screen.findByText("Rent")).toBeTruthy();
    expect(screen.getByText("Tomorrow")).toBeTruthy();
    expect(screen.getByText("₹25,000")).toBeTruthy();
    const link = screen.getByRole("link", { name: /Asha: birthday/ });
    expect(link.getAttribute("href")).toBe("/people/p1");
  });

  test("shows nothing at all when nothing is coming", async () => {
    const { container } = view();
    await new Promise((r) => setTimeout(r, 30));
    expect(container.textContent).toBe("");
  });
});
