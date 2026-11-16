import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

import { loadQueue } from "@/offline";
import type { Budget } from "@/services/wedding";
import { WeddingBudget } from "./WeddingBudget";

let calls: { url: string; method: string; body?: unknown; key?: string }[];
let budget: Budget;
let offline = false;
const realFetch = globalThis.fetch;

const view = () =>
  render(
    <MemoryRouter>
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <WeddingBudget goalId="g1" />
      </QueryClientProvider>
    </MemoryRouter>,
  );

const item = (id: string, name: string, category: string, estimate: number, quote: number | null, milestones: Budget["items"][number]["milestones"] = []) => ({
  item_id: id,
  name,
  category,
  estimate_paise: estimate,
  estimate_input: String(estimate / 100),
  quote_paise: quote,
  quote_input: quote === null ? "" : String(quote / 100),
  vendor: "",
  notes: "",
  planned_paise: quote ?? estimate,
  paid_paise: 0,
  remaining_paise: quote ?? estimate,
  milestones,
});

beforeEach(() => {
  calls = [];
  offline = false;
  localStorage.clear();
  budget = {
    goal_id: "g1",
    name: "Wedding",
    target_paise: 50_000_000,
    target_date: null,
    saved_paise: 6_000_000,
    on_hand_paise: 6_000_000,
    expected_monthly_paise: 0,
    categories: ["venue", "catering", "misc"],
    items: [
      item("i1", "Banquet hall", "venue", 10_000_000, 8_000_000, [
        { milestone_id: "m1", item_id: "i1", due_on: "2026-12-01", amount_paise: 7_000_000, amount_input: "70000", paid: false, paid_on: null, overdue: false },
      ]),
      item("i2", "Caterer", "catering", 3_000_000, null),
    ],
    summary: {
      categories: [
        { category: "venue", items: 1, planned_paise: 8_000_000, committed_paise: 8_000_000, estimated_paise: 0, paid_paise: 0, remaining_paise: 8_000_000 },
        { category: "catering", items: 1, planned_paise: 3_000_000, committed_paise: 0, estimated_paise: 3_000_000, paid_paise: 0, remaining_paise: 3_000_000 },
      ],
      planned_paise: 11_000_000,
      committed_paise: 8_000_000,
      estimated_paise: 3_000_000,
      paid_paise: 0,
      remaining_paise: 11_000_000,
      scheduled_paise: 7_000_000,
      unscheduled_paise: 4_000_000,
      overdue_paise: 0,
    },
    verdict: { status: "short", gap_paise: 1_000_000, by: "2026-12-01", line: "Short by ₹10,000 by 1 Dec 2026" },
    next_due: { milestone_id: "m1", item_name: "Banquet hall", due_on: "2026-12-01", amount_paise: 7_000_000, days_away: 15 },
  };
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    const headers = new Headers(init?.headers);
    calls.push({ url: String(url), method: init?.method ?? "GET", body: init?.body ? JSON.parse(String(init.body)) : undefined, key: headers.get("Idempotency-Key") ?? undefined });
    if (offline && init?.method === "POST") throw new TypeError("network down");
    if (init?.method === "POST") return new Response(JSON.stringify({ status: true }), { status: 200 });
    return new Response(JSON.stringify(budget), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("WeddingBudget", () => {
  test("opens with the verdict, then each category with its items and payments", async () => {
    view();
    expect(await screen.findByText("Short by ₹10,000 by 1 Dec 2026")).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Venue" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Catering" })).toBeTruthy();
    expect(screen.getByText("Banquet hall")).toBeTruthy();
    expect(screen.getByText(/Quote ₹80,000 \(estimate ₹1,00,000\)/)).toBeTruthy();
    expect(screen.getByText(/Estimate ₹30,000/)).toBeTruthy();
    expect(screen.getByText(/₹70,000 due 1 Dec/)).toBeTruthy();
    expect(screen.getByText(/Planned ₹1,10,000/)).toBeTruthy();
  });

  test("an item added in rupees is sent in paise", async () => {
    view();
    fireEvent.click(await screen.findByText(/add an item/i));
    fireEvent.change(screen.getByLabelText("Item name new"), { target: { value: "Photographer" } });
    fireEvent.change(screen.getByLabelText("Category new"), { target: { value: "misc" } });
    fireEvent.change(screen.getByLabelText("Estimate new"), { target: { value: "45,000" } });
    fireEvent.change(screen.getByLabelText("Quote new"), { target: { value: "40000.50" } });
    fireEvent.click(screen.getByRole("button", { name: /^add item$/i }));
    await waitFor(() => expect(calls.some((c) => c.url.endsWith("/g1/items"))).toBe(true));
    expect(calls.find((c) => c.url.endsWith("/g1/items"))?.body).toEqual({
      name: "Photographer",
      category: "misc",
      estimate_paise: 4_500_000,
      quote_paise: 4_000_050,
      vendor: "",
      notes: "",
    });
  });

  test("changing a quote sends the new amount in paise", async () => {
    view();
    fireEvent.click(await screen.findByText(/edit banquet hall/i));
    fireEvent.change(screen.getByLabelText("Quote i1"), { target: { value: "95000" } });
    fireEvent.click(screen.getAllByRole("button", { name: /save changes/i })[0]!);
    await waitFor(() => expect(calls.some((c) => c.url.endsWith("/items/i1"))).toBe(true));
    expect((calls.find((c) => c.url.endsWith("/items/i1"))?.body as { quote_paise: number }).quote_paise).toBe(9_500_000);
  });

  test("a payment is added with its date and amount in paise", async () => {
    view();
    fireEvent.change(await screen.findByLabelText("Due date for Caterer"), { target: { value: "2027-02-01" } });
    fireEvent.change(screen.getByLabelText("Payment amount for Caterer"), { target: { value: "10,000" } });
    fireEvent.click(screen.getAllByRole("button", { name: /add payment/i })[1]!);
    await waitFor(() => expect(calls.some((c) => c.url.endsWith("/items/i2/milestones"))).toBe(true));
    expect(calls.find((c) => c.url.endsWith("/items/i2/milestones"))?.body).toEqual({ due_on: "2027-02-01", amount_paise: 1_000_000 });
  });

  test("marking a payment paid sends it once-only with an idempotency key", async () => {
    view();
    fireEvent.click(await screen.findByRole("button", { name: /mark banquet hall ₹70,000 paid/i }));
    await waitFor(() => expect(calls.some((c) => c.url.endsWith("/milestones/m1/pay"))).toBe(true));
    const sent = calls.find((c) => c.url.endsWith("/milestones/m1/pay"))!;
    expect(sent.method).toBe("POST");
    expect(sent.key).toBeTruthy();
    expect((sent.body as { idempotency_key: string }).idempotency_key).toBeTruthy();
  });

  test("with no signal a payment is kept on the phone and the person is told", async () => {
    offline = true;
    view();
    fireEvent.click(await screen.findByRole("button", { name: /mark banquet hall ₹70,000 paid/i }));
    expect(await screen.findByText(/saved on this phone/i)).toBeTruthy();
    const queued = loadQueue();
    expect(queued).toHaveLength(1);
    expect(queued[0]!.url).toBe("/api/couple-goals/g1/milestones/m1/pay");
  });
});
