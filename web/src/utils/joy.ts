// Pure helpers for the Joy date budget. Money stays in integer paise; formatting goes through `formatINR`.

import { daysAgo, formatDay } from "@/utils/dates";

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

/** When Joy last answered: "today", "yesterday", "6 Nov", or "an earlier sync" when unknown. */
export function syncedLabel(ts: string | null, now: Date = new Date()): string {
  if (!ts) return "an earlier sync";
  const days = daysAgo(ts, now);
  if (days === 0) return "today";
  if (days === 1) return "yesterday";
  return formatDay(ts.slice(0, 10));
}
