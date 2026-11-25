import { Gift } from "lucide-react";

import type { Reward } from "@/services/rewards";
import { STATE_LABEL, triggerLabel } from "@/utils/rewards";
import { cn } from "@/utils/cn";

// Earned glows in party colours; claimed is a quiet grape; locked waits in grey with a dashed edge.
const TONE = {
  locked: "border border-dashed border-border bg-muted text-muted-foreground",
  earned:
    "bg-[linear-gradient(110deg,hsl(var(--c-sun)/0.3),hsl(var(--c-coral)/0.25),hsl(var(--c-rose)/0.25))] text-[hsl(var(--foreground))] shadow-[0_4px_14px_-6px_hsl(var(--c-coral))] dark:text-[hsl(var(--foreground))]",
  claimed: "pill-grape",
} as const;

/** Small "reward earned / claimed / locked" tags for a goal or a streak. Nothing when none hang on it. */
export function RewardChips({
  rewards,
  goalName,
}: {
  rewards: Reward[];
  goalName?: string;
}) {
  if (rewards.length === 0) return null;
  return (
    <ul className="m-0 flex list-none flex-wrap gap-2 p-0" aria-label="Rewards">
      {rewards.map((r) => (
        <li
          key={r.reward_id}
          title={triggerLabel(r, goalName)}
          className={cn(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium",
            TONE[r.state],
          )}
        >
          <Gift
            className={cn(
              "h-3.5 w-3.5",
              r.state === "earned" && "[animation:wiggle_2.6s_ease-in-out_infinite]",
            )}
            aria-hidden
          />
          {r.name}: {STATE_LABEL[r.state].toLowerCase()}
        </li>
      ))}
    </ul>
  );
}
