// Pure helpers for the At a glance page. Money is integer paise; nothing here uses float money
// except to scale a bar.

import type { HistoryView } from "@/joy/kept/types";
import type { AnalyticsMonth } from "@/services/analytics";
import type { Milestone } from "@/services/road";
import type { YearView } from "@/services/year";
import { toPaise } from "@/utils/money";

const LAKH = 100_000 * 100;
const CRORE = 100 * LAKH;
const THOUSAND = 1_000 * 100;

/** Axis and bar labels: 1234500 -> "₹12k", 15000000 -> "₹1.5L", 2500000000 -> "₹2.5Cr". */
export function shortINR(paise: number): string {
  const sign = paise < 0 ? "−" : "";
  const p = Math.abs(paise);
  const one = (v: number) =>
    Number.isInteger(v) ? String(v) : v.toFixed(1).replace(/\.0$/, "");
  if (p >= CRORE) return `${sign}₹${one(Math.round((p / CRORE) * 10) / 10)}Cr`;
  if (p >= LAKH) return `${sign}₹${one(Math.round((p / LAKH) * 10) / 10)}L`;
  if (p >= THOUSAND) return `${sign}₹${Math.round(p / THOUSAND)}k`;
  return `${sign}₹${Math.round(p / 100)}`;
}

/** "2026-04" -> "Apr"; with `year`, "Apr '26". */
export function monthShort(ym: string, year = false): string {
  const [y, m] = ym.split("-").map(Number);
  const name = new Date(Date.UTC(y, (m || 1) - 1, 1)).toLocaleString("en-IN", {
    month: "short",
    timeZone: "UTC",
  });
  return year ? `${name} '${String(y).slice(2)}` : name;
}

/** "2026-12-10" -> "10 Dec". */
export function dayShort(day: string): string {
  const [y, m, d] = day.slice(0, 10).split("-").map(Number);
  return new Date(Date.UTC(y, (m || 1) - 1, d || 1)).toLocaleString("en-IN", {
    day: "numeric",
    month: "short",
    timeZone: "UTC",
  });
}

/** The months worth charting: from the first month with any spending or investing, at most `max`. */
export function chartMonths(
  months: AnalyticsMonth[],
  max = 12,
): AnalyticsMonth[] {
  const first = months.findIndex(
    (m) => m.spend_paise > 0 || m.invested_paise > 0,
  );
  if (first < 0) return [];
  return months.slice(first).slice(-max);
}

export interface OutflowMonth {
  month: string;
  fixed_paise: number;
  lumpy_paise: number;
  total_paise: number;
  items: string[];
}

/** What is already promised each month of the year view: fixed payments plus one-off amounts going out. */
export function outflowMonths(view: YearView): OutflowMonth[] {
  return view.months.map((m) => ({
    month: m.month,
    fixed_paise: m.fixed_paise,
    lumpy_paise: m.lumpy_out_paise,
    total_paise: m.fixed_paise + m.lumpy_out_paise,
    items: m.lumpy.filter((l) => l.direction === "out").map((l) => l.label),
  }));
}

export interface DatesStats {
  kept: number;
  rated: number;
  /** Average of the rated dates, one decimal, or null when none is rated. */
  average: number | null;
  spent_paise: number;
}

/** Counts every kept date in the history view, with the average rating and the real cost. */
export function datesStats(view: HistoryView): DatesStats {
  const items = view.months.flatMap((m) => m.items);
  const rated = items.filter(
    (i) => i.rating !== null && i.rating !== undefined,
  );
  const sum = rated.reduce((s, i) => s + (i.rating ?? 0), 0);
  const spent = items.reduce(
    (s, i) => s + (toPaise(i.cost_actual_raw || "") ?? 0),
    0,
  );
  return {
    kept: items.length,
    rated: rated.length,
    average: rated.length ? Math.round((sum / rated.length) * 10) / 10 : null,
    spent_paise: spent,
  };
}

export interface Getaway {
  id: string;
  title: string;
  cost_paise: number;
}

/** The jar's priced ideas ("Coorg · ₹46,100"), biggest first. */
export function getaways(view: HistoryView, max = 8): Getaway[] {
  return view.savings.choices
    .map((c) => {
      const at = c.label.lastIndexOf(" · ");
      const title = at >= 0 ? c.label.slice(0, at) : c.label;
      const cost = at >= 0 ? toPaise(c.label.slice(at + 3)) : null;
      return { id: c.value, title, cost_paise: cost ?? 0 };
    })
    .filter((g) => g.cost_paise > 0)
    .sort((a, b) => b.cost_paise - a.cost_paise)
    .slice(0, max);
}

/** This month's dates spend and budget, in paise, from the view's rupee text. */
export function jarBudget(view: HistoryView): {
  spent_paise: number;
  budget_paise: number;
} {
  return {
    spent_paise: toPaise(view.budget.spent) ?? 0,
    budget_paise: toPaise(view.budget.budget_raw || view.budget.budget) ?? 0,
  };
}

/** Milestones grouped by year, in date order. */
export function byYear(
  ms: Milestone[],
): { year: string; items: Milestone[] }[] {
  const out: { year: string; items: Milestone[] }[] = [];
  for (const m of [...ms].sort((a, b) => a.on.localeCompare(b.on))) {
    const y = m.on.slice(0, 4);
    const last = out[out.length - 1];
    if (last && last.year === y) last.items.push(m);
    else out.push({ year: y, items: [m] });
  }
  return out;
}

/** A bar's share of the largest, 0-100, never below a sliver for a non-zero value. */
export function pct(v: number, max: number): number {
  if (max <= 0 || v <= 0) return 0;
  return Math.max(2, Math.min(100, (v / max) * 100));
}

/** Three or four round ticks from 0 to just over `max` (paise). */
export function niceTicks(max: number): number[] {
  const raw = max / 3;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step =
    [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? 10 * mag;
  const out: number[] = [];
  for (let v = 0; v <= max + step * 0.999; v += step) out.push(Math.round(v));
  return out.length > 1 ? out : [0, Math.round(step)];
}
