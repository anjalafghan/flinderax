import { describe, expect, it } from "bun:test";
import { formatINR, paiseToInput, toPaise } from "@/utils/money";

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
