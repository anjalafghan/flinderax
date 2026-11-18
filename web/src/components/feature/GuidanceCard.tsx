import { useState, type FormEvent } from "react";
import { useMutation } from "@tanstack/react-query";
import { Plus, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { buildGuidanceInput, getGuidance, type Guidance, type GuidanceDraft } from "@/services/guidance";
import { formatINR } from "@/utils/money";

const START: GuidanceDraft = {
  salary: "",
  sip: "",
  spend_pct: "30",
  saved: "",
  fixed: [{ label: "", amount: "" }],
  shared: [],
};

export function GuidanceResult({ g }: { g: Guidance }) {
  return (
    <div className="stack" data-testid="guidance-result">
      {g.warnings.map((w) => (
        <p key={w} role="alert" className="m-0 text-sm text-over">
          {w}
        </p>
      ))}
      <ol className="m-0 list-none divide-y divide-border p-0">
        {g.steps.map((s) => (
          <li key={s.what} className="py-4">
            <p className="m-0 font-display text-xl leading-snug">{s.what}</p>
            <p className="m-0 mt-2 text-sm">
              <span className="font-medium">What it changes: </span>
              {s.impact}
            </p>
            <p className="m-0 mt-1 text-sm text-muted-foreground">
              <span className="font-medium">Why: </span>
              {s.why}
            </p>
          </li>
        ))}
      </ol>
      <p className="m-0 text-xs text-muted-foreground">
        Left over each month: {formatINR(g.surplus_paise)}. Emergency target: {formatINR(g.emergency_target_paise)}, from what you really spend ({formatINR(g.monthly_cost_paise)} a month).
      </p>
    </div>
  );
}

/** Salary, fixed costs, SIP and a strict spending share in; what to do, what it changes and why out. */
export function GuidanceCard() {
  const [d, setD] = useState<GuidanceDraft>(START);
  const [error, setError] = useState<string | null>(null);
  const run = useMutation({ mutationFn: getGuidance, onError: (e) => setError(e instanceof Error ? e.message : "Couldn't work it out.") });
  const set = (patch: Partial<GuidanceDraft>) => setD((x) => ({ ...x, ...patch }));

  const submit = (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    try {
      run.mutate(buildGuidanceInput(d));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Check the figures.");
    }
  };

  return (
    <section className="rounded-[28px] border border-border bg-card px-6 py-6 sm:px-10" aria-labelledby="guidance-title">
      <h2 id="guidance-title" className="m-0 mb-4 font-display text-[32px] leading-none">
        What to do this month
      </h2>
      <form onSubmit={submit} className="stack" aria-label="Month plan">
        <div className="form-grid">
          <div className="field">
            <Label htmlFor="g-salary">Monthly salary (₹)</Label>
            <MoneyInput id="g-salary" value={d.salary} onChange={(e) => set({ salary: e.target.value })} />
          </div>
          <div className="field">
            <Label htmlFor="g-sip">SIP a month (₹)</Label>
            <MoneyInput id="g-sip" value={d.sip} onChange={(e) => set({ sip: e.target.value })} />
          </div>
          <div className="field">
            <Label htmlFor="g-pct">Spend at most this share of salary (%)</Label>
            <Input id="g-pct" inputMode="decimal" value={d.spend_pct} onChange={(e) => set({ spend_pct: e.target.value })} />
          </div>
          <div className="field">
            <Label htmlFor="g-saved">Emergency fund so far (₹)</Label>
            <MoneyInput id="g-saved" value={d.saved} onChange={(e) => set({ saved: e.target.value })} />
          </div>
        </div>

        <fieldset className="m-0 min-w-0 border-0 p-0">
          <legend className="mb-2 p-0 text-sm font-medium">Fixed costs a month</legend>
          {d.fixed.map((f, i) => (
            <div key={i} className="mb-2 grid grid-cols-[1fr_1fr_auto] items-end gap-2">
              <Input aria-label={`Fixed cost ${i + 1} name`} placeholder="Rent" value={f.label} onChange={(e) => set({ fixed: d.fixed.map((x, j) => (j === i ? { ...x, label: e.target.value } : x)) })} />
              <MoneyInput aria-label={`Fixed cost ${i + 1} amount`} value={f.amount} onChange={(e) => set({ fixed: d.fixed.map((x, j) => (j === i ? { ...x, amount: e.target.value } : x)) })} />
              <Button type="button" variant="ghost" size="icon" aria-label={`Remove fixed cost ${i + 1}`} onClick={() => set({ fixed: d.fixed.filter((_, j) => j !== i) })}>
                <X className="h-4 w-4" aria-hidden />
              </Button>
            </div>
          ))}
          <Button type="button" variant="outline" size="sm" onClick={() => set({ fixed: [...d.fixed, { label: "", amount: "" }] })}>
            <Plus className="mr-1 h-4 w-4" aria-hidden /> Add a fixed cost
          </Button>
        </fieldset>

        <fieldset className="m-0 min-w-0 border-0 p-0">
          <legend className="mb-2 p-0 text-sm font-medium">Shared spending inside that share (every month)</legend>
          {d.shared.map((s, i) => (
            <div key={i} className="mb-2 grid grid-cols-[1fr_1fr_1fr_auto] items-end gap-2">
              <Input aria-label={`Shared ${i + 1} name`} placeholder="Together" value={s.label} onChange={(e) => set({ shared: d.shared.map((x, j) => (j === i ? { ...x, label: e.target.value } : x)) })} />
              <MoneyInput aria-label={`Shared ${i + 1} total`} placeholder="Total" value={s.total} onChange={(e) => set({ shared: d.shared.map((x, j) => (j === i ? { ...x, total: e.target.value } : x)) })} />
              <MoneyInput aria-label={`Shared ${i + 1} partner's part`} placeholder="Partner pays" value={s.partner} onChange={(e) => set({ shared: d.shared.map((x, j) => (j === i ? { ...x, partner: e.target.value } : x)) })} />
              <Button type="button" variant="ghost" size="icon" aria-label={`Remove shared ${i + 1}`} onClick={() => set({ shared: d.shared.filter((_, j) => j !== i) })}>
                <X className="h-4 w-4" aria-hidden />
              </Button>
            </div>
          ))}
          <Button type="button" variant="outline" size="sm" onClick={() => set({ shared: [...d.shared, { label: "", total: "", partner: "" }] })}>
            <Plus className="mr-1 h-4 w-4" aria-hidden /> Add shared spending
          </Button>
        </fieldset>

        {error && (
          <p role="alert" className="m-0 text-sm text-over">
            {error}
          </p>
        )}
        <div className="flex justify-end">
          <Button type="submit" disabled={run.isPending}>
            {run.isPending ? "Working it out..." : "Tell me what to do"}
          </Button>
        </div>
      </form>
      {run.data && <GuidanceResult g={run.data} />}
    </section>
  );
}
