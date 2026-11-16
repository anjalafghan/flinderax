import { describe, expect, test } from "bun:test";

import { barPct, monthLabel } from "./year";

describe("year helpers", () => {
  test("monthLabel names the month", () => {
    expect(monthLabel("2027-03")).toBe("Mar 2027");
    expect(monthLabel("2026-12")).toBe("Dec 2026");
    expect(monthLabel("garbage")).toBe("garbage");
  });

  test("barPct is a share of the tallest bar and never negative", () => {
    expect(barPct(50, 100)).toBe(50);
    expect(barPct(100, 100)).toBe(100);
    expect(barPct(-10, 100)).toBe(0);
    expect(barPct(10, 0)).toBe(0);
    expect(barPct(500, 100)).toBe(100);
  });
});
