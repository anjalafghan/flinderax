import { describe, expect, test } from "bun:test";
import { monthName, payoutLine, pct, triggerLabel } from "./rewards";
import type { Reward } from "@/services/rewards";

const fmt = (p: number) => `₹${p / 100}`;

describe("reward fund helpers", () => {
  test("percent rounds down", () => {
    expect(pct(9_999)).toBe("99%");
    expect(pct(10_000)).toBe("100%");
    expect(pct(-5)).toBe("0%");
  });

  test("month names", () => {
    expect(monthName("2026-10")).toContain("2026");
    expect(monthName("junk")).toBe("junk");
  });

  test("the payout line says why it is held back", () => {
    expect(payoutLine({ earned_paise: 0, saved_paise: 0, payout_paise: 0 }, fmt)).toContain("Nothing earned");
    expect(payoutLine({ earned_paise: 100_000, saved_paise: 0, payout_paise: 0 }, fmt)).toContain("nothing is saved yet");
    expect(payoutLine({ earned_paise: 100_000, saved_paise: 40_000, payout_paise: 40_000 }, fmt)).toContain("₹400 of it");
    expect(payoutLine({ earned_paise: 100_000, saved_paise: 900_000, payout_paise: 100_000 }, fmt)).toContain("all covered");
  });

  test("a wish is bought from the wallet", () => {
    const r = { trigger_kind: "wish", goal_pct: null, streak_kind: null, streak_target: null } as unknown as Reward;
    expect(triggerLabel(r)).toBe("From your reward wallet");
  });
});
