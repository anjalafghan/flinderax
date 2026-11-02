// Pure helpers for the spending (ingest) pages. Money stays in integer paise; formatting goes
// through `formatINR` from ./money.

import { formatINR } from "@/utils/money";

/** Category ids are fixed by the server. Order is the order shown in the category <select>. */
export const CATEGORY_IDS: readonly string[] = [
  "food",
  "groceries",
  "shopping",
  "transport",
  "fuel",
  "bills",
  "entertainment",
  "health",
  "travel",
  "rent",
  "emi",
  "cash",
  "transfer",
  "income",
  "other",
  "uncategorized",
];

export const CATEGORY_LABEL: Record<string, string> = {
  food: "Food & dining",
  groceries: "Groceries",
  shopping: "Shopping",
  transport: "Transport",
  fuel: "Fuel",
  bills: "Bills & utilities",
  entertainment: "Entertainment",
  health: "Health",
  travel: "Travel",
  rent: "Rent",
  emi: "EMI & loans",
  cash: "Cash withdrawals",
  transfer: "Transfers (not spending)",
  income: "Income",
  other: "Other",
  uncategorized: "Uncategorized",
};

/** Transfers and income are money moving, not spending. */
const NOT_SPENDING = new Set(["transfer", "income"]);

export function categoryLabel(id: string): string {
  return CATEGORY_LABEL[id] ?? id;
}

export function isSpendingCategory(id: string): boolean {
  return !NOT_SPENDING.has(id);
}

export interface CategoryAmount {
  category: string;
  amount_paise: number;
}

/** Spending categories with money in them, largest first. Transfers and income are dropped. */
export function spendingBreakdown(rows: CategoryAmount[]): CategoryAmount[] {
  return rows
    .filter((r) => isSpendingCategory(r.category) && r.amount_paise > 0)
    .sort((a, b) => b.amount_paise - a.amount_paise);
}

/** Share of a total as a whole percent. Tiny non-zero shares read "<1%" rather than "0%". */
export function formatShare(part: number, total: number): string {
  if (total <= 0 || part <= 0) return "0%";
  const pct = (part / total) * 100;
  if (pct < 1) return "<1%";
  return `${Math.round(pct)}%`;
}

/** Bar width from 0 to 100, relative to the largest value so the top bar fills its track. */
export function barWidth(value: number, max: number): number {
  if (max <= 0 || value <= 0) return 0;
  return Math.min(100, (value / max) * 100);
}

const MONTH_NAMES = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

/** Local "YYYY-MM" for a date. */
export function monthKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

/** Local "YYYY-MM-DD" for a date (the default date for a manual entry). */
export function dateKey(d: Date): string {
  return `${monthKey(d)}-${String(d.getDate()).padStart(2, "0")}`;
}

/** "2026-11" shifted by whole months: shiftMonth("2026-01", -1) = "2025-12". */
export function shiftMonth(ym: string, delta: number): string {
  const [y, m] = ym.split("-").map(Number);
  const index = y * 12 + (m - 1) + delta;
  const year = Math.floor(index / 12);
  const month = index - year * 12 + 1;
  return `${year}-${String(month).padStart(2, "0")}`;
}

/** "2026-11" -> "November 2026". Parsed from the string so the viewer's timezone can't shift it. */
export function monthTitle(ym: string): string {
  const [y, m] = ym.split("-").map(Number);
  return `${MONTH_NAMES[m - 1]} ${y}`;
}

/** "2026-11" -> "Nov". */
export function monthShort(ym: string): string {
  const [, m] = ym.split("-").map(Number);
  return MONTH_NAMES[m - 1].slice(0, 3);
}

export interface SignedTxn {
  direction: "debit" | "credit";
  amount_paise: number;
}

/** Debits read as money out (−), credits as money in (+). */
export function signedAmount(t: SignedTxn): string {
  return `${t.direction === "debit" ? "−" : "+"}${formatINR(t.amount_paise)}`;
}

const SOURCE_LABEL: Record<string, string> = {
  sms: "SMS",
  email: "Email",
  paste: "Pasted",
  manual: "Manual",
};

export function sourceLabel(source: string): string {
  return SOURCE_LABEL[source] ?? source;
}

/** "HDFC Credit Card ••1234", or whichever parts are known. Null when neither is known. */
export function sourceDetail(sourceName: string | null, last4: string | null): string | null {
  if (sourceName && last4) return `${sourceName} ••${last4}`;
  if (sourceName) return sourceName;
  if (last4) return `••${last4}`;
  return null;
}

/** Plural-aware count: 1 "transaction", 2 "transactions". */
export function plural(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

/** The path the phone posts alerts to, relative to the site origin. */
export const INGEST_WEBHOOK_PATH = "/api/hook/ingest";
