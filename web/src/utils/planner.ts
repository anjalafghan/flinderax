import type { AffordPay, Bill, GoalStatus } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { toPaise } from "@/utils/money";

// Pure helpers for the Bills, Goals and Can-I-afford screens. Dates are "YYYY-MM-DD" strings
// and are compared as UTC midnights so the viewer's timezone can't move a bill by a day.

const DAY_MS = 86_400_000;

const toUtc = (iso: string): number => {
  const [y, m, d] = iso.split("-").map(Number);
  return Date.UTC(y, m - 1, d);
};

/** Whole days from `from` to `to` ("YYYY-MM-DD" both). 0 = same day. */
export function daysBetween(from: string, to: string): number {
  return Math.round((toUtc(to) - toUtc(from)) / DAY_MS);
}

/** Today's date in the viewer's local calendar, as "YYYY-MM-DD". */
export function localIsoDate(now: Date = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** The bills due within `days` days of `today` (inclusive), soonest first, at most `limit`. */
export function nextBills(bills: Bill[], today: string, days = 7, limit = 3): Bill[] {
  return bills
    .filter((b) => {
      const d = daysBetween(today, b.date);
      return d >= 0 && d <= days;
    })
    .sort((a, b) => a.date.localeCompare(b.date))
    .slice(0, limit);
}

export interface BillWeek {
  key: string;
  label: string;
  bills: Bill[];
}

/**
 * Groups bills into weeks counted from today: "Today and this week", "Next week", then
 * "Week of 24 Nov". Within a group bills keep their date order.
 */
export function groupBillsByWeek(bills: Bill[], today: string): BillWeek[] {
  const weeks = new Map<number, Bill[]>();
  const sorted = [...bills].sort((a, b) => a.date.localeCompare(b.date));
  for (const b of sorted) {
    const d = daysBetween(today, b.date);
    if (d < 0) continue;
    const idx = Math.floor(d / 7);
    weeks.set(idx, [...(weeks.get(idx) ?? []), b]);
  }
  return [...weeks.entries()]
    .sort(([a], [b]) => a - b)
    .map(([idx, list]) => {
      const startOffset = idx * 7;
      const start = new Date(toUtc(today) + startOffset * DAY_MS).toISOString().slice(0, 10);
      const label =
        idx === 0 ? "This week" : idx === 1 ? "Next week" : `Week of ${formatDay(start)}`;
      return { key: `w${idx}`, label, bills: list };
    });
}

/** "₹2,000 of ₹5,000 moved" for a card bill that is partly covered, else null. */
export function coveredLine(bill: Bill, format: (paise: number) => string): string | null {
  if (bill.kind !== "card_bill" || bill.covered_paise <= 0) return null;
  return `${format(bill.covered_paise)} of ${format(bill.amount_paise)} moved`;
}

/** Share of a target reached, 0-100, never NaN or above 100. */
export function progressPercent(saved: number, target: number): number {
  if (target <= 0) return 0;
  return Math.min(100, Math.max(0, Math.round((saved * 100) / target)));
}

export const GOAL_STATUS_LABEL: Record<GoalStatus, string> = {
  done: "Done",
  on_track: "On track",
  behind: "Behind",
  open: "No deadline",
};

/** Tone classes for a goal status chip, light and dark. */
export const GOAL_STATUS_TONE: Record<GoalStatus, string> = {
  done: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
  on_track: "bg-sky-500/15 text-sky-700 dark:text-sky-300",
  behind: "bg-amber-500/15 text-amber-700 dark:text-amber-300",
  open: "bg-muted text-muted-foreground",
};

/** 42 -> "4.2 months", 10 -> "1 month". Null when the savings cover no months to show. */
export function monthsCoveredLabel(tenths: number | null): string | null {
  if (tenths === null) return null;
  const months = tenths / 10;
  return `${months.toFixed(1)} ${months === 1 ? "month" : "months"}`;
}

/**
 * A percentage typed by the user ("12.5", "12.5%") to basis points (1250). Null when it is not a
 * plain number from 0 to 100 with at most two decimals.
 */
export function percentToBps(text: string): number | null {
  const t = text.trim().replace(/%$/, "").trim();
  if (!/^\d{1,3}(\.\d{1,2})?$/.test(t)) return null;
  const bps = Math.round(Number(t) * 100);
  return bps <= 10_000 ? bps : null;
}

/** 1250 -> "12.5%", for pre-filling the rate box. */
export function bpsToPercentInput(bps: number): string {
  return String(bps / 100);
}

/** Sinking-fund progress, plus the plain sentence shown under it. */
export function sinkingFundLine(
  fund: { saved_paise: number; amount_paise: number },
  format: (paise: number) => string,
): string {
  return `${format(fund.saved_paise)} set aside of ${format(fund.amount_paise)}`;
}

/** Pure: turns the form into the request body, or the message to show. */
export function buildAffordPay(
  mode: "now" | "emi",
  emi: { months: string; rate: string; fee: string },
): { pay: AffordPay } | { error: string } {
  if (mode === "now") return { pay: { mode: "now" } };
  const months = Number(emi.months);
  if (!Number.isInteger(months) || months < 1 || months > 120) {
    return { error: "Months must be a whole number from 1 to 120." };
  }
  const rate_bps = percentToBps(emi.rate || "0");
  if (rate_bps === null) return { error: "Interest must be a percentage like 14 or 14.5." };
  const fee = emi.fee.trim() ? toPaise(emi.fee) : 0;
  if (fee === null || fee < 0) return { error: "The processing fee must be zero or more." };
  return { pay: { mode: "emi", months, rate_bps, fee_paise: fee } };
}
