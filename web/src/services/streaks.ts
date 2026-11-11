import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

export interface Streak {
  current: number;
  best: number;
}

export interface StreaksOut {
  /** Days in a row with a transaction logged. */
  logging: Streak;
  /** Months in a row with income, and spending no more than it. */
  saved: Streak;
  /** Months in a row with a date; null outside a couple. */
  dates: Streak | null;
}

export const getStreaks = () => api.get<StreaksOut>("/api/dashboard/streaks").then((r) => r.data);

/** Logging a transaction, a statement or a date all move a streak, so it shares the dashboard refresh cycle. */
export const streaksKey = ["streaks"] as const;

export const useStreaks = () => useQuery({ queryKey: streaksKey, queryFn: getStreaks, retry: 1 });
