import { useState, type FormEvent } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { GraduationCap, Plus, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import {
  MAX_STAGES,
  buildEduPlan,
  educationKey,
  forgetEducation,
  getEducation,
  saveEducation,
  stageWhen,
  toEduDraft,
  type EduDraft,
  type EducationOut,
} from "@/services/education";
import { formatPercent, formatWholeINR } from "@/utils/money";

/** Planning figures in whole rupees: paise are noise over a 15-year horizon. */
const rupees = formatWholeINR;

const RANGE_HUE: Record<string, string> = {
  low: "mint",
  base: "grape",
  high: "coral",
};

/** What each stage costs then, the monthly top-up and its low/base/high range. Shows guesses as guesses. */
export function EducationSummary({ data }: { data: EducationOut }) {
  const { summary, lifeline } = data;
  const { low, base, high } = summary;
  return (
    <div className="stack" data-testid="education-summary">
      <ul className="m-0 list-none divide-y divide-border p-0">
        {summary.stages.map((s) => (
          <li
            key={`${s.name}-${s.start_age}`}
            className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-3"
          >
            <span>
              <span className="font-medium">{s.name}</span>
              <span className="ml-2 text-sm text-muted-foreground">
                at age {s.start_age} · {stageWhen(s)}
              </span>
            </span>
            <span className="text-right">
              <span className="font-display text-2xl leading-none">
                {rupees(s.future_cost_paise)}
              </span>
              <span className="ml-2 text-sm text-muted-foreground">
                {s.started
                  ? "now"
                  : `then, from ${rupees(s.today_cost_paise)} today`}
              </span>
            </span>
          </li>
        ))}
      </ul>

      {summary.nothing_left ? (
        <p className="m-0 text-sm text-muted-foreground">
          Every stage has begun or passed, so there is nothing left to save for.
        </p>
      ) : (
        <div>
          <p className="m-0 text-sm text-muted-foreground">
            To have it ready, put aside about
          </p>
          <p className="m-0 font-display text-4xl leading-tight">
            {rupees(base.monthly_top_up_paise)}{" "}
            <span className="text-base text-muted-foreground">a month</span>
          </p>
          <dl
            className="mt-3 grid grid-cols-3 gap-3 text-sm"
            aria-label="Range of monthly top-up"
          >
            {[
              {
                key: "low",
                scenario: low,
                hint: "Fees rise less, savings earn more",
              },
              { key: "base", scenario: base, hint: "Your assumptions" },
              {
                key: "high",
                scenario: high,
                hint: "Fees rise more, savings earn less",
              },
            ].map(({ key, scenario, hint }) => (
              <div
                key={key}
                className="rounded-2xl border px-3 py-2"
                style={{
                  // Low is the kind case (mint), base the plan (grape), high the hard one (coral).
                  borderColor: `hsl(var(--c-${RANGE_HUE[key]}) / 0.4)`,
                  background: `hsl(var(--c-${RANGE_HUE[key]}) / 0.1)`,
                }}
              >
                <dt className="text-muted-foreground">{scenario.label}</dt>
                <dd className="m-0 font-medium">
                  {rupees(scenario.monthly_top_up_paise)}
                </dd>
                <dd className="m-0 text-xs text-muted-foreground">
                  {hint}: fees +{formatPercent(scenario.inflation_bps)}, returns{" "}
                  {formatPercent(scenario.return_bps)}
                </dd>
              </div>
            ))}
          </dl>
        </div>
      )}

      <div>
        <h3 className="m-0 text-sm font-medium">
          What it does to your life line
        </h3>
        <ul className="m-0 mt-1 list-disc pl-5 text-sm">
          {lifeline.consequences.lines.map((l) => (
            <li key={l}>{l}</li>
          ))}
        </ul>
        <p className="m-0 mt-1 text-xs text-muted-foreground">
          {lifeline.note}
        </p>
      </div>

      <div
        className="rounded-2xl bg-muted/40 px-4 py-3 text-sm text-muted-foreground"
        role="note"
      >
        {summary.notes.map((n) => (
          <p key={n} className="m-0 [&+p]:mt-1">
            {n}
          </p>
        ))}
      </div>
    </div>
  );
}

function EducationForm({
  data,
  onDone,
}: {
  data: EducationOut;
  onDone: () => void;
}) {
  const qc = useQueryClient();
  const [draft, setDraft] = useState<EduDraft>(() => toEduDraft(data.plan));
  const [error, setError] = useState<string | null>(null);
  const save = useMutation({
    mutationFn: saveEducation,
    onSuccess: (out) => {
      qc.setQueryData(educationKey, out);
      onDone();
    },
    onError: (e) =>
      setError(e instanceof Error ? e.message : "Couldn't save the plan."),
  });
  const forget = useMutation({
    mutationFn: forgetEducation,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: educationKey });
      onDone();
    },
  });
  const set = (patch: Partial<EduDraft>) =>
    setDraft((d) => ({ ...d, ...patch }));
  const setStage = (i: number, patch: Partial<EduDraft["stages"][number]>) =>
    set({
      stages: draft.stages.map((s, j) => (j === i ? { ...s, ...patch } : s)),
    });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      save.mutate(buildEduPlan(draft));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Check the figures.");
    }
  };

  return (
    <form onSubmit={submit} className="stack" aria-label="Education plan">
      <div className="form-grid">
        <div className="field">
          <Label htmlFor="edu-year">Child's birth year</Label>
          <Input
            id="edu-year"
            inputMode="numeric"
            value={draft.child_birth_year}
            onChange={(e) => set({ child_birth_year: e.target.value })}
          />
        </div>
        <div className="field">
          <Label htmlFor="edu-corpus">Already set aside for this (₹)</Label>
          <MoneyInput
            id="edu-corpus"
            value={draft.corpus}
            onChange={(e) => set({ corpus: e.target.value })}
          />
        </div>
        <div className="field">
          <Label htmlFor="edu-infl">Fee inflation a year (%)</Label>
          <Input
            id="edu-infl"
            inputMode="decimal"
            value={draft.inflation}
            onChange={(e) => set({ inflation: e.target.value })}
          />
          <p className="m-0 text-[13px] text-muted-foreground">
            A guess. Fees have often risen faster than prices in general.
          </p>
        </div>
        <div className="field">
          <Label htmlFor="edu-return">Expected return a year (%)</Label>
          <Input
            id="edu-return"
            inputMode="decimal"
            value={draft.return_pct}
            onChange={(e) => set({ return_pct: e.target.value })}
          />
          <p className="m-0 text-[13px] text-muted-foreground">
            A guess, not a promise. Markets can return far less, or lose money.
          </p>
        </div>
      </div>

      <fieldset className="m-0 min-w-0 border-0 p-0">
        <legend className="mb-2 p-0 text-sm font-medium">Stages</legend>
        {draft.stages.map((s, i) => (
          <div
            key={i}
            className="mb-3 grid grid-cols-[1fr_5rem_1fr_auto] items-end gap-2"
          >
            <div className="field">
              <Label htmlFor={`edu-name-${i}`}>Name</Label>
              <Input
                id={`edu-name-${i}`}
                value={s.name}
                onChange={(e) => setStage(i, { name: e.target.value })}
              />
            </div>
            <div className="field">
              <Label htmlFor={`edu-age-${i}`}>Age</Label>
              <Input
                id={`edu-age-${i}`}
                inputMode="numeric"
                value={s.start_age}
                onChange={(e) => setStage(i, { start_age: e.target.value })}
              />
            </div>
            <div className="field">
              <Label htmlFor={`edu-cost-${i}`}>Cost today (₹)</Label>
              <MoneyInput
                id={`edu-cost-${i}`}
                value={s.cost}
                onChange={(e) => setStage(i, { cost: e.target.value })}
              />
            </div>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={`Remove ${s.name || `stage ${i + 1}`}`}
              onClick={() =>
                set({ stages: draft.stages.filter((_, j) => j !== i) })
              }
            >
              <X className="h-4 w-4" aria-hidden />
            </Button>
          </div>
        ))}
        {draft.stages.length < MAX_STAGES && (
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() =>
              set({
                stages: [
                  ...draft.stages,
                  { name: "", start_age: "", cost: "" },
                ],
              })
            }
          >
            <Plus className="mr-1 h-4 w-4" aria-hidden /> Add a stage
          </Button>
        )}
      </fieldset>

      {error && (
        <p role="alert" className="m-0 text-sm text-over">
          {error}
        </p>
      )}
      <div className="flex flex-wrap justify-end gap-2">
        {data.saved && (
          <Button
            type="button"
            variant="ghost"
            disabled={forget.isPending}
            onClick={() => forget.mutate()}
          >
            Forget this plan
          </Button>
        )}
        <Button type="button" variant="outline" onClick={onDone}>
          Cancel
        </Button>
        <Button type="submit" disabled={save.isPending}>
          {save.isPending ? "Saving..." : "Save plan"}
        </Button>
      </div>
    </form>
  );
}

/** The Life page's child and education card. */
export function EducationCard() {
  const { data, isLoading, isError } = useQuery({
    queryKey: educationKey,
    queryFn: getEducation,
  });
  const [editing, setEditing] = useState(false);

  return (
    <section
      className="glass rounded-[28px] px-6 py-6 sm:px-10"
      aria-labelledby="education-title"
    >
      <div className="mb-4 flex flex-wrap items-center justify-between gap-2">
        <h2
          id="education-title"
          className="m-0 flex items-center gap-2 font-display text-[32px] leading-none"
        >
          <span
            aria-hidden
            className="grid h-10 w-10 place-items-center rounded-full bg-grape/15 text-grape"
          >
            <GraduationCap className="h-5 w-5" />
          </span>
          Education fund
        </h2>
        {data && !editing && (
          <Button variant="outline" size="sm" onClick={() => setEditing(true)}>
            {data.saved ? "Edit plan" : "Set up a plan"}
          </Button>
        )}
      </div>
      {isLoading && (
        <div
          className="h-32 animate-pulse rounded-2xl bg-muted/40"
          aria-label="Loading the education plan"
        />
      )}
      {isError && !data && (
        <p className="m-0 text-sm text-muted-foreground">
          Couldn't load the education plan.
        </p>
      )}
      {data && editing && (
        <EducationForm data={data} onDone={() => setEditing(false)} />
      )}
      {data && !editing && (
        <>
          {!data.saved && (
            <p className="mb-3 mt-0 text-sm text-muted-foreground">
              A starter plan, with made-up costs, to show how this works. Set up
              your own to save it.
            </p>
          )}
          <EducationSummary data={data} />
        </>
      )}
    </section>
  );
}
