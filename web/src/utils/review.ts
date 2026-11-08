// Pure helpers for the Review page (month close and tax). Money stays in integer paise.

import type { TaxRegime, TaxReminder, TaxSectionId } from "@/services/types";
import { formatINR, toPaise } from "@/utils/money";
import { monthKey, shiftMonth } from "@/utils/spending";

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

/** The month the Review page opens on: the last complete calendar month, "YYYY-MM". */
export function lastCompleteMonth(today: Date = new Date()): string {
  return shiftMonth(monthKey(today), -1);
}

/** Financial year (India, April to March) for a date, as "2026-27". */
export function fyForDate(d: Date = new Date()): string {
  const startYear = d.getMonth() >= 3 ? d.getFullYear() : d.getFullYear() - 1;
  return `${startYear}-${String((startYear + 1) % 100).padStart(2, "0")}`;
}

/**
 * The month one step from `ym`, or null when that step would pass `last` (the newest month allowed).
 * "YYYY-MM" strings compare correctly as text.
 */
export function stepMonth(ym: string, delta: number, last: string): string | null {
  const next = shiftMonth(ym, delta);
  return next > last ? null : next;
}

/** "2026-27" -> "FY 2026-27". */
export function fyLabel(fy: string): string {
  return `FY ${fy}`;
}

/** Signed rupees for a change: "+₹1,200", "−₹300", "₹0". */
export function signedINR(paise: number): string {
  if (paise === 0) return formatINR(0);
  return `${paise > 0 ? "+" : "−"}${formatINR(Math.abs(paise))}`;
}

/** Part of a whole as a 0 to 100 bar width. A zero whole reads 100 only when there is a part. */
export function ratioPercent(part: number, whole: number): number {
  if (whole <= 0) return part > 0 ? 100 : 0;
  return clamp((part / whole) * 100, 0, 100);
}

/**
 * Stacked bar for a tax section: the claimed amount (solid) and the projected amount on top of it
 * (lighter). Both are percentages of the limit and together never exceed the bar. No limit, no bar.
 */
export function stackedWidths(
  claimed: number,
  projected: number,
  limit: number | null,
): { claimed: number; projected: number } {
  if (limit === null || limit <= 0) return { claimed: 0, projected: 0 };
  const solid = clamp((claimed / limit) * 100, 0, 100);
  const extra = clamp((projected / limit) * 100, 0, 100 - solid);
  return { claimed: solid, projected: extra };
}

/** Headline for the regime comparison: the saving is the gap between the two tax amounts. */
export function regimeHeadline(regime: TaxRegime): string {
  const gap = Math.abs(regime.old_tax_paise - regime.new_tax_paise);
  if (gap === 0) return "Both regimes come to the same tax";
  const name = regime.better === "new" ? "New" : "Old";
  return `${name} regime saves you ${formatINR(gap)}`;
}

/** "Today", "Tomorrow", "12 days left", or "Passed" for a date that has gone. */
export function daysLeftLabel(days: number): string {
  if (days < 0) return "Passed";
  if (days === 0) return "Today";
  if (days === 1) return "Tomorrow";
  return `${days} days left`;
}

/** The soonest reminder that has not passed yet, or null when there are none. */
export function nextReminder(reminders: TaxReminder[]): TaxReminder | null {
  let best: TaxReminder | null = null;
  for (const r of reminders) {
    if (r.days_left < 0) continue;
    if (best === null || r.days_left < best.days_left) best = r;
  }
  return best;
}

const SECTION_LABEL: Record<string, string> = {
  "80C": "80C (PF, ELSI, tuition)",
  "80CCD1B": "80CCD(1B) NPS",
  "80D": "80D health insurance",
  HRA: "HRA rent",
  "24B": "Home loan interest",
  other: "Other",
};

export function sectionLabel(section: TaxSectionId | string): string {
  return SECTION_LABEL[section] ?? section;
}

/** Section choices for a select, in the order shown. */
export const TAX_SECTION_IDS: readonly TaxSectionId[] = ["80C", "80CCD1B", "80D", "HRA", "24B", "other"];

/** Why a deduction cannot be saved yet, or null when it can. */
export function deductionError(label: string, amountText: string): string | null {
  if (!label.trim()) return "Name the deduction, for example “PPF deposit”.";
  const paise = toPaise(amountText);
  if (paise === null || paise <= 0) return "Type an amount above zero.";
  return null;
}
