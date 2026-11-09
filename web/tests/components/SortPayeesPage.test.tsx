import { afterEach, describe, expect, it, mock } from "bun:test";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { AuthContext } from "@/context/AuthContext";

const toasts: string[] = [];
mock.module("sonner", () => ({
  toast: { success: (m: string) => toasts.push(m), error: () => {}, info: () => {} },
}));

import SortPayeesPage from "@/pages/SortPayeesPage";
import type { PayeesOut } from "@/services/payees";

// Synthetic data only.
const fixture: PayeesOut = {
  total_uncategorized_paise: 100000,
  covered_paise: 0,
  payees: [
    {
      payee: "SWIGGY INDIA",
      count: 3,
      paise: 60000,
      last_on: "2026-11-08",
      share_bps: 6000,
      suggestion: "food",
      sample: "UPI/XXXX/SWIGGY@ybl/XX1234",
    },
    {
      payee: "CORNER STORE",
      count: 1,
      paise: 40000,
      last_on: "2026-10-02",
      share_bps: 4000,
      suggestion: null,
      sample: "POS XXXX CORNER STORE",
    },
  ],
};

const originalFetch = globalThis.fetch;
let posts: { url: string; body: unknown }[] = [];

function stubFetch(payees: PayeesOut) {
  posts = [];
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    if (init?.method === "POST") {
      posts.push({ url: String(url), body: JSON.parse(String(init.body)) });
      return new Response(JSON.stringify({ updated: 3 }), { status: 200, headers: { "Content-Type": "application/json" } });
    }
    return new Response(JSON.stringify(payees), { status: 200, headers: { "Content-Type": "application/json" } });
  }) as typeof fetch;
}

function renderPage(payees: PayeesOut) {
  stubFetch(payees);
  // staleTime Infinity keeps the seeded data from refetching on mount.
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(["payees"], payees);
  return render(
    <AuthContext.Provider value={{ isAuthenticated: true, token: "t", login: () => {}, logout: () => {}, isLoading: false }}>
      <QueryClientProvider client={qc}>
        <MemoryRouter>
          <SortPayeesPage />
        </MemoryRouter>
      </QueryClientProvider>
    </AuthContext.Provider>,
  );
}

afterEach(() => {
  globalThis.fetch = originalFetch;
  toasts.length = 0;
});

describe("SortPayeesPage", () => {
  it("renders payees with title-cased names, metadata and the masked sample", () => {
    renderPage(fixture);
    expect(screen.getByRole("heading", { name: "Sort your payees" })).toBeInTheDocument();
    expect(screen.getByText("Tag each payee once. joy remembers it for past and future payments.")).toBeInTheDocument();
    expect(screen.getByText("Swiggy India")).toBeInTheDocument();
    expect(screen.getByText("3 payments · ₹600 · last 8 Nov 2026")).toBeInTheDocument();
    expect(screen.getByTitle("UPI/XXXX/SWIGGY@ybl/XX1234")).toBeInTheDocument();
    expect(screen.getByText("₹1,000 still untagged")).toBeInTheDocument();
  });

  it("puts the suggestion chip first and highlights it", () => {
    renderPage(fixture);
    const group = screen.getByRole("group", { name: "Category for Swiggy India" });
    const chips = within(group).getAllByRole("button");
    expect(chips[0]).toHaveAccessibleName("Tag Swiggy India as Food");
    expect(chips[0].className).toContain("tone-good");
    // The suggested category is not repeated among the presets.
    expect(within(group).getAllByRole("button", { name: "Tag Swiggy India as Food" })).toHaveLength(1);
    expect(within(group).getByRole("button", { name: "Tag Swiggy India as Groceries" })).toBeInTheDocument();
  });

  it("tagging with a chip calls the categorize endpoint with remember true, then removes the row", async () => {
    renderPage(fixture);
    const group = screen.getByRole("group", { name: "Category for Corner Store" });
    fireEvent.click(within(group).getByRole("button", { name: "Tag Corner Store as Shopping" }));

    await waitFor(() => expect(screen.queryByText("Corner Store")).toBeNull());
    expect(posts).toHaveLength(1);
    expect(posts[0].url).toBe("/api/ingest/payees/categorize");
    expect(posts[0].body).toEqual({ payee: "CORNER STORE", category: "shopping", remember: true });
    expect(toasts).toEqual(["Tagged 1 payment as Shopping"]);
    // Tagged money counts toward progress: ₹400 of ₹1,000 is 40%.
    expect(screen.getByText("40%")).toBeInTheDocument();
    expect(screen.getByText("₹600 still untagged")).toBeInTheDocument();
  });

  it("shows the empty state when nothing is uncategorised", () => {
    renderPage({ total_uncategorized_paise: 0, covered_paise: 0, payees: [] });
    expect(screen.getByText("Everything is tagged. Nice.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Money" })).toHaveAttribute("href", "/money");
  });
});
