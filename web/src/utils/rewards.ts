import type { Reward, RewardState } from "@/services/rewards";
import { unitLabel } from "@/utils/streaks";

export const STATE_LABEL: Record<RewardState, string> = {
  locked: "Locked",
  earned: "Earned",
  claimed: "Claimed",
};

/** What unlocks it, said plainly: "Reach half of Trip" / "A 7 days streak of keeping up". */
export function triggerLabel(r: Reward, goalName?: string): string {
  if (r.trigger_kind === "wish") return "From your reward wallet";
  if (r.trigger_kind === "goal") {
    const pct = r.goal_pct ?? 100;
    const goal = goalName ?? "a goal";
    return pct === 100 ? `Finish ${goal}` : `Reach ${pct}% of ${goal}`;
  }
  const what = { logging: "keeping up", saved: "saving", dates: "dates", activity: "adding your data" }[r.streak_kind ?? "logging"];
  return `${unitLabel(r.streak_kind ?? "logging", r.streak_target ?? 0)} in a row of ${what}`;
}

/** The rewards that hang on one goal, locked ones too, so the goal can show what is waiting. */
export const forGoal = (rewards: Reward[] | undefined, goalId: string) =>
  (rewards ?? []).filter((r) => r.trigger_kind === "goal" && r.goal_id === goalId);

export const forStreak = (rewards: Reward[] | undefined, kind: string) =>
  (rewards ?? []).filter((r) => r.trigger_kind === "streak" && r.streak_kind === kind);

/** "2026-10" -> "October 2026". */
export function monthName(key: string): string {
  const [y, m] = key.split("-").map(Number);
  if (!y || !m) return key;
  return new Date(y, m - 1, 1).toLocaleDateString("en-IN", { month: "long", year: "numeric" });
}

/** Basis points as a whole percent, rounded down so 99.9% never shows as 100%. */
export const pct = (bps: number) => `${Math.floor(Math.max(0, bps) / 100)}%`;

/** One plain line on what this month pays so far and why it isn't more. */
export function payoutLine(s: { earned_paise: number; saved_paise: number; payout_paise: number }, fmt: (p: number) => string): string {
  if (s.earned_paise === 0) return "Nothing earned yet. Small steps in any area start it.";
  if (s.payout_paise < s.earned_paise) {
    return s.saved_paise === 0
      ? `You've earned ${fmt(s.earned_paise)}, but it is only paid from money saved under your allowance, and nothing is saved yet.`
      : `You've earned ${fmt(s.earned_paise)}; ${fmt(s.payout_paise)} of it is covered by what you've saved so far.`;
  }
  return `${fmt(s.payout_paise)} so far, all covered by what you've saved.`;
}
