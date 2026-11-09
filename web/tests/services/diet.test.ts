import { describe, expect, it } from "bun:test";

import { kgLabel, ordersPerWeek, parseTenths, signedKg, todayIso } from "@/services/diet";

// Pure helpers only: the wire is tenths of a kilo and whole days, and the text is shown as the user reads it.

describe("diet helpers", () => {
  it("shows weights in kg to one decimal", () => {
    expect(kgLabel(705)).toBe("70.5 kg");
    expect(kgLabel(700)).toBe("70.0 kg");
  });

  it("signs a change with a true minus sign", () => {
    expect(signedKg(-12)).toBe("−1.2 kg");
    expect(signedKg(5)).toBe("+0.5 kg");
    expect(signedKg(0)).toBe("0.0 kg");
  });

  it("parses one-decimal input to tenths, within the range", () => {
    expect(parseTenths("70.5", 200, 3000)).toBe(705);
    expect(parseTenths("70", 200, 3000)).toBe(700);
    expect(parseTenths(" 84 ", 300, 2000)).toBe(840);
    expect(parseTenths("70.55", 200, 3000)).toBeNull();
    expect(parseTenths("abc", 200, 3000)).toBeNull();
    expect(parseTenths("10", 200, 3000)).toBeNull();
    expect(parseTenths("301", 200, 3000)).toBeNull();
    expect(parseTenths("", 200, 3000)).toBeNull();
  });

  it("writes orders a week without a trailing .0", () => {
    expect(ordersPerWeek(350)).toBe("3.5");
    expect(ordersPerWeek(300)).toBe("3");
    expect(ordersPerWeek(0)).toBe("0");
  });

  it("gives today in the local calendar as YYYY-MM-DD", () => {
    expect(todayIso(new Date(2026, 10, 9, 23, 30))).toBe("2026-11-09");
    expect(todayIso(new Date(2026, 0, 1, 0, 5))).toBe("2026-01-01");
  });
});
