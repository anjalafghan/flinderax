import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useLocation } from "react-router-dom";
import { ChevronDown, Trash2 } from "lucide-react";

import { ChoiceForm } from "@/components/feature/life/ChoiceForm";
import { LineChart, type ChartMarker, type ChartSeries } from "@/components/feature/LineChart";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import {
  clearChoices,
  describeChoice,
  dropChoice,
  freeLabel,
  getLifeline,
  keepChoice,
  lastsLabel,
  lifelineKey,
  tryChoices,
  updateChoice,
  type Choice,
  type Line,
  type LifelineOut,
  type SavedChoice,
  type TryOut,
} from "@/services/lifeline";
import { cn } from "@/utils/cn";
import { formatCompactINR, formatINR } from "@/utils/money";

/** What is being compared with the plan right now. `draft` is set while the choice is not saved yet. */
interface Preview {
  title: string;
  out: TryOut;
  draft?: Choice;
}

const rupees = (paise: number) => formatINR(Math.round(paise / 100) * 100);

function Stat({ label, value, was, tone }: { label: string; value: string; was?: string; tone?: "good" | "over" }) {
  return (
    <div className="flex min-w-0 flex-col gap-1.5 rounded-[18px] border border-border px-4 py-4">
      <span className="text-[13px] text-muted-foreground">{label}</span>
      <span className={cn("font-display text-[26px] leading-none", tone === "good" && "text-good", tone === "over" && "text-over")}>{value}</span>
      {was !== undefined && was !== value && <span className="text-[13px] text-muted-foreground">Now: {was}</span>}
    </div>
  );
}

function Stats({ line, plan }: { line: Line; plan?: Line }) {
  const m = line.marks;
  const p = plan?.marks;
  return (
    <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
      <Stat label="Work becomes optional" value={freeLabel(m)} was={p && freeLabel(p)} tone={m.free_at_age === null ? "over" : undefined} />
      <Stat label="Money lasts" value={lastsLabel(m)} was={p && lastsLabel(p)} tone={m.runs_out_at_age === null ? "good" : "over"} />
      {m.at_60_today_paise !== null && (
        <Stat
          label="At 60, in today's money"
          value={formatCompactINR(m.at_60_today_paise)}
          was={p && p.at_60_today_paise !== null ? formatCompactINR(p.at_60_today_paise) : undefined}
        />
      )}
      <Stat label={`At ${m.end_age}, in today's money`} value={formatCompactINR(m.at_end_today_paise)} was={p && formatCompactINR(p.at_end_today_paise)} />
    </div>
  );
}

function Chart({ plan, preview }: { plan: Line; preview: Preview | null }) {
  const shown = preview ? preview.out.with : plan;
  const labels = shown.rows.map((r) => String(r.age));
  const values = (l: Line) => l.rows.map((r) => r.net_worth_today_paise);
  const series: ChartSeries[] = preview
    ? [
        { label: "Without it", values: values(preview.out.base), stroke: "stroke-muted-foreground", dashed: true },
        { label: "With it", values: values(preview.out.with), stroke: "stroke-emerald-500" },
      ]
    : [{ label: "Your plan", values: values(plan), stroke: "stroke-sky-500" }];
  const markers: ChartMarker[] = shown.rows.flatMap((r, index) =>
    r.short_paise > 0 ? [{ index, label: `Age ${r.age}: short by ${rupees(r.short_paise)}` }] : [],
  );
  return (
    <LineChart
      labels={labels}
      series={series}
      markers={markers}
      markerLabel="A year the money falls short"
      ariaLabel="Net worth by age, in today's money"
    />
  );
}

function Years({ line }: { line: Line }) {
  const rows = line.rows.filter((r) => r.events.length > 0);
  if (rows.length === 0) return null;
  return (
    <ol className="m-0 grid list-none gap-2 p-0 sm:grid-cols-2" aria-label="The years ahead">
      {rows.map((r) => (
        <li key={r.year} className="flex items-baseline gap-3 text-sm">
          <span className="w-12 shrink-0 font-display text-lg leading-none">{r.year}</span>
          <span className="min-w-0">
            <span className="text-muted-foreground">Age {r.age} · </span>
            {r.events.join(", ")}
          </span>
        </li>
      ))}
    </ol>
  );
}

function Notes({ data, line }: { data: LifelineOut; line: Line }) {
  const w = data.working;
  return (
    <details className="group rounded-[18px] border border-border">
      <summary className="flex min-h-11 cursor-pointer list-none items-center justify-between gap-4 px-4 py-3 text-sm">
        What this is worked out from
        <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden />
      </summary>
      <div className="grid gap-3 px-4 pb-4 text-sm text-muted-foreground">
        <p className="m-0">
          Take-home {rupees(w.income_paise)} a month, spending {rupees(w.expenses_paise)} a month, and {rupees(w.wealth_paise)} to start with (
          {rupees(Math.max(w.accounts_paise, w.plan_saved_paise))} owned, {rupees(w.loans_paise)} of loans). {w.note}
        </p>
        <ul className="m-0 grid list-none gap-2 p-0">
          {line.notes.map((n) => {
            const estimate = n.startsWith("Estimate: ");
            return (
              <li key={n} className="flex items-start gap-2">
                {estimate && <span className="tone-warn mt-0.5 shrink-0 rounded-full px-2 text-[12px] font-semibold">Estimate</span>}
                <span>{estimate ? n.slice("Estimate: ".length) : n}</span>
              </li>
            );
          })}
        </ul>
      </div>
    </details>
  );
}

function Result({ preview, busy, onKeep, onClose }: { preview: Preview; busy: boolean; onKeep: (inPlan: boolean) => void; onClose: () => void }) {
  const c = preview.out.consequences;
  return (
    <div className="grid gap-4 rounded-[18px] border border-border bg-[hsl(var(--chip))] px-5 py-5" role="status">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h3 className="m-0 text-[17px] font-medium">{preview.title}</h3>
        <span className={cn("rounded-full px-3 py-1 text-[13px] font-semibold", c.better ? "tone-good" : "tone-quiet")}>
          {c.better ? "Leaves you better off" : "What it costs"}
        </span>
      </div>
      <ul className="m-0 grid list-disc gap-1.5 pl-5 text-base">
        {c.lines.map((l) => (
          <li key={l}>{l}</li>
        ))}
      </ul>
      <div className="flex flex-wrap gap-2">
        {preview.draft && (
          <>
            <Button onClick={() => onKeep(true)} disabled={busy}>
              Keep in my plan
            </Button>
            <Button variant="outline" onClick={() => onKeep(false)} disabled={busy}>
              Save for later
            </Button>
          </>
        )}
        <Button variant="ghost" onClick={onClose}>
          {preview.draft ? "Discard" : "Done"}
        </Button>
      </div>
    </div>
  );
}

/** The life line: net worth by age to 90, the four marks, and choices to try against it. */
export function LifeLine() {
  const qc = useQueryClient();
  const location = useLocation();
  // Another page (Work) can hand over a choice to start from.
  const handed = (location.state as { choice?: Choice } | null)?.choice;
  const { data, isLoading, isError, refetch } = useQuery({ queryKey: lifelineKey, queryFn: getLifeline });
  const [preview, setPreview] = useState<Preview | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fail = (err: unknown) => setError(err instanceof Error ? err.message : "That didn't work. Try again.");
  const refresh = () => qc.invalidateQueries({ queryKey: lifelineKey });

  const attempt = useMutation({
    mutationFn: (v: { base: Choice[]; with: Choice[]; title: string; draft?: Choice }) => tryChoices(v.base, v.with),
    onSuccess: (out, v) => {
      setError(null);
      setPreview({ title: v.title, out, draft: v.draft });
    },
    onError: fail,
  });
  const keep = useMutation({
    mutationFn: keepChoice,
    onSuccess: () => {
      setPreview(null);
      return refresh();
    },
    onError: fail,
  });
  const toggle = useMutation({
    mutationFn: (c: SavedChoice) => updateChoice(c.id, { in_plan: !c.in_plan }),
    onSuccess: () => {
      setPreview(null);
      return refresh();
    },
    onError: fail,
  });
  const clearAll = useMutation({
    mutationFn: clearChoices,
    onSuccess: () => {
      setPreview(null);
      return refresh();
    },
    onError: fail,
  });
  const drop = useMutation({
    mutationFn: (c: SavedChoice) => dropChoice(c.id),
    onSuccess: () => {
      setPreview(null);
      return refresh();
    },
    onError: fail,
  });

  if (isLoading) return <div className="h-80 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your life line" />;
  if (isError || !data) {
    return (
      <div className="rounded-[28px] border border-dashed border-border px-6 py-8 text-center text-sm text-muted-foreground">
        Couldn't load your life line.{" "}
        <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  const inPlan = data.choices.filter((c) => c.in_plan).map((c) => c.choice);
  const tryDraft = (choice: Choice) => attempt.mutate({ base: inPlan, with: [...inPlan, choice], title: describeChoice(choice), draft: choice });
  const seeEffect = (c: SavedChoice) => {
    const others = data.choices.filter((x) => x.in_plan && x.id !== c.id).map((x) => x.choice);
    attempt.mutate({ base: others, with: [...others, c.choice], title: c.label });
  };
  const shown = preview ? preview.out.with : data.line;
  const end = data.line.marks.end_age;

  return (
    <section aria-labelledby="lifeline-title" className="flex flex-col gap-6 rounded-[28px] border border-border bg-card px-5 py-7 sm:px-8 sm:py-9">
      <div className="flex flex-col gap-2">
        <h2 id="lifeline-title" className="m-0 text-[15px] font-medium text-muted-foreground">
          {preview ? "If you did this" : `Your plan, from ${data.line.rows[0]?.age ?? "now"} to ${end}`}
        </h2>
        {!data.saved_profile && (
          <p className="m-0 text-sm text-muted-foreground">This runs on placeholder numbers. Fill in your details below to make it yours.</p>
        )}
      </div>

      <Stats line={shown} plan={preview ? data.line : undefined} />
      <Chart plan={data.line} preview={preview} />
      {shown.marks.unfunded.length > 0 && (
        <p className="tone-over m-0 rounded-[14px] px-4 py-3 text-sm">Savings can't fully pay for: {shown.marks.unfunded.join(", ")}.</p>
      )}
      <Years line={shown} />

      {preview && (
        <Result
          preview={preview}
          busy={keep.isPending}
          onClose={() => setPreview(null)}
          onKeep={(in_plan) => preview.draft && keep.mutate({ label: preview.title.slice(0, 60), choice: preview.draft, in_plan })}
        />
      )}
      {error && (
        <p role="alert" className="m-0 text-sm text-destructive">
          {error}
        </p>
      )}

      {data.choices.length > 0 && (
        <div className="grid gap-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="m-0 text-[17px] font-medium">Your choices</h3>
            <ResetButton
              label="Reset: remove all choices"
              disabled={clearAll.isPending}
              confirm="Take every choice off your life line? Your details stay."
              onReset={() => clearAll.mutate()}
            />
          </div>
          <ul className="m-0 grid list-none gap-2 p-0">
            {data.choices.map((c) => (
              <li key={c.id} className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 rounded-[14px] border border-border px-4 py-3">
                <label className="flex min-h-11 min-w-0 flex-1 cursor-pointer items-center gap-3 text-sm">
                  <input
                    type="checkbox"
                    role="switch"
                    className="h-5 w-5 shrink-0"
                    checked={c.in_plan}
                    disabled={toggle.isPending}
                    onChange={() => toggle.mutate(c)}
                    aria-label={`${c.label}: in my plan`}
                  />
                  <span className="min-w-0">
                    {c.label}
                    <span className="block text-[13px] text-muted-foreground">{c.in_plan ? "In my plan" : "Not in my plan"}</span>
                  </span>
                </label>
                <span className="flex shrink-0 items-center gap-1">
                  <Button variant="outline" className="h-11 rounded-full" onClick={() => seeEffect(c)} disabled={attempt.isPending}>
                    See its effect
                  </Button>
                  <Button variant="ghost" size="icon" className="h-11 w-11" onClick={() => drop.mutate(c)} disabled={drop.isPending} aria-label={`Remove ${c.label}`}>
                    <Trash2 className="h-4 w-4" aria-hidden />
                  </Button>
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      <div className="grid gap-4">
        <h3 className="m-0 text-[17px] font-medium">Try a choice</h3>
        <ChoiceForm key={handed ? JSON.stringify(handed) : "blank"} initial={handed} busy={attempt.isPending} onTry={tryDraft} />
      </div>

      <Notes data={data} line={shown} />
    </section>
  );
}
