import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import type { Choice, ChoiceKind } from "@/services/lifeline";
import { paiseToInput, toPaise } from "@/utils/money";

type FieldType = "money" | "int" | "pct" | "month" | "text" | "check";

interface FieldSpec {
  key: string;
  label: string;
  type: FieldType;
  hint?: string;
  /** What a blank field becomes. Without it, the field is required. */
  blank?: null | 0 | "";
  /** The text the field starts with. */
  start?: string;
  min?: number;
  max?: number;
}

const thisYear = new Date().getFullYear();
const nextMonth = (() => {
  const d = new Date();
  d.setMonth(d.getMonth() + 1, 1);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
})();

/** Every kind of choice: what to call it and which fields it needs. The field keys are the API's. */
const CHOICE_SPECS: Record<ChoiceKind, { title: string; fields: FieldSpec[] }> = {
  job_switch: {
    title: "Switch jobs",
    fields: [
      { key: "month", label: "Start month", type: "month", start: nextMonth },
      { key: "take_home_paise", label: "New take-home a month (₹)", type: "money" },
      { key: "gap_months", label: "Months without pay in between", type: "int", start: "0", min: 0, max: 36 },
      { key: "raise_bps", label: "Yearly raise after (%)", type: "pct", blank: null, hint: "Blank uses your usual raise." },
    ],
  },
  house: {
    title: "Buy a home",
    fields: [
      { key: "year", label: "Year", type: "int", start: String(thisYear + 3), min: thisYear, max: thisYear + 80 },
      { key: "price_paise", label: "Price, in today's rupees (₹)", type: "money" },
      { key: "down_bps", label: "Down payment (%)", type: "pct", start: "20" },
      { key: "loan_bps", label: "Loan rate (%)", type: "pct", start: "8.5" },
      { key: "tenure_years", label: "Loan years", type: "int", start: "20", min: 1, max: 40 },
      { key: "rent_saved_paise", label: "Rent you stop paying, a month (₹)", type: "money", blank: 0 },
    ],
  },
  child: {
    title: "Have a child",
    fields: [
      { key: "birth_year", label: "Birth year", type: "int", start: String(thisYear + 2), min: thisYear - 17, max: thisYear + 25 },
      { key: "another", label: "This is another child, not the one already in my plan", type: "check" },
    ],
  },
  retire_at: {
    title: "Retire at a different age",
    fields: [{ key: "age", label: "Retirement age", type: "int", start: "55", min: 30, max: 80 }],
  },
  big_spend: {
    title: "A big spend",
    fields: [
      { key: "name", label: "What for", type: "text", blank: "", hint: "A car, a trip, a course." },
      { key: "year", label: "Year", type: "int", start: String(thisYear + 1), min: thisYear, max: thisYear + 80 },
      { key: "amount_paise", label: "Cost, in today's rupees (₹)", type: "money" },
      { key: "every_years", label: "Repeat every (years)", type: "int", blank: null, min: 1, max: 50, hint: "Blank for once." },
    ],
  },
  break: {
    title: "Take a break from work",
    fields: [
      { key: "start", label: "Start month", type: "month", start: nextMonth },
      { key: "months", label: "Months without your pay", type: "int", start: "6", min: 1, max: 60 },
    ],
  },
  save_more: {
    title: "Spend less each month",
    fields: [{ key: "monthly_paise", label: "Less a month (₹)", type: "money", hint: "While you are working. It is invested instead." }],
  },
  stay_active: {
    title: "Stay active",
    fields: [
      {
        key: "medical_cut_bps",
        label: "Medical costs after 60, lower by (%)",
        type: "pct",
        start: "30",
        hint: "An estimate, not a promise. Change it to what you believe.",
      },
    ],
  },
};

const KINDS = Object.keys(CHOICE_SPECS) as ChoiceKind[];

type Draft = Record<string, string | boolean>;

function startDraft(kind: ChoiceKind, from?: Choice): Draft {
  const draft: Draft = {};
  const given = from && from.kind === kind ? (from as unknown as Record<string, unknown>) : undefined;
  for (const f of CHOICE_SPECS[kind].fields) {
    const v = given?.[f.key];
    if (f.type === "check") draft[f.key] = v === true;
    else if (v === undefined || v === null) draft[f.key] = given ? "" : (f.start ?? "");
    else if (f.type === "money") draft[f.key] = paiseToInput(Number(v));
    else if (f.type === "pct") draft[f.key] = String(Number(v) / 100);
    else draft[f.key] = String(v);
  }
  return draft;
}

/** Turns the typed fields into a choice, or throws an Error whose message names the field. */
function build(kind: ChoiceKind, draft: Draft): Choice {
  const out: Record<string, unknown> = { kind };
  for (const f of CHOICE_SPECS[kind].fields) {
    const raw = draft[f.key];
    if (f.type === "check") {
      out[f.key] = raw === true;
      continue;
    }
    const text = String(raw ?? "").trim();
    if (text === "") {
      if (f.blank === undefined) throw new Error(`${f.label}: fill this in.`);
      out[f.key] = f.blank;
      continue;
    }
    if (f.type === "text") out[f.key] = text;
    else if (f.type === "month") {
      if (!/^\d{4}-\d{2}$/.test(text)) throw new Error(`${f.label}: pick a month.`);
      out[f.key] = text;
    } else if (f.type === "money") {
      const p = toPaise(text);
      if (p === null || p < 0) throw new Error(`${f.label}: enter a plain rupee amount, like 250000.`);
      out[f.key] = p;
    } else {
      const n = Number(text);
      const ok = f.type === "pct" ? Number.isFinite(n) && n >= 0 && n <= 100 : Number.isInteger(n);
      if (!ok) throw new Error(`${f.label}: enter a ${f.type === "pct" ? "percentage" : "whole number"}.`);
      if (f.min !== undefined && n < f.min) throw new Error(`${f.label} must be ${f.min} or more.`);
      if (f.max !== undefined && n > f.max) throw new Error(`${f.label} must be ${f.max} or less.`);
      out[f.key] = f.type === "pct" ? Math.round(n * 100) : n;
    }
  }
  return out as unknown as Choice;
}

/** Pick a kind of choice, fill it in, and hand it to `onTry`. Saves nothing itself. */
export function ChoiceForm({ initial, busy, onTry }: { initial?: Choice; busy: boolean; onTry: (c: Choice) => void }) {
  const [kind, setKind] = useState<ChoiceKind>(initial?.kind ?? "job_switch");
  const [draft, setDraft] = useState<Draft>(() => startDraft(initial?.kind ?? "job_switch", initial));
  const [error, setError] = useState<string | null>(null);

  const pick = (k: ChoiceKind) => {
    setKind(k);
    setDraft(startDraft(k));
    setError(null);
  };

  const submit = (e: FormEvent) => {
    e.preventDefault();
    try {
      const choice = build(kind, draft);
      setError(null);
      onTry(choice);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Check the details and try again.");
    }
  };

  return (
    <form onSubmit={submit} noValidate className="grid gap-5">
      <div className="field">
        <Label htmlFor="choice-kind">What if I...</Label>
        <Select id="choice-kind" value={kind} onChange={(e) => pick(e.target.value as ChoiceKind)}>
          {KINDS.map((k) => (
            <option key={k} value={k}>
              {CHOICE_SPECS[k].title}
            </option>
          ))}
        </Select>
      </div>
      <div className="form-grid">
        {CHOICE_SPECS[kind].fields.map((f) => {
          const id = `choice-${f.key}`;
          if (f.type === "check") {
            return (
              <label key={f.key} htmlFor={id} className="col-span-full flex min-h-11 items-center gap-3 text-sm">
                <input
                  id={id}
                  type="checkbox"
                  className="h-5 w-5"
                  checked={draft[f.key] === true}
                  onChange={(e) => setDraft((d) => ({ ...d, [f.key]: e.target.checked }))}
                />
                {f.label}
              </label>
            );
          }
          const common = {
            id,
            value: String(draft[f.key] ?? ""),
            onChange: (e: { target: { value: string } }) => setDraft((d) => ({ ...d, [f.key]: e.target.value })),
          };
          return (
            <div key={f.key} className="field">
              <Label htmlFor={id}>{f.label}</Label>
              {f.type === "money" ? (
                <MoneyInput {...common} />
              ) : f.type === "month" ? (
                <Input {...common} type="month" />
              ) : f.type === "text" ? (
                <Input {...common} maxLength={40} />
              ) : (
                <Input {...common} inputMode={f.type === "pct" ? "decimal" : "numeric"} />
              )}
              {f.hint && <p className="m-0 text-[13px] leading-snug text-muted-foreground">{f.hint}</p>}
            </div>
          );
        })}
      </div>
      {error && (
        <p role="alert" className="m-0 text-sm text-destructive">
          {error}
        </p>
      )}
      <div className="form-actions">
        <Button type="submit" disabled={busy}>
          {busy ? "Working it out..." : "See what changes"}
        </Button>
      </div>
    </form>
  );
}
