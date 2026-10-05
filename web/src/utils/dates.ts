const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** "2026-10-30" -> "30 Oct". Parsed from the string so the viewer's timezone can't shift it. */
export function formatDay(iso: string): string {
  const [, m, d] = iso.split("-").map(Number);
  return `${d} ${MONTHS[m - 1]}`;
}

/** Whole days between an RFC 3339 / SQLite timestamp and now (0 = today). */
export function daysAgo(ts: string, now: Date = new Date()): number {
  const t = new Date(ts.includes("T") ? ts : ts.replace(" ", "T") + "Z").getTime();
  return Math.max(0, Math.floor((now.getTime() - t) / 86_400_000));
}

/** 1 -> "1st", 22 -> "22nd", 13 -> "13th". */
export function ordinal(n: number): string {
  const v = n % 100;
  if (v >= 11 && v <= 13) return `${n}th`;
  return `${n}${["th", "st", "nd", "rd"][n % 10] ?? "th"}`;
}
