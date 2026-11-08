import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

// Mirrors InsightsOut in the life-plan API contract (section 2). Money is integer paise.

export interface InsightRefund {
  txn_id: string;
  occurred_on: string;
  amount_paise: number;
  merchant: string | null;
  rail: "p2p" | "p2m" | "unknown";
  due_by: string;
  days_overdue: number;
  compensation_paise: number;
  status: "waiting" | "overdue";
}

export interface InsightCharges {
  last_12m_paise: number;
  items: { label: string; amount_paise: number; count: number }[];
}

export interface InsightDuplicate {
  merchant: string;
  amount_paise: number;
  occurred_on: string;
  count: number;
  txn_ids: string[];
}

export interface InsightUnusual {
  category: string;
  month: string;
  amount_paise: number;
  typical_paise: number;
}

export interface InsightHealth {
  score: number;
  savings_rate_bps: number | null;
  emergency_months_tenths: number | null;
  income_cv_bps: number | null;
  actions: { text: string; impact: "high" | "medium" | "low" }[];
}

export interface SpendToPayday {
  days: number;
  p10_paise: number;
  p50_paise: number;
  p90_paise: number;
}

export interface InsightsOut {
  refunds: InsightRefund[];
  charges: InsightCharges;
  duplicates: InsightDuplicate[];
  unusual: InsightUnusual[];
  health: InsightHealth;
  spend_to_payday: SpendToPayday | null;
}

export const getInsights = () => api.get<InsightsOut>("/api/dashboard/insights").then((r) => r.data);

/** Insights are derived from every transaction, so they refresh with the rest of the planner data. */
export const useInsights = () => useQuery({ queryKey: ["insights"], queryFn: getInsights, retry: 1 });
