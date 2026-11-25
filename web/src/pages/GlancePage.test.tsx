import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

import GlancePage from "./GlancePage";

const realFetch = globalThis.fetch;

const summary = {
  safe_to_spend_paise: 0,
  per_week_paise: 0,
  status: "red",
  until: "2026-11-30",
  days_left: 5,
  period_start: "2026-10-30",
  period_label: "November money",
  next_income: null,
  short_by_paise: 0,
  breakdown: [],
  budget: {},
  pots: {
    spending: {
      account_name: "HDFC savings",
      balance_paise: 11_170_315,
      committed_paise: 0,
      to_move_paise: 0,
      free_paise: 0,
    },
    reserve: {
      account_name: "Kotak",
      balance_paise: 2600,
      required_paise: 0,
      free_savings_paise: 0,
      floor_paise: 0,
      below_floor: false,
    },
    locked: { total_paise: 8_737_500, count: 1 },
  },
  spending: {
    tracked: true,
    spent_paise: 5_007_747,
    before_tracking_paise: 0,
    allowance_paise: 2_220_000,
    left_paise: 0,
    overspend_paise: 4_157_767,
  },
  recovery: null,
  next_period: {
    label: "December money",
    arrives: "2026-11-30",
    income_paise: 7_485_900,
    committed_paise: 4_180_000,
    left_paise: 3_305_900,
  },
  upcoming: [
    {
      date: "2026-12-10",
      label: "SIP",
      amount_paise: 2_500_000,
      kind: "commitment",
    },
  ],
  actions: [],
  stale: [],
};

const answers: Record<string, unknown> = {
  "/api/dashboard/summary": summary,
  "/api/dashboard/analytics": {
    coverage: null,
    months: [
      {
        month: "2026-10",
        income_paise: 0,
        spend_paise: 3_987_244,
        invested_paise: 17_623_000,
        transfers_paise: 0,
        refunds_paise: 0,
        net_paise: 0,
      },
    ],
    categories: [
      { category: "food", paise: 500_144, share_bps: 559, count: 34 },
    ],
  },
  "/api/year": {
    months: [],
    first_short_month: null,
    lowest_paise: 0,
    total_lumpy_out_paise: 0,
  },
  "/api/goals": {
    goals: [
      {
        id: "g1",
        name: "Emergency fund",
        target_paise: 10_000_000,
        saved_paise: 0,
        this_month_paise: 0,
        needs_monthly_paise: 909_091,
        months_left: 11,
        status: "on_track",
        reached_by: null,
        fixes: [],
        target_date: "2027-09-01",
      },
    ],
    available_paise: 0,
    unallocated_paise: 0,
    months_covered_tenths: null,
    emergency_suggested_paise: null,
  },
  "/api/rewards/fund": {
    wallet_paise: 0,
    paid_in_paise: 0,
    spent_paise: 0,
    max_payout_paise: 200_000,
    fund_since: "2026-10",
    months: [],
    this_month: {
      month: "2026-11",
      days_left: 6,
      score: {
        areas: [
          {
            area: "money",
            label: "Money",
            weight_bps: 4000,
            raw: 0,
            target: 20,
            score_bps: 0,
            tip: "Log today's spending.",
          },
        ],
        score_bps: 6000,
        earned_paise: 120_000,
        saved_paise: 0,
        payout_paise: 0,
        held_back_paise: 120_000,
        focus: "money",
      },
    },
  },
  "/api/plan/road": {
    milestones: [
      {
        on: "2029-02-14",
        title: "Wedding",
        area: "family",
        source: "life",
        cost_paise: 83_215_100,
        saved_paise: 0,
        per_month_paise: 1_335_500,
        months_away: 27,
        link: "/life",
      },
    ],
    prompts: [],
    pace_all_paise: 1_335_500,
    biggest: null,
  },
  "/api/couple": { in_couple: false },
};

beforeEach(() => {
  globalThis.fetch = (async (input: RequestInfo | URL) => {
    const path = new URL(String(input), "http://x").pathname;
    const body = answers[path];
    return body === undefined
      ? new Response("{}", { status: 404 })
      : new Response(JSON.stringify(body), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("GlancePage", () => {
  test("says where spending stands and shows the tiles, goals and the road", async () => {
    render(
      <QueryClientProvider
        client={
          new QueryClient({ defaultOptions: { queries: { retry: false } } })
        }
      >
        <MemoryRouter>
          <GlancePage />
        </MemoryRouter>
      </QueryClientProvider>,
    );
    expect(
      await screen.findByText(
        /Spending is ₹41,578 past its ₹22,200 allowance with 5 days to go/,
      ),
    ).toBeTruthy();
    expect(screen.getByText("₹1,11,703")).toBeTruthy();
    expect(screen.getByText("₹33,059")).toBeTruthy();
    expect(await screen.findByText("Emergency fund")).toBeTruthy();
    expect(await screen.findByRole("link", { name: /Wedding/ })).toBeTruthy();
    expect(screen.getByText(/the wedding alone needs/)).toBeTruthy();
    expect(screen.getByText(/Start the jar with your partner/)).toBeTruthy();
  });
});
