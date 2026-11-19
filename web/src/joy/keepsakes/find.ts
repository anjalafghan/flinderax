// Finds a kept date's own keepsake in the catalogue: by its slip's title (lower-cased, as the API
// sends it) and, when several slips share a title, by which visit this is.
import { CATALOG } from "./catalog";
import type { Spec } from "./spec";

const byKey = new Map<string, Spec[]>();
for (const s of CATALOG) {
  const k = s.key.trim().toLowerCase();
  byKey.set(k, [...(byKey.get(k) ?? []), s].sort((a, b) => (a.visit ?? 1) - (b.visit ?? 1)));
}

/** The spec for this slip and visit, or null when the catalogue has none (a slip written later). */
export function specFor(slipKey: string | undefined, visit = 1): Spec | null {
  if (!slipKey) return null;
  const all = byKey.get(slipKey.trim().toLowerCase());
  if (!all?.length) return null;
  // Same title, several objects: the n-th visit gets the n-th; later visits go round again.
  return all.find((s) => (s.visit ?? 1) === visit) ?? all[(Math.max(1, visit) - 1) % all.length];
}
