import { useState } from "react";
import { Trash2 } from "lucide-react";

import type { TaxCommitment, TaxEntry, TaxRegime, TaxSectionId, TaxSectionRow } from "@/services/types";
import {
  useAddTaxEntry,
  useDeleteTaxEntry,
  useSetCommitmentTaxSection,
  useTaxReview,
} from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { formatINR, toPaise } from "@/utils/money";
import { cn } from "@/utils/cn";
import {
  daysLeftLabel,
  deductionError,
  fyLabel,
  nextReminder,
  regimeHeadline,
  sectionLabel,
  stackedWidths,
  TAX_SECTION_IDS,
} from "@/utils/review";

// ---------- regime ----------

function RegimeBox({ name, tax, better }: { name: string; tax: number; better: boolean }) {
  return (
    <div
      className={cn(
        "min-w-0 flex-1 glass rounded-2xl p-3",
        better && "border-emerald-500/50 bg-emerald-500/10",
      )}
    >
      <p className="text-xs text-muted-foreground">
        {name}
        {better && <span className="ml-1 font-semibold text-emerald-700 dark:text-emerald-300">· better</span>}
      </p>
      <p className="text-lg font-bold tabular-nums">{formatINR(tax)}</p>
      <p className="text-xs text-muted-foreground">tax</p>
    </div>
  );
}

function RegimeCard({ regime }: { regime: TaxRegime }) {
  return (
    <Card>
      <CardContent className="space-y-3 p-4">
        <p className="text-xl font-bold leading-snug sm:text-2xl">{regimeHeadline(regime)}</p>
        <div className="flex gap-3">
          <RegimeBox name="Old regime" tax={regime.old_tax_paise} better={regime.better === "old"} />
          <RegimeBox name="New regime" tax={regime.new_tax_paise} better={regime.better === "new"} />
        </div>
        <p className="text-xs text-muted-foreground">{regime.note}</p>
      </CardContent>
    </Card>
  );
}

// ---------- sections ----------

function SectionRow({ row }: { row: TaxSectionRow }) {
  const bar = stackedWidths(row.claimed_paise, row.projected_paise, row.limit_paise);
  return (
    <li className="space-y-1.5 py-2.5 text-sm">
      <div className="flex items-baseline justify-between gap-2">
        <span className="min-w-0 truncate font-medium">{sectionLabel(row.section)}</span>
        <span className="shrink-0 tabular-nums">
          <span className="font-semibold">{formatINR(row.claimed_paise)}</span>
          {row.limit_paise !== null && (
            <span className="text-muted-foreground"> of {formatINR(row.limit_paise)}</span>
          )}
        </span>
      </div>
      {row.limit_paise !== null && (
        <>
          <div className="flex h-2 w-full overflow-hidden rounded-full bg-muted" aria-hidden>
            <div className="h-full bg-primary" style={{ width: `${bar.claimed}%` }} />
            <div className="h-full bg-primary/30" style={{ width: `${bar.projected}%` }} />
          </div>
          <p className="text-xs text-muted-foreground">
            {row.left_paise !== null && row.left_paise > 0 ? `${formatINR(row.left_paise)} left` : "Limit used up"}
            {row.projected_paise > 0 && ` · ${formatINR(row.projected_paise)} more to come`}
          </p>
        </>
      )}
      {row.note && <p className="text-xs text-muted-foreground">{row.note}</p>}
    </li>
  );
}

// ---------- commitments ----------

function CommitmentRow({ c }: { c: TaxCommitment }) {
  const setSection = useSetCommitmentTaxSection();
  const total = c.paid_paise + c.projected_paise;
  return (
    <li className="flex flex-col gap-2 py-2.5 text-sm sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p className="truncate font-medium">{c.label}</p>
        <p className="text-xs text-muted-foreground tabular-nums">{formatINR(total)} this year</p>
      </div>
      <Select
        aria-label={`Tax section for ${c.label}`}
        className="h-9 w-full sm:w-52"
        value={c.tax_section ?? ""}
        disabled={setSection.isPending}
        onChange={(e) =>
          setSection.mutate({
            commitment_id: c.commitment_id,
            tax_section: e.target.value === "" ? null : (e.target.value as TaxSectionId),
          })
        }
      >
        <option value="">Doesn't save tax</option>
        {TAX_SECTION_IDS.map((id) => (
          <option key={id} value={id}>
            {sectionLabel(id)}
          </option>
        ))}
      </Select>
    </li>
  );
}

// ---------- deductions ----------

function AddDeductionForm() {
  const [section, setSection] = useState<TaxSectionId>("80C");
  const [label, setLabel] = useState("");
  const [amount, setAmount] = useState("");
  const [error, setError] = useState<string | null>(null);
  const add = useAddTaxEntry();

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const problem = deductionError(label, amount);
    if (problem) return setError(problem);
    setError(null);
    add.mutate(
      { section, label: label.trim(), amount_paise: toPaise(amount) ?? 0 },
      {
        onSuccess: () => {
          setLabel("");
          setAmount("");
        },
      },
    );
  };

  return (
    <form onSubmit={submit} className="grid grid-cols-2 gap-2 glass rounded-2xl p-3" noValidate>
      <div className="col-span-2 space-y-1 sm:col-span-1">
        <Label htmlFor="tax-section">Section</Label>
        <Select id="tax-section" value={section} onChange={(e) => setSection(e.target.value as TaxSectionId)}>
          {TAX_SECTION_IDS.map((id) => (
            <option key={id} value={id}>
              {sectionLabel(id)}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-1">
        <Label htmlFor="tax-amount">Amount</Label>
        <MoneyInput id="tax-amount" value={amount} onChange={(e) => setAmount(e.target.value)} />
      </div>
      <div className="col-span-2 space-y-1">
        <Label htmlFor="tax-label">What is it</Label>
        <Input
          id="tax-label"
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="e.g. PPF deposit"
        />
      </div>
      {error && <p role="alert" className="col-span-2 text-sm text-destructive">{error}</p>}
      <div className="col-span-2">
        <Button type="submit" size="sm" disabled={add.isPending}>
          Add deduction
        </Button>
      </div>
    </form>
  );
}

function DeductionList({ entries }: { entries: TaxEntry[] }) {
  const del = useDeleteTaxEntry();
  if (entries.length === 0) return <p className="text-sm text-muted-foreground">No deductions added yet.</p>;
  return (
    <ul className="divide-y">
      {entries.map((e) => (
        <li key={e.entry_id} className="flex items-center justify-between gap-3 py-2.5 text-sm">
          <div className="min-w-0">
            <p className="truncate font-medium">{e.label}</p>
            <p className="text-xs text-muted-foreground">{sectionLabel(e.section)}</p>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <p className="font-semibold tabular-nums">{formatINR(e.amount_paise)}</p>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              aria-label={`Delete ${e.label}`}
              disabled={del.isPending}
              onClick={() => {
                if (window.confirm(`Remove "${e.label}"?`)) del.mutate(e.entry_id);
              }}
            >
              <Trash2 className="h-4 w-4" aria-hidden />
            </Button>
          </div>
        </li>
      ))}
    </ul>
  );
}

// ---------- panel ----------

export function TaxPanel() {
  const { data: tax, isLoading, isError, error } = useTaxReview();
  const reminder = tax ? nextReminder(tax.reminders) : null;

  return (
    <section aria-label="Tax" className="space-y-4 glass rounded-3xl p-4 md:p-6">
      <div>
        <h2 className="font-display text-3xl leading-none">Tax (India)</h2>
        <p className="text-sm text-muted-foreground">
          {tax ? `${fyLabel(tax.fy)} · April to March` : "Deductions and the regime comparison for this year."}
        </p>
      </div>

      {isLoading && <p className="text-sm text-muted-foreground">Working out your tax…</p>}
      {isError && (
        <p role="alert" className="text-sm text-destructive">
          {error instanceof Error && error.message ? error.message : "Couldn't load your tax summary."}
        </p>
      )}

      {tax && (
        <>
          <RegimeCard regime={tax.regime} />

          {reminder && (
            <Card>
              <CardContent className="flex items-center justify-between gap-3 p-4 text-sm">
                <div className="min-w-0">
                  <p className="font-semibold">{reminder.text}</p>
                  <p className="text-xs text-muted-foreground">{reminder.date}</p>
                </div>
                <p className="shrink-0 font-semibold tabular-nums">{daysLeftLabel(reminder.days_left)}</p>
              </CardContent>
            </Card>
          )}

          <div>
            <h3 className="mb-1 font-semibold">Deductions so far</h3>
            <ul className="divide-y">
              {tax.sections.map((row) => (
                <SectionRow key={row.section} row={row} />
              ))}
            </ul>
          </div>

          <div className="space-y-2">
            <h3 className="font-semibold">Fixed payments that save tax</h3>
            {tax.commitments.length === 0 ? (
              <p className="text-sm text-muted-foreground">No fixed payments yet.</p>
            ) : (
              <ul className="divide-y">
                {tax.commitments.map((c) => (
                  <CommitmentRow key={c.commitment_id} c={c} />
                ))}
              </ul>
            )}
          </div>

          <div className="space-y-2">
            <h3 className="font-semibold">Add a deduction</h3>
            <AddDeductionForm />
            <DeductionList entries={tax.entries} />
          </div>

          {tax.interest_income_paise > 0 && (
            <p className="text-sm">
              <span className="font-semibold tabular-nums">{formatINR(tax.interest_income_paise)}</span> interest
              earned this FY — it's taxable.
            </p>
          )}
        </>
      )}
    </section>
  );
}
