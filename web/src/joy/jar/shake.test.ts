import { describe, expect, test } from "bun:test";
import { magnitude, newShake, shakeStep } from "./shake";

const feed = (mags: [number, number][], busy = false) => {
  let s = newShake();
  let draws = 0;
  for (const [mag, t] of mags) {
    const r = shakeStep(s, mag, t, busy);
    s = r.state;
    if (r.draw) draws++;
  }
  return draws;
};

describe("shake to draw", () => {
  test("three hard shakes inside 700 ms draw", () => {
    expect(feed([[20, 1000], [20, 1250], [20, 1500]])).toBe(1);
  });
  test("two shakes are not enough", () => {
    expect(feed([[20, 1000], [20, 1250]])).toBe(0);
  });
  test("slow shakes don't add up", () => {
    expect(feed([[20, 1000], [20, 1500], [20, 2000]])).toBe(0);
  });
  test("gentle motion only rattles the jar", () => {
    const r = shakeStep(newShake(), 8, 1000, false);
    expect(r.draw).toBe(false);
    expect(r.jiggle).toBeCloseTo(0.4, 5);
  });
  test("not while a draw is under way, and not twice inside two seconds", () => {
    expect(feed([[20, 1000], [20, 1200], [20, 1400]], true)).toBe(0);
    expect(feed([[20, 1000], [20, 1200], [20, 1400], [20, 1600], [20, 1800], [20, 2000]])).toBe(1);
    expect(feed([[20, 1000], [20, 1200], [20, 1400], [20, 3500], [20, 3700], [20, 3900]])).toBe(2);
  });
  test("magnitude: with gravity removed, or from gravity alone", () => {
    expect(magnitude({ acceleration: { x: 3, y: 4, z: 0 } })).toBe(5);
    expect(magnitude({ acceleration: { x: null, y: null, z: null }, accelerationIncludingGravity: { x: 0, y: 0, z: 9.81 } })).toBeCloseTo(0, 5);
    expect(magnitude({})).toBe(0);
  });
});
