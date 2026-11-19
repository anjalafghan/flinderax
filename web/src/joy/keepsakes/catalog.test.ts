import { describe, expect, test } from "bun:test";
import { CATALOG } from "./catalog";
import { ANIMS, FORMS } from "./spec";
import { hasMotif } from "./icons";
import { specFor } from "./find";

// WCAG relative luminance and contrast, for the ink on each object's paper.
const lum = (hex: string) => {
  const n = parseInt(hex.slice(1), 16);
  const c = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => {
    const s = v / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
};
const contrast = (a: string, b: string) => {
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
};

describe("the keepsake catalogue", () => {
  test("covers every slip we had when it was written", () => {
    expect(CATALOG.length).toBe(335);
  });

  test("every spec is something the renderer can draw", () => {
    for (const s of CATALOG) {
      expect(FORMS).toContain(s.form);
      expect(ANIMS).toContain(s.anim);
      expect(hasMotif(s.motif)).toBe(true);
      for (const c of [s.paper, s.ink, s.accent]) expect(c).toMatch(/^#[0-9a-fA-F]{6}$/);
      expect(s.label.length).toBeGreaterThan(0);
      expect(s.head.length).toBeLessThanOrEqual(30);
      expect(s.lines.length).toBeLessThanOrEqual(2);
      for (const l of s.lines) expect(l.length).toBeLessThanOrEqual(46);
    }
  });

  test("ink stays readable on its paper", () => {
    const poor = CATALOG.filter((s) => contrast(s.ink, s.paper) < 7).map((s) => `${s.label} ${contrast(s.ink, s.paper).toFixed(1)}`);
    expect(poor).toEqual([]);
  });

  test("no two slips get the same object", () => {
    const seen = new Map<string, string>();
    for (const s of CATALOG) {
      const k = s.label.toLowerCase();
      expect(seen.get(k)).toBeUndefined();
      seen.set(k, s.key);
    }
    const looks = new Set(CATALOG.map((s) => `${s.form}|${s.motif}`));
    expect(looks.size).toBe(CATALOG.length);
  });

  test("each slip (and each visit of a shared title) has exactly one spec", () => {
    const keys = CATALOG.map((s) => `${s.key}#${s.visit ?? 1}`);
    expect(new Set(keys).size).toBe(keys.length);
  });

  test("a kept date finds its keepsake from the key the API sends", () => {
    expect(specFor("cat café")?.form).toBe("paw");
    expect(specFor("Dress up for KOKO")?.motif).toBe("x:chopsticks");
    expect(specFor("blue tokai", 1)?.label).not.toBe(specFor("blue tokai", 2)?.label);
    expect(specFor("blue tokai", 3)).not.toBeNull(); // later visits go round again
    expect(specFor("a slip we write next year")).toBeNull();
    expect(specFor(undefined)).toBeNull();
  });
});
