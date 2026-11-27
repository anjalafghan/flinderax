import { useState } from "react";
import { Gift, PartyPopper } from "lucide-react";
import { toast } from "sonner";

import { useGoals } from "@/hooks/usePlanner";
import {
  useAddReward,
  useClaimReward,
  useRemoveReward,
  useRewardSuggestions,
  useRewards,
  useSetRewardCap,
  useUnclaimReward,
  type Reward,
  type RewardInput,
  type TriggerKind,
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
import HoldButton from "@/components/bits/HoldButton";
import { MoneyCounter } from "@/components/feature/SafeToSpendHero";

/** What each state looks like: earned is a party, claimed a quiet grape, locked waits in grey. */
const STATE_TONE = {
  earned:
    "bg-[linear-gradient(110deg,hsl(var(--c-sun)/0.3),hsl(var(--c-coral)/0.25))] text-[hsl(var(--foreground))] dark:text-[hsl(var(--foreground))]",
  claimed: "pill-grape",
  locked: "bg-muted text-muted-foreground",
} as const;

const STREAK_LABEL: Record<StreakKind, string> = {
  logging: "Days in a row keeping up",
  saved: "Months in a row saving",
  dates: "Months in a row with a date",
  activity: "Days in a row adding your data",
};

// ---------- add form ----------

function RewardForm({
  onDone,
  preset,
}: {
  onDone: () => void;
  preset?: { name: string; cost_paise: number };
}) {
  const { data: goals } = useGoals();
  const add = useAddReward();
  const [name, setName] = useState(preset?.name ?? "");
  const [cost, setCost] = useState(
    preset && preset.cost_paise > 0 ? paiseToInput(preset.cost_paise) : "",
  );
  const [trigger, setTrigger] = useState<TriggerKind>("wish");
  const [goalId, setGoalId] = useState("");
  const [pct, setPct] = useState("100");
  const [streakKind, setStreakKind] = useState<StreakKind>("logging");
  const [streakTarget, setStreakTarget] = useState("7");
  const [error, setError] = useState<string | null>(null);

  const reset = () => {
    setName("");
    setCost("");
    setTrigger("wish");
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
    if (paise === null || paise < 0)
      return setError("The cost must be zero or more.");
    const body: RewardInput = {
      name: name.trim(),
      cost_paise: paise,
      trigger_kind: trigger,
    };
    if (trigger === "goal") {
      const id = goalId || goals?.goals[0]?.id;
      if (!id) return setError("Add a goal first, or pick a streak.");
      body.goal_id = id;
      body.goal_pct = Number(pct);
    } else if (trigger === "streak") {
      const n = Number(streakTarget);
      if (!Number.isInteger(n) || n < 1 || n > 3650)
        return setError("Pick a streak between 1 and 3650.");
      body.streak_kind = streakKind;
      body.streak_target = n;
    }
    setError(null);
    add.mutate(body, {
      onSuccess: () => {
        reset();
        onDone();
      },
    });
  };

  return (
    <form
      onSubmit={submit}
      className="form-grid border-t border-border pt-6"
      noValidate
    >
      <div className="field col-span-full">
        <Label htmlFor="reward-name">The treat</Label>
        <Input
          id="reward-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. A nice dinner out"
          maxLength={80}
        />
      </div>
      <div className="field">
        <Label htmlFor="reward-cost">What it costs (optional)</Label>
        <MoneyInput
          id="reward-cost"
          value={cost}
          onChange={(e) => setCost(e.target.value)}
        />
      </div>
      <div className="field">
        <Label htmlFor="reward-trigger">Unlocked by</Label>
        <Select
          id="reward-trigger"
          value={trigger}
          onChange={(e) => setTrigger(e.target.value as TriggerKind)}
        >
          <option value="wish">
            Nothing: buy it once the wallet has enough
          </option>
          <option value="goal">Reaching a goal</option>
          <option value="streak">A streak</option>
        </Select>
      </div>
      {trigger === "wish" ? null : trigger === "goal" ? (
        <>
          <div className="field">
            <Label htmlFor="reward-goal">Which goal</Label>
            <Select
              id="reward-goal"
              value={goalId || goals?.goals[0]?.id || ""}
              onChange={(e) => setGoalId(e.target.value)}
            >
              {(goals?.goals ?? []).map((g) => (
                <option key={g.id} value={g.id}>
                  {g.name}
                </option>
              ))}
            </Select>
          </div>
          <div className="field">
            <Label htmlFor="reward-pct">When it reaches</Label>
            <Select
              id="reward-pct"
              value={pct}
              onChange={(e) => setPct(e.target.value)}
            >
              {[25, 50, 75, 100].map((p) => (
                <option key={p} value={p}>
                  {p === 100 ? "All of it (done)" : `${p}% of the target`}
                </option>
              ))}
            </Select>
          </div>
        </>
      ) : (
        <>
          <div className="field">
            <Label htmlFor="reward-streak-kind">Which streak</Label>
            <Select
              id="reward-streak-kind"
              value={streakKind}
              onChange={(e) => setStreakKind(e.target.value as StreakKind)}
            >
              {(Object.keys(STREAK_LABEL) as StreakKind[]).map((k) => (
                <option key={k} value={k}>
                  {STREAK_LABEL[k]}
                </option>
              ))}
            </Select>
          </div>
          <div className="field">
            <Label htmlFor="reward-streak-target">How long</Label>
            <Input
              id="reward-streak-target"
              inputMode="numeric"
              value={streakTarget}
              onChange={(e) => setStreakTarget(e.target.value)}
            />
          </div>
        </>
      )}
      {error && (
        <p role="alert" className="col-span-full text-sm text-destructive">
          {error}
        </p>
      )}
      <div className="form-actions">
        <ResetButton onReset={reset} />
        <Button type="button" variant="ghost" onClick={onDone}>
          Cancel
        </Button>
        <Button type="submit" disabled={add.isPending}>
          Add reward
        </Button>
      </div>
    </form>
  );
}

// ---------- the budget ----------

function CapForm({
  cap,
  spent,
  left,
}: {
  cap: number | null;
  spent: number;
  left: number | null;
}) {
  const [value, setValue] = useState(cap === null ? "" : paiseToInput(cap));
  const [error, setError] = useState<string | null>(null);
  const save = useSetRewardCap();
  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    if (value.trim() === "") return save.mutate(null);
    const paise = toPaise(value);
    if (paise === null || paise < 0)
      return setError(
        "Type an amount of zero or more, or leave it empty for no limit.",
      );
    setError(null);
    save.mutate(paise);
  };
  return (
    <form
      onSubmit={submit}
      className="flex flex-wrap items-end gap-3 rounded-2xl border border-border p-4"
      noValidate
    >
      <div className="field min-w-0 flex-1">
        <Label htmlFor="reward-cap">Most to spend on treats each month</Label>
        <MoneyInput
          id="reward-cap"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="No limit"
        />
        <p className="mt-1 text-[13px] text-muted-foreground tabular-nums">
          {formatINR(spent)} claimed this month
          {left !== null && ` · ${formatINR(left)} left`}
        </p>
      </div>
      <div className="flex gap-2">
        <ResetButton
          onReset={() => {
            setValue(cap === null ? "" : paiseToInput(cap));
            setError(null);
          }}
        />
        <Button type="submit" size="sm" disabled={save.isPending}>
          Save limit
        </Button>
      </div>
      {error && (
        <p role="alert" className="w-full text-sm text-destructive">
          {error}
        </p>
      )}
    </form>
  );
}

// ---------- one reward ----------

function RewardRow({
  r,
  goalName,
  wallet,
}: {
  r: Reward;
  goalName?: string;
  wallet: number;
}) {
  const claim = useClaimReward();
  const unclaim = useUnclaimReward();
  const remove = useRemoveReward();
  return (
    <li
      className={cn(
        "space-y-2 border-t border-border py-4 text-sm first:border-t-0 first:pt-1",
        r.state === "earned" &&
          "-mx-3 rounded-2xl border-t-0 bg-[linear-gradient(120deg,hsl(var(--c-sun)/0.12),transparent_70%)] px-3",
      )}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="truncate text-[15px] font-medium" title={r.name}>
            {r.name}
          </p>
          <p className="text-[13px] text-muted-foreground tabular-nums">
            {triggerLabel(r, goalName)}
            {r.cost_paise > 0 && ` · ${formatINR(r.cost_paise)}`}
            {r.claimed_on && ` · claimed ${formatDay(r.claimed_on)}`}
          </p>
        </div>
        <span
          className={cn(
            "shrink-0 rounded-full px-2.5 py-1 text-xs font-semibold",
            STATE_TONE[r.state],
          )}
        >
          {STATE_LABEL[r.state]}
        </span>
      </div>
      <div className="flex flex-wrap gap-2">
        {r.state === "earned" && r.cost_paise <= wallet && (
          // Hold to claim: a treat comes out of real savings, so it asks for a moment, not a stray tap.
          <HoldButton
            size="md"
            radius={999}
            holdTime={900}
            backgroundColor="hsl(var(--good-tint))"
            textColor="hsl(var(--good))"
            fillColor="hsl(var(--good))"
            fillTextColor="hsl(var(--card))"
            icon={<Gift className="h-4 w-4" aria-hidden />}
            doneIcon={<PartyPopper className="h-4 w-4" aria-hidden />}
            doneLabel="Enjoy it!"
            disabled={claim.isPending}
            onHold={() => claim.mutate(r.reward_id)}
            onTap={() => toast("Press and hold to claim it")}
            className="min-h-11 font-semibold"
          >
            {r.cost_paise > 0 ? `Buy it from the wallet` : "Claim it"}
          </HoldButton>
        )}
        {r.state === "earned" && r.cost_paise > wallet && (
          <p className="m-0 self-center text-[13px] text-muted-foreground tabular-nums">
            {formatINR(r.cost_paise - wallet)} more in the wallet and it's yours
          </p>
        )}
        {r.state === "claimed" && (
          <ResetButton
            label="Undo claim"
            disabled={unclaim.isPending}
            onReset={() => unclaim.mutate(r.reward_id)}
          />
        )}
        <Button
          size="sm"
          variant="ghost"
          aria-label={`Remove ${r.name}`}
          disabled={remove.isPending}
          onClick={() => {
            if (
              window.confirm(
                `Remove "${r.name}"? Anything you claimed stays in your history.`,
              )
            )
              remove.mutate(r.reward_id);
          }}
        >
          Remove
        </Button>
      </div>
    </li>
  );
}

export function RewardsPanel() {
  const { data, isLoading, isError } = useRewards();
  const { data: goals } = useGoals();
  const { data: ideas } = useRewardSuggestions();
  const [adding, setAdding] = useState(false);
  const [preset, setPreset] = useState<
    { name: string; cost_paise: number } | undefined
  >();
  const goalName = (id: string | null) =>
    goals?.goals.find((g) => g.id === id)?.name;

  return (
    <Card id="rewards" className="scroll-mt-24">
      <CardHeader className="space-y-2 pb-4">
        <CardTitle className="text-[32px]">Rewards</CardTitle>
        <p className="max-w-prose text-sm text-muted-foreground">
          Treats you look forward to, bought from your reward fund's wallet.
          Some can also wait for a goal or a streak first.
        </p>
      </CardHeader>
      <CardContent className="space-y-6">
        {isLoading && (
          <p className="text-sm text-muted-foreground">Loading rewards...</p>
        )}
        {isError && (
          <p className="text-sm text-destructive">
            Could not load rewards. Try again in a moment.
          </p>
        )}

        {data && (
          <div className="flex flex-wrap items-baseline justify-between gap-2 rounded-2xl border border-mint/40 bg-[linear-gradient(120deg,hsl(var(--c-mint)/0.16),hsl(var(--c-sky)/0.08))] p-4">
            <span className="text-sm font-medium text-muted-foreground">
              In your reward wallet
            </span>
            <span className="font-display text-[28px] leading-none tabular-nums text-[hsl(var(--foreground))] dark:text-mint">
              <MoneyCounter paise={data.wallet_paise} />
            </span>
          </div>
        )}
        {data && (
          <details className="text-sm">
            <summary className="cursor-pointer text-muted-foreground">
              An extra monthly limit (optional)
            </summary>
            <div className="mt-3">
              <CapForm
                key={data.monthly_cap_paise ?? "none"}
                cap={data.monthly_cap_paise}
                spent={data.spent_this_month_paise}
                left={data.cap_left_paise}
              />
            </div>
          </details>
        )}

        {data && data.rewards.length === 0 && (
          <EmptyScene scene="sprout" title="No treats yet">
            Add something you'd love; the wallet fills at the end of each month.
          </EmptyScene>
        )}
        {data && data.rewards.length > 0 && (
          <ul className="ledger">
            {data.rewards.map((r) => (
              <RewardRow
                key={r.reward_id}
                r={r}
                goalName={goalName(r.goal_id)}
                wallet={data.wallet_paise}
              />
            ))}
          </ul>
        )}

        {data && data.history.length > 0 && (
          <details className="rounded-2xl border border-border px-4 py-3 text-sm">
            <summary className="cursor-pointer font-medium">
              Claimed so far ({data.history.length})
            </summary>
            <ul className="ledger mt-2">
              {data.history.map((h) => (
                <li key={h.reward_id} className="ledger-row">
                  <span className="min-w-0 truncate font-medium">{h.name}</span>
                  <span className="text-[13px] text-muted-foreground tabular-nums">
                    {formatDay(h.claimed_on)}
                    {h.cost_paise > 0 && ` · ${formatINR(h.cost_paise)}`}
                  </span>
                </li>
              ))}
            </ul>
          </details>
        )}

        {adding ? (
          <RewardForm
            key={preset?.name ?? "blank"}
            preset={preset}
            onDone={() => {
              setAdding(false);
              setPreset(undefined);
            }}
          />
        ) : (
          <div className="space-y-3">
            {ideas && ideas.length > 0 && (
              <div>
                <p className="mb-2 text-[13px] text-muted-foreground">
                  Ideas to start from (you choose what unlocks them):
                </p>
                <ul
                  className="m-0 flex list-none flex-wrap gap-2 p-0"
                  aria-label="Reward ideas"
                >
                  {ideas.map((i) => (
                    <li key={i.name} className="max-w-full">
                      <Button
                        size="sm"
                        variant="outline"
                        className="max-w-full max-md:h-auto max-md:min-h-11 max-md:whitespace-normal max-md:py-2 max-md:text-left"
                        title={i.note}
                        onClick={() => {
                          setPreset(i);
                          setAdding(true);
                        }}
                      >
                        {i.name}
                        {i.cost_paise > 0 && ` · ${formatINR(i.cost_paise)}`}
                      </Button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
            <Button
              variant="outline"
              onClick={() => {
                setPreset(undefined);
                setAdding(true);
              }}
            >
              Add a reward
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
