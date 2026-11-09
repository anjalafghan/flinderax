import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import { spyOn } from "bun:test";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";

import { AuthContext } from "@/context/AuthContext";
import type { DietOverview, Mark } from "@/services/diet";

// Synthetic data only: made-up habits, amounts and dates.

const WEEK = 7 * 24 * 60 * 60 * 1000;
const CURRENT = "2026-11-02";

/** The last twelve week starts, newest first, the first being the current week. */
const recent = Array.from({ length: 12 }, (_, i) => {
  const d = new Date(Date.parse(`${CURRENT}T00:00:00Z`) - i * WEEK);
  return {
    week_start: d.toISOString().slice(0, 10),
    logged: i < 6,
    hit_bps: i < 6 ? 4000 : null,
  };
});

export const homeDinner = { id: "h1", label: "Home-cooked dinner", preset_key: "home_dinner", target_days: 5, active: true };
export const walk = { id: "h2", label: "Walk 30 minutes", preset_key: "walk", target_days: 4, active: true };

export const overview: DietOverview = {
  presets: [
    { key: "home_dinner", label: "Home-cooked dinner", target_days: 5 },
    { key: "no_sugary", label: "No sugary drinks", target_days: 6 },
    { key: "fruit_veg", label: "Fruit or salad every day", target_days: 5 },
    { key: "walk", label: "Walk 30 minutes", target_days: 4 },
  ],
  habits: [homeDinner, walk],
  week: { week_start: CURRENT, feel: 4, note: "", entries: [{ habit_id: "h1", days_hit: 2 }] },
  recent,
  money: {
    months: [
      { month: "2026-06", eating_out_paise: 420_000, groceries_paise: 900_000, delivery_orders: 6 },
      { month: "2026-07", eating_out_paise: 380_000, groceries_paise: 880_000, delivery_orders: 5 },
      { month: "2026-08", eating_out_paise: 510_000, groceries_paise: 910_000, delivery_orders: 9 },
      { month: "2026-09", eating_out_paise: 300_000, groceries_paise: 870_000, delivery_orders: 4 },
      { month: "2026-10", eating_out_paise: 460_000, groceries_paise: 940_000, delivery_orders: 8 },
      { month: "2026-11", eating_out_paise: 120_000, groceries_paise: 300_000, delivery_orders: 2 },
    ],
    eating_out_share_bps: 4400,
    delivery_per_week_x100: 350,
    cap_paise: null,
  },
  body: {
    weight_x10: 705,
    weight_on: "2026-10-12",
    prev_weight_x10: 717,
    waist_x10: null,
    last_checkup_on: "2026-03-12",
    checkup_due_on: "2027-03-12",
  },
};

export const marksFor = (kind: string): Mark[] =>
  kind === "weight"
    ? [
        { id: "m1", kind: "weight", measured_on: "2026-10-12", value_x10: 705, note: "" },
        { id: "m0", kind: "weight", measured_on: "2026-08-12", value_x10: 717, note: "" },
      ]
    : [];

/** Renders with the same providers the app has: auth, router and a react-query client with retries off. */
export function renderWithProviders(ui: ReactNode) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity }, mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={qc}>
      <AuthContext.Provider value={{ isAuthenticated: true, token: "t", login: () => {}, logout: () => {}, isLoading: false }}>
        <MemoryRouter>{ui}</MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  );
}

/** Stubs fetch for the PUT, PATCH and DELETE calls. The cast is needed because a stub has no `preconnect`. */
export const stubFetch = (impl: () => Promise<Response>) => spyOn(globalThis, "fetch").mockImplementation(impl as never);

export const respond = (data: unknown) => Promise.resolve({ data, status: 200, statusText: "OK" }) as never;

/** A fetch response for the PUT, PATCH and DELETE calls. */
export const jsonResponse = (data: unknown, status = 200) =>
  Promise.resolve(
    new Response(data === undefined ? null : JSON.stringify(data), {
      status,
      headers: { "Content-Type": "application/json" },
    }),
  );
