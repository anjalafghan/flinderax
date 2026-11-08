import { describe, expect, it } from "bun:test";
import { barPercent, budgetTone, dayLabel, isOverBudget, spentPercent, syncedLabel } from "@/utils/joy";

describe("spentPercent", () => {
  it("rounds spent as a share of the budget", () => {
    expect(spentPercent(730_000, 2_000_000)).toBe(37);
    expect(spentPercent(1_000_000, 1_000_000)).toBe(100);
    expect(spentPercent(0, 2_000_000)).toBe(0);
  });

  it("is null without a usable budget", () => {
    expect(spentPercent(500, null)).toBeNull();
    expect(spentPercent(500, 0)).toBeNull();
  });
});

describe("barPercent", () => {
  it("caps the fill at 100 and never goes negative", () => {
    expect(barPercent(3_000, 1_000)).toBe(100);
    expect(barPercent(-50, 1_000)).toBe(0);
    expect(barPercent(500, null)).toBe(0);
    expect(barPercent(500, 1_000)).toBe(50);
  });
});

describe("isOverBudget", () => {
  it("is true only when spent is past a real budget", () => {
    expect(isOverBudget(2_000_001, 2_000_000)).toBe(true);
    expect(isOverBudget(2_000_000, 2_000_000)).toBe(false);
    expect(isOverBudget(5_000, null)).toBe(false);
    expect(isOverBudget(5_000, 0)).toBe(false);
  });
});

describe("budgetTone", () => {
  const budget = 2_000_000;
  it("is none without a budget", () => {
    expect(budgetTone(9_999_999, null)).toBe("none");
  });
  it("is ok up to 80%, warn above 80%, over past 100%", () => {
    expect(budgetTone(1_600_000, budget)).toBe("ok");
    expect(budgetTone(1_600_100, budget)).toBe("warn");
    expect(budgetTone(2_000_000, budget)).toBe("warn");
    expect(budgetTone(2_000_100, budget)).toBe("over");
  });
});

describe("dayLabel", () => {
  it("formats plain dates and passes anything else through", () => {
    expect(dayLabel("2026-11-06")).toBe("6 Nov");
    expect(dayLabel("sometime")).toBe("sometime");
  });
});

describe("syncedLabel", () => {
  const now = new Date("2026-11-08T12:00:00Z");
  it("says today, yesterday, or the date", () => {
    expect(syncedLabel("2026-11-08T08:00:00Z", now)).toBe("today");
    expect(syncedLabel("2026-11-07T08:00:00Z", now)).toBe("yesterday");
    expect(syncedLabel("2026-11-03T08:00:00Z", now)).toBe("3 Nov");
  });
  it("handles a missing sync time", () => {
    expect(syncedLabel(null, now)).toBe("an earlier sync");
  });
});
