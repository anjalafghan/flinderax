import { useState } from "react";
import { Archive, Pencil, PiggyBank } from "lucide-react";

import type { Goal, GoalInput } from "@/services/types";
import { useArchiveGoal, useContribute, useGoals, useSaveGoal } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { formatDay } from "@/utils/dates";
import { formatINR, paiseToInput, toPaise } from "@/utils/money";
import { cn } from "@/utils/cn";
import { GOAL_STATUS_LABEL, GOAL_STATUS_TONE, monthsCoveredLabel, progressPercent } from "@/utils/planner";

// ---------- add / edit form ----------

interface GoalFormProps {
  initial?: Goal;
  onDone: () => void;
}

function GoalForm({ initial, onDone }: GoalFormProps) {
  const [name, setName] = useState(initial?.name ?? "");
  const [target, setTarget] = useState(initial ? paiseToInput(initial.target_paise) : "");
  const [date, setDate] = useState(initial?.target_date ?? "");
  const [priority, setPriority] = useState(String(initial?.priority ?? 3));
  const [error, setError] = useState<string | null>(null);
  const save = useSaveGoal();

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(target);
    if (!name.trim() || amount === null || amount <= 0) {
      return setError("Add a name and a target above zero.");
    }
    setError(null);
    const body: GoalInput = {
      goal_id: initial?.id,
      name: name.trim(),
      kind: initial?.kind ?? "custom",
      target_paise: amount,
      target_date: date || null,
      priority: Number(priority),
      account_id: initial?.account_id ?? null,
    };
    save.mutate(body, { onSuccess: onDone });
  };

  return (
    <form onSubmit={submit} className="grid grid-cols-2 gap-2 glass rounded-2xl p-3" noValidate>
      <div className="col-span-2 space-y-1">
        <Label htmlFor={`goal-name-${initial?.id ?? "new"}`}>Name</Label>
        <Input
          id={`goal-name-${initial?.id ?? "new"}`}
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. New laptop"
        />
      </div>
      <div className="space-y-1">
        <Label htmlFor={`goal-target-${initial?.id ?? "new"}`}>Target</Label>
        <MoneyInput
          id={`goal-target-${initial?.id ?? "new"}`}
          value={target}
          onChange={(e) => setTarget(e.target.value)}
        />
      </div>
      <div className="space-y-1">
        <Label htmlFor={`goal-date-${initial?.id ?? "new"}`}>By (optional)</Label>
        <Input
          id={`goal-date-${initial?.id ?? "new"}`}
          type="date"
          value={date}
          onChange={(e) => setDate(e.target.value)}
        />
      </div>
      <div className="col-span-2 space-y-1 sm:col-span-1">
        <Label htmlFor={`goal-priority-${initial?.id ?? "new"}`}>Priority</Label>
        <Select
          id={`goal-priority-${initial?.id ?? "new"}`}
          value={priority}
          onChange={(e) => setPriority(e.target.value)}
        >
          {[1, 2, 3, 4, 5].map((p) => (
            <option key={p} value={p}>
              {p} {p === 1 ? "(first)" : p === 5 ? "(last)" : ""}
            </option>
          ))}
        </Select>
      </div>
      {error && <p role="alert" className="col-span-2 text-sm text-destructive">{error}</p>}
      <div className="col-span-2 flex flex-wrap gap-2">
        <Button type="submit" size="sm" disabled={save.isPending}>
          {initial ? "Save goal" : "Add goal"}
        </Button>
        <Button type="button" size="sm" variant="ghost" onClick={onDone}>
          Cancel
        </Button>
      </div>
    </form>
  );
}

// ---------- one goal ----------

function ContributeForm({ goal, onDone }: { goal: Goal; onDone: () => void }) {
  const [amount, setAmount] = useState("");
  const [error, setError] = useState<string | null>(null);
  const contribute = useContribute();
  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const paise = toPaise(amount);
    if (paise === null || paise === 0) return setError("Type an amount. Use a minus sign to take money out.");
    setError(null);
    contribute.mutate({ goal_id: goal.id, amount_paise: paise }, { onSuccess: onDone });
  };
  return (
    <form onSubmit={submit} className="flex flex-wrap items-end gap-2" noValidate>
      <div className="min-w-0 flex-1 space-y-1">
        <Label htmlFor={`contrib-${goal.id}`}>Amount put in</Label>
        <MoneyInput id={`contrib-${goal.id}`} value={amount} onChange={(e) => setAmount(e.target.value)} />
      </div>
      <Button type="submit" size="sm" disabled={contribute.isPending}>
        Save
      </Button>
      <Button type="button" size="sm" variant="ghost" onClick={onDone}>
        Cancel
      </Button>
      {error && <p role="alert" className="w-full text-sm text-destructive">{error}</p>}
    </form>
  );
}

function GoalItem({ goal }: { goal: Goal }) {
  const [mode, setMode] = useState<"none" | "contribute" | "edit">("none");
  const archive = useArchiveGoal();
  const pct = progressPercent(goal.saved_paise, goal.target_paise);
  const done = goal.status === "done";

  const onArchive = () => {
    if (window.confirm(`Archive "${goal.name}"? It will leave this list.`)) archive.mutate(goal.id);
  };

  return (
    <li className="space-y-3 glass rounded-2xl p-4 text-sm">
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="truncate font-semibold">{goal.name}</p>
          <p className="text-xs text-muted-foreground tabular-nums">
            {formatINR(goal.saved_paise)} of {formatINR(goal.target_paise)}
            {goal.target_date && !done && ` · by ${formatDay(goal.target_date)}`}
            {goal.months_left != null && !done && ` · ${goal.months_left} months left`}
            {done && goal.reached_by && ` · reached ${formatDay(goal.reached_by)}`}
          </p>
        </div>
        <span className={cn("shrink-0 rounded-full px-2.5 py-1 text-xs font-semibold", GOAL_STATUS_TONE[goal.status])}>
          {GOAL_STATUS_LABEL[goal.status]}
        </span>
      </div>

      <div
        role="progressbar"
        aria-label={`${goal.name} progress`}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
        className="h-2 w-full overflow-hidden rounded-full bg-muted"
      >
        <div className="h-full rounded-full bg-emerald-500 transition-[width]" style={{ width: `${pct}%` }} />
      </div>

      {!done && goal.this_month_paise > 0 && (
        <p className="font-medium">This month: put in {formatINR(goal.this_month_paise)}</p>
      )}

      {goal.status === "behind" && goal.fixes.length > 0 && (
        <div className="rounded-md bg-amber-500/10 p-3 text-amber-800 dark:text-amber-200">
          <p className="mb-1 font-medium">To get back on track:</p>
          <ul className="list-disc space-y-1 pl-5">
            {goal.fixes.map((f) => (
              <li key={f}>{f}</li>
            ))}
          </ul>
        </div>
      )}

      {mode === "contribute" && <ContributeForm goal={goal} onDone={() => setMode("none")} />}
      {mode === "edit" && <GoalForm initial={goal} onDone={() => setMode("none")} />}

      {mode === "none" && (
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline" onClick={() => setMode("contribute")}>
            <PiggyBank className="mr-1.5 h-4 w-4" />
            I put money in
          </Button>
          <Button size="sm" variant="ghost" onClick={() => setMode("edit")} aria-label={`Edit ${goal.name}`}>
            <Pencil className="mr-1.5 h-4 w-4" />
            Edit
          </Button>
          <Button size="sm" variant="ghost" onClick={onArchive} disabled={archive.isPending} aria-label={`Archive ${goal.name}`}>
            <Archive className="mr-1.5 h-4 w-4" />
            Archive
          </Button>
        </div>
      )}
    </li>
  );
}

// ---------- the section ----------

export function GoalsPanel() {
  const { data, isLoading, isError } = useGoals();
  const save = useSaveGoal();
  const [adding, setAdding] = useState(false);
  const covered = monthsCoveredLabel(data?.months_covered_tenths ?? null);

  return (
    <Card id="goals" className="scroll-mt-24">
      <CardHeader className="pb-2">
        <CardTitle className="text-3xl">Goals</CardTitle>
        <p className="text-sm text-muted-foreground">Things you are saving for. Money put in here is set aside each month.</p>
        {covered && (
          <p className="pt-1 text-sm font-medium">
            Emergency cover: <span className="tabular-nums">{covered}</span>
          </p>
        )}
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading && <p className="text-sm text-muted-foreground">Loading goals...</p>}
        {isError && <p className="text-sm text-destructive">Could not load goals. Try again in a moment.</p>}

        {data?.emergency_suggested_paise != null && data.emergency_suggested_paise > 0 && (
          <div className="flex flex-col gap-3 rounded-lg border border-sky-500/30 bg-sky-500/10 p-4 text-sm sm:flex-row sm:items-center sm:justify-between">
            <p className="font-medium">Start an emergency fund of {formatINR(data.emergency_suggested_paise)} (6 months of needs)</p>
            <Button
              size="sm"
              className="shrink-0"
              disabled={save.isPending}
              onClick={() =>
                save.mutate({
                  name: "Emergency fund",
                  kind: "emergency",
                  target_paise: data.emergency_suggested_paise!,
                  priority: 1,
                  target_date: null,
                  account_id: null,
                })
              }
            >
              Start it
            </Button>
          </div>
        )}

        {data && data.goals.length === 0 && <p className="text-sm text-muted-foreground">No goals yet.</p>}
        {data && data.goals.length > 0 && (
          <ul className="space-y-3">
            {data.goals.map((g) => (
              <GoalItem key={g.id} goal={g} />
            ))}
          </ul>
        )}

        {data && data.unallocated_paise > 0 && (
          <p className="rounded-md bg-muted p-3 text-sm">
            {formatINR(data.unallocated_paise)} of this month's savings not assigned yet.
          </p>
        )}

        {adding ? (
          <GoalForm onDone={() => setAdding(false)} />
        ) : (
          <Button variant="outline" className="w-full" onClick={() => setAdding(true)}>
            Add a goal
          </Button>
        )}
      </CardContent>
    </Card>
  );
}
