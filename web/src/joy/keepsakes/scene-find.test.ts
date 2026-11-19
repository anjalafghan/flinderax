import { describe, expect, test } from "bun:test";
import { sceneFor, visitsOf } from "./scene-find";
import { SCENES } from "./scenes.gen";

describe("sceneFor", () => {
  test("finds a scene by slip title, any case", () => {
    expect(sceneFor("Cat café")?.id).toBe("catcafe");
    expect(sceneFor(" dress up for KOKO ")?.id).toBe("koko");
  });
  test("a repeat visit gets its own object, and later visits keep the last", () => {
    expect(sceneFor("blue tokai", 1)?.id).toBe("tokai1");
    expect(sceneFor("blue tokai", 2)?.id).toBe("tokai2");
    expect(sceneFor("blue tokai", 5)?.id).toBe("tokai2");
  });
  test("slips nobody drew yet have none", () => {
    expect(sceneFor("a slip written tomorrow")).toBeNull();
    expect(sceneFor(undefined)).toBeNull();
  });
  test("every scene's ids and motion are scoped to it", () => {
    for (const s of SCENES) {
      for (const m of s.svg.matchAll(/\bid="([^"]+)"/g)) expect(m[1]!.startsWith(s.id + "-")).toBe(true);
      expect(s.svg).not.toContain("<script");
    }
  });
});

describe("visitsOf", () => {
  test("counts visits per title in day order", () => {
    const v = visitsOf([
      { id: 9, title: "Blue Tokai", day: "2026-11-12" },
      { id: 3, title: "Cat café", day: "2026-11-09" },
      { id: 7, title: "blue tokai", day: "2026-11-10" },
    ]);
    expect(v.get(7)).toBe(1);
    expect(v.get(9)).toBe(2);
    expect(v.get(3)).toBe(1);
  });
});
