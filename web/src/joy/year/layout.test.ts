import { describe, expect, test } from "bun:test";
import {
  coverCrop, dealParams, drawData, fitNumberSize, H, M, MAX_PHOTOS, monthBars, needsDense, pickPhotos, polaroidPile, rng, shareControls,
  shareFile, slackFor, uniquePlaces, W, wrapText, type ShareData,
} from "./layout";

const data = (o: Partial<ShareData> = {}): ShareData => ({
  year: 2026, dates: 12, months: [0, 1, 0, 3, 0, 0, 2, 0, 0, 3, 0, 3], vibe: "Around town", tags: [], best_title: "Chai", best_rating: 5, best_more: 0,
  vibe_key: "local", avg_rating: "4.2", streak_weeks: 3, places: ["A", "B", "A", ""], spent: "₹4,000", photos: [{ id: 1, url: "/a" }, { id: 2, url: "/b" }], ...o,
});

describe("seeded randomness", () => {
  test("the same seed draws the same paper", () => {
    const a = rng(2026 * 97 + 12);
    const b = rng(2026 * 97 + 12);
    expect([a(), a(), a()]).toEqual([b(), b(), b()]);
    const r = rng(5)();
    expect(r).toBeGreaterThanOrEqual(0);
    expect(r).toBeLessThan(1);
  });
  test("dealt angles are stable per year and spread by index", () => {
    expect(dealParams(2026, 3)).toEqual(dealParams(2026, 3));
    expect(dealParams(2026, 3)).not.toEqual(dealParams(2026, 4));
    expect(Math.abs(dealParams(2026, 0).rot)).toBeLessThanOrEqual(18);
  });
});

describe("what the picture may hold", () => {
  test("places and spend only when ticked; places de-duplicated", () => {
    const off = drawData(data(), { places: false, spent: false, photos: [] });
    expect(off.places).toEqual([]);
    expect(off.spent).toBe("");
    const on = drawData(data(), { places: true, spent: true, photos: [] });
    expect(on.places).toEqual(["A", "B"]);
    expect(on.spent).toBe("₹4,000");
  });
  test("photos: only ids in the share data, once each, at most nine", () => {
    const many = Array.from({ length: 14 }, (_, i) => ({ id: i + 1, url: `/p${i}` }));
    expect(pickPhotos([1, 99, 1, 2], data().photos).map((p) => p.id)).toEqual([1, 2]);
    expect(pickPhotos(many.map((p) => p.id), many)).toHaveLength(MAX_PHOTOS);
  });
  test("controls hide when there is nothing behind them", () => {
    expect(shareControls(data({ places: [""], spent: "", photos: [] }))).toEqual({ places: false, spent: false, photos: false });
    expect(shareControls(data())).toEqual({ places: true, spent: true, photos: true });
    expect(uniquePlaces(["x", "x", ""])).toEqual(["x"]);
  });
  test("file name and type follow the photos", () => {
    expect(shareFile(2026, true)).toEqual({ name: "our-2026.jpg", type: "image/jpeg" });
    expect(shareFile(2026, false)).toEqual({ name: "our-2026.png", type: "image/png" });
  });
});

describe("wrapping", () => {
  const m = (t: string) => t.length * 10;
  test("breaks on words", () => expect(wrapText(m, "one two three four", 100, 5)).toEqual(["one two", "three four"]));
  test("cuts with an ellipsis when it runs out of lines", () => {
    const out = wrapText(m, "alpha beta gamma delta epsilon", 100, 2);
    expect(out).toHaveLength(2);
    expect(out[1]?.endsWith("…")).toBe(true);
    expect(out.every((l) => m(l) <= 100)).toBe(true);
  });
  test("a word wider than the line never overflows", () => {
    const out = wrapText(m, "supercalifragilistic", 80, 1);
    expect(out.every((l) => m(l) <= 80)).toBe(true);
  });
});

describe("paint decisions", () => {
  test("denser only without photos and when the extras run past the sign-off", () => {
    expect(needsDense(H - 100, false)).toBe(true);
    expect(needsDense(H - 200, false)).toBe(false);
    expect(needsDense(H, true)).toBe(false);
  });
  test("spare height is shared out, capped, and only without photos", () => {
    expect(slackFor(1000, false)).toBe(380);
    expect(slackFor(H - 200 - 100, false)).toBe(100);
    expect(slackFor(H - 200 - 10, false)).toBe(0);
    expect(slackFor(1000, true)).toBe(0);
  });
  test("a big number shrinks to fit", () => {
    expect(fitNumberSize(380, 400, 560)).toBe(380);
    expect(fitNumberSize(380, 1120, 560)).toBe(190);
  });
});

describe("months ledger", () => {
  test("bars are relative to the busiest; empty months are rings", () => {
    const bars = monthBars([0, 1, 3, 3], 190);
    expect(bars[0]?.h).toBe(0);
    expect(bars[2]?.h).toBe(190);
    expect(bars[1]?.h).toBeCloseTo(190 / 3);
    expect(bars.map((b) => b.busiest)).toEqual([false, false, true, true]);
    expect(bars[0]?.cx).toBeGreaterThan(M);
    expect(monthBars([0, 0], 100).every((b) => !b.busiest)).toBe(true);
  });
});

describe("polaroid pile", () => {
  test("none, one row, two rows", () => {
    expect(polaroidPile(0, 1000)).toEqual([]);
    expect(new Set(polaroidPile(3, 1000).map((s) => Math.round(s.y / 100))).size).toBeLessThanOrEqual(2);
    const nine = polaroidPile(9, 1000);
    expect(nine).toHaveLength(9);
    expect(Math.max(...nine.map((s) => s.y))).toBeGreaterThan(Math.min(...nine.map((s) => s.y)));
  });
  test("everything stays on the page, above the sign-off", () => {
    for (const n of [1, 2, 4, 9]) {
      for (const s of polaroidPile(n, 900)) {
        expect(s.x).toBeGreaterThanOrEqual(M - 1);
        expect(s.x + s.pw).toBeLessThanOrEqual(W - 40);
        expect(s.y + s.fh).toBeLessThanOrEqual(H - 100);
      }
    }
  });
  test("cover-crop takes the middle at the tile's ratio", () => {
    const wide = coverCrop(2000, 1000, 100, 125);
    expect(wide.sh).toBe(1000);
    expect(wide.sw / wide.sh).toBeCloseTo(0.8);
    expect(wide.sx).toBeCloseTo((2000 - wide.sw) / 2);
    const tall = coverCrop(500, 2000, 100, 125);
    expect(tall.sw).toBe(500);
    expect(tall.sy).toBeCloseTo((2000 - tall.sh) / 2);
  });
});
