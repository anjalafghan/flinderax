import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { forecastKey, type Forecast } from "@/services/forecast";
import { MemoryRouter } from "react-router-dom";

import LifePage from "@/pages/LifePage";
import { AuthContext } from "@/context/AuthContext";
import { lifeKey, type LifeOut } from "@/services/life";
import { lifelineKey, type Line, type LifelineOut } from "@/services/lifeline";

const line: Line = {
  rows: [
    { year: 2026, age: 28, income_paise: 50_000_000, spend_paise: 18_000_000, short_paise: 0, net_worth_paise: 40_000_000, net_worth_today_paise: 40_000_000, events: [] },
    { year: 2028, age: 30, income_paise: 300_000_000, spend_paise: 380_000_000, short_paise: 20_000_000, net_worth_paise: 0, net_worth_today_paise: 0, events: ["Wedding"] },
    { year: 2058, age: 60, income_paise: 0, spend_paise: 500_000_000, short_paise: 0, net_worth_paise: 90_000_000_00, net_worth_today_paise: 15_000_000_00, events: ["Retire at 60"] },
    { year: 2088, age: 90, income_paise: 0, spend_paise: 900_000_000, short_paise: 0, net_worth_paise: 30_000_000_00, net_worth_today_paise: 2_000_000_00, events: [] },
  ],
  marks: { free_at_age: 57, runs_out_at_age: null, retire_age: 60, end_age: 90, at_60_today_paise: 15_000_000_00, at_end_today_paise: 2_000_000_00, unfunded: ["Wedding in 2028"] },
  notes: ["Both incomes stop when you retire.", "Estimate: medical costs of ₹1,00,000 a year in today's money from age 60."],
};

const lifeline: LifelineOut = {
  saved_profile: true,
  line,
  choices: [{ id: "c1", label: "Retire at 55", in_plan: false, choice: { kind: "retire_at", age: 55 } }],
  working: { accounts_paise: 40_000_000, plan_saved_paise: 0, loans_paise: 0, wealth_paise: 40_000_000, expenses_paise: 9_000_000, income_paise: 25_000_000, note: "Starts from your account balances, less loans." },
};

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
  qc.setQueryData(lifelineKey, lifeline);
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

  it("leads with the life line: the marks, the years, what can't be paid, and the saved choices", () => {
    withData(fixture);
    expect(screen.getByRole("heading", { name: "Life line" })).toBeInTheDocument();
    expect(screen.getByText("Your plan, from 28 to 90")).toBeInTheDocument();
    expect(screen.getByText("At 57")).toBeInTheDocument();
    expect(screen.getByText("Past 90")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Net worth by age, in today's money" })).toBeInTheDocument();
    expect(screen.getByText("Savings can't fully pay for: Wedding in 2028.")).toBeInTheDocument();
    expect(screen.getByRole("list", { name: "The years ahead" })).toHaveTextContent("Retire at 60");
    expect(screen.getByRole("switch", { name: "Retire at 55: in my plan" })).not.toBeChecked();
    expect(screen.getByRole("button", { name: "See what changes" })).toBeInTheDocument();
    // An estimate is labelled as one.
    expect(screen.getByText("Estimate")).toBeInTheDocument();
  });

  it("collapses the profile form once the details are saved", () => {
    withData(fixture);
    const details = screen.getByText("Your details").closest("details");
    expect(details).not.toBeNull();
    expect(details).not.toHaveAttribute("open");
  });
});
