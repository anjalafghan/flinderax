import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { RewardChips } from "@/components/feature/RewardChips";
import { RewardsPanel } from "@/components/feature/RewardsPanel";
import type { Reward, RewardsOut } from "@/services/rewards";
import { forGoal, forStreak, triggerLabel } from "@/utils/rewards";

// Synthetic data only.
const reward = (o: Partial<Reward> & { reward_id: string; name: string }): Reward => ({
  cost_paise: 50000,
  trigger_kind: "goal",
  goal_id: "g1",
  goal_pct: 100,
  streak_kind: null,
  streak_target: null,
  state: "locked",
  claimed_on: null,
  ...o,
});

const out: RewardsOut = {
  rewards: [
    reward({ reward_id: "a", name: "Dinner out", state: "earned" }),
    reward({ reward_id: "b", name: "New book", goal_pct: 50, state: "claimed", claimed_on: "2026-11-03" }),
    reward({ reward_id: "c", name: "Cake", trigger_kind: "streak", goal_id: null, goal_pct: null, streak_kind: "logging", streak_target: 7, cost_paise: 0 }),
  ],
  history: [{ reward_id: "b", name: "New book", cost_paise: 50000, claimed_on: "2026-11-03" }],
  monthly_cap_paise: 100000,
  spent_this_month_paise: 50000,
  cap_left_paise: 50000,
};

function withData(ui: React.ReactElement) {
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(["rewards"], out);
  qc.setQueryData(["goals"], { goals: [{ id: "g1", name: "Trip" }], available_paise: 0, unallocated_paise: 0, months_covered_tenths: null, emergency_suggested_paise: null });
  return render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>);
}

describe("rewards", () => {
  it("says what unlocks a reward in plain words", () => {
    expect(triggerLabel(out.rewards[0]!, "Trip")).toBe("Finish Trip");
    expect(triggerLabel(out.rewards[1]!, "Trip")).toBe("Reach 50% of Trip");
    expect(triggerLabel(out.rewards[2]!)).toBe("7 days in a row of keeping up");
  });

  it("picks out the rewards that hang on a goal or a streak", () => {
    expect(forGoal(out.rewards, "g1").map((r) => r.name)).toEqual(["Dinner out", "New book"]);
    expect(forGoal(out.rewards, "other")).toEqual([]);
    expect(forStreak(out.rewards, "logging").map((r) => r.name)).toEqual(["Cake"]);
  });

  it("shows earned, claimed and locked tags, and nothing when there are none", () => {
    const { unmount } = withData(<RewardChips rewards={out.rewards} goalName="Trip" />);
    expect(screen.getByText("Dinner out: earned")).toBeTruthy();
    expect(screen.getByText("New book: claimed")).toBeTruthy();
    expect(screen.getByText("Cake: locked")).toBeTruthy();
    unmount();
    withData(<RewardChips rewards={[]} />);
    expect(screen.queryByRole("list", { name: "Rewards" })).toBeNull();
  });

  it("lists rewards with a claim button only on the earned one, plus the budget and history", () => {
    withData(<RewardsPanel />);
    expect(screen.getAllByRole("button", { name: /Claim it/ })).toHaveLength(1);
    expect(screen.getByRole("button", { name: "Undo claim" })).toBeTruthy();
    expect(screen.getByText(/claimed this month/)).toHaveTextContent("₹500");
    expect(screen.getByText("Claimed so far (1)")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Add a reward" })).toBeTruthy();
  });
});
