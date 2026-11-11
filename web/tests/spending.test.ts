import { describe, expect, it } from "bun:test";
import {
  barWidth,
  categoryLabel,
  dateKey,
  formatShare,
  isSpendingCategory,
  monthKey,
  monthShort,
  monthTitle,
  plural,
  shiftMonth,
  signedAmount,
  sourceDetail,
  sourceLabel,
  spendingBreakdown,
} from "@/utils/spending";

describe("categoryLabel", () => {
  it("gives friendly labels and falls back to the id", () => {
    expect(categoryLabel("food")).toBe("Food & dining");
    expect(categoryLabel("transfer")).toBe("Transfers (not spending)");
    expect(categoryLabel("uncategorized")).toBe("Uncategorized");
    expect(categoryLabel("mystery")).toBe("mystery");
  });
});

describe("spendingBreakdown", () => {
  it("drops transfers, income and zero rows, and sorts largest first", () => {
    const rows = [
      { category: "transfer", amount_paise: 900000 },
      { category: "groceries", amount_paise: 120000 },
      { category: "income", amount_paise: 500000 },
      { category: "food", amount_paise: 0 },
      { category: "rent", amount_paise: 800000 },
    ];
    expect(spendingBreakdown(rows)).toEqual([
      { category: "rent", amount_paise: 800000 },
      { category: "groceries", amount_paise: 120000 },
    ]);
  });

  it("treats only transfer and income as not spending", () => {
    expect(isSpendingCategory("transfer")).toBe(false);
    expect(isSpendingCategory("income")).toBe(false);
    expect(isSpendingCategory("uncategorized")).toBe(true);
  });
});

describe("formatShare and barWidth", () => {
  it("rounds shares to whole percents", () => {
    expect(formatShare(1, 3)).toBe("33%");
    expect(formatShare(2, 3)).toBe("67%");
    expect(formatShare(100, 100)).toBe("100%");
  });

  it("marks tiny non-zero shares as under 1%, and zero as 0%", () => {
    expect(formatShare(1, 1000)).toBe("<1%");
    expect(formatShare(0, 1000)).toBe("0%");
    expect(formatShare(5, 0)).toBe("0%");
  });

  it("scales bars against the largest value and clamps to 0-100", () => {
    expect(barWidth(50, 100)).toBe(50);
    expect(barWidth(100, 100)).toBe(100);
    expect(barWidth(150, 100)).toBe(100);
    expect(barWidth(0, 100)).toBe(0);
    expect(barWidth(10, 0)).toBe(0);
  });
});

describe("months", () => {
  it("shifts across year boundaries", () => {
    expect(shiftMonth("2026-11", 1)).toBe("2026-12");
    expect(shiftMonth("2026-12", 1)).toBe("2027-01");
    expect(shiftMonth("2026-01", -1)).toBe("2025-12");
    expect(shiftMonth("2026-11", -6)).toBe("2026-05");
  });

  it("formats titles and short labels from the string, not a Date", () => {
    expect(monthTitle("2026-11")).toBe("November 2026");
    expect(monthShort("2026-01")).toBe("Jan");
  });

  it("builds local month and date keys", () => {
    const d = new Date(2026, 10, 2); // 2 Nov 2026, local time
    expect(monthKey(d)).toBe("2026-11");
    expect(dateKey(d)).toBe("2026-11-02");
  });
});

describe("signedAmount", () => {
  it("shows debits as money out and credits as money in", () => {
    expect(signedAmount({ direction: "debit", amount_paise: 125000 })).toBe("−₹1,250");
    expect(signedAmount({ direction: "credit", amount_paise: 50 })).toBe("+₹0.50");
  });
});

describe("sourceDetail and sourceLabel", () => {
  it("combines the source name and last 4 digits when both are known", () => {
    expect(sourceDetail("HDFC Credit Card", "1234")).toBe("HDFC Credit Card ••1234");
    expect(sourceDetail("HDFC Savings", null)).toBe("HDFC Savings");
    expect(sourceDetail(null, "9876")).toBe("••9876");
    expect(sourceDetail(null, null)).toBeNull();
  });

  it("labels each source", () => {
    expect(sourceLabel("sms")).toBe("SMS");
    expect(sourceLabel("paste")).toBe("Pasted");
    expect(sourceLabel("manual")).toBe("Manual");
  });
});

describe("plural", () => {
  it("picks the singular only for exactly one", () => {
    expect(plural(1, "transaction", "transactions")).toBe("1 transaction");
    expect(plural(0, "transaction", "transactions")).toBe("0 transactions");
    expect(plural(3, "other", "others")).toBe("3 others");
  });
});

import { suggestCategory, topCategories } from "@/utils/spending";

describe("suggestCategory", () => {
  const history = [
    { merchant: "Swiggy", direction: "debit", category: "food" },
    { merchant: "swiggy ", direction: "debit", category: "food" },
    { merchant: "Swiggy", direction: "debit", category: "groceries" },
    { merchant: "Swiggy", direction: "credit", category: "refund" },
    { merchant: "Ola", direction: "debit", category: "uncategorized" },
  ];

  it("picks the category you used most for the same merchant, ignoring case and spaces", () => {
    expect(suggestCategory({ merchant: "SWIGGY", direction: "debit" }, history)).toBe("food");
  });

  it("keeps debits and credits apart", () => {
    expect(suggestCategory({ merchant: "Swiggy", direction: "credit" }, history)).toBe("refund");
  });

  it("does not guess without history, or for a missing merchant, or from uncategorised rows", () => {
    expect(suggestCategory({ merchant: "Ola", direction: "debit" }, history)).toBeNull();
    expect(suggestCategory({ merchant: "Zomato", direction: "debit" }, history)).toBeNull();
    expect(suggestCategory({ merchant: null, direction: "debit" }, history)).toBeNull();
  });
});

describe("topCategories", () => {
  it("lists the most used first and tops up with common ones", () => {
    const rows = [
      { category: "fuel", direction: "debit" },
      { category: "fuel", direction: "debit" },
      { category: "health", direction: "debit" },
      { category: "uncategorized", direction: "debit" },
    ];
    expect(topCategories(rows, "debit", 4)).toEqual(["fuel", "health", "food", "groceries"]);
    expect(topCategories([], "credit", 3)).toEqual(["income", "refund", "transfer"]);
  });
});
