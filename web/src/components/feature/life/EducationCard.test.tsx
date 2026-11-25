import { afterEach, describe, expect, test } from "bun:test";
import { cleanup, render, screen } from "@testing-library/react";

import type { EducationOut, EduScenario } from "@/services/education";
import { EducationSummary } from "./EducationCard";

afterEach(cleanup);

const scenario = (
  label: string,
  top_up: number,
  infl: number,
  ret: number,
): EduScenario => ({
  label,
  inflation_bps: infl,
  return_bps: ret,
  future_costs_paise: [0],
  total_future_cost_paise: 0,
  monthly_top_up_paise: top_up,
  months_to_save: 100,
});

const data = (patch: Partial<EducationOut["summary"]> = {}): EducationOut => ({
  saved: true,
  plan: {
    child_birth_year: 2026,
    stages: [],
    inflation_bps: 1000,
    return_bps: 1150,
    corpus_paise: 0,
  },
  summary: {
    child_birth_year: 2026,
    child_age: 0,
    corpus_paise: 0,
    stages: [
      {
        name: "School",
        start_age: 5,
        year: 2031,
        months_away: 60,
        today_cost_paise: 3_000_000,
        future_cost_paise: 4_900_000,
        started: false,
      },
      {
        name: "College",
        start_age: 18,
        year: 2044,
        months_away: 216,
        today_cost_paise: 15_000_000,
        future_cost_paise: 82_000_000,
        started: false,
      },
    ],
    low: scenario("Low", 2_500_000, 800, 1350),
    base: scenario("Base", 3_500_000, 1000, 1150),
    high: scenario("High", 5_000_000, 1200, 950),
    nothing_left: false,
    notes: [
      "Fee inflation and the return on savings are guesses, not forecasts.",
      "This is planning arithmetic, not investment advice.",
    ],
    ...patch,
  },
  lifeline: {
    consequences: {
      lines: ["No change you would notice."],
      at_60_change_paise: null,
      at_end_change_paise: 0,
      better: false,
    },
    without_at_end_today_paise: 1,
    with_at_end_today_paise: 1,
    note: "Compared with paying out of savings.",
  },
});

describe("EducationSummary", () => {
  test("shows each stage's cost then, the top-up and the whole range", () => {
    render(<EducationSummary data={data()} />);
    expect(screen.getByText("School")).toBeTruthy();
    expect(screen.getByText("College")).toBeTruthy();
    expect(screen.getByText("₹8,20,000")).toBeTruthy();
    expect(screen.getAllByText("₹35,000").length).toBeGreaterThan(0);
    const range = screen.getByLabelText("Range of monthly top-up");
    expect(range.textContent).toContain("Low");
    expect(range.textContent).toContain("₹25,000");
    expect(range.textContent).toContain("High");
    expect(range.textContent).toContain("₹50,000");
  });

  test("says the assumptions are guesses and that this is not investment advice", () => {
    render(<EducationSummary data={data()} />);
    const note = screen.getByRole("note");
    expect(note.textContent).toContain("guesses");
    expect(note.textContent).toContain("not investment advice");
  });

  test("shows what it does to the life line", () => {
    render(<EducationSummary data={data()} />);
    expect(screen.getByText("What it does to your life line")).toBeTruthy();
    expect(screen.getByText("No change you would notice.")).toBeTruthy();
  });

  test("when every stage has begun there is no top-up to show", () => {
    const d = data({
      nothing_left: true,
      stages: [
        {
          name: "School",
          start_age: 5,
          year: 2020,
          months_away: -70,
          today_cost_paise: 3_000_000,
          future_cost_paise: 3_000_000,
          started: true,
        },
      ],
    });
    render(<EducationSummary data={d} />);
    expect(screen.getByText(/nothing left to save for/)).toBeTruthy();
    expect(screen.queryByLabelText("Range of monthly top-up")).toBeNull();
    expect(screen.getByText(/Has begun/)).toBeTruthy();
  });
});
