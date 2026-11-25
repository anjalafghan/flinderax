import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import ReviewPage from "./ReviewPage";

const month = {
  month: "2026-10",
  income_paise: 7_000_000,
  spent_paise: 2_000_000,
  allowance_paise: 2_500_000,
  over_by_paise: 0,
  top_categories: [{ category: "food", amount_paise: 600_000 }],
  dates: null,
  savings_rate_bps: 4000,
  net_worth_change_paise: 900_000,
  sentences: ["You spent less than your allowance in October."],
};

const tax = {
  fy: "2026-27",
  from: "2026-04-01",
  to: "2027-03-31",
  sections: [
    {
      section: "80C",
      claimed_paise: 5_000_000,
      projected_paise: 0,
      limit_paise: 15_000_000,
      left_paise: 10_000_000,
      note: "",
    },
  ],
  entries: [],
  commitments: [],
  interest_income_paise: 0,
  gross_salary_paise: 0,
  regime: {
    old_tax_paise: 100_000,
    new_tax_paise: 80_000,
    better: "new",
    note: "The new regime is cheaper for you.",
    break_even_deductions_paise: null,
    deductions_needed_paise: null,
  },
  reminders: [],
  ltcg: {
    exemption_paise: 12_500_000,
    used_paise: 0,
    left_paise: 12_500_000,
    days_left_in_fy: 120,
    note: "",
  },
};

describe("ReviewPage", () => {
  test("shows the month in plain words, the tax picture and saved analyses", async () => {
    mockApi({
      "/api/review/month": month,
      "/api/review/tax": tax,
      "/api/analyses": {
        analyses: [
          {
            id: "a1",
            title: "Where the subscriptions went",
            body: "Three overlapping streaming plans.",
            source: "claude",
            created_on: "2026-11-02",
          },
        ],
      },
    });
    renderPage(<ReviewPage />);
    expect(await screen.findByText(/You spent less than your allowance/)).toBeTruthy();
    expect(await screen.findByText("A good month")).toBeTruthy();
    expect(await screen.findByText("Where the subscriptions went")).toBeTruthy();
    expect(await screen.findByText("Deductions so far")).toBeTruthy();
  });

  test("shows loading placeholders before anything answers", () => {
    mockApi({
      "/api/review/month": pending,
      "/api/review/tax": pending,
      "/api/analyses": pending,
    });
    renderPage(<ReviewPage />);
    expect(screen.getByText("Loading the month…")).toBeTruthy();
    expect(screen.getByLabelText("Loading analyses")).toBeTruthy();
  });

  test("says what is empty and what failed, panel by panel", async () => {
    mockApi({
      "/api/review/month": failWith(500, { error: "month closed badly" }),
      "/api/review/tax": failWith(500),
      "/api/analyses": { analyses: [] },
    });
    renderPage(<ReviewPage />);
    expect(await screen.findByText(/Nothing here yet/)).toBeTruthy();
    expect((await screen.findAllByRole("alert")).length).toBeGreaterThan(0);
  });
});
