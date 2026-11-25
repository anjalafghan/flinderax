import { describe, expect, test } from "bun:test";

import { buildGuidanceInput, type GuidanceDraft } from "@/services/guidance";

const draft: GuidanceDraft = {
  salary: "74000",
  sip: "25000",
  spend_pct: "30",
  saved: "",
  fixed: [
    { label: "House", amount: "4000" },
    { label: "", amount: "" },
  ],
  shared: [{ label: "Together", total: "18000", partner: "6000" }],
};

describe("buildGuidanceInput", () => {
  test("turns rupees into paise and the share into basis points, skipping empty rows", () => {
    const r = buildGuidanceInput(draft);
    expect(r.salary_paise).toBe(7_400_000);
    expect(r.spend_bps).toBe(3000);
    expect(r.fixed).toEqual([{ label: "House", amount_paise: 400_000 }]);
    expect(r.shared[0]).toEqual({
      label: "Together",
      total_paise: 1_800_000,
      partner_paise: 600_000,
    });
    expect(r.emergency_saved_paise).toBe(0);
  });
  test("refuses a missing salary and a bad amount", () => {
    expect(() => buildGuidanceInput({ ...draft, salary: "" })).toThrow();
    expect(() => buildGuidanceInput({ ...draft, sip: "abc" })).toThrow();
  });
});
