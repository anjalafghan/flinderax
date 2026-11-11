import type { Streak } from "@/services/streaks";

export type StreakKind = "logging" | "saved" | "dates";

/** The runs that earn a small celebration, the first time you reach them. */
const MILESTONES: Record<StreakKind, readonly number[]> = {
  logging: [3, 7, 14, 30, 60, 100, 365],
  saved: [2, 3, 6, 12, 24],
  dates: [2, 3, 6, 12, 24],
};

const UNIT: Record<StreakKind, [one: string, many: string]> = {
  logging: ["day", "days"],
  saved: ["month", "months"],
  dates: ["month", "months"],
};

export function unitLabel(kind: StreakKind, n: number): string {
  return `${n} ${UNIT[kind][n === 1 ? 0 : 1]}`;
}

/** What the streak is about, said as a headline: "5 days in a row". */
export function streakHeadline(kind: StreakKind, s: Streak): string {
  if (s.current > 0) return `${unitLabel(kind, s.current)} in a row`;
  return s.best > 0 ? `Best: ${unitLabel(kind, s.best)}` : "Start your first";
}

const WHAT: Record<StreakKind, string> = {
  logging: "keeping your spending up to date",
  saved: "spending less than you earned",
  dates: "making time for a date",
};

/** One encouraging line: warm when it is going well, and never scolding when it is not. */
export function streakLine(kind: StreakKind, s: Streak): string {
  if (s.current === 0) {
    return s.best > 0 ? `You managed ${unitLabel(kind, s.best)} before. Start a new run ${kind === "logging" ? "today" : "this month"}.` : `A new run starts with ${WHAT[kind]}.`;
  }
  if (s.current >= s.best && s.current > 1) return `A new personal best for ${WHAT[kind]}!`;
  if (s.current === 1) return `A great start on ${WHAT[kind]}.`;
  return `Lovely work ${WHAT[kind]}. Your best is ${unitLabel(kind, s.best)}.`;
}

/** The milestone just reached, or null when the current run is not on one. */
export function milestoneFor(kind: StreakKind, current: number): number | null {
  return MILESTONES[kind].includes(current) ? current : null;
}

export function milestoneKey(kind: StreakKind, n: number): string {
  return `streak-seen:${kind}:${n}`;
}
