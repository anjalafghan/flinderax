import { describe, expect, it } from "bun:test";
import { render, screen, within } from "@testing-library/react";

import { DietReportSection } from "@/components/feature/health/DietReportSection";
import type { DietReport } from "@/services/checkin";

const report: DietReport = {
  eating_out_paise: 2_200_000,
  groceries_paise: 3_000_000,
  prev_eating_out_paise: 2_000_000,
  prev_groceries_paise: null,
  eating_out_share_bps: 4400,
  delivery_per_week_x100: 350,
  weeks_logged: 6,
  weeks_in_period: 13,
  habits: [{ label: "Home-cooked dinner", target_days: 5, hit_bps: 6900, weeks_on_target: 9, weeks_logged: 13 }],
  weight_x10: 705,
  weight_change_x10: -12,
  checkup_due_on: "2027-03-12",
  wins: ["Eating out fell to 31% of food, from 44%."],
  watch: ["Ordered in 3.5 times a week, up from 2."],
};

describe("DietReportSection", () => {
  it("shows the food figures with the change from the quarter before", () => {
    render(<DietReportSection report={report} />);
    expect(screen.getByRole("heading", { name: "Food and health" })).toBeInTheDocument();
    expect(screen.getByText("₹22,000")).toBeInTheDocument();
    expect(screen.getByText("+10%")).toBeInTheDocument();
    expect(screen.getByText("No earlier spend")).toBeInTheDocument();
    expect(screen.getByText("44%")).toBeInTheDocument();
    expect(screen.getByText("3.5")).toBeInTheDocument();
    expect(screen.getByText("6 of 13 weeks logged")).toBeInTheDocument();
  });

  it("shows each habit's rate and the weight change, and the check-up date", () => {
    render(<DietReportSection report={report} />);
    expect(screen.getByText("Home-cooked dinner")).toBeInTheDocument();
    expect(screen.getByText("On target in 9 of 13 weeks, 69% of days")).toBeInTheDocument();
    expect(screen.getByText("Weight: −1.2 kg")).toBeInTheDocument();
    expect(screen.getByText("Yearly check-up due 12 Mar.")).toBeInTheDocument();
  });

  it("lists wins and watch without any weight figure in the text", () => {
    render(<DietReportSection report={report} />);
    expect(screen.getByText("Eating out fell to 31% of food, from 44%.")).toBeInTheDocument();
    expect(screen.getByText("Ordered in 3.5 times a week, up from 2.")).toBeInTheDocument();
  });

  it("leaves the weight line out when there is no change to show", () => {
    render(<DietReportSection report={{ ...report, weight_change_x10: null, weight_x10: 705 }} />);
    expect(screen.queryByText(/Weight:/)).toBeNull();
  });

  it("uses plain fallbacks for an empty side of wins and watch, and a missing share", () => {
    render(
      <DietReportSection
        report={{ ...report, eating_out_share_bps: null, wins: [], habits: [], checkup_due_on: null }}
      />,
    );
    const section = screen.getByRole("heading", { name: "Food and health" }).parentElement as HTMLElement;
    expect(within(section).getByText("Nothing standing out yet.")).toBeInTheDocument();
    expect(within(section).getByText("Ordered in 3.5 times a week, up from 2.")).toBeInTheDocument();
    expect(within(section).getByText("—")).toBeInTheDocument();
    expect(within(section).queryByText(/check-up due/)).toBeNull();
  });

  it("leaves wins and watch out entirely when both are empty", () => {
    render(<DietReportSection report={{ ...report, wins: [], watch: [] }} />);
    expect(screen.queryByText("Nothing standing out yet.")).toBeNull();
    expect(screen.queryByText("Nothing to flag this time.")).toBeNull();
  });
});
