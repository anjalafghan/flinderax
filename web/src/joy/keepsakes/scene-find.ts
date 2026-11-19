// Finds the illustrated keepsake drawn for a kept date (scenes.gen.ts), by slip and visit.
import { SCENES } from "./scenes.gen";
import type { SceneDef } from "./scene-types";

const byKey = new Map<string, SceneDef[]>();
for (const s of SCENES) byKey.set(s.key, [...(byKey.get(s.key) ?? []), s].sort((a, b) => a.visit - b.visit));

/** The scene drawn for this slip and visit; later visits keep the last one (the regulars' card). */
export function sceneFor(slipKey: string | undefined, visit = 1): SceneDef | null {
  if (!slipKey) return null;
  const all = byKey.get(slipKey.trim().toLowerCase());
  if (!all?.length) return null;
  return all.find((s) => s.visit === visit) ?? all[all.length - 1] ?? null;
}

/** Which visit each date is, counting dates with the same title in day order (for views without `visit`). */
export function visitsOf<T extends { id: number; title: string; day: string }>(items: T[]): Map<number, number> {
  const sorted = [...items].sort((a, b) => a.day.localeCompare(b.day) || a.id - b.id);
  const seen = new Map<string, number>();
  const out = new Map<number, number>();
  for (const it of sorted) {
    const k = it.title.trim().toLowerCase();
    const n = (seen.get(k) ?? 0) + 1;
    seen.set(k, n);
    out.set(it.id, n);
  }
  return out;
}

