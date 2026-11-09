import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

// Mirrors AnalyticsOut in the statement contract (section "Analytics"). Money is integer paise.

export interface AnalyticsCoverage {
  first_date: string;
  last_date: string;
  imports: number;
}

export interface AnalyticsMonth {
  month: string;
  income_paise: number;
  spend_paise: number;
  invested_paise: number;
  transfers_paise: number;
  refunds_paise: number;
  net_paise: number;
}

export interface AnalyticsCategory {
  category: string;
  paise: number;
  share_bps: number;
  count: number;
}

export interface AnalyticsPayee {
  name: string;
  paise: number;
  count: number;
  kind: "shop" | "person" | "merchant";
}

export interface AnalyticsSalary {
  typical_days: number[];
  /** Short pay window: "Month-end (29th–2nd)", "Around the 7th". */
  usual: string;
  avg_paise: number;
  last_paise: number;
  last_on: string;
  missing_months: string[];
  next_expected: string;
}

export interface AnalyticsSip {
  label: string;
  amount_paise: number;
  usual_day: number;
  months_seen: number;
  last_on: string;
}

export interface AnalyticsAutopay {
  name: string;
  amount_paise: number;
  last_on: string;
  count: number;
}

export interface AnalyticsOwnTransfers {
  count: number;
  out_paise: number;
  in_paise: number;
}

export interface AnalyticsFrequentSmall {
  name: string;
  count: number;
  paise: number;
}

export interface AnalyticsOut {
  coverage: AnalyticsCoverage | null;
  months: AnalyticsMonth[];
  savings_rate_bps: number | null;
  categories: AnalyticsCategory[];
  top_payees: AnalyticsPayee[];
  small_ticket: { per_month_count: number; per_month_paise: number; threshold_paise: number };
  salary: AnalyticsSalary | null;
  sips: AnalyticsSip[];
  autopay: AnalyticsAutopay[];
  own_transfers: AnalyticsOwnTransfers | null;
  frequent_small: AnalyticsFrequentSmall[];
  weekday_spend_paise: number[];
  notes: string[];
}

export const getAnalytics = () => api.get<AnalyticsOut>("/api/dashboard/analytics").then((r) => r.data);

/** Derived from every transaction and statement import, so it refreshes with the rest of the money data. */
export const useAnalytics = () => useQuery({ queryKey: ["analytics"], queryFn: getAnalytics, retry: 1 });
