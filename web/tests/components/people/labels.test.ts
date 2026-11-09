import { describe, expect, it } from "bun:test";

import {
  circleLabel,
  dateTitle,
  daysAwayLabel,
  everyLabel,
  lastCaughtUpLabel,
  localDay,
  plural,
  weekdayDay,
} from "@/services/people";

// Pure label and date helpers. Fixed 2026 and 2028 dates only.

describe("people labels", () => {
  it("reads a catch-up rhythm in weeks when it divides by seven, otherwise in days", () => {
    expect(everyLabel(28)).toBe("every 4 weeks");
    expect(everyLabel(7)).toBe("every 1 week");
    expect(everyLabel(10)).toBe("every 10 days");
    expect(everyLabel(null)).toBe("");
  });

  it("says when a person was last caught up, or that it was never logged", () => {
    expect(lastCaughtUpLabel("2026-10-20")).toBe("Last caught up 20 Oct");
    expect(lastCaughtUpLabel(null)).toBe("No catch-up logged yet.");
  });

  it("names the weekday from the date itself, including a leap day", () => {
    expect(weekdayDay("2026-11-14")).toBe("Sat 14 Nov");
    expect(weekdayDay("2028-02-29")).toBe("Tue 29 Feb");
  });

  it("says how far away a date is in plain words", () => {
    expect(daysAwayLabel(0)).toBe("today");
    expect(daysAwayLabel(1)).toBe("tomorrow");
    expect(daysAwayLabel(5)).toBe("in 5 days");
  });

  it("uses the label in place of the kind when one is set", () => {
    expect(dateTitle("Person A", "birthday", "")).toBe("Person A's birthday");
    expect(dateTitle("Aunt B", "other", " Diwali ")).toBe("Aunt B's Diwali");
    expect(dateTitle("Person A", "remembrance", "")).toBe("Person A's remembrance day");
  });

  it("gives the circle names and picks the right plural", () => {
    expect(circleLabel("close")).toBe("Close friends");
    expect(plural(1, "person", "people")).toBe("1 person");
    expect(plural(0, "person", "people")).toBe("0 people");
  });

  it("builds the viewer's local day, so a date input's max is today for them", () => {
    expect(localDay(new Date(2026, 10, 9, 23, 30))).toBe("2026-11-09");
    expect(localDay(new Date(2026, 0, 5))).toBe("2026-01-05");
  });
});
