import { describe, expect, it } from "bun:test";
import { daysAgo, formatDay, ordinal } from "@/utils/dates";

describe("dates", () => {
  it("formats ISO days without timezone drift", () => {
    expect(formatDay("2026-10-30")).toBe("30 Oct");
    expect(formatDay("2026-01-01")).toBe("1 Jan");
  });

  it("builds ordinals", () => {
    expect([1, 2, 3, 4, 11, 12, 13, 21, 22, 23, 30, 31].map(ordinal)).toEqual([
      "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "23rd", "30th", "31st",
    ]);
  });

  it("counts whole days since a SQLite or RFC 3339 timestamp", () => {
    const now = new Date("2026-10-12T10:00:00Z");
    expect(daysAgo("2026-10-05 10:00:00", now)).toBe(7);
    expect(daysAgo("2026-10-12T09:00:00Z", now)).toBe(0);
  });
});
