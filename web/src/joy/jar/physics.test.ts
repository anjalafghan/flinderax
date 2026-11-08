import { describe, expect, test } from "bun:test";
import {
  assign, CAP, dropIn, FLOOR, hash, insetAt, jostle, makeParts, makeSlots, MOUTH, mix, parseColor, rgba, rng, sample, step, takeOut, WALL_L, WALL_R,
  type SlipInfo,
} from "./physics";

const slips = (n: number): SlipInfo[] => Array.from({ length: n }, (_, i) => ({ id: i + 1, setting: i % 2 ? "home" : "local", outdoor: i % 3 === 0 }));
const fixed = () => 0.5;
const settle = (parts: ReturnType<typeof makeParts>, reduced = false) => {
  let n = 0;
  while (step(parts, 1 / 60, reduced) && n++ < 2000);
  return n;
};
const setup = (n: number) => {
  const parts = makeParts(slips(n));
  const slots = makeSlots(n + 1);
  assign(parts, slots, { matchSet: null, rainy: false, animate: false, reduced: false });
  return { parts, slots };
};

describe("slots", () => {
  test("are deterministic, inside the glass, and enough for every slip", () => {
    const a = makeSlots(40);
    expect(a).toHaveLength(40);
    expect(makeSlots(40)).toEqual(a);
    for (const s of a) {
      expect(s.x).toBeGreaterThanOrEqual(WALL_L);
      expect(s.x).toBeLessThanOrEqual(WALL_R);
      expect(s.y).toBeLessThanOrEqual(FLOOR);
    }
  });
  test("the glass curves in near the floor", () => {
    expect(insetAt(100)).toBe(0);
    expect(insetAt(FLOOR)).toBeCloseTo(24, 5);
  });
  test("the rng and hash are stable", () => {
    const r = rng(1);
    const x = r();
    expect(rng(1)()).toBe(x);
    expect(hash(7)).toBe(hash(7));
  });
  test("a big jar is sampled down to CAP, the same way each time", () => {
    const big = slips(CAP + 25);
    const s = sample(big);
    expect(s).toHaveLength(CAP);
    expect(sample(big).map((x) => x.id)).toEqual(s.map((x) => x.id));
    expect(sample(slips(5))).toHaveLength(5);
  });
});

describe("step", () => {
  test("a nudged pile settles back on its slots and then stops asking for frames", () => {
    const { parts } = setup(20);
    jostle(parts, 1, fixed);
    expect(step(parts, 1 / 60, false)).toBe(true);
    const frames = settle(parts);
    expect(frames).toBeLessThan(2000);
    for (const p of parts) {
      expect(p.x).toBeCloseTo(p.tx, 0);
      expect(p.y).toBeCloseTo(p.ty, 0);
    }
    expect(step(parts, 1 / 60, false)).toBe(false);
  });
  test("a slip never sinks through the floor", () => {
    const { parts } = setup(3);
    parts[0].y = FLOOR + 50;
    parts[0].vy = 500;
    step(parts, 1 / 60, false);
    expect(parts[0].y).toBeLessThanOrEqual(FLOOR - 4);
  });
  test("reduced motion jumps straight to the slot and fades in 150 ms", () => {
    const { parts, slots } = setup(6);
    parts[0].x = 5;
    parts[0].y = 5;
    assign(parts, slots, { matchSet: new Set([999]), rainy: false, animate: true, reduced: true });
    step(parts, 0.05, true);
    expect(parts[0].x).toBe(parts[0].tx);
    expect(parts[0].dim).toBeGreaterThan(0);
    expect(parts[0].dim).toBeLessThan(1);
    step(parts, 0.2, true);
    expect(parts[0].dim).toBe(1);
  });
  test("a falling slip bounces, then lands", () => {
    const { parts } = setup(2);
    const p = parts[0];
    p.y = p.ty - 60;
    p.vy = 0;
    p.falling = true;
    let bounced = false;
    for (let i = 0; i < 600 && p.falling; i++) {
      step(parts, 1 / 60, false);
      if (p.vy < 0) bounced = true;
    }
    expect(bounced).toBe(true);
    expect(p.falling).toBe(false);
  });
});

describe("dimming", () => {
  test("slips that don't match grey out; the ones that do stay lit and sit on top", () => {
    const { parts, slots } = setup(10);
    assign(parts, slots, { matchSet: new Set([1, 2, 3]), rainy: false, animate: true, reduced: false, rand: fixed });
    settle(parts);
    for (const p of parts) expect(p.dim).toBe([1, 2, 3].includes(p.id) ? 0 : 1);
    const lit = parts.filter((p) => p.dimT === 0).map((p) => p.ty);
    const dim = parts.filter((p) => p.dimT === 1).map((p) => p.ty);
    expect(Math.max(...dim)).toBeGreaterThan(Math.min(...lit)); // sorted into the slots in order: dim first (bottom)
  });
  test("clearing the match lights everything again", () => {
    const { parts, slots } = setup(6);
    assign(parts, slots, { matchSet: new Set([1]), rainy: false, animate: true, reduced: false, rand: fixed });
    settle(parts);
    assign(parts, slots, { matchSet: null, rainy: false, animate: true, reduced: false, rand: fixed });
    settle(parts);
    expect(parts.every((p) => p.dim === 0)).toBe(true);
  });
  test("when rain is likely cozy slips float up and outdoor ones sink", () => {
    const parts = makeParts([
      { id: 1, setting: "local", outdoor: true },
      { id: 2, setting: "home", outdoor: false },
    ]);
    const slots = makeSlots(3);
    assign(parts, slots, { matchSet: null, rainy: true, animate: false, reduced: false });
    const [outdoor, cozy] = [parts[0], parts[1]];
    expect(cozy.ty).toBe(slots[1].y); // later slot = higher up the pile
    expect(outdoor.ty).toBe(slots[0].y);
  });
});

describe("taking a slip out and putting it back", () => {
  test("the drawn slip leaves from where it was, and the pile closes up", () => {
    const { parts, slots } = setup(8);
    const at = takeOut(parts, 3, "home");
    expect(at?.setting).toBeTruthy();
    expect(parts.find((p) => p.id === 3)?.out).toBe(true);
    assign(parts, slots, { matchSet: null, rainy: false, animate: true, reduced: false });
    expect(parts.filter((p) => !p.out)).toHaveLength(7);
  });
  test("an unsimulated slip is stood in for by the top slip that fits", () => {
    const { parts } = setup(4);
    const at = takeOut(parts, 999, "travel");
    expect(at).not.toBeNull();
    const stand = parts.find((p) => p.subFor === 999);
    expect(stand?.out).toBe(true);
    expect(stand?.setting).toBe("travel");
  });
  test("it drops back in through the mouth and falls", () => {
    const { parts } = setup(4);
    takeOut(parts, 2);
    expect(dropIn(parts, 2, slips(4), false, fixed)).toBe(true);
    const p = parts.find((q) => q.id === 2)!;
    expect(p.out).toBe(false);
    expect(p.y).toBe(MOUTH.y + 6);
    expect(p.falling).toBe(true);
    settle(parts);
    expect(p.y).toBeGreaterThan(MOUTH.y + 10);
  });
  test("a stand-in gets its own colour back; nothing to drop in is a no-op", () => {
    const shown = slips(4);
    const { parts } = setup(4);
    takeOut(parts, 999, "travel");
    dropIn(parts, 999, shown, true);
    const stand = parts.find((p) => p.subFor === null && p.setting !== "travel");
    expect(stand).toBeDefined();
    expect(dropIn(parts, 5, shown, true)).toBe(false);
  });
});

describe("colours", () => {
  test("hex and rgba tokens parse; junk falls back to grey", () => {
    expect(parseColor("#f5eee3")).toEqual([245, 238, 227, 1]);
    expect(parseColor(" #fff ")).toEqual([255, 255, 255, 1]);
    expect(parseColor("rgba(255, 190, 110, .35)")).toEqual([255, 190, 110, 0.35]);
    expect(parseColor("")).toEqual([128, 128, 128, 1]);
  });
  test("mix and rgba", () => {
    expect(mix([0, 0, 0], [100, 200, 50], 0.5)).toEqual([50, 100, 25]);
    expect(rgba([10.9, 20, 30], 0.5)).toBe("rgba(10,20,30,0.5)");
  });
});
