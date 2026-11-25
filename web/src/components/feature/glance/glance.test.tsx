import { afterEach, describe, expect, test } from "bun:test";
import { cleanup, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";

import type { HistoryView } from "@/joy/kept/types";
import type { Road } from "@/services/road";
import type { YearView } from "@/services/year";
import {
  byYear,
  chartMonths,
  datesStats,
  getaways,
  jarBudget,
  niceTicks,
  outflowMonths,
  pct,
  shortINR,
} from "./glance";
import { RoadAhead } from "./RoadAhead";

afterEach(cleanup);

const month = (m: string, spend: number, invested: number) => ({
  month: m,
  income_paise: 0,
  spend_paise: spend,
  invested_paise: invested,
  transfers_paise: 0,
  refunds_paise: 0,
  net_paise: 0,
});

const kept = {
  months: [
    {
      items: [
        {
          id: 2,
          title: "KOKO",
          day: "2026-11-17",
          rating: 5,
          cost_actual_raw: "4500",
        },
        {
          id: 3,
          title: "Blue Tokai",
          day: "2026-11-10",
          rating: null,
          cost_actual_raw: "500.50",
        },
      ],
    },
    {
      items: [
        {
          id: 1,
          title: "No-plan date",
          day: "2026-10-17",
          rating: 4,
          cost_actual_raw: "",
        },
      ],
    },
  ],
  skipped: [],
  budget: { spent: "₹13,037", budget: "₹18,000", budget_raw: "18000" },
  savings: {
    choices: [
      { value: "1", label: "Alibaug · ₹18,450", selected: false },
      {
        value: "2",
        label: "Udaipur · Lake Pichola · ₹50,750",
        selected: false,
      },
      { value: "3", label: "No price", selected: false },
    ],
  },
} as unknown as HistoryView;

describe("glance helpers", () => {
  test("shortINR reads in thousands, lakhs and crores", () => {
    expect(shortINR(0)).toBe("₹0");
    expect(shortINR(1_234_500)).toBe("₹12k");
    expect(shortINR(15_000_000)).toBe("₹1.5L");
    expect(shortINR(10_000_000)).toBe("₹1L");
    expect(shortINR(2_500_000_000)).toBe("₹2.5Cr");
  });

  test("chartMonths starts at the first month with money moving", () => {
    const ms = [
      month("2026-01", 0, 0),
      month("2026-02", 0, 0),
      month("2026-03", 100, 0),
      month("2026-04", 0, 200),
    ];
    expect(chartMonths(ms).map((m) => m.month)).toEqual(["2026-03", "2026-04"]);
    expect(chartMonths([month("2026-01", 0, 0)])).toEqual([]);
  });

  test("outflowMonths adds fixed payments and one-offs going out", () => {
    const view = {
      months: [
        {
          month: "2026-12",
          income_paise: 0,
          fixed_paise: 1000,
          lumpy_out_paise: 500,
          lumpy_in_paise: 900,
          net_paise: 0,
          closing_paise: 0,
          short: false,
          lumpy: [
            {
              on: "2026-12-01",
              label: "Mobile",
              amount_paise: 500,
              direction: "out",
              kind: "other",
            },
            {
              on: "2026-12-02",
              label: "Bonus",
              amount_paise: 900,
              direction: "in",
              kind: "other",
            },
          ],
        },
      ],
      first_short_month: null,
      lowest_paise: 0,
      total_lumpy_out_paise: 500,
    } as YearView;
    expect(outflowMonths(view)).toEqual([
      {
        month: "2026-12",
        fixed_paise: 1000,
        lumpy_paise: 500,
        total_paise: 1500,
        items: ["Mobile"],
      },
    ]);
  });

  test("datesStats counts kept dates, averages the rated ones and sums real costs in paise", () => {
    expect(datesStats(kept)).toEqual({
      kept: 3,
      rated: 2,
      average: 4.5,
      spent_paise: 450_000 + 50_050,
    });
  });

  test("getaways splits the price off the last separator and sorts biggest first", () => {
    expect(getaways(kept)).toEqual([
      { id: "2", title: "Udaipur · Lake Pichola", cost_paise: 5_075_000 },
      { id: "1", title: "Alibaug", cost_paise: 1_845_000 },
    ]);
    expect(jarBudget(kept)).toEqual({
      spent_paise: 1_303_700,
      budget_paise: 1_800_000,
    });
  });

  test("byYear groups in date order; pct and ticks stay sane", () => {
    const ms = [
      { on: "2029-02-14" },
      { on: "2027-03-01" },
      { on: "2027-01-01" },
    ] as Road["milestones"];
    expect(byYear(ms).map((g) => [g.year, g.items.map((i) => i.on)])).toEqual([
      ["2027", ["2027-01-01", "2027-03-01"]],
      ["2029", ["2029-02-14"]],
    ]);
    expect(pct(0, 10)).toBe(0);
    expect(pct(1, 1000)).toBe(2);
    expect(pct(20, 10)).toBe(100);
    const t = niceTicks(12_437_401);
    expect(t[0]).toBe(0);
    expect(t[t.length - 1]).toBeGreaterThanOrEqual(12_437_401);
    expect(t.length).toBeLessThanOrEqual(5);
  });
});

describe("RoadAhead", () => {
  test("lists milestones by year with their cost and monthly need, and the prompts", () => {
    const road: Road = {
      milestones: [
        {
          on: "2029-02-14",
          title: "Wedding",
          area: "family",
          source: "life",
          cost_paise: 83_215_100,
          saved_paise: 0,
          per_month_paise: 1_335_500,
          months_away: 27,
          link: "/life",
        },
        {
          on: "2027-09-01",
          title: "Emergency fund",
          area: "money",
          source: "goal",
          cost_paise: 10_000_000,
          saved_paise: 0,
          per_month_paise: 909_100,
          months_away: 10,
          link: "/plan",
        },
      ],
      prompts: [
        {
          title: "Pick a year for a child",
          body: "A birth year lets joy plan.",
          link: "/life",
        },
      ],
      pace_all_paise: 2_244_600,
      biggest: null,
    };
    render(
      <MemoryRouter>
        <RoadAhead road={road} />
      </MemoryRouter>,
    );
    const years = screen.getAllByText(/^20\d\d$/).map((e) => e.textContent);
    expect(years).toEqual(["2027", "2029"]);
    expect(
      screen.getByRole("link", { name: /Wedding/ }).getAttribute("href"),
    ).toBe("/life");
    expect(screen.getByText(/₹13,355 a month from now/)).toBeTruthy();
    expect(
      screen.getByRole("link", { name: /Pick a year for a child/ }),
    ).toBeTruthy();
  });
});
