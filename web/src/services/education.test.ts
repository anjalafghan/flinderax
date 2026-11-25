import { describe, expect, test } from "bun:test";

import {
  bpsToText,
  buildEduPlan,
  percentToBps,
  stageWhen,
  toEduDraft,
  type EduDraft,
  type EduPlan,
} from "./education";

const plan: EduPlan = {
  child_birth_year: 2026,
  stages: [
    { name: "School", start_age: 5, today_cost_paise: 30_000_000 },
    { name: "College", start_age: 18, today_cost_paise: 150_000_050 },
  ],
  inflation_bps: 1000,
  return_bps: 1150,
  corpus_paise: 0,
};

const draft = (patch: Partial<EduDraft> = {}): EduDraft => ({
  ...toEduDraft(plan),
  ...patch,
});

describe("education form", () => {
  test("a plan survives a trip through the form without float drift", () => {
    expect(buildEduPlan(toEduDraft(plan))).toEqual(plan);
    expect(toEduDraft(plan).return_pct).toBe("11.5");
    expect(bpsToText(1150)).toBe("11.5");
  });

  test("rupees become paise through toPaise and a blank corpus is zero", () => {
    const out = buildEduPlan(
      draft({
        corpus: "2,50,000.5",
        stages: [{ name: "College", start_age: "18", cost: "₹1,50,000" }],
      }),
    );
    expect(out.corpus_paise).toBe(25_000_050);
    expect(out.stages[0]?.today_cost_paise).toBe(15_000_000);
    expect(buildEduPlan(draft({ corpus: "  " })).corpus_paise).toBe(0);
  });

  test("percentages are read from digits into basis points", () => {
    expect(percentToBps("x", "10", 20)).toBe(1000);
    expect(percentToBps("x", "11.5", 30)).toBe(1150);
    expect(percentToBps("x", "0.05", 30)).toBe(5);
    expect(() => percentToBps("Return", "31", 30)).toThrow("between 0 and 30");
    expect(() => percentToBps("Return", "abc", 30)).toThrow("percentage");
    expect(() => percentToBps("Return", "-1", 30)).toThrow("percentage");
  });

  test("bad input says what is wrong", () => {
    expect(() => buildEduPlan(draft({ stages: [] }))).toThrow(
      "at least one stage",
    );
    expect(() =>
      buildEduPlan(
        draft({ stages: [{ name: " ", start_age: "5", cost: "1" }] }),
      ),
    ).toThrow("needs a name");
    expect(() =>
      buildEduPlan(
        draft({ stages: [{ name: "A", start_age: "5", cost: "lots" }] }),
      ),
    ).toThrow("plain rupee amount");
    expect(() =>
      buildEduPlan(
        draft({ stages: [{ name: "A", start_age: "50", cost: "1" }] }),
      ),
    ).toThrow("starting age");
    expect(() => buildEduPlan(draft({ corpus: "-5" }))).toThrow(
      "Already set aside",
    );
    expect(() => buildEduPlan(draft({ child_birth_year: "20x6" }))).toThrow(
      "Birth year",
    );
  });

  test("a stage says when it is due or that it has begun", () => {
    expect(stageWhen({ year: 2044, started: false })).toBe("Due 2044");
    expect(stageWhen({ year: 2020, started: true })).toBe("Has begun");
  });
});
