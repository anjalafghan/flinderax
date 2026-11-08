import { describe, expect, it } from "bun:test";
import {
  daysLeftLabel,
  deductionError,
  fyForDate,
  fyLabel,
  lastCompleteMonth,
  nextReminder,
  ratioPercent,
  regimeHeadline,
  sectionLabel,
  signedINR,
  stackedWidths,
  stepMonth,
  TAX_SECTION_IDS,
} from "@/utils/review";

describe("month helpers", () => {
  it("opens on the last complete calendar month", () => {
    expect(lastCompleteMonth(new Date(2026, 10, 8))).toBe("2026-10");
    expect(lastCompleteMonth(new Date(2026, 0, 15))).toBe("2025-12");
  });

  it("steps months and stops at the last allowed month", () => {
    expect(stepMonth("2026-09", 1, "2026-10")).toBe("2026-10");
    expect(stepMonth("2026-10", 1, "2026-10")).toBeNull();
    expect(stepMonth("2026-01", -1, "2026-10")).toBe("2025-12");
  });
});

describe("financial year", () => {
  it("runs April to March", () => {
    expect(fyForDate(new Date(2026, 2, 31))).toBe("2025-26");
    expect(fyForDate(new Date(2026, 3, 1))).toBe("2026-27");
    expect(fyForDate(new Date(2099, 11, 1))).toBe("2099-00");
  });

  it("labels a year", () => {
    expect(fyLabel("2026-27")).toBe("FY 2026-27");
  });
});

describe("money display", () => {
  it("signs changes with a unicode minus", () => {
    expect(signedINR(120000)).toBe("+₹1,200");
    expect(signedINR(-30000)).toBe("−₹300");
    expect(signedINR(0)).toBe("₹0");
  });

  it("turns a part of a whole into a clamped percent", () => {
    expect(ratioPercent(50, 200)).toBe(25);
    expect(ratioPercent(300, 200)).toBe(100);
    expect(ratioPercent(5, 0)).toBe(100);
    expect(ratioPercent(0, 0)).toBe(0);
  });
});

describe("tax helpers", () => {
  it("stacks claimed under projected without exceeding the limit", () => {
    expect(stackedWidths(30, 20, 100)).toEqual({ claimed: 30, projected: 20 });
    expect(stackedWidths(90, 50, 100)).toEqual({ claimed: 90, projected: 10 });
    expect(stackedWidths(10, 10, null)).toEqual({ claimed: 0, projected: 0 });
  });

  it("headlines the cheaper regime", () => {
    expect(
      regimeHeadline({ old_tax_paise: 100000, new_tax_paise: 50000, better: "new", note: "", break_even_deductions_paise: null, deductions_needed_paise: null }),
    ).toBe("New regime saves you ₹500");
    expect(
      regimeHeadline({ old_tax_paise: 40000, new_tax_paise: 90000, better: "old", note: "", break_even_deductions_paise: null, deductions_needed_paise: null }),
    ).toBe("Old regime saves you ₹500");
    expect(
      regimeHeadline({ old_tax_paise: 7000, new_tax_paise: 7000, better: "old", note: "", break_even_deductions_paise: null, deductions_needed_paise: null }),
    ).toBe("Both regimes come to the same tax");
  });

  it("reads days left and picks the next reminder that has not passed", () => {
    expect(daysLeftLabel(-2)).toBe("Passed");
    expect(daysLeftLabel(0)).toBe("Today");
    expect(daysLeftLabel(1)).toBe("Tomorrow");
    expect(daysLeftLabel(12)).toBe("12 days left");

    const reminders = [
      { date: "2026-10-01", text: "old", days_left: -38 },
      { date: "2026-12-31", text: "later", days_left: 53 },
      { date: "2026-11-15", text: "soon", days_left: 7 },
    ];
    expect(nextReminder(reminders)?.text).toBe("soon");
    expect(nextReminder([{ date: "2026-10-01", text: "old", days_left: -1 }])).toBeNull();
  });

  it("names every section and offers them in order", () => {
    expect(sectionLabel("80CCD1B")).toBe("80CCD(1B) NPS");
    expect(sectionLabel("24B")).toBe("Home loan interest");
    expect(TAX_SECTION_IDS).toEqual(["80C", "80CCD1B", "80D", "HRA", "24B", "other"]);
  });

  it("explains why a deduction cannot be saved", () => {
    expect(deductionError("", "100")).not.toBeNull();
    expect(deductionError("PPF", "")).not.toBeNull();
    expect(deductionError("PPF", "0")).not.toBeNull();
    expect(deductionError("PPF", "1500.50")).toBeNull();
  });
});
