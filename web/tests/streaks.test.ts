import { describe, expect, it } from "bun:test";

import { milestoneFor, streakHeadline, streakLine, unitLabel } from "@/utils/streaks";

describe("streak words", () => {
  it("counts in the right unit and number", () => {
    expect(unitLabel("logging", 1)).toBe("1 day");
    expect(unitLabel("saved", 3)).toBe("3 months");
  });

  it("headlines a live run, a past best, and a blank slate", () => {
    expect(streakHeadline("logging", { current: 5, best: 9 })).toBe("5 days in a row");
    expect(streakHeadline("saved", { current: 0, best: 4 })).toBe("Best: 4 months");
    expect(streakHeadline("dates", { current: 0, best: 0 })).toBe("Start your first");
  });

  it("is encouraging in every state and never scolds", () => {
    expect(streakLine("logging", { current: 0, best: 0 })).toBe("A new run starts with keeping your spending up to date.");
    expect(streakLine("saved", { current: 0, best: 3 })).toBe("You managed 3 months before. Start a new run this month.");
    expect(streakLine("logging", { current: 1, best: 4 })).toBe("A great start on keeping your spending up to date.");
    expect(streakLine("dates", { current: 4, best: 4 })).toBe("A new personal best for making time for a date!");
    expect(streakLine("saved", { current: 2, best: 5 })).toBe("Lovely work spending less than you earned. Your best is 5 months.");
  });

  it("knows which runs are milestones", () => {
    expect(milestoneFor("logging", 7)).toBe(7);
    expect(milestoneFor("logging", 8)).toBeNull();
    expect(milestoneFor("saved", 6)).toBe(6);
    expect(milestoneFor("dates", 0)).toBeNull();
  });
});
