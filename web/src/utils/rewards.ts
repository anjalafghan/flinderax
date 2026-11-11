import type { Reward, RewardState } from "@/services/rewards";
import { unitLabel } from "@/utils/streaks";

export const STATE_LABEL: Record<RewardState, string> = {
  locked: "Locked",
  earned: "Earned",
  claimed: "Claimed",
};

/** What unlocks it, said plainly: "Reach half of Trip" / "A 7 days streak of keeping up". */
export function triggerLabel(r: Reward, goalName?: string): string {
  if (r.trigger_kind === "goal") {
    const pct = r.goal_pct ?? 100;
    const goal = goalName ?? "a goal";
    return pct === 100 ? `Finish ${goal}` : `Reach ${pct}% of ${goal}`;
  }
  const what = { logging: "keeping up", saved: "saving", dates: "dates" }[r.streak_kind ?? "logging"];
  return `${unitLabel(r.streak_kind ?? "logging", r.streak_target ?? 0)} in a row of ${what}`;
}

/** The rewards that hang on one goal, locked ones too, so the goal can show what is waiting. */
export const forGoal = (rewards: Reward[] | undefined, goalId: string) =>
  (rewards ?? []).filter((r) => r.trigger_kind === "goal" && r.goal_id === goalId);

export const forStreak = (rewards: Reward[] | undefined, kind: string) =>
  (rewards ?? []).filter((r) => r.trigger_kind === "streak" && r.streak_kind === kind);
