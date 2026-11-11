import { useMutation, useQuery, useQueryClient, type QueryKey } from "@tanstack/react-query";
import { toast } from "sonner";

import {
  accountApi,
  affordApi,
  billsApi,
  cardPlanApi,
  dashboardApi,
  getDashboardHistory,
  goalsApi,
  ingestApi,
  joyApi,
  movesApi,
  planApi,
  reviewApi,
} from "@/services/api";
import type { AffordRequest, GoalInput, NewCommitment, NewTaxEntry, TaxSectionId } from "@/services/types";

export const keys = {
  txns: ["ingest-transactions"] as const,
  spendSummary: ["ingest-summary"] as const,
  ingestKey: ["ingest-key"] as const,
  ingestRules: ["ingest-rules"] as const,
  summary: ["dashboard-summary"] as const,
  accounts: ["accounts"] as const,
  commitments: ["commitments"] as const,
  incomes: ["incomes"] as const,
  settings: ["plan-settings"] as const,
  moves: ["moves"] as const,
  history: ["dashboard-history"] as const,
  breakdown: (cardId: string) => ["card-breakdown", cardId] as const,
  joy: ["joy-summary"] as const,
  bills: ["bills"] as const,
  billSuggestions: ["bill-suggestions"] as const,
  goals: ["goals"] as const,
  reviewMonth: (month: string | undefined) => ["review-month", month ?? "last"] as const,
  // A prefix: every tax query (any financial year) is refreshed by a tax write.
  tax: ["review-tax"] as const,
  taxYear: (fy: string | undefined) => ["review-tax", fy ?? "current"] as const,
};

export const useSummary = () => useQuery({ queryKey: keys.summary, queryFn: dashboardApi.summary });
export const useAccounts = () => useQuery({ queryKey: keys.accounts, queryFn: accountApi.list });
export const useCommitments = () => useQuery({ queryKey: keys.commitments, queryFn: planApi.commitments });
export const useIncomes = () => useQuery({ queryKey: keys.incomes, queryFn: planApi.incomes });
export const useMoves = () => useQuery({ queryKey: keys.moves, queryFn: movesApi.list });
export const useDashboardHistory = (months = 12) =>
  useQuery({
    queryKey: [...keys.history, months],
    queryFn: () => getDashboardHistory(months),
    retry: 1,
  });
export const useTransactions = (month: string) =>
  useQuery({ queryKey: [...keys.txns, month], queryFn: () => ingestApi.transactions(month) });
export const useSpendSummary = (months = 6) =>
  useQuery({
    queryKey: [...keys.spendSummary, months],
    queryFn: () => ingestApi.summary(months),
    retry: 1,
  });
export const useIngestKey = () => useQuery({ queryKey: keys.ingestKey, queryFn: ingestApi.keyStatus });
export const useIngestRules = () => useQuery({ queryKey: keys.ingestRules, queryFn: ingestApi.rules });
/** Everything a spending write can change. The dashboard summary and history are added by usePlannerMutation. */
export const INGEST_INVALIDATE: QueryKey[] = [keys.txns, keys.spendSummary, keys.ingestRules, ['streaks']];
export const useJoySummary = () => useQuery({ queryKey: keys.joy, queryFn: joyApi.summary, retry: 1 });
/** Joy writes can re-label transactions as dates, so the spending data is refreshed too. */
export const JOY_INVALIDATE: QueryKey[] = [keys.joy, keys.txns, keys.spendSummary];
export const useJoySetPartner = () =>
  usePlannerMutation((partner_paise: number) => joyApi.setPartner(partner_paise), JOY_INVALIDATE, {
    success: "Partner amount saved",
  });

// Bills and goals. Adding a suggestion also adds a fixed payment, so the commitments are refreshed.
export const useBills = (days = 30) =>
  useQuery({ queryKey: [...keys.bills, days], queryFn: () => billsApi.list(days), retry: 1 });
export const useBillSuggestions = () =>
  useQuery({ queryKey: keys.billSuggestions, queryFn: billsApi.suggestions, retry: 1 });
export const useAddSuggestion = () =>
  usePlannerMutation(
    (commitment: NewCommitment) => planApi.createCommitment(commitment),
    [keys.bills, keys.billSuggestions, keys.commitments],
    { success: "Added to your fixed payments" },
  );
export const useDismissSuggestion = () =>
  usePlannerMutation((merchant: string) => billsApi.dismiss(merchant), [keys.billSuggestions], {
    success: "Got it. We won't suggest it again.",
  });

export const useGoals = () => useQuery({ queryKey: keys.goals, queryFn: goalsApi.list, retry: 1 });
const GOAL_INVALIDATE: QueryKey[] = [keys.goals];
export const useSaveGoal = () =>
  usePlannerMutation(
    (g: GoalInput) => (g.goal_id ? goalsApi.update({ ...g, goal_id: g.goal_id }) : goalsApi.create(g)),
    GOAL_INVALIDATE,
    { success: "Goal saved" },
  );
export const useArchiveGoal = () =>
  usePlannerMutation((goal_id: string) => goalsApi.archive(goal_id), GOAL_INVALIDATE, { success: "Goal archived" });
export const useContribute = () =>
  usePlannerMutation(
    (v: { goal_id: string; amount_paise: number }) => goalsApi.contribute(v.goal_id, v.amount_paise),
    GOAL_INVALIDATE,
    { success: "Saved" },
  );

/** A read-only check, so it does not refresh anything. */
export const useAfford = () =>
  useMutation({
    mutationFn: (r: AffordRequest) => affordApi.check(r),
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : "Could not check that"),
  });

export const useSettings = () => useQuery({ queryKey: keys.settings, queryFn: planApi.settings });
export const useBreakdown = (cardId: string | undefined) =>
  useQuery({
    queryKey: keys.breakdown(cardId ?? ""),
    queryFn: () => cardPlanApi.breakdown(cardId!),
    enabled: !!cardId,
  });

/**
 * A write that refreshes the given query keys and the dashboard summary (every planner input
 * feeds Safe to Spend and the history, so both are always stale after a write).
 */
export function usePlannerMutation<TVars, TResult>(
  fn: (vars: TVars) => Promise<TResult>,
  invalidate: QueryKey[],
  messages: { success?: string; error?: string } = {},
) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      for (const key of [...invalidate, keys.summary, keys.history]) qc.invalidateQueries({ queryKey: key });
      if (messages.success) toast.success(messages.success);
    },
    onError: (e: unknown) => {
      toast.error(messages.error ?? (e instanceof Error ? e.message : "Something went wrong"));
    },
  });
}

// Review: month close (read-only) and tax (writes refresh every tax query).
export const useReviewMonth = (month: string | undefined) =>
  useQuery({ queryKey: keys.reviewMonth(month), queryFn: () => reviewApi.month(month), retry: false });

export const useTaxReview = (fy?: string) =>
  useQuery({ queryKey: keys.taxYear(fy), queryFn: () => reviewApi.tax(fy), retry: 1 });

/** Tax writes only change tax data, so this refreshes the tax queries and nothing else. */
function useTaxMutation<TVars, TResult>(fn: (vars: TVars) => Promise<TResult>, success?: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: keys.tax });
      if (success) toast.success(success);
    },
    onError: (e: unknown) => toast.error(e instanceof Error ? e.message : "Something went wrong"),
  });
}

export const useAddTaxEntry = () =>
  useTaxMutation((e: NewTaxEntry) => reviewApi.addTaxEntry(e), "Deduction added");
export const useDeleteTaxEntry = () =>
  useTaxMutation((entry_id: string) => reviewApi.deleteTaxEntry(entry_id), "Deduction removed");
export const useSetCommitmentTaxSection = () =>
  useTaxMutation(
    (v: { commitment_id: string; tax_section: TaxSectionId | null }) =>
      reviewApi.setCommitmentTaxSection(v.commitment_id, v.tax_section),
    "Saved",
  );
