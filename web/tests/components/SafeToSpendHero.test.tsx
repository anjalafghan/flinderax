import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

import { SafeToSpendHero } from "@/components/feature/SafeToSpendHero";
import type { DashboardSummary } from "@/services/types";

const base: DashboardSummary = {
  safe_to_spend_paise: 1_301_699,
  per_week_paise: 325_424,
  status: "green",
  until: "2026-10-30",
  days_left: 25,
  period_start: "2026-09-30",
  period_label: "October money",
  next_income: { label: "November money", arrives: "2026-10-30" },
  short_by_paise: 0,
  breakdown: [],
  budget: {
    income_paise: 7_549_900,
    needs_paise: 700_000,
    spending_paise: 1_887_500,
    savings_paise: 4_962_400,
    sip_paise: 3_000_000,
    rest_savings_paise: 1_962_400,
    allocation_paise: 2_662_400,
  },
  pots: {
    spending: { account_name: "HDFC", balance_paise: 4_301_699, committed_paise: 3_000_000, to_move_paise: 0, free_paise: 1_301_699 },
    reserve: { account_name: "Kotak", balance_paise: 6_914_400, required_paise: 3_004_200, free_savings_paise: 3_910_200, floor_paise: 0, below_floor: false },
    locked: { total_paise: 20_904_400, count: 2 },
  },
  spending: { tracked: true, spent_paise: 0, before_tracking_paise: 0, allowance_paise: 1_887_500, left_paise: 1_875_600, overspend_paise: 0 },
  recovery: null,
  next_period: null,
  upcoming: [],
  actions: [],
  stale: [],
};

const renderHero = (s: DashboardSummary) =>
  render(
    <MemoryRouter>
      <SafeToSpendHero summary={s} />
    </MemoryRouter>,
  );

describe("SafeToSpendHero", () => {
  it("shows the big number in whole rupees, the weekly figure and the until date", () => {
    renderHero(base);
    expect(screen.getByText("₹13,017")).toBeInTheDocument();
    expect(screen.getByText(/₹3,254 a week/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Safe to spend until 30 Oct" })).toBeInTheDocument();
    expect(screen.getByText(/for the next 25 days\. On track\./)).toBeInTheDocument();
    expect(screen.getByText(/November money arrives 30 Oct and isn’t counted here/)).toBeInTheDocument();
  });

  it("shows the pay period as one bar per day", () => {
    renderHero(base);
    expect(screen.getByRole("img", { name: /of 30 in this pay period/ })).toBeInTheDocument();
  });

  it("shows how much has been spent against the allowance when no income is due", () => {
    renderHero({ ...base, next_income: null, spending: { ...base.spending, spent_paise: 1_000_000 } });
    expect(screen.getByText(/₹10,000 spent so far of a ₹18,875 allowance/)).toBeInTheDocument();
  });

  it("says so when spending isn't tracked yet", () => {
    renderHero({ ...base, next_income: null, spending: { ...base.spending, tracked: false } });
    expect(screen.getByText(/Spending isn’t tracked yet/)).toBeInTheDocument();
  });

  it("links to the working page", () => {
    renderHero(base);
    expect(screen.getByText(/See the working/).closest("a")?.getAttribute("href")).toBe("/working");
  });

  it("names the status in words", () => {
    const { container } = renderHero({ ...base, status: "red", safe_to_spend_paise: 0 });
    expect(container.querySelector("[data-status='red']")).not.toBeNull();
    expect(screen.getByText("A bit over plan")).toBeInTheDocument();
    expect(screen.getByText(/Pause non-essentials until 30 Oct\./)).toBeInTheDocument();
  });

  it("names the account that is short, in rupees", () => {
    renderHero({ ...base, status: "red", safe_to_spend_paise: 0, short_by_paise: 4_164_901 });
    expect(screen.getByText(/HDFC is short by ₹41,649.01/)).toBeInTheDocument();
  });

  it("reports an overspend", () => {
    renderHero({ ...base, spending: { ...base.spending, overspend_paise: 124_400 } });
    expect(screen.getByText(/over this month’s spending allowance/)).toBeInTheDocument();
  });

  it("marks a week-old number as approximate and offers an update", () => {
    renderHero({ ...base, stale: [{ what: "Kotak balance", days_old: 9 }] });
    expect(screen.getByText("~₹13,017")).toBeInTheDocument();
    expect(screen.getByText("Probably fine")).toBeInTheDocument();
    expect(screen.getByText(/Kotak balance from .* · 9 days old/)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Update" })).toHaveAttribute("href", "/money");
  });

  it("calls a three-week-old number a guess", () => {
    renderHero({ ...base, status: "yellow", stale: [{ what: "HDFC balance", days_old: 23 }] });
    expect(screen.getByText(/This is a guess, based on HDFC balance from/)).toBeInTheDocument();
    expect(screen.getByText("Probably tight")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Update balance" })).toHaveAttribute("href", "/money");
  });
});
