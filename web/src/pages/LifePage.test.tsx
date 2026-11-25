import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import LifePage from "./LifePage";

const profile = {
  my_age: 32,
  partner_age: 30,
  my_take_home_paise: 9_000_000,
  partner_take_home_paise: 6_000_000,
  monthly_expenses_paise: 5_000_000,
  wedding_date: "2029-02-14",
  wedding_budget_paise: 83_215_100,
  wedding_saved_paise: 0,
  child_birth_year: 2031,
  child_education_today_paise: 0,
  child_saved_paise: 0,
  retirement_age: 60,
  retirement_saved_paise: 0,
  emergency_saved_paise: 0,
  term_cover_paise: 0,
  health_cover_paise: 0,
  loans_paise: 0,
};

const goal = {
  key: "wedding",
  label: "Wedding",
  when: "2029-02-14",
  years_away: 2,
  today_cost_paise: 83_215_100,
  future_cost_paise: 90_000_000,
  saved_paise: 0,
  gap_paise: 90_000_000,
  monthly_sip_paise: 3_000_000,
  step_up_sip_paise: 2_800_000,
  mix: "debt",
  mix_note: "Short horizon, so mostly debt funds.",
  my_share_paise: 1_700_000,
  partner_share_paise: 1_100_000,
  status: "needs_more",
  note: "",
};

const life = (goals: unknown[]) => ({
  saved: true,
  profile,
  plan: {
    assumptions: {
      general_inflation_bps: 600,
      education_inflation_bps: 900,
      medical_inflation_bps: 800,
      wedding_inflation_bps: 700,
      equity_return_bps: 1200,
      hybrid_return_bps: 900,
      debt_return_bps: 700,
      step_up_bps: 1000,
      swr_bps: 350,
    },
    goals,
    insurance: {
      term_needed_paise: 100_000_000,
      term_have_paise: 0,
      term_gap_paise: 100_000_000,
      health_needed_paise: 10_000_000,
      health_have_paise: 0,
      health_gap_paise: 10_000_000,
      notes: [],
    },
    monthly: {
      total_sip_paise: 3_000_000,
      total_step_up_sip_paise: 2_800_000,
      surplus_paise: 4_000_000,
      fits: true,
      shortfall_paise: 0,
    },
    timeline: [],
  },
});

describe("LifePage", () => {
  test("shows the plan, the goals ledger and the insurance gaps", async () => {
    mockApi({ "/api/plan/life": life([goal]) });
    renderPage(<LifePage />);
    expect(screen.getByRole("heading", { name: "Life line" })).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Your plan" })).toBeTruthy();
    expect(await screen.findByRole("heading", { name: "Goals" })).toBeTruthy();
    expect(screen.getAllByText("Wedding").length).toBeGreaterThan(0);
    expect(screen.queryByText(/No goals yet/)).toBeNull();
  });

  test("says there are no goals yet when the plan has none", async () => {
    mockApi({ "/api/plan/life": life([]) });
    renderPage(<LifePage />);
    expect(await screen.findByText(/No goals yet/)).toBeTruthy();
  });

  test("shows a skeleton while the plan loads", () => {
    mockApi({ "/api/plan/life": pending });
    renderPage(<LifePage />);
    expect(screen.getByLabelText("Loading your life plan")).toBeTruthy();
  });

  test("offers a retry when the plan fails", async () => {
    mockApi({ "/api/plan/life": failWith(500) });
    renderPage(<LifePage />);
    expect(await screen.findByText(/Couldn't load your life plan/)).toBeTruthy();
    expect(screen.queryByRole("heading", { name: "Goals" })).toBeNull();
  });
});
