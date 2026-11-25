import { describe, expect, it } from "bun:test";
import {
  daysAgo,
  formatDay,
  ordinal,
  addMonths,
  firstBuyMonth,
  formatYearMonth,
  thisYearMonth,
} from "@/utils/dates";

describe("dates", () => {
  it("formats ISO days without timezone drift", () => {
    expect(formatDay("2026-10-30")).toBe("30 Oct");
    expect(formatDay("2026-01-01")).toBe("1 Jan");
  });

  it("builds ordinals", () => {
    expect([1, 2, 3, 4, 11, 12, 13, 21, 22, 23, 30, 31].map(ordinal)).toEqual([
      "1st",
      "2nd",
      "3rd",
      "4th",
      "11th",
      "12th",
      "13th",
      "21st",
      "22nd",
      "23rd",
      "30th",
      "31st",
    ]);
  });

  it("counts whole days since a SQLite or RFC 3339 timestamp", () => {
    const now = new Date("2026-10-12T10:00:00Z");
    expect(daysAgo("2026-10-05 10:00:00", now)).toBe(7);
    expect(daysAgo("2026-10-12T09:00:00Z", now)).toBe(0);
  });
});

describe("months for payments every few months", () => {
  it("adds months across a year end", () => {
    expect(addMonths("2026-12", 4)).toBe("2027-04");
    expect(addMonths("2026-11", 1)).toBe("2026-12");
    expect(addMonths("2027-01", -1)).toBe("2026-12");
  });

  it("buys after a full round of saving: a perfume every 4 months saved from December is bought in April", () => {
    expect(firstBuyMonth("2026-12", 4)).toBe("2027-04");
    expect(formatYearMonth("2027-04")).toBe("Apr 2027");
  });

  it("reads this month on the device", () => {
    expect(thisYearMonth(new Date(2026, 10, 25))).toBe("2026-11");
  });
});
