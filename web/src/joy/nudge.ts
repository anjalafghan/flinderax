// "Dates at 85% with 9 days left: keep the next 2 dates under ₹1,500 each". Pure.
import { rupees } from "./money";

export interface Nudge {
  pct: number;
  daysLeft: number;
  /** How many dates the rest of the month could hold. */
  dates: number;
  /** Per-date ceiling in paise (0 when the budget is already spent). */
  perDatePaise: number;
  text: string;
}

export const NUDGE_FROM_PCT = 80;

/**
 * Not a feature of the server: just arithmetic on what is left. A date roughly every week, at
 * least one, at most three; the ceiling is what's left shared between them, rounded down to ₹50.
 */
export function nudgeFor(budgetPaise: number | null | undefined, spentPaise: number, daysLeft: number): Nudge | null {
  if (!budgetPaise || budgetPaise <= 0 || spentPaise < 0) return null;
  const pct = Math.floor((spentPaise / budgetPaise) * 100);
  if (pct < NUDGE_FROM_PCT) return null;
  const days = Math.max(0, Math.floor(daysLeft));
  const left = Math.max(0, budgetPaise - spentPaise);
  const head = `Dates at ${pct}% with ${days} ${days === 1 ? "day" : "days"} left`;
  if (left === 0) {
    return { pct, daysLeft: days, dates: 1, perDatePaise: 0, text: `${head}: the month's budget is spent, so keep the next date free` };
  }
  const dates = Math.min(3, Math.max(1, Math.ceil(days / 7)));
  const perDatePaise = Math.floor(left / dates / 5000) * 5000;
  if (perDatePaise <= 0) {
    return { pct, daysLeft: days, dates: 1, perDatePaise: 0, text: `${head}: only ${rupees(left)} is left, so keep the next date free` };
  }
  const text =
    dates === 1
      ? `${head}: keep the next date under ${rupees(perDatePaise)}`
      : `${head}: keep the next ${dates} dates under ${rupees(perDatePaise)} each`;
  return { pct, daysLeft: days, dates, perDatePaise, text };
}

/** Whole days from `now` to the end of its month, counting today's remainder as a day. */
export function daysLeftInMonth(now: Date = new Date()): number {
  const last = new Date(now.getFullYear(), now.getMonth() + 1, 0).getDate();
  return last - now.getDate() + 1;
}
