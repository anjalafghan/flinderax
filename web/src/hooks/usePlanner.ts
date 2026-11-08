import { useMutation, useQuery, useQueryClient, type QueryKey } from "@tanstack/react-query";
import { toast } from "sonner";

import { accountApi, cardPlanApi, dashboardApi, getDashboardHistory, ingestApi, joyApi, movesApi, planApi } from "@/services/api";
import type { JoyConnect } from "@/services/types";

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
export const INGEST_INVALIDATE: QueryKey[] = [keys.txns, keys.spendSummary, keys.ingestRules];
export const useJoySummary = () => useQuery({ queryKey: keys.joy, queryFn: joyApi.summary, retry: 1 });
/** Joy writes can re-label transactions as dates, so the spending data is refreshed too. */
export const JOY_INVALIDATE: QueryKey[] = [keys.joy, keys.txns, keys.spendSummary];
export const useJoyConnect = () =>
  usePlannerMutation((c: JoyConnect) => joyApi.connect(c), JOY_INVALIDATE, { success: "Joy connected" });
export const useJoySetPartner = () =>
  usePlannerMutation((partner_paise: number) => joyApi.setPartner(partner_paise), JOY_INVALIDATE, {
    success: "Partner amount saved",
  });
export const useJoyDisconnect = () =>
  usePlannerMutation(() => joyApi.disconnect(), JOY_INVALIDATE, { success: "Joy disconnected" });
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
