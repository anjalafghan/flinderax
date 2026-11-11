import { Gift } from "lucide-react";

import type { Reward } from "@/services/rewards";
import { STATE_LABEL, triggerLabel } from "@/utils/rewards";
import { cn } from "@/utils/cn";

const TONE = {
  locked: "bg-muted text-muted-foreground",
  earned: "bg-good-tint text-good",
  claimed: "bg-chip text-muted-foreground",
} as const;

/** Small "reward earned / claimed / locked" tags for a goal or a streak. Nothing when none hang on it. */
export function RewardChips({ rewards, goalName }: { rewards: Reward[]; goalName?: string }) {
  if (rewards.length === 0) return null;
  return (
    <ul className="m-0 flex list-none flex-wrap gap-2 p-0" aria-label="Rewards">
      {rewards.map((r) => (
        <li
          key={r.reward_id}
          title={triggerLabel(r, goalName)}
          className={cn("inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium", TONE[r.state])}
        >
          <Gift className="h-3.5 w-3.5" aria-hidden />
          {r.name}: {STATE_LABEL[r.state].toLowerCase()}
        </li>
      ))}
    </ul>
  );
}
