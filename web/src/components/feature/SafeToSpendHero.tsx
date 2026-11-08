import { AlertTriangle, ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";

import type { DashboardSummary, Status } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const TONE: Record<Status, string> = {
  green: "You're fine",
  yellow: "Spend carefully",
  red: "Don't spend",
};

/** 1533333 paise -> ["₹15,333", ".33"]: the rupees big, the paise small and italic. */
function splitINR(paise: number): [string, string] {
  const full = formatINR(paise);
  const dot = full.lastIndexOf(".");
  return dot === -1 ? [full, ""] : [full.slice(0, dot), full.slice(dot)];
}

/** The one number: what is safe to spend from now until the next payday. */
export function SafeToSpendHero({ summary }: { summary: DashboardSummary }) {
  const spendName = summary.pots.spending.account_name ?? "your spending account";
  const [rupees, paise] = splitINR(summary.safe_to_spend_paise);
  return (
    <section
      aria-labelledby="safe-title"
      data-status={summary.status}
      className={cn(
        "glass streaks relative flex flex-col gap-9 overflow-hidden rounded-[2rem] p-6 sm:p-10",
        summary.status === "red" && "border-destructive/50",
      )}
    >
      <div className="relative flex flex-wrap items-center justify-between gap-3 eyebrow">
        <span>until {formatDay(summary.until)}</span>
        <span className="rounded-full border border-foreground/30 px-3 py-1.5 text-foreground">{TONE[summary.status]}</span>
      </div>

      <div className="relative flex flex-wrap items-end justify-between gap-x-10 gap-y-6">
        <div className="flex min-w-0 flex-col gap-1.5">
          <h1 id="safe-title" className="font-display text-3xl text-foreground/80">
            Safe to spend
          </h1>
          <p className="font-serif-num glow-text text-[84px] leading-[0.92] text-foreground sm:text-[128px] lg:text-[160px]">
            {rupees}
            <span className="font-display text-[0.55em] tracking-tight text-foreground/80">{paise}</span>
          </p>
          <p className="mt-3 text-base text-foreground/80">
            <span className="font-serif-num text-2xl text-foreground">{formatINR(summary.per_week_paise)}</span> a week, for the
            next {summary.days_left} days
          </p>
        </div>
        <div className="flex flex-col items-end text-right">
          <span className="font-display text-[88px] leading-[0.85] sm:text-[110px]">{summary.days_left}</span>
          <span className="eyebrow">days until {summary.next_income ? formatDay(summary.next_income.arrives) : formatDay(summary.until)}</span>
        </div>
      </div>

      <div className="relative flex flex-wrap items-center justify-between gap-4 border-t border-border pt-6">
        <div className="flex flex-col gap-2 text-sm text-muted-foreground">
          <span>
            {summary.spending.tracked
              ? `${formatINR(summary.spending.spent_paise)} spent on your cards since tracking began, out of a ${formatINR(summary.spending.allowance_paise)} monthly allowance.`
              : "Spending isn't tracked yet. Update a card from your bank app to start."}
          </span>
          {summary.short_by_paise > 0 && (
            <span className="text-foreground">
              {spendName} is short by {formatINR(summary.short_by_paise)} for the moves below. Mark what you've already dealt with,
              or skip this month's savings top-up.
            </span>
          )}
          {summary.spending.tracked && summary.spending.overspend_paise > 0 && (
            <span className="text-foreground">
              You've gone {formatINR(summary.spending.overspend_paise)} over this month's spending allowance.
            </span>
          )}
          {summary.next_income && (
            <span>
              {summary.next_income.label} (arrives {formatDay(summary.next_income.arrives)}) is not counted.
            </span>
          )}
          {summary.stale.map((st) => (
            <span key={st.what} className="inline-flex items-center gap-2 text-primary">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              {st.what} is {st.days_old} days old. Update it for a reliable number.
            </span>
          ))}
        </div>
        <Link to="/working" className="font-display inline-flex min-h-11 items-center gap-2.5 text-[22px] text-primary hover:text-foreground">
          See the working
          <ArrowRight className="h-[18px] w-[18px]" aria-hidden />
        </Link>
      </div>
    </section>
  );
}
