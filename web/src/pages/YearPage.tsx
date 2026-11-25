import { useState } from "react";

import { Header } from "@/components/layout/Header";
import { hue, ink, type Hue } from "@/components/feature/life/hue";
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

const PANEL =
  "glass flex flex-col gap-6 rounded-[28px] px-6 py-8 sm:px-12 sm:py-10";
const TITLE = "m-0 font-display text-[32px] leading-none brand-text";

/** Each kind of large amount has its colour: premiums sky, tax coral, fees sun, maturities mint, the wedding rose. */
const KIND_HUE: Record<string, Hue> = {
  premium: "sky",
  tax: "coral",
  fee: "sun",
  maturity: "mint",
  wedding: "rose",
  card: "sun",
  other: "grape",
};
const kindHue = (k: string): Hue => KIND_HUE[k] ?? "grape";

/** A month's colour by how much is left at its close: short coral, thin sun, then mint to sky as it fills. */
function monthHue(closing: number, max: number, short: boolean): Hue {
  if (short || closing < 0) return "coral";
  const share = max > 0 ? closing / max : 0;
  if (share < 0.25) return "sun";
  if (share < 0.6) return "mint";
  return "sky";
}

/** Twelve months as one strip of coloured tiles, a heatmap of the year at a glance. */
function Heat({
  months,
  max,
}: {
  months: { month: string; closing_paise: number; short: boolean; lumpy: unknown[] }[];
  max: number;
}) {
  return (
    <div aria-hidden className="grid grid-cols-6 gap-1.5 sm:grid-cols-12">
      {months.map((m) => {
        const h = monthHue(m.closing_paise, max, m.short);
        const share =
          max > 0 ? Math.max(0, Math.min(1, m.closing_paise / max)) : 0;
        return (
          <div
            key={m.month}
            className="relative flex aspect-square flex-col items-center justify-center rounded-xl text-[11px] font-semibold"
            style={{
              background: hue(h, m.short ? 0.45 : 0.16 + share * 0.4),
              color: ink(h),
              boxShadow: `inset 0 0 0 1px ${hue(h, 0.45)}`,
            }}
          >
            {monthLabel(m.month).slice(0, 3)}
            {m.lumpy.length > 0 && (
              <span
                className="absolute right-1 top-1 h-1.5 w-1.5 rounded-full"
                style={{ background: ink(h) }}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}
const NOTE = "m-0 text-sm text-muted-foreground";

function Strip() {
  const { data, isLoading, isError } = useYear();
  if (isLoading) return <p className={NOTE}>Working out the year…</p>;
  if (isError || !data)
    return (
      <p className={NOTE}>Could not load the year. Try again in a moment.</p>
    );
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
          {`It starts from what you hold now, adds your income and takes out fixed payments, card bills already owed, the large amounts below and ${formatINR(data.everyday_monthly_paise)} a month of everyday spending (your spending allowance).`}
        </p>
      </header>
      <div className="flex flex-col gap-2">
        <Heat months={data.months} max={max} />
        <p className="m-0 flex flex-wrap gap-x-4 gap-y-1 text-[12px] text-muted-foreground">
          {(
            [
              ["sky", "Plenty left"],
              ["mint", "Comfortable"],
              ["sun", "Thin"],
              ["coral", "Short"],
            ] as [Hue, string][]
          ).map(([h, name]) => (
            <span key={h} className="inline-flex items-center gap-1.5">
              <i
                aria-hidden
                className="inline-block h-2.5 w-2.5 rounded-sm"
                style={{ background: hue(h) }}
              />
              {name}
            </span>
          ))}
          <span>A dot marks a month with a large amount.</span>
        </p>
      </div>
      <ol className="flex flex-col gap-3">
        {data.months.map((m) => (
          <li
            key={m.month}
            className="flex flex-col gap-2 border-t border-border pt-3 first:border-t-0 first:pt-0"
          >
            <div className="flex items-baseline justify-between gap-4">
              <span className="text-[15px] font-medium">
                {monthLabel(m.month)}
              </span>
              <span
                className={`text-[15px] font-medium tabular-nums ${m.short ? "text-destructive" : ""}`}
              >
                {formatINR(m.closing_paise)}
              </span>
            </div>
            <p className="m-0 flex flex-wrap gap-x-3 text-[12px] tabular-nums text-muted-foreground">
              <span>{`In ${formatINR(m.income_paise + m.lumpy_in_paise)}`}</span>
              <span>{`Bills ${formatINR(m.fixed_paise)}`}</span>
              <span>{`Everyday ${formatINR(m.everyday_paise)}`}</span>
              {m.lumpy_out_paise > 0 && (
                <span>{`Large ${formatINR(m.lumpy_out_paise)}`}</span>
              )}
            </p>
            <div
              className="h-2 w-full overflow-hidden rounded-full bg-[hsl(var(--chip))]"
              aria-hidden="true"
            >
              <div
                className="h-full rounded-full"
                style={{
                  width: `${barPct(m.closing_paise, max)}%`,
                  background: m.short
                    ? "hsl(var(--destructive))"
                    : `linear-gradient(90deg, ${hue(monthHue(m.closing_paise, max, m.short), 0.7)}, ${hue(monthHue(m.closing_paise, max, m.short))})`,
                }}
              />
            </div>
            {m.lumpy.length > 0 && (
              <ul className="flex flex-col gap-1">
                {m.lumpy.map((l, i) => (
                  <li
                    key={`${l.on}-${l.label}-${i}`}
                    className="flex justify-between gap-4 text-[13px] text-muted-foreground"
                  >
                    <span className="inline-flex items-center gap-2">
                      <i
                        aria-hidden
                        className="inline-block h-2 w-2 shrink-0 rounded-full"
                        style={{
                          background:
                            l.direction === "in"
                              ? hue("mint")
                              : hue(kindHue(l.kind)),
                        }}
                      />
                      {`${formatDay(l.on)} · ${l.label}`}
                    </span>
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
    if (paise === null || paise <= 0)
      return setError("Enter an amount above zero.");
    if (!/^\d{4}-\d{2}-\d{2}$/.test(on)) return setError("Pick a date.");
    setError(null);
    add.mutate(
      {
        label: label.trim(),
        on_day: on,
        amount_paise: paise,
        direction,
        kind,
        yearly,
      },
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
    <form
      onSubmit={submit}
      className="form-grid border-t border-border pt-6"
      noValidate
    >
      <div className="field col-span-full">
        <Label htmlFor="out-label">What is it</Label>
        <Input
          id="out-label"
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          maxLength={80}
          placeholder="e.g. Term insurance premium"
        />
      </div>
      <div className="field">
        <Label htmlFor="out-amount">Amount</Label>
        <MoneyInput
          id="out-amount"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
        />
      </div>
      <div className="field">
        <Label htmlFor="out-date">Date</Label>
        <Input
          id="out-date"
          type="date"
          value={on}
          onChange={(e) => setOn(e.target.value)}
        />
      </div>
      <div className="field">
        <Label htmlFor="out-kind">Kind</Label>
        <Select
          id="out-kind"
          value={kind}
          onChange={(e) => setKind(e.target.value as OutflowKind)}
        >
          {(Object.keys(KIND_LABEL) as OutflowKind[]).map((k) => (
            <option key={k} value={k}>
              {KIND_LABEL[k]}
            </option>
          ))}
        </Select>
      </div>
      <div className="field">
        <Label htmlFor="out-dir">Money</Label>
        <Select
          id="out-dir"
          value={direction}
          onChange={(e) => setDirection(e.target.value as "out" | "in")}
        >
          <option value="out">Going out</option>
          <option value="in">Coming in</option>
        </Select>
      </div>
      <label className="col-span-full flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={yearly}
          onChange={(e) => setYearly(e.target.checked)}
        />
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
        <p className={NOTE}>
          Premiums, tax, annual fees and maturities that are not fixed monthly
          payments. Add them so they show up months early.
        </p>
      </header>
      {data && data.length > 0 ? (
        <ul className="ledger">
          {data.map((o) => (
            <li key={o.outflow_id} className="ledger-row">
              <span className="flex min-w-0 items-start gap-3">
                <span
                  aria-hidden
                  className="mt-1 h-3 w-3 shrink-0 rounded-full"
                  style={{
                    background: hue(kindHue(o.kind)),
                    boxShadow: `0 0 0 3px ${hue(kindHue(o.kind), 0.2)}`,
                  }}
                />
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">{o.label}</span>
                <span className="text-[13px] text-muted-foreground">
                  {`${KIND_LABEL[o.kind]} · ${o.next_on ? formatDay(o.next_on) : "done"}${o.yearly ? " · every year" : ""}`}
                  {o.set_aside_monthly_paise
                    ? ` · set aside ${formatINR(o.set_aside_monthly_paise)} a month`
                    : ""}
                </span>
              </span>
              </span>
              <span className="flex shrink-0 items-center gap-3">
                <span
                  className={`tabular-nums font-medium ${o.direction === "in" ? "text-good" : ""}`}
                >{`${o.direction === "in" ? "+" : ""}${formatINR(o.amount_paise)}`}</span>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => remove.mutate(o.outflow_id)}
                  aria-label={`Remove ${o.label}`}
                >
                  Remove
                </Button>
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="empty-note m-0">Nothing added yet.</p>
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
