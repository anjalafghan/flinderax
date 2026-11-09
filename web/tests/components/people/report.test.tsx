import { afterEach, describe, expect, it, mock } from "bun:test";
import { cleanup, render, screen } from "@testing-library/react";

import { PeopleReportSection } from "@/components/feature/people/PeopleReportSection";
import type { PeopleReport } from "@/services/checkin";

// Synthetic data only. The section must never show a person's name.

const report: PeopleReport = {
  catch_ups: 12,
  overdue_people: 3,
  gifts_given_paise: 480_000,
  gifts_budgeted_paise: 600_000,
  next_quarter_gifts_paise: 1_400_000,
  next_quarter_occasions: 5,
  visit_cost_paise: 92_000,
  sent_to_people_paise: 310_000,
  prev_sent_to_people_paise: 250_000,
  wins: ["You caught up 12 times this quarter."],
  watch: ["3 people are past their catch-up date."],
};

describe("PeopleReportSection", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("shows catch-ups, gifts, the next quarter, visits and money sent, with the wins and watch", () => {
    render(<PeopleReportSection report={report} />);
    expect(screen.getByRole("heading", { level: 3, name: "People" })).toBeInTheDocument();

    expect(screen.getByText("12")).toBeInTheDocument();
    expect(screen.getByText("3 people past catch-up date")).toBeInTheDocument();
    expect(screen.getByText("₹4,800")).toBeInTheDocument();
    expect(screen.getByText("of ₹6,000 budgeted")).toBeInTheDocument();
    expect(screen.getByText("₹14,000")).toBeInTheDocument();
    expect(screen.getByText("across 5 occasions")).toBeInTheDocument();
    expect(screen.getByText("₹920")).toBeInTheDocument();
    expect(screen.getByText("₹3,100")).toBeInTheDocument();
    expect(screen.getByText("Before: ₹2,500")).toBeInTheDocument();
    expect(screen.getByText("You caught up 12 times this quarter.")).toBeInTheDocument();
    expect(screen.getByText("3 people are past their catch-up date.")).toBeInTheDocument();
  });

  it("uses singular wording and a no-earlier-spend note when there is nothing to compare", () => {
    render(
      <PeopleReportSection
        report={{ ...report, overdue_people: 0, next_quarter_occasions: 1, prev_sent_to_people_paise: null, wins: [], watch: [] }}
      />,
    );
    expect(screen.getByText("Nobody overdue")).toBeInTheDocument();
    expect(screen.getByText("across 1 occasion")).toBeInTheDocument();
    expect(screen.getByText("No earlier spend")).toBeInTheDocument();
    expect(screen.getByText("Nothing standing out yet.")).toBeInTheDocument();
    expect(screen.getByText("Nothing to flag this time.")).toBeInTheDocument();
  });
});
