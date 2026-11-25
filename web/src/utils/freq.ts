// Mirrors `Freq::from_columns` in src/planner/mod.rs. A commitment's cadence is stored as
// `frequency` + `every_months` + `first_month`; read it here, never by comparing the strings,
// so the page agrees with the planner, bills, year view and auto-post.

export type Freq =
  | { kind: "monthly" }
  | { kind: "yearly" }
  /** Every `months` (2-11) months, counting from `first` ("YYYY-MM"). */
  | { kind: "every"; months: number; first: string };

export interface FreqColumns {
  frequency: string;
  every_months: number;
  first_month: string | null;
}

/** True for "YYYY-MM" with a month of 1-12 (the server's `parse_year_month`). */
function isYearMonth(s: string | null): s is string {
  const m = s === null ? null : /^(\d+)-(\d+)$/.exec(s);
  return m !== null && Number(m[2]) >= 1 && Number(m[2]) <= 12;
}

/**
 * "yearly" is always yearly. A monthly row with 2-11 `every_months` and a readable
 * `first_month` is "every"; anything else falls back to plain monthly (the safe side).
 */
export function freqOf(row: FreqColumns): Freq {
  if (row.frequency === "yearly") return { kind: "yearly" };
  if (
    Number.isInteger(row.every_months) &&
    row.every_months >= 2 &&
    row.every_months <= 11 &&
    isYearMonth(row.first_month)
  ) {
    return { kind: "every", months: row.every_months, first: row.first_month };
  }
  return { kind: "monthly" };
}
