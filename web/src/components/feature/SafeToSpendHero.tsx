import { AlertTriangle } from "lucide-react";

import type { DashboardSummary, Status } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const TONE: Record<Status, { box: string; label: string }> = {
  green: {
    box: "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
    label: "You're fine",
  },
  yellow: {
    box: "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300",
    label: "Spend carefully",
  },
  red: {
    box: "border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-300",
    label: "Don't spend",
  },
};

export function SafeToSpendHero({ summary }: { summary: DashboardSummary }) {
  const tone = TONE[summary.status];
  const low = summary.lowest_point;
  return (
    <section
      aria-label="Safe to spend"
      data-status={summary.status}
      className={cn("rounded-3xl border p-6 md:p-8", tone.box)}
    >
      <div className="flex items-center justify-between gap-2 text-sm font-medium">
        <span>Safe to spend until {formatDay(summary.until)}</span>
        <span className="rounded-full bg-background/60 px-3 py-1 text-xs font-semibold">{tone.label}</span>
      </div>
      <p className="mt-3 text-5xl font-extrabold tracking-tight tabular-nums md:text-6xl">
        {formatINR(summary.safe_to_spend_paise)}
      </p>
      <p className="mt-2 text-lg font-semibold tabular-nums">
        {formatINR(summary.per_week_paise)} <span className="text-sm font-medium opacity-80">per week</span>
      </p>

      <dl className="mt-5 grid gap-x-8 gap-y-1 text-sm sm:grid-cols-2">
        {summary.period_label && (
          <div className="flex justify-between gap-4">
            <dt className="opacity-80">This period</dt>
            <dd className="font-medium">{summary.period_label}</dd>
          </div>
        )}
        {summary.next_income && (
          <div className="flex justify-between gap-4">
            <dt className="opacity-80">Next income</dt>
            <dd className="font-medium">
              {summary.next_income.label}, arrives {formatDay(summary.next_income.arrives)}
            </dd>
          </div>
        )}
        <div className="flex justify-between gap-4">
          <dt className="opacity-80">Lowest balance</dt>
          <dd className="font-medium tabular-nums">
            {formatINR(low.amount_paise)} on {formatDay(low.date)}
          </dd>
        </div>
      </dl>

      {summary.stale.length > 0 && (
        <ul className="mt-4 space-y-1 text-sm">
          {summary.stale.map((s) => (
            <li key={s.what} className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              {s.what} is {s.days_old} days old, update it for a reliable number.
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
