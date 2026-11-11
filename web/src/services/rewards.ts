import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import api from "@/services/api";
import type { StreakKind } from "@/utils/streaks";

export type RewardState = "locked" | "earned" | "claimed";

export interface Reward {
  reward_id: string;
  name: string;
  cost_paise: number;
  trigger_kind: "goal" | "streak";
  goal_id: string | null;
  /** 25, 50, 75 or 100 (the goal is done). */
  goal_pct: number | null;
  streak_kind: StreakKind | null;
  streak_target: number | null;
  state: RewardState;
  claimed_on: string | null;
}

export interface RewardHistory {
  reward_id: string;
  name: string;
  cost_paise: number;
  claimed_on: string;
}

export interface RewardsOut {
  rewards: Reward[];
  history: RewardHistory[];
  monthly_cap_paise: number | null;
  spent_this_month_paise: number;
  cap_left_paise: number | null;
}

export interface RewardSuggestion {
  name: string;
  cost_paise: number;
  note: string;
}

export interface RewardInput {
  name: string;
  cost_paise: number;
  trigger_kind: "goal" | "streak";
  goal_id?: string;
  goal_pct?: number;
  streak_kind?: StreakKind;
  streak_target?: number;
}

export const rewardsKey = ["rewards"] as const;

const data = <T>(p: Promise<{ data: T }>) => p.then((r) => r.data);

export const rewardsApi = {
  list: () => data(api.get<RewardsOut>("/api/rewards")),
  suggestions: () => data(api.get<RewardSuggestion[]>("/api/rewards/suggestions")),
  create: (r: RewardInput) => data(api.post<{ reward_id: string }>("/api/rewards/create", r)),
  remove: (reward_id: string) => data(api.post("/api/rewards/remove", { reward_id })),
  claim: (reward_id: string) => data(api.post("/api/rewards/claim", { reward_id })),
  unclaim: (reward_id: string) => data(api.post("/api/rewards/unclaim", { reward_id })),
  setCap: (monthly_cap_paise: number | null) => data(api.post("/api/rewards/cap", { monthly_cap_paise })),
};

export const useRewardSuggestions = () => useQuery({ queryKey: [...rewardsKey, "suggestions"], queryFn: rewardsApi.suggestions, retry: 1 });

export const useRewards = () => useQuery({ queryKey: rewardsKey, queryFn: rewardsApi.list, retry: 1 });

function useRewardMutation<V>(fn: (v: V) => Promise<unknown>, success: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      toast.success(success);
      void qc.invalidateQueries({ queryKey: rewardsKey });
    },
    onError: (e: unknown) => toast.error(rewardError(e)),
  });
}

/** The server's plain-language refusal when it sent one (e.g. over the monthly budget). */
export function rewardError(e: unknown): string {
  const body = (e as { response?: { data?: unknown } })?.response?.data;
  if (typeof body === "string" && body.trim()) return body;
  return e instanceof Error ? e.message : "Something went wrong. Try again.";
}

export const useAddReward = () => useRewardMutation((r: RewardInput) => rewardsApi.create(r), "Reward added");
export const useRemoveReward = () => useRewardMutation((id: string) => rewardsApi.remove(id), "Reward removed");
export const useClaimReward = () => useRewardMutation((id: string) => rewardsApi.claim(id), "Enjoy it, you earned it!");
export const useUnclaimReward = () => useRewardMutation((id: string) => rewardsApi.unclaim(id), "Claim taken back");
export const useSetRewardCap = () => useRewardMutation((cap: number | null) => rewardsApi.setCap(cap), "Reward budget saved");
