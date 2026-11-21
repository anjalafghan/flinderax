import { describe, expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { sceneFor, visitsOf } from "./scene-find";
import { SCENES } from "./scenes.gen";

const PUBLIC = path.join(import.meta.dir, "..", "..", "..", "public", "keepsakes");

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
  test("a title with a name filled in still finds its slip's keepsake", () => {
    expect(sceneFor("Hindi rap listening session (watch Diya react)")?.id).toBe("hindirap");
  });
  test("slips nobody drew yet have none", () => {
    expect(sceneFor("a slip written tomorrow")).toBeNull();
    expect(sceneFor(undefined)).toBeNull();
  });
});

describe("the keepsake files", () => {
  test("every scene has its file, with its ids and motion scoped to it and nothing that runs", () => {
    for (const s of SCENES) {
      const f = path.join(PUBLIC, s.file);
      expect(existsSync(f)).toBe(true);
      const svg = readFileSync(f, "utf8");
      for (const m of svg.matchAll(/\bid="([^"]+)"/g)) expect(m[1]!.startsWith(s.id + "-")).toBe(true);
      expect(svg).not.toMatch(/<script|\son\w+=|javascript:/i);
      for (const rule of svg.match(/<style>([\s\S]*?)<\/style>/)?.[1]?.split("}") ?? []) {
        const sel = rule.split("{")[0]!.trim();
        if (sel && !sel.startsWith("@") && !/^(from|to|\d+%)/.test(sel) && sel.includes(".")) expect(sel).toContain(`ksc-${s.id}`);
      }
    }
  });
  test("ids are unique and every file name carries a content hash", () => {
    expect(new Set(SCENES.map((s) => s.id)).size).toBe(SCENES.length);
    for (const s of SCENES) expect(s.file).toMatch(new RegExp(`^${s.id}-[0-9a-z]{8}\\.svg$`));
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
