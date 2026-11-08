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
  it("shows the big number, the weekly figure and the until date", () => {
    renderHero(base);
    expect(screen.getByText("₹13,016.99")).toBeInTheDocument();
    expect(screen.getByText("₹3,254.24")).toBeInTheDocument();
    expect(screen.getByText(/until 30 Oct \(25 days\)/)).toBeInTheDocument();
    expect(screen.getByText(/November money \(arrives 30 Oct\) is not counted/)).toBeInTheDocument();
  });

  it("shows how much has been spent against the allowance", () => {
    renderHero({ ...base, spending: { ...base.spending, spent_paise: 1_000_000 } });
    expect(screen.getByText(/₹10,000 spent on your cards since tracking began, out of a ₹18,875 monthly allowance/)).toBeInTheDocument();
  });

  it("says so when spending isn't tracked yet", () => {
    renderHero({ ...base, spending: { ...base.spending, tracked: false } });
    expect(screen.getByText(/Spending isn't tracked yet/)).toBeInTheDocument();
  });

  it("links to the working page", () => {
    renderHero(base);
    expect(screen.getByText("See the working").closest("a")?.getAttribute("href")).toBe("/working");
  });

  it("reflects the status colour band", () => {
    const { container } = renderHero({ ...base, status: "red", safe_to_spend_paise: 0 });
    expect(container.querySelector("[data-status='red']")).not.toBeNull();
    expect(screen.getByText("Don't spend")).toBeInTheDocument();
  });

  it("names the account that is short, in rupees", () => {
    renderHero({ ...base, status: "red", safe_to_spend_paise: 0, short_by_paise: 4_164_901 });
    expect(screen.getByText(/HDFC is short by ₹41,649.01/)).toBeInTheDocument();
  });

  it("reports an overspend", () => {
    renderHero({ ...base, spending: { ...base.spending, overspend_paise: 124_400 } });
    expect(screen.getByText(/over this month's spending allowance/)).toBeInTheDocument();
  });

  it("warns about stale data", () => {
    renderHero({ ...base, stale: [{ what: "Kotak balance", days_old: 9 }] });
    expect(screen.getByText(/Kotak balance is 9 days old/)).toBeInTheDocument();
  });
});
