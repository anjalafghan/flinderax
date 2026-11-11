import { useState } from "react";
import { Gift } from "lucide-react";

import { useGoals } from "@/hooks/usePlanner";
import {
  useAddReward,
  useClaimReward,
  useRemoveReward,
  useRewards,
  useSetRewardCap,
  useUnclaimReward,
  type Reward,
  type RewardInput,
} from "@/services/rewards";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyScene } from "@/components/ui/EmptyScene";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { ResetButton } from "@/components/ui/reset-button";
import { Select } from "@/components/ui/select";
import { formatDay } from "@/utils/dates";
import { formatINR, paiseToInput, toPaise } from "@/utils/money";
import { STATE_LABEL, triggerLabel } from "@/utils/rewards";
import type { StreakKind } from "@/utils/streaks";
import { cn } from "@/utils/cn";

const STREAK_LABEL: Record<StreakKind, string> = {
  logging: "Days in a row keeping up",
  saved: "Months in a row saving",
  dates: "Months in a row with a date",
};

// ---------- add form ----------

function RewardForm({ onDone }: { onDone: () => void }) {
  const { data: goals } = useGoals();
  const add = useAddReward();
  const [name, setName] = useState("");
  const [cost, setCost] = useState("");
  const [trigger, setTrigger] = useState<"goal" | "streak">("goal");
  const [goalId, setGoalId] = useState("");
  const [pct, setPct] = useState("100");
  const [streakKind, setStreakKind] = useState<StreakKind>("logging");
  const [streakTarget, setStreakTarget] = useState("7");
  const [error, setError] = useState<string | null>(null);

  const reset = () => {
    setName("");
    setCost("");
    setTrigger("goal");
    setGoalId("");
    setPct("100");
    setStreakKind("logging");
    setStreakTarget("7");
    setError(null);
  };

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const paise = cost.trim() === "" ? 0 : toPaise(cost);
    if (!name.trim()) return setError("Give the treat a name.");
    if (paise === null || paise < 0) return setError("The cost must be zero or more.");
    const body: RewardInput = { name: name.trim(), cost_paise: paise, trigger_kind: trigger };
    if (trigger === "goal") {
      const id = goalId || goals?.goals[0]?.id;
      if (!id) return setError("Add a goal first, or pick a streak.");
      body.goal_id = id;
      body.goal_pct = Number(pct);
    } else {
      const n = Number(streakTarget);
      if (!Number.isInteger(n) || n < 1 || n > 3650) return setError("Pick a streak between 1 and 3650.");
      body.streak_kind = streakKind;
      body.streak_target = n;
    }
    setError(null);
    add.mutate(body, { onSuccess: () => { reset(); onDone(); } });
  };

  return (
    <form onSubmit={submit} className="form-grid border-t border-border pt-6" noValidate>
      <div className="field col-span-full">
        <Label htmlFor="reward-name">The treat</Label>
        <Input id="reward-name" value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. A nice dinner out" maxLength={80} />
      </div>
      <div className="field">
        <Label htmlFor="reward-cost">What it costs (optional)</Label>
        <MoneyInput id="reward-cost" value={cost} onChange={(e) => setCost(e.target.value)} />
      </div>
      <div className="field">
        <Label htmlFor="reward-trigger">Unlocked by</Label>
        <Select id="reward-trigger" value={trigger} onChange={(e) => setTrigger(e.target.value as "goal" | "streak")}>
          <option value="goal">Reaching a goal</option>
          <option value="streak">A streak</option>
        </Select>
      </div>
      {trigger === "goal" ? (
        <>
          <div className="field">
            <Label htmlFor="reward-goal">Which goal</Label>
            <Select id="reward-goal" value={goalId || goals?.goals[0]?.id || ""} onChange={(e) => setGoalId(e.target.value)}>
              {(goals?.goals ?? []).map((g) => (
                <option key={g.id} value={g.id}>{g.name}</option>
              ))}
            </Select>
          </div>
          <div className="field">
            <Label htmlFor="reward-pct">When it reaches</Label>
            <Select id="reward-pct" value={pct} onChange={(e) => setPct(e.target.value)}>
              {[25, 50, 75, 100].map((p) => (
                <option key={p} value={p}>{p === 100 ? "All of it (done)" : `${p}% of the target`}</option>
              ))}
            </Select>
          </div>
        </>
      ) : (
        <>
          <div className="field">
            <Label htmlFor="reward-streak-kind">Which streak</Label>
            <Select id="reward-streak-kind" value={streakKind} onChange={(e) => setStreakKind(e.target.value as StreakKind)}>
              {(Object.keys(STREAK_LABEL) as StreakKind[]).map((k) => (
                <option key={k} value={k}>{STREAK_LABEL[k]}</option>
              ))}
            </Select>
          </div>
          <div className="field">
            <Label htmlFor="reward-streak-target">How long</Label>
            <Input id="reward-streak-target" inputMode="numeric" value={streakTarget} onChange={(e) => setStreakTarget(e.target.value)} />
          </div>
        </>
      )}
      {error && <p role="alert" className="col-span-full text-sm text-destructive">{error}</p>}
      <div className="form-actions">
        <ResetButton onReset={reset} />
        <Button type="button" variant="ghost" onClick={onDone}>Cancel</Button>
        <Button type="submit" disabled={add.isPending}>Add reward</Button>
      </div>
    </form>
  );
}

// ---------- the budget ----------

function CapForm({ cap, spent, left }: { cap: number | null; spent: number; left: number | null }) {
  const [value, setValue] = useState(cap === null ? "" : paiseToInput(cap));
  const [error, setError] = useState<string | null>(null);
  const save = useSetRewardCap();
  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (value.trim() === "") return save.mutate(null);
    const paise = toPaise(value);
    if (paise === null || paise < 0) return setError("Type an amount of zero or more, or leave it empty for no limit.");
    setError(null);
    save.mutate(paise);
  };
  return (
    <form onSubmit={submit} className="flex flex-wrap items-end gap-3 rounded-2xl border border-border p-4" noValidate>
      <div className="field min-w-0 flex-1">
        <Label htmlFor="reward-cap">Most to spend on treats each month</Label>
        <MoneyInput id="reward-cap" value={value} onChange={(e) => setValue(e.target.value)} placeholder="No limit" />
        <p className="mt-1 text-[13px] text-muted-foreground tabular-nums">
          {formatINR(spent)} claimed this month{left !== null && ` · ${formatINR(left)} left`}
        </p>
      </div>
      <div className="flex gap-2">
        <ResetButton onReset={() => { setValue(cap === null ? "" : paiseToInput(cap)); setError(null); }} />
        <Button type="submit" size="sm" disabled={save.isPending}>Save limit</Button>
      </div>
      {error && <p role="alert" className="w-full text-sm text-destructive">{error}</p>}
    </form>
  );
}

// ---------- one reward ----------

function RewardRow({ r, goalName }: { r: Reward; goalName?: string }) {
  const claim = useClaimReward();
  const unclaim = useUnclaimReward();
  const remove = useRemoveReward();
  return (
    <li className="space-y-2 border-t border-border py-4 text-sm first:border-t-0 first:pt-1">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="truncate text-[15px] font-medium" title={r.name}>{r.name}</p>
          <p className="text-[13px] text-muted-foreground tabular-nums">
            {triggerLabel(r, goalName)}
            {r.cost_paise > 0 && ` · ${formatINR(r.cost_paise)}`}
            {r.claimed_on && ` · claimed ${formatDay(r.claimed_on)}`}
          </p>
        </div>
        <span
          className={cn(
            "shrink-0 rounded-full px-2.5 py-1 text-xs font-semibold",
            r.state === "earned" ? "bg-good-tint text-good" : "bg-muted text-muted-foreground",
          )}
        >
          {STATE_LABEL[r.state]}
        </span>
      </div>
      <div className="flex flex-wrap gap-2">
        {r.state === "earned" && (
          <Button size="sm" disabled={claim.isPending} onClick={() => claim.mutate(r.reward_id)}>
            <Gift className="mr-1.5 h-4 w-4" aria-hidden />
            Claim it
          </Button>
        )}
        {r.state === "claimed" && (
          <ResetButton label="Undo claim" disabled={unclaim.isPending} onReset={() => unclaim.mutate(r.reward_id)} />
        )}
        <Button size="sm" variant="ghost" aria-label={`Remove ${r.name}`} disabled={remove.isPending}
          onClick={() => { if (window.confirm(`Remove "${r.name}"? Anything you claimed stays in your history.`)) remove.mutate(r.reward_id); }}>
          Remove
        </Button>
      </div>
    </li>
  );
}

export function RewardsPanel() {
  const { data, isLoading, isError } = useRewards();
  const { data: goals } = useGoals();
  const [adding, setAdding] = useState(false);
  const goalName = (id: string | null) => goals?.goals.find((g) => g.id === id)?.name;

  return (
    <Card id="rewards" className="scroll-mt-24">
      <CardHeader className="space-y-2 pb-4">
        <CardTitle className="text-[32px]">Rewards</CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Small treats you pick now and unlock by reaching a goal or keeping a streak. Set a monthly limit so they never get in the goals' way.
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        {isLoading && <p className="text-sm text-muted-foreground">Loading rewards...</p>}
        {isError && <p className="text-sm text-destructive">Could not load rewards. Try again in a moment.</p>}

        {data && <CapForm key={data.monthly_cap_paise ?? "none"} cap={data.monthly_cap_paise} spent={data.spent_this_month_paise} left={data.cap_left_paise} />}

        {data && data.rewards.length === 0 && <EmptyScene scene="sprout" title="No rewards yet">Pick a treat to look forward to when a goal lands.</EmptyScene>}
        {data && data.rewards.length > 0 && (
          <ul className="ledger">
            {data.rewards.map((r) => (
              <RewardRow key={r.reward_id} r={r} goalName={goalName(r.goal_id)} />
            ))}
          </ul>
        )}

        {data && data.history.length > 0 && (
          <details className="rounded-2xl border border-border px-4 py-3 text-sm">
            <summary className="cursor-pointer font-medium">Claimed so far ({data.history.length})</summary>
            <ul className="ledger mt-2">
              {data.history.map((h) => (
                <li key={h.reward_id} className="ledger-row">
                  <span className="min-w-0 truncate font-medium">{h.name}</span>
                  <span className="text-[13px] text-muted-foreground tabular-nums">
                    {formatDay(h.claimed_on)}{h.cost_paise > 0 && ` · ${formatINR(h.cost_paise)}`}
                  </span>
                </li>
              ))}
            </ul>
          </details>
        )}

        {adding ? (
          <RewardForm onDone={() => setAdding(false)} />
        ) : (
          <div>
            <Button variant="outline" onClick={() => setAdding(true)}>Add a reward</Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
