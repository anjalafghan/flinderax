import { describe, expect, test } from "bun:test";
import { axisDecision, dragPose, FOLLOW, MOST, shown, swipeOutcome, YES } from "./swipe";

const both = { canYes: true, canBack: true, vx: 0 };

describe("swipe commit threshold", () => {
  test("110 px commits; just under does not", () => {
    expect(YES).toBe(110);
    expect(swipeOutcome({ ...both, dx: 111 })).toBe("yes");
    expect(swipeOutcome({ ...both, dx: 110 })).toBe("none");
    expect(swipeOutcome({ ...both, dx: -111 })).toBe("back");
    expect(swipeOutcome({ ...both, dx: -109 })).toBe("none");
  });
  test("a quick flick commits from 50 px", () => {
    expect(swipeOutcome({ ...both, dx: 60, vx: 0.8 })).toBe("yes");
    expect(swipeOutcome({ ...both, dx: -60, vx: -0.8 })).toBe("back");
    expect(swipeOutcome({ ...both, dx: 40, vx: 0.9 })).toBe("none");
    expect(swipeOutcome({ ...both, dx: 60, vx: 0.2 })).toBe("none");
  });
  test("a direction with no button behind it never commits", () => {
    expect(swipeOutcome({ canYes: false, canBack: true, dx: 300, vx: 1 })).toBe("none");
    expect(swipeOutcome({ canYes: true, canBack: false, dx: -300, vx: -1 })).toBe("none");
  });
  test("a cancelled drag does nothing", () => {
    expect(swipeOutcome({ ...both, dx: 300, cancelled: true })).toBe("none");
  });
});

describe("the slip follows the finger at 0.2x", () => {
  test("and never further than 26 px", () => {
    expect(FOLLOW).toBe(0.2);
    expect(shown(100)).toBe(20);
    expect(shown(-100)).toBe(-20);
    expect(shown(1000)).toBe(MOST);
    expect(shown(-1000)).toBe(-MOST);
  });
  test("a direction that can't commit barely moves, and the slip tilts and shrinks a touch", () => {
    expect(dragPose(100, false).x).toBe(4);
    const p = dragPose(130, true);
    expect(p.x).toBe(26);
    expect(p.r).toBeCloseTo(0.52, 5);
    expect(p.s).toBeCloseTo(0.94, 5);
    expect(p.y).toBeCloseTo(2.6, 5);
  });
});

describe("when a drag starts", () => {
  test("sideways starts, vertical is left to scroll", () => {
    expect(axisDecision(12, 2)).toBe("start");
    expect(axisDecision(3, 3)).toBe("wait");
    expect(axisDecision(4, 14)).toBe("abort");
    expect(axisDecision(12, 12)).toBe("abort");
  });
});
