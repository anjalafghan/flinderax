import { describe, expect, it } from "bun:test";
import { formatCompactINR, formatINR, formatPercent, paiseToInput, parseAmountOrPercent, toPaise } from "@/utils/money";

describe("toPaise", () => {
  it("parses whole and decimal rupees without float error", () => {
    expect(toPaise("1234")).toBe(123400);
    expect(toPaise("1234.5")).toBe(123450);
    expect(toPaise("1234.05")).toBe(123405);
    expect(toPaise("0.07")).toBe(7);
    expect(toPaise("19.99")).toBe(1999);
    expect(toPaise(".5")).toBe(50);
  });

  it("ignores ₹, commas and spaces", () => {
    expect(toPaise("₹ 61,488")).toBe(6148800);
    expect(toPaise("1,23,456.78")).toBe(12345678);
  });

  it("allows negatives (balances can be overdrawn)", () => {
    expect(toPaise("-50")).toBe(-5000);
  });

  it("rejects anything that is not a plain amount", () => {
    for (const bad of ["", " ", "abc", "12.345", "1.2.3", "--5", "-", "."]) {
      expect(toPaise(bad)).toBeNull();
    }
  });
});

describe("formatINR", () => {
  it("uses Indian grouping and drops zero paise", () => {
    expect(formatINR(0)).toBe("₹0");
    expect(formatINR(6148800)).toBe("₹61,488");
    expect(formatINR(754990000)).toBe("₹75,49,900");
    expect(formatINR(150)).toBe("₹1.50");
    expect(formatINR(-5000)).toBe("-₹50");
  });
});

describe("paiseToInput", () => {
  it("round-trips through toPaise", () => {
    for (const p of [0, 7, 50, 100, 123405, 123450, -5000]) {
      expect(toPaise(paiseToInput(p))).toBe(p);
    }
  });
});

describe("formatCompactINR", () => {
  it("uses k, L and Cr like Indian chart axes", () => {
    expect(formatCompactINR(0)).toBe("₹0");
    expect(formatCompactINR(45_000)).toBe("₹450");
    expect(formatCompactINR(4_520_000)).toBe("₹45.2k");
    expect(formatCompactINR(12_345_600)).toBe("₹1.2L");
    expect(formatCompactINR(150_000_000)).toBe("₹15L");
    expect(formatCompactINR(-5_000_000)).toBe("-₹50k");
    expect(formatCompactINR(30_000_000_000)).toBe("₹30Cr");
  });
});

describe("parseAmountOrPercent", () => {
  it("reads percentages as basis points", () => {
    expect(parseAmountOrPercent("25%")).toEqual({ kind: "percent", bps: 2500 });
    expect(parseAmountOrPercent(" 22.5 % ")).toEqual({ kind: "percent", bps: 2250 });
    expect(parseAmountOrPercent("33.33%")).toEqual({ kind: "percent", bps: 3333 });
    expect(parseAmountOrPercent("0%")).toEqual({ kind: "percent", bps: 0 });
    expect(parseAmountOrPercent("100%")).toEqual({ kind: "percent", bps: 10000 });
  });

  it("reads plain numbers as rupee amounts", () => {
    expect(parseAmountOrPercent("18875")).toEqual({ kind: "amount", paise: 1_887_500 });
    expect(parseAmountOrPercent("₹18,875.50")).toEqual({ kind: "amount", paise: 1_887_550 });
  });

  it("treats blank as empty and junk as invalid", () => {
    expect(parseAmountOrPercent("  ")).toEqual({ kind: "empty" });
    for (const bad of ["abc", "%", "25 %%", "101%", "-5%", "12.345%", "-100"]) {
      expect(parseAmountOrPercent(bad)).toEqual({ kind: "invalid" });
    }
  });
});

describe("formatPercent", () => {
  it("drops needless decimals", () => {
    expect(formatPercent(2500)).toBe("25%");
    expect(formatPercent(2250)).toBe("22.5%");
  });
});
