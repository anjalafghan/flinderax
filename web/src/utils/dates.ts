const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

/** "2026-10-30" -> "30 Oct". Parsed from the string so the viewer's timezone can't shift it. */
export function formatDay(iso: string): string {
  const [, m, d] = iso.split("-").map(Number);
  return `${d} ${MONTHS[m - 1]}`;
}

/** Whole days between an RFC 3339 / SQLite timestamp and now (0 = today). */
export function daysAgo(ts: string, now: Date = new Date()): number {
  const t = new Date(
    ts.includes("T") ? ts : ts.replace(" ", "T") + "Z",
  ).getTime();
  return Math.max(0, Math.floor((now.getTime() - t) / 86_400_000));
}

/** 1 -> "1st", 22 -> "22nd", 13 -> "13th". */
export function ordinal(n: number): string {
  const v = n % 100;
  if (v >= 11 && v <= 13) return `${n}th`;
  return `${n}${["th", "st", "nd", "rd"][n % 10] ?? "th"}`;
}

/** "2026-11" for this month, on this device. */
export function thisYearMonth(now: Date = new Date()): string {
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

/** "2026-12" plus 4 months is "2027-04". */
export function addMonths(ym: string, k: number): string {
  const [y, m] = ym.split("-").map(Number);
  const mi = y * 12 + (m - 1) + k;
  return `${Math.floor(mi / 12)}-${String((mi % 12) + 1).padStart(2, "0")}`;
}

/** "2027-04" -> "Apr 2027". */
export function formatYearMonth(ym: string): string {
  const [y, m] = ym.split("-").map(Number);
  return `${["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"][m - 1]} ${y}`;
}

/**
 * For a payment every few months saved up from `start`: the month to buy it first. A perfume every
 * 4 months saved from December (December, January, February, March) is bought in April.
 */
export function firstBuyMonth(start: string, everyMonths: number): string {
  return addMonths(start, everyMonths);
}
