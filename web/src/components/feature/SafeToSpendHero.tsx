import { AlertTriangle } from "lucide-react";
import { Link } from "react-router-dom";

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

/** The one number: what is safe to spend from now until the next payday. */
export function SafeToSpendHero({ summary }: { summary: DashboardSummary }) {
  const tone = TONE[summary.status];
  const spendName = summary.pots.spending.account_name ?? "your spending account";
  return (
    <section
      aria-label="Safe to spend"
      data-status={summary.status}
      className={cn("rounded-3xl border p-6 md:p-8", tone.box)}
    >
      <div className="flex items-center justify-between gap-2 text-sm font-medium">
        <span>
          Safe to spend until {formatDay(summary.until)} ({summary.days_left} days)
        </span>
        <span className="rounded-full bg-background/60 px-3 py-1 text-xs font-semibold">{tone.label}</span>
      </div>
      <p className="mt-3 text-5xl font-extrabold tracking-tight tabular-nums md:text-6xl">
        {formatINR(summary.safe_to_spend_paise)}
      </p>
      <p className="mt-2 text-lg font-semibold tabular-nums">
        {formatINR(summary.per_week_paise)} <span className="text-sm font-medium opacity-80">per week</span>
      </p>

      <p className="mt-3 text-sm opacity-90">
        {summary.spending.tracked
          ? `${formatINR(summary.spending.spent_paise)} spent on your cards since tracking began, out of a ${formatINR(summary.spending.allowance_paise)} monthly allowance.`
          : "Spending isn't tracked yet. Update a card from your bank app to start."}
      </p>

      {summary.short_by_paise > 0 && (
        <p className="mt-3 text-sm font-medium">
          {spendName} is short by {formatINR(summary.short_by_paise)} for the moves below. Mark what you've already
          dealt with, or skip this month's savings top-up.
        </p>
      )}
      {summary.spending.tracked && summary.spending.overspend_paise > 0 && (
        <p className="mt-3 text-sm font-medium">
          You've gone {formatINR(summary.spending.overspend_paise)} over this month's spending allowance.
        </p>
      )}

      <div className="mt-4 flex flex-wrap items-center justify-between gap-2 text-sm">
        <span className="opacity-80">
          Only this period's money.{" "}
          {summary.next_income ? `${summary.next_income.label} (arrives ${formatDay(summary.next_income.arrives)}) is not counted.` : ""}
        </span>
        <Link to="/working" className="font-semibold underline underline-offset-4">
          See the working
        </Link>
      </div>

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
