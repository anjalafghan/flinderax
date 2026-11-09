import { useQuery } from "@tanstack/react-query";

import { INGEST_INVALIDATE, usePlannerMutation } from "@/hooks/usePlanner";
import api from "@/services/api";

// Uncategorised payees, grouped from the statement data. Money is integer paise.

export interface PayeeRow {
  payee: string;
  count: number;
  paise: number;
  /** "YYYY-MM-DD" */
  last_on: string;
  share_bps: number;
  /** A category id the server thinks fits, or null. */
  suggestion: string | null;
  /** A masked narration from one of the payments. */
  sample: string;
}

export interface PayeesOut {
  total_uncategorized_paise: number;
  covered_paise: number;
  payees: PayeeRow[];
}

export interface CategorizeIn {
  payee: string;
  category: string;
  remember: boolean;
}

export interface CategorizeOut {
  updated: number;
}

export const PAYEES_KEY = ["payees"] as const;

export const getPayees = () =>
  api.get<PayeesOut>("/api/ingest/payees?status=uncategorized&days=365&limit=40").then((r) => r.data);

export const categorizePayee = (c: CategorizeIn) =>
  api.post<CategorizeOut>("/api/ingest/payees/categorize", c).then((r) => r.data);

export const usePayees = () => useQuery({ queryKey: [...PAYEES_KEY], queryFn: getPayees, retry: 1 });

/** Tagging a payee re-categorises its past and future payments, so every spending view is refreshed. */
export const useCategorizePayee = () =>
  usePlannerMutation(categorizePayee, [
    [...PAYEES_KEY],
    ["analytics"],
    ["habits"],
    ["insights"],
    ...INGEST_INVALIDATE,
  ]);
