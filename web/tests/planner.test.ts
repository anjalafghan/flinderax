import { describe, expect, it } from "bun:test";
import type { Bill } from "@/services/types";
import {
  bpsToPercentInput,
  coveredLine,
  daysBetween,
  groupBillsByWeek,
  localIsoDate,
  monthsCoveredLabel,
  nextBills,
  percentToBps,
  progressPercent,
} from "@/utils/planner";

const fmt = (p: number) => `₹${p / 100}`;
const bill = (date: string, extra: Partial<Bill> = {}): Bill => ({
  date,
  label: `Bill ${date}`,
  amount_paise: 100_00,
  kind: "monthly",
  account_id: null,
  covered_paise: 0,
  ...extra,
});

describe("daysBetween", () => {
  it("counts whole calendar days across month and year ends", () => {
    expect(daysBetween("2026-11-08", "2026-11-08")).toBe(0);
    expect(daysBetween("2026-11-08", "2026-11-10")).toBe(2);
    expect(daysBetween("2026-12-30", "2027-01-02")).toBe(3);
    expect(daysBetween("2026-11-10", "2026-11-08")).toBe(-2);
  });
});

describe("localIsoDate", () => {
  it("uses the local calendar date with zero padding", () => {
    expect(localIsoDate(new Date(2026, 0, 5, 0, 30))).toBe("2026-01-05");
  });
});

describe("nextBills", () => {
  const bills = [bill("2026-11-20"), bill("2026-11-09"), bill("2026-11-08"), bill("2026-11-16"), bill("2026-11-30")];

  it("keeps bills within 7 days, soonest first, at most 3", () => {
    expect(nextBills(bills, "2026-11-08").map((b) => b.date)).toEqual(["2026-11-08", "2026-11-09"]);
    const many = [bill("2026-11-12"), bill("2026-11-10"), bill("2026-11-11"), bill("2026-11-09")];
    expect(nextBills(many, "2026-11-08").map((b) => b.date)).toEqual(["2026-11-09", "2026-11-10", "2026-11-11"]);
  });

  it("returns nothing when no bill is due within the window", () => {
    expect(nextBills([bill("2026-12-01")], "2026-11-08")).toEqual([]);
  });

  it("ignores bills in the past", () => {
    expect(nextBills([bill("2026-11-01")], "2026-11-08")).toEqual([]);
  });
});

describe("groupBillsByWeek", () => {
  it("groups by weeks counted from today, in date order", () => {
    const groups = groupBillsByWeek(
      [bill("2026-11-20"), bill("2026-11-08"), bill("2026-11-14"), bill("2026-11-30")],
      "2026-11-08",
    );
    expect(groups.map((g) => g.label)).toEqual(["This week", "Next week", "Week of 29 Nov"]);
    expect(groups[0].bills.map((b) => b.date)).toEqual(["2026-11-08", "2026-11-14"]);
    expect(groups[1].bills.map((b) => b.date)).toEqual(["2026-11-20"]);
    expect(groups[2].bills.map((b) => b.date)).toEqual(["2026-11-30"]);
  });

  it("drops past bills and returns no groups for an empty list", () => {
    expect(groupBillsByWeek([bill("2026-11-01")], "2026-11-08")).toEqual([]);
  });
});

describe("coveredLine", () => {
  it("shows the moved part of a partly covered card bill", () => {
    expect(coveredLine(bill("2026-11-10", { kind: "card_bill", amount_paise: 500000, covered_paise: 200000 }), fmt)).toBe(
      "₹2000 of ₹5000 moved",
    );
  });

  it("is null for other bills and for card bills with nothing moved", () => {
    expect(coveredLine(bill("2026-11-10", { covered_paise: 5 }), fmt)).toBeNull();
    expect(coveredLine(bill("2026-11-10", { kind: "card_bill" }), fmt)).toBeNull();
  });
});

describe("progressPercent", () => {
  it("clamps to 0-100 and handles a zero target", () => {
    expect(progressPercent(8000, 12000)).toBe(67);
    expect(progressPercent(20000, 12000)).toBe(100);
    expect(progressPercent(-500, 12000)).toBe(0);
    expect(progressPercent(100, 0)).toBe(0);
  });
});

describe("monthsCoveredLabel", () => {
  it("formats tenths of a month", () => {
    expect(monthsCoveredLabel(42)).toBe("4.2 months");
    expect(monthsCoveredLabel(10)).toBe("1.0 month");
    expect(monthsCoveredLabel(null)).toBeNull();
  });
});

describe("percentToBps", () => {
  it("converts a typed percentage to basis points without float drift", () => {
    expect(percentToBps("12")).toBe(1200);
    expect(percentToBps("12.5%")).toBe(1250);
    expect(percentToBps(" 3.15 ")).toBe(315);
    expect(percentToBps("100")).toBe(10000);
  });

  it("rejects anything that is not a percentage from 0 to 100", () => {
    for (const bad of ["", "%", "abc", "101", "1.234", "-5", "12.5.1"]) {
      expect(percentToBps(bad)).toBeNull();
    }
  });

  it("round-trips with bpsToPercentInput", () => {
    expect(bpsToPercentInput(1250)).toBe("12.5");
    expect(percentToBps(bpsToPercentInput(1250))).toBe(1250);
  });
});

import { buildAffordPay } from "@/utils/planner";

describe("buildAffordPay", () => {
  it("builds a pay-now request with no extra fields", () => {
    expect(buildAffordPay("now", { months: "", rate: "", fee: "" })).toEqual({ pay: { mode: "now" } });
  });

  it("converts an EMI rate in percent to basis points and the fee to paise", () => {
    expect(buildAffordPay("emi", { months: "12", rate: "14.5", fee: "499" })).toEqual({
      pay: { mode: "emi", months: 12, rate_bps: 1450, fee_paise: 49900 },
    });
  });

  it("treats an empty rate and fee as zero", () => {
    expect(buildAffordPay("emi", { months: "6", rate: "", fee: "" })).toEqual({
      pay: { mode: "emi", months: 6, rate_bps: 0, fee_paise: 0 },
    });
  });

  it("explains a bad month count, rate or fee", () => {
    expect(buildAffordPay("emi", { months: "0", rate: "", fee: "" })).toHaveProperty("error");
    expect(buildAffordPay("emi", { months: "1.5", rate: "", fee: "" })).toHaveProperty("error");
    expect(buildAffordPay("emi", { months: "12", rate: "abc", fee: "" })).toHaveProperty("error");
    expect(buildAffordPay("emi", { months: "12", rate: "", fee: "-5" })).toHaveProperty("error");
  });
});
