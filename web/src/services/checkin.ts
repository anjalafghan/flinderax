import { useMutation, useQuery } from "@tanstack/react-query";

import api from "@/services/api";
import type { Habit } from "@/services/habits";
import { formatINR } from "@/utils/money";

// Mirrors the check-in contract, sections A (GET /api/checkin, ...) and B (reminder feed).
// Money is integer paise, dates are "YYYY-MM-DD", percentages are basis points.

export interface Upload {
  kind: "account" | "card";
  id: string;
  name: string;
  /** Latest statement end date, or null when none has been uploaded. */
  covered_to: string | null;
  needed: boolean;
}

export interface Step {
  key: "statements" | "balances" | "payees" | "report";
  title: string;
  detail: string;
  done: boolean;
  count: number;
}

export interface Estimate {
  account_id: string;
  name: string;
  known_paise: number;
  known_on: string;
  estimate_paise: number;
  days_since: number;
  basis: string;
}

export interface CheckinStatus {
  last_done_on: string | null;
  /** A check-in was finished today; `period` is the one it covered. */
  done_today: boolean;
  next_due_on: string;
  status: "overdue" | "due_soon" | "ok";
  period: { start: string; end: string };
  uploads: Upload[];
  steps: Step[];
  estimates: Estimate[];
}

export interface Totals {
  income_paise: number;
  spend_paise: number;
  invested_paise: number;
  savings_rate_bps: number | null;
}

export interface CategoryChange {
  category: string;
  paise: number;
  prev_paise: number;
  change_bps: number | null;
}

export interface QuarterReport {
  period: { start: string; end: string };
  previous: { start: string; end: string };
  totals: Totals;
  prev: Totals | null;
  categories: CategoryChange[];
  wins: string[];
  watch: string[];
  habits: Habit[];
}

export interface CheckinHistoryItem {
  done_on: string;
  period: { start: string; end: string };
  totals: Totals;
}

export interface CheckinHistory {
  checkins: CheckinHistoryItem[];
}

export interface ReminderLink {
  url: string;
}

// Query keys: the status is the prefix of the report and history, so invalidating ["checkin"] refreshes all three.
export const checkinKey = ["checkin"] as const;
export const checkinReportKey = ["checkin", "report"] as const;
export const checkinHistoryKey = ["checkin", "history"] as const;

export const getCheckinStatus = () => api.get<CheckinStatus>("/api/checkin").then((r) => r.data);
/** No query string: the server answers for the status period, so the report and the status always agree. */
export const getCheckinReport = () => api.get<QuarterReport>("/api/checkin/report").then((r) => r.data);
export const getCheckinHistory = () => api.get<CheckinHistory>("/api/checkin/history").then((r) => r.data);
export const completeCheckin = () => api.post<CheckinStatus>("/api/checkin/complete", {}).then((r) => r.data);
/** Fetched on demand only: the first call creates the user's private feed. */
export const getReminderLink = () => api.get<ReminderLink>("/api/reminders/link").then((r) => r.data);

export const useCheckinStatus = () => useQuery({ queryKey: checkinKey, queryFn: getCheckinStatus, retry: 1 });
export const useCheckinReport = (enabled = true) =>
  useQuery({ queryKey: checkinReportKey, queryFn: getCheckinReport, retry: 1, enabled });
export const useCheckinHistory = () =>
  useQuery({ queryKey: checkinHistoryKey, queryFn: getCheckinHistory, retry: 1 });
export const useReminderLink = () => useMutation({ mutationFn: getReminderLink });
export const useCompleteCheckin = () => useMutation({ mutationFn: completeCheckin });

/**
 * True once this period has been finished: the period then starts after it ends
 * (the server moves the start to the day after the last check-in). ISO dates compare as strings.
 */
export const isSaved = (s: CheckinStatus): boolean => s.done_today;

/** Whole rupees with Indian grouping, for the planning figures on this page. */
export const rupees = (paise: number): string => formatINR(Math.round(paise / 100) * 100);

/** Whole percent ("22%"), or a dash when there is no income to measure against. */
export const rate = (bps: number | null): string => (bps === null ? "—" : `${Math.round(bps / 100)}%`);

/** A signed change: "+38%", "-12%", "0%". Null (no earlier spend) reads as "No earlier spend". */
export const signedChange = (bps: number | null): string => {
  if (bps === null) return "No earlier spend";
  const whole = Math.round(Math.abs(bps) / 100);
  if (whole === 0) return "0%";
  return `${bps > 0 ? "+" : "-"}${whole}%`;
};
