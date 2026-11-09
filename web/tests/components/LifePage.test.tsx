import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { forecastKey, type Forecast } from "@/services/forecast";
import { MemoryRouter } from "react-router-dom";

import LifePage from "@/pages/LifePage";
import { AuthContext } from "@/context/AuthContext";
import { lifeKey, type LifeOut } from "@/services/life";

const profile = {
  my_age: 28,
  partner_age: 27,
  my_take_home_paise: 15_000_000,
  partner_take_home_paise: 10_000_000,
  monthly_expenses_paise: 9_000_000,
  wedding_date: "2028-02-10",
  wedding_budget_paise: 2_500_000_00,
  wedding_saved_paise: 0,
  child_birth_year: 2029,
  child_education_today_paise: 3_000_000_00,
  child_saved_paise: 0,
  retirement_age: 60,
  retirement_saved_paise: 0,
  emergency_saved_paise: 0,
  term_cover_paise: 0,
  health_cover_paise: 0,
  loans_paise: 0,
};

const fixture: LifeOut = {
  saved: true,
  profile,
  plan: {
    assumptions: {
      general_inflation_bps: 600,
      education_inflation_bps: 1000,
      medical_inflation_bps: 1000,
      wedding_inflation_bps: 800,
      equity_return_bps: 1150,
      hybrid_return_bps: 900,
      debt_return_bps: 700,
      step_up_bps: 1000,
      swr_bps: 325,
    },
    goals: [
      {
        key: "wedding",
        label: "Wedding",
        when: "2028-02-10",
        years_away: 1,
        today_cost_paise: 2_500_000_00,
        future_cost_paise: 2_700_000_00,
        saved_paise: 0,
        gap_paise: 2_700_000_00,
        monthly_sip_paise: 9_000_000,
        step_up_sip_paise: 8_500_000,
        mix: "debt",
        mix_note: "Under 3 years: RD/FD or a liquid fund, not equity.",
        my_share_paise: 5_000_000,
        partner_share_paise: 3_500_000,
        status: "needs_more",
        note: "You are short by about ₹1,20,000 a month on this one.",
      },
      {
        key: "child_education",
        label: "Child's education",
        when: "2047-06-01",
        years_away: 18,
        today_cost_paise: 3_000_000_00,
        future_cost_paise: 15_000_000_00,
        saved_paise: 0,
        gap_paise: 15_000_000_00,
        monthly_sip_paise: 4_000_000,
        step_up_sip_paise: 3_000_000,
        mix: "equity",
        mix_note: "Over 7 years: equity index funds.",
        my_share_paise: 2_000_000,
        partner_share_paise: 1_000_000,
        status: "on_track",
        note: "On track with the current SIP.",
      },
      {
        key: "retirement",
        label: "Retirement",
        when: "2058-11-09",
        years_away: 30,
        today_cost_paise: 0,
        future_cost_paise: 40_000_000_00,
        saved_paise: 0,
        gap_paise: 40_000_000_00,
        monthly_sip_paise: 6_000_000,
        step_up_sip_paise: 4_000_000,
        mix: "equity",
        mix_note: "Over 7 years: equity index funds.",
        my_share_paise: 1_000_000,
        partner_share_paise: 1_000_000,
        status: "needs_more",
        note: "Keep stepping up the SIP each year.",
      },
    ],
    insurance: {
      term_needed_paise: 30_000_000_00,
      term_have_paise: 0,
      term_gap_paise: 30_000_000_00,
      health_needed_paise: 1_500_000_00,
      health_have_paise: 1_000_000_00,
      health_gap_paise: 500_000_00,
      notes: ["Term cover is 15 years of take-home for each of you."],
    },
    monthly: {
      total_sip_paise: 19_000_000,
      total_step_up_sip_paise: 15_500_000,
      surplus_paise: 16_000_000,
      fits: false,
      shortfall_paise: 12_000_000,
    },
    timeline: [
      { year: 2028, age: 29, events: ["Wedding"] },
      { year: 2029, age: 30, events: ["Baby's first year"] },
    ],
  },
};

function withData(data: LifeOut) {
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(lifeKey, data);
  // The forecast panel sits on this page; an empty forecast keeps it from calling the network.
  qc.setQueryData(forecastKey, {
    basis: { source: "profile", months_used: 0, income_paise: 0, spend_paise: 0, invested_paise: 0, surplus_paise: 0, surplus_sd_paise: 0, entered_income_paise: 0, note: null },
    goals: [],
    path: [],
    summary: "",
    saved_profile: false,
  } satisfies Forecast);
  return render(
    <QueryClientProvider client={qc}>
      <AuthContext.Provider value={{ isAuthenticated: true, token: "t", login: () => {}, logout: () => {}, isLoading: false }}>
        <MemoryRouter>
          <LifePage />
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  );
}

describe("LifePage", () => {
  it("shows the goals ledger with each goal's status and the plan's fit", () => {
    withData(fixture);
    expect(screen.getByRole("heading", { name: "Wedding" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Child's education" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Retirement" })).toBeInTheDocument();
    expect(screen.getAllByText("Needs more").length).toBe(2);
    expect(screen.getByText("On track")).toBeInTheDocument();
    expect(screen.getByText(/Short by ₹1,20,000 a month/)).toBeInTheDocument();
    expect(screen.getByText("You ₹80,000 · Partner ₹55,000 a month", { exact: false })).toBeInTheDocument();
  });

  it("collapses the profile form once the details are saved", () => {
    withData(fixture);
    const details = screen.getByText("Your details").closest("details");
    expect(details).not.toBeNull();
    expect(details).not.toHaveAttribute("open");
  });
});
