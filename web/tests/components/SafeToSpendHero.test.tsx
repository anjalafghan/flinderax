import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";

import { SafeToSpendHero } from "@/components/feature/SafeToSpendHero";
import type { DashboardSummary } from "@/services/types";

const base: DashboardSummary = {
  safe_to_spend_paise: 1_500_000,
  per_week_paise: 375_000,
  status: "green",
  until: "2026-10-30",
  period_label: "October money",
  next_income: { label: "November money", arrives: "2026-10-30" },
  lowest_point: { amount_paise: 2_000_000, date: "2026-10-29" },
  upcoming: [],
  actions: [],
  stale: [],
};

describe("SafeToSpendHero", () => {
  it("shows the big number, the weekly figure and the until date", () => {
    render(<SafeToSpendHero summary={base} />);
    expect(screen.getByText("₹15,000")).toBeInTheDocument();
    expect(screen.getByText("₹3,750")).toBeInTheDocument();
    expect(screen.getByText(/until 30 Oct/)).toBeInTheDocument();
    expect(screen.getByText(/November money, arrives 30 Oct/)).toBeInTheDocument();
  });

  it("reflects the status colour band", () => {
    const { container } = render(<SafeToSpendHero summary={{ ...base, status: "red", safe_to_spend_paise: 0 }} />);
    expect(container.querySelector("[data-status='red']")).not.toBeNull();
    expect(screen.getByText("Don't spend")).toBeInTheDocument();
  });

  it("warns about stale data", () => {
    render(<SafeToSpendHero summary={{ ...base, stale: [{ what: "Kotak balance", days_old: 9 }] }} />);
    expect(screen.getByText(/Kotak balance is 9 days old/)).toBeInTheDocument();
  });
});
