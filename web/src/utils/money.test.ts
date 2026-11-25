import { describe, expect, test } from "bun:test";
import {
  formatINR,
  formatRupeesFixed,
  formatWholeINR,
  toPaise,
} from "./money";

describe("formatters", () => {
  test("formatINR shows paise only when non-zero", () => {
    expect(formatINR(1234500)).toBe("₹12,345");
    expect(formatINR(150)).toBe("₹1.50");
  });
  test("formatWholeINR drops paise with rounding", () => {
    expect(formatWholeINR(150049)).toBe("₹1,500");
    expect(formatWholeINR(150050)).toBe("₹1,501");
    expect(formatWholeINR(0)).toBe("₹0");
  });
  test("formatRupeesFixed always has two decimals, takes rupees", () => {
    expect(formatRupeesFixed(1234)).toBe("₹1,234.00");
    expect(formatRupeesFixed(123456.5)).toBe("₹1,23,456.50");
  });
});

describe("toPaise", () => {
  test("parses plain amounts without floats", () => {
    expect(toPaise("1,200.50")).toBe(120050);
    expect(toPaise("₹19.99")).toBe(1999);
    expect(toPaise("-50")).toBe(-5000);
  });
  test("invalid input is null", () => {
    expect(toPaise("12.345")).toBeNull();
    expect(toPaise("")).toBeNull();
    expect(toPaise("abc")).toBeNull();
  });
});
