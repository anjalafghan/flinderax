import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import MoneyPage from "./MoneyPage";

const analytics = {
  coverage: null,
  months: [],
  savings_rate_bps: null,
  categories: [],
  top_payees: [],
  small_ticket: { per_month_count: 0, per_month_paise: 0, threshold_paise: 0 },
  salary: null,
  sips: [],
  autopay: [],
  own_transfers: null,
  frequent_small: [],
  weekday_spend_paise: [0, 0, 0, 0, 0, 0, 0],
  notes: [],
};

const insights = {
  refunds: [],
  charges: { last_12m_paise: 0, items: [] },
  duplicates: [],
  unusual: [],
  health: {
    savings_rate_bps: null,
    emergency_months_tenths: null,
    income_cv_bps: null,
  },
  spend_to_payday: null,
};

const history = {
  months: [
    {
      month: "2026-10",
      liquid_paise: 5_000_000,
      locked_paise: 1_000_000,
      card_debt_paise: 0,
      net_worth_paise: 6_000_000,
    },
    {
      month: "2026-11",
      liquid_paise: 5_500_000,
      locked_paise: 1_000_000,
      card_debt_paise: 0,
      net_worth_paise: 6_500_000,
    },
  ],
  cards: [],
  maturities: [],
  auto_postings: [],
  auto_post: false,
};

describe("MoneyPage", () => {
  test("shows net worth and asks for a statement when there is none", async () => {
    mockApi({
      "/api/dashboard/history": history,
      "/api/dashboard/analytics": analytics,
      "/api/dashboard/insights": insights,
    });
    renderPage(<MoneyPage />);
    expect(await screen.findByRole("heading", { name: "Net worth" })).toBeTruthy();
    expect(await screen.findByText("Upload a statement to see this.")).toBeTruthy();
    expect(await screen.findByText(/Not enough transactions yet/)).toBeTruthy();
  });

  test("says net worth starts building when there is no history", async () => {
    mockApi({
      "/api/dashboard/history": { ...history, months: [] },
      "/api/dashboard/analytics": analytics,
      "/api/dashboard/insights": insights,
    });
    renderPage(<MoneyPage />);
    expect(await screen.findByText(/Net worth history starts building/)).toBeTruthy();
  });

  test("shows loading skeletons", () => {
    mockApi({
      "/api/dashboard/history": pending,
      "/api/dashboard/analytics": pending,
      "/api/dashboard/insights": pending,
    });
    renderPage(<MoneyPage />);
    expect(screen.getByLabelText("Loading your money")).toBeTruthy();
    expect(screen.getByLabelText("Loading insights")).toBeTruthy();
  });

  test("offers a retry on each panel that fails", async () => {
    mockApi({
      "/api/dashboard/history": history,
      "/api/dashboard/analytics": failWith(500),
      "/api/dashboard/insights": failWith(500),
    });
    renderPage(<MoneyPage />);
    expect(await screen.findByText(/Couldn't load your insights/)).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Try again" }).length).toBe(2);
  });
});
