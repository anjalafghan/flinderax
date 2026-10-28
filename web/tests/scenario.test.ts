import { describe, expect, it } from "bun:test";
import { defaultStart, newDraft, toScenarioItem } from "@/utils/scenario";

describe("toScenarioItem", () => {
  it("turns a complete RD draft into a request item in paise and basis points", () => {
    const d = { ...newDraft("rd", new Date(2026, 9, 5)), amount: "10,000", rate: "7.1", target: "1,50,000", targetDate: "2027-11-10" };
    expect(toScenarioItem(d)).toEqual({
      name: "Wedding RD",
      kind: "rd",
      amount_paise: 1_000_000,
      start: "2026-11-10",
      tenure_months: 12,
      rate_bps: 710,
      funded_from: null,
      payout_to: null,
      renewal: null,
      target_paise: 15_000_000,
      target_date: "2027-11-10",
    });
  });

  it("returns null until the draft is valid", () => {
    const base = newDraft("rd", new Date(2026, 9, 5));
    expect(toScenarioItem({ ...base, amount: "" })).toBeNull();
    expect(toScenarioItem({ ...base, amount: "abc" })).toBeNull();
    expect(toScenarioItem({ ...base, tenure: "0" })).toBeNull();
    expect(toScenarioItem({ ...base, rate: "-1" })).toBeNull();
    expect(toScenarioItem({ ...base, start: "10 Nov" })).toBeNull();
    expect(toScenarioItem({ ...base, target: "xyz" })).toBeNull();
  });

  it("one-off spends carry no tenure, rate or renewal", () => {
    const d = { ...newDraft("one_off", new Date(2026, 9, 5)), amount: "50000", start: "2026-12-05" };
    const item = toScenarioItem(d)!;
    expect([item.tenure_months, item.rate_bps, item.renewal]).toEqual([null, null, null]);
  });

  it("FDs keep their renewal choice", () => {
    const d = { ...newDraft("fd", new Date(2026, 9, 5)), amount: "300000", renewal: "renew_all" as const };
    expect(toScenarioItem(d)!.renewal).toBe("renew_all");
  });
});

describe("defaultStart", () => {
  it("is the 10th of next month, rolling over the year", () => {
    expect(defaultStart(new Date(2026, 9, 5))).toBe("2026-11-10");
    expect(defaultStart(new Date(2026, 11, 20))).toBe("2027-01-10");
  });
});
