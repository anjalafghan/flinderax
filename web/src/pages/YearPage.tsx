import { useState } from "react";

import { Header } from "@/components/layout/Header";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import {
  barPct,
  KIND_LABEL,
  monthLabel,
  useAddOutflow,
  useOutflows,
  useRemoveOutflow,
  useYear,
  type OutflowKind,
} from "@/services/year";
import { formatDay } from "@/utils/dates";
import { formatINR, toPaise } from "@/utils/money";

const PANEL = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
const TITLE = "m-0 font-display text-[32px] leading-none";
const NOTE = "m-0 text-sm text-muted-foreground";

function Strip() {
  const { data, isLoading, isError } = useYear();
  if (isLoading) return <p className={NOTE}>Working out the year…</p>;
  if (isError || !data) return <p className={NOTE}>Could not load the year. Try again in a moment.</p>;
  const max = Math.max(...data.months.map((m) => m.closing_paise), 0);
  return (
    <section aria-labelledby="year-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="year-title" className={TITLE}>
          The year ahead
        </h2>
        <p className={NOTE}>
          {data.first_short_month
            ? `You would run short in ${monthLabel(data.first_short_month)} if nothing else changes. The lowest point is ${formatINR(data.lowest_paise)}.`
            : `No month ends below zero. The lowest point is ${formatINR(data.lowest_paise)}.`}
        </p>
        <p className={NOTE}>
          This counts your income, fixed payments and the large amounts below, and what you hold now. It leaves out everyday spending, so treat it as a ceiling.
        </p>
      </header>
      <ol className="flex flex-col gap-3">
        {data.months.map((m) => (
          <li key={m.month} className="flex flex-col gap-2 border-t border-border pt-3 first:border-t-0 first:pt-0">
            <div className="flex items-baseline justify-between gap-4">
              <span className="text-[15px] font-medium">{monthLabel(m.month)}</span>
              <span className={`text-[15px] tabular-nums ${m.short ? "text-destructive" : ""}`}>{formatINR(m.closing_paise)}</span>
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-[hsl(var(--chip))]" aria-hidden="true">
              <div className={`h-full rounded-full ${m.short ? "bg-destructive" : "bg-primary"}`} style={{ width: `${barPct(m.closing_paise, max)}%` }} />
            </div>
            {m.lumpy.length > 0 && (
              <ul className="flex flex-col gap-1">
                {m.lumpy.map((l, i) => (
                  <li key={`${l.on}-${l.label}-${i}`} className="flex justify-between gap-4 text-[13px] text-muted-foreground">
                    <span>{`${formatDay(l.on)} · ${l.label}`}</span>
                    <span className="tabular-nums">{`${l.direction === "in" ? "+" : "−"}${formatINR(l.amount_paise)}`}</span>
                  </li>
                ))}
              </ul>
            )}
          </li>
        ))}
      </ol>
    </section>
  );
}

function AddForm() {
  const add = useAddOutflow();
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");
  const [on, setOn] = useState("");
  const [kind, setKind] = useState<OutflowKind>("premium");
  const [direction, setDirection] = useState<"out" | "in">("out");
  const [yearly, setYearly] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const paise = toPaise(amount);
    if (!label.trim()) return setError("Give it a name.");
    if (paise === null || paise <= 0) return setError("Enter an amount above zero.");
    if (!/^\d{4}-\d{2}-\d{2}$/.test(on)) return setError("Pick a date.");
    setError(null);
    add.mutate(
      { label: label.trim(), on_day: on, amount_paise: paise, direction, kind, yearly },
      {
        onSuccess: () => {
          setLabel("");
          setAmount("");
          setOn("");
          setYearly(false);
        },
      },
    );
  };

  return (
    <form onSubmit={submit} className="form-grid border-t border-border pt-6" noValidate>
      <div className="field col-span-full">
        <Label htmlFor="out-label">What is it</Label>
        <Input id="out-label" value={label} onChange={(e) => setLabel(e.target.value)} maxLength={80} placeholder="e.g. Term insurance premium" />
      </div>
      <div className="field">
        <Label htmlFor="out-amount">Amount</Label>
        <MoneyInput id="out-amount" value={amount} onChange={(e) => setAmount(e.target.value)} />
      </div>
      <div className="field">
        <Label htmlFor="out-date">Date</Label>
        <Input id="out-date" type="date" value={on} onChange={(e) => setOn(e.target.value)} />
      </div>
      <div className="field">
        <Label htmlFor="out-kind">Kind</Label>
        <Select id="out-kind" value={kind} onChange={(e) => setKind(e.target.value as OutflowKind)}>
          {(Object.keys(KIND_LABEL) as OutflowKind[]).map((k) => (
            <option key={k} value={k}>
              {KIND_LABEL[k]}
            </option>
          ))}
        </Select>
      </div>
      <div className="field">
        <Label htmlFor="out-dir">Money</Label>
        <Select id="out-dir" value={direction} onChange={(e) => setDirection(e.target.value as "out" | "in")}>
          <option value="out">Going out</option>
          <option value="in">Coming in</option>
        </Select>
      </div>
      <label className="col-span-full flex items-center gap-2 text-sm">
        <input type="checkbox" checked={yearly} onChange={(e) => setYearly(e.target.checked)} />
        Repeats every year
      </label>
      {error && (
        <p role="alert" className="col-span-full text-sm text-destructive">
          {error}
        </p>
      )}
      <div className="form-actions">
        <Button type="submit" disabled={add.isPending}>
          Add
        </Button>
      </div>
    </form>
  );
}

function Outflows() {
  const { data } = useOutflows();
  const remove = useRemoveOutflow();
  return (
    <section aria-labelledby="out-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="out-title" className={TITLE}>
          Large amounts
        </h2>
        <p className={NOTE}>Premiums, tax, annual fees and maturities that are not fixed monthly payments. Add them so they show up months early.</p>
      </header>
      {data && data.length > 0 ? (
        <ul className="ledger">
          {data.map((o) => (
            <li key={o.outflow_id} className="ledger-row">
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">{o.label}</span>
                <span className="text-[13px] text-muted-foreground">
                  {`${KIND_LABEL[o.kind]} · ${o.next_on ? formatDay(o.next_on) : "done"}${o.yearly ? " · every year" : ""}`}
                  {o.set_aside_monthly_paise ? ` · set aside ${formatINR(o.set_aside_monthly_paise)} a month` : ""}
                </span>
              </span>
              <span className="flex shrink-0 items-center gap-3">
                <span className="tabular-nums">{`${o.direction === "in" ? "+" : ""}${formatINR(o.amount_paise)}`}</span>
                <Button type="button" variant="ghost" size="sm" onClick={() => remove.mutate(o.outflow_id)} aria-label={`Remove ${o.label}`}>
                  Remove
                </Button>
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className={NOTE}>Nothing added yet.</p>
      )}
      <AddForm />
    </section>
  );
}

/** The year ahead: twelve months of money, with the large amounts that would otherwise surprise you. */
export default function YearPage() {
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page page-narrow stack">
        <header>
          <h1 className="page-title">Year ahead</h1>
        </header>
        <Strip />
        <Outflows />
      </main>
    </div>
  );
}
