// Searching the slips list. Pure.
import type { ActivityView, Group } from "./api";

export const norm = (s: string) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

export function haystack(a: ActivityView): string {
  return norm([a.title, a.place, a.tags.join(" "), a.setting, a.notes].join(" "));
}

/** Every word of the query has to appear somewhere in the slip. */
export function matches(a: ActivityView, query: string): boolean {
  const words = norm(query).trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return true;
  const h = haystack(a);
  return words.every((w) => h.includes(w));
}

export interface Filtered {
  group: Group;
  items: ActivityView[];
}

export function filterGroups(groups: Group[], query: string): { shown: Filtered[]; total: number } {
  const shown = groups
    .map((group) => ({ group, items: group.items.filter((a) => matches(a, query)) }))
    .filter((g) => g.items.length > 0 || query.trim() === "");
  return { shown, total: shown.reduce((n, g) => n + g.items.length, 0) };
}

export const slipsWord = (n: number) => (n === 1 ? "slip" : "slips");
