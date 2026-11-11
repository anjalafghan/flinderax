// Pure helpers for the Joy date budget. Money stays in integer paise; formatting goes through `formatINR`.

import { formatDay } from "@/utils/dates";

/** Above this share of the budget the bar turns amber. */
export const JOY_WARN_PERCENT = 80;

export type BudgetTone = "none" | "ok" | "warn" | "over";

/** Spent as a whole percent of the budget, or null when there is no budget to measure against. */
export function spentPercent(spent: number, budget: number | null): number | null {
  if (budget === null || budget <= 0) return null;
  return Math.round((spent / budget) * 100);
}

/** Bar fill from 0 to 100. */
export function barPercent(spent: number, budget: number | null): number {
  const pct = spentPercent(spent, budget);
  if (pct === null || pct <= 0) return 0;
  return Math.min(100, pct);
}

export function isOverBudget(spent: number, budget: number | null): boolean {
  return budget !== null && budget > 0 && spent > budget;
}

/** "none" without a budget, "over" past it, "warn" above 80%, otherwise "ok". */
export function budgetTone(spent: number, budget: number | null): BudgetTone {
  if (budget === null || budget <= 0) return "none";
  if (isOverBudget(spent, budget)) return "over";
  // Exact integer comparison, so 80.005% counts as past 80% even though it rounds to 80.
  return spent * 100 > budget * JOY_WARN_PERCENT ? "warn" : "ok";
}

/** "2026-11-06" -> "6 Nov". Anything that isn't a plain date is shown as it came. */
export function dayLabel(day: string): string {
  return /^\d{4}-\d{2}-\d{2}$/.test(day) ? formatDay(day) : day;
}

const DAY_MS = 86_400_000;

/** Parses an RFC 3339 or SQLite timestamp. A timestamp without a zone is UTC. Null when it can't be read. */
function parseTimestamp(ts: string): Date | null {
  const d = new Date(ts.includes("T") ? ts : ts.replace(" ", "T") + "Z");
  return Number.isNaN(d.getTime()) ? null : d;
}

/**
 * Whole calendar days between the local date of `ts` and the local date of `now` (0 = same day).
 * Counts days, not elapsed hours, so 23:00 yesterday is one day ago at 09:00 today.
 */
export function calendarDaysAgo(ts: string, now: Date = new Date()): number | null {
  const t = parseTimestamp(ts);
  if (!t) return null;
  const dayStart = (d: Date) => Date.UTC(d.getFullYear(), d.getMonth(), d.getDate());
  // Round, so a daylight-saving shift inside the span can't leave a fraction.
  return Math.max(0, Math.round((dayStart(now) - dayStart(t)) / DAY_MS));
}
