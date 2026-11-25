import { describe, expect, test } from "bun:test";
import { freqOf } from "./freq";

const row = (
  frequency: string,
  every_months: number,
  first_month: string | null,
) => ({ frequency, every_months, first_month });

// Same cases as `freq_from_columns_reads_the_stored_cadence_and_falls_back_to_the_safe_side`.
describe("freqOf", () => {
  test("plain monthly and yearly", () => {
    expect(freqOf(row("monthly", 1, null))).toEqual({ kind: "monthly" });
    expect(freqOf(row("yearly", 1, null))).toEqual({ kind: "yearly" });
  });
  test("every few months needs a gap of 2-11 and a first month", () => {
    expect(freqOf(row("monthly", 4, "2026-11"))).toEqual({
      kind: "every",
      months: 4,
      first: "2026-11",
    });
    expect(freqOf(row("monthly", 2, "2026-01")).kind).toBe("every");
    expect(freqOf(row("monthly", 11, "2026-01")).kind).toBe("every");
  });
  test("falls back to monthly on the safe side", () => {
    expect(freqOf(row("monthly", 4, null))).toEqual({ kind: "monthly" });
    expect(freqOf(row("monthly", 4, "soon"))).toEqual({ kind: "monthly" });
    expect(freqOf(row("monthly", 4, "2026-13"))).toEqual({ kind: "monthly" });
    expect(freqOf(row("monthly", 12, "2026-11"))).toEqual({ kind: "monthly" });
    expect(freqOf(row("monthly", 0, "2026-11"))).toEqual({ kind: "monthly" });
    expect(freqOf(row("monthly", -3, "2026-11"))).toEqual({ kind: "monthly" });
  });
  test("a yearly bill ignores a leftover gap", () => {
    expect(freqOf(row("yearly", 4, "2026-11"))).toEqual({ kind: "yearly" });
  });
});
