import { describe, expect, it } from "bun:test";
import { barPercent, budgetTone, calendarDaysAgo, dayLabel, isOverBudget, spentPercent, syncedLabel } from "@/utils/joy";

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

describe("calendarDaysAgo", () => {
  // Local-time fixtures, so the expectations hold in any timezone the tests run in.
  const now = new Date(2026, 10, 8, 9, 0);
  it("counts calendar days, not elapsed hours", () => {
    expect(calendarDaysAgo(new Date(2026, 10, 8, 0, 5).toISOString(), now)).toBe(0);
    // 23:55 yesterday is only ~9 hours before now, but it is yesterday
    expect(calendarDaysAgo(new Date(2026, 10, 7, 23, 55).toISOString(), now)).toBe(1);
    expect(calendarDaysAgo(new Date(2026, 10, 1, 12, 0).toISOString(), now)).toBe(7);
  });
  it("reads a zone-less SQLite timestamp as UTC", () => {
    const utc = new Date(Date.UTC(2026, 10, 7, 20, 0));
    const sqlite = utc.toISOString().slice(0, 19).replace("T", " ");
    expect(calendarDaysAgo(sqlite, now)).toBe(calendarDaysAgo(utc.toISOString(), now));
  });
  it("is null for an unreadable timestamp and never negative for the future", () => {
    expect(calendarDaysAgo("not a date", now)).toBeNull();
    expect(calendarDaysAgo(new Date(2026, 10, 9, 9, 0).toISOString(), now)).toBe(0);
  });
});

describe("syncedLabel", () => {
  const now = new Date(2026, 10, 8, 12, 0);
  it("says today, yesterday, or the date", () => {
    expect(syncedLabel(new Date(2026, 10, 8, 8, 0).toISOString(), now)).toBe("today");
    expect(syncedLabel(new Date(2026, 10, 7, 8, 0).toISOString(), now)).toBe("yesterday");
    expect(syncedLabel(new Date(2026, 10, 3, 8, 0).toISOString(), now)).toBe("3 Nov");
  });
  it("uses the local calendar date, not the UTC one", () => {
    // 00:30 local on the 6th is still the 5th in UTC for timezones east of UTC
    const early = new Date(2026, 10, 6, 0, 30);
    expect(syncedLabel(early.toISOString(), new Date(2026, 10, 8, 12, 0))).toBe("6 Nov");
    expect(syncedLabel(early.toISOString(), new Date(2026, 10, 7, 12, 0))).toBe("yesterday");
  });
  it("handles a missing sync time", () => {
    expect(syncedLabel(null, now)).toBe("an earlier sync");
  });
});
