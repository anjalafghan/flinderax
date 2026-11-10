import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";

import { LifeLine } from "@/components/feature/life/LifeLine";
import api from "@/services/api";
import { describeChoice, type Line, type LifelineOut, type TryOut } from "@/services/lifeline";
import { renderWithProviders, respond } from "./health/fixture";

const line = (free: number | null, end: number): Line => ({
  rows: [
    { year: 2026, age: 29, income_paise: 30_000_000, spend_paise: 14_000_000, short_paise: 0, net_worth_paise: 100_000_000, net_worth_today_paise: 100_000_000, events: [] },
    { year: 2057, age: 60, income_paise: 0, spend_paise: 500_000_000, short_paise: 0, net_worth_paise: 90_000_000_00, net_worth_today_paise: end * 2, events: ["Retire at 60"] },
    { year: 2087, age: 90, income_paise: 0, spend_paise: 900_000_000, short_paise: 0, net_worth_paise: 30_000_000_00, net_worth_today_paise: end, events: [] },
  ],
  marks: { free_at_age: free, runs_out_at_age: null, retire_age: 60, end_age: 90, at_60_today_paise: end * 2, at_end_today_paise: end, unfunded: [] },
  notes: ["Both incomes stop when you retire."],
});

const overview: LifelineOut = {
  saved_profile: true,
  line: line(58, 2_000_000_00),
  choices: [],
  working: { accounts_paise: 100_000_000, plan_saved_paise: 0, loans_paise: 0, wealth_paise: 100_000_000, expenses_paise: 7_000_000, income_paise: 15_000_000, note: "Starts from your account balances, less loans." },
};

const tried: TryOut = {
  base: overview.line,
  with: line(54, 6_000_000_00),
  consequences: { lines: ["Work becomes optional 4 years earlier, at 54.", "₹40,00,000 more at 90, in today's money."], at_60_change_paise: 8_000_000_00, at_end_change_paise: 4_000_000_00, better: true },
};

describe("LifeLine", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("tries a job switch against the plan, shows what changes, and keeps it in the plan", async () => {
    spyOn(api, "get").mockImplementation(() => respond(overview));
    const post = spyOn(api, "post").mockImplementation((url: string) => respond(url.endsWith("/try") ? tried : { id: "c1" }));
    renderWithProviders(<LifeLine />);

    expect(await screen.findByText("At 58")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("New take-home a month (₹)"), { target: { value: "220000" } });
    fireEvent.change(screen.getByLabelText("Start month"), { target: { value: "2027-04" } });
    fireEvent.click(screen.getByRole("button", { name: "See what changes" }));

    expect(await screen.findByText("Work becomes optional 4 years earlier, at 54.")).toBeInTheDocument();
    const choice = { kind: "job_switch", month: "2027-04", take_home_paise: 22_000_000, gap_months: 0, raise_bps: null };
    expect(post).toHaveBeenCalledWith("/api/plan/lifeline/try", { base: [], with: [choice] });
    // The marks move to the tried line and say what they are now.
    expect(screen.getByText("At 54")).toBeInTheDocument();
    expect(screen.getByText("Now: At 58")).toBeInTheDocument();
    expect(screen.getByText("Leaves you better off")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Keep in my plan" }));
    await waitFor(() =>
      expect(post).toHaveBeenCalledWith("/api/plan/lifeline/choices", {
        label: "New job at ₹2,20,000 a month from Apr 2027",
        choice,
        in_plan: true,
      }),
    );
  });

  it("names a field that is missing instead of calling the server", async () => {
    spyOn(api, "get").mockImplementation(() => respond(overview));
    const post = spyOn(api, "post").mockImplementation(() => respond(tried));
    renderWithProviders(<LifeLine />);
    await screen.findByText("At 58");
    fireEvent.click(screen.getByRole("button", { name: "See what changes" }));
    expect(screen.getByRole("alert")).toHaveTextContent("New take-home a month (₹): fill this in.");
    expect(post).not.toHaveBeenCalled();
  });

  it("describes each kind of choice in plain words", () => {
    expect(describeChoice({ kind: "retire_at", age: 55 })).toBe("Retire at 55");
    expect(describeChoice({ kind: "house", year: 2030, price_paise: 8_000_000_00, down_bps: 2000, loan_bps: 850, tenure_years: 20, rent_saved_paise: 0 })).toBe("Buy a ₹80,00,000 home in 2030");
    expect(describeChoice({ kind: "big_spend", name: "Car", year: 2030, amount_paise: 1_200_000_00, every_years: 10 })).toBe("Car: ₹12,00,000 in 2030, then every 10 years");
    expect(describeChoice({ kind: "break", start: "2028-01", months: 6 })).toBe("6 months off work from Jan 2028");
    expect(describeChoice({ kind: "stay_active", medical_cut_bps: null })).toBe("Stay active (medical costs 30% lower after 60)");
  });
});
