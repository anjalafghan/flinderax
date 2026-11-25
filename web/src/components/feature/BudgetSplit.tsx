import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

function Bar({
  label,
  amount,
  income,
  sub,
  className,
}: {
  label: string;
  amount: number;
  income: number;
  sub?: string;
  className: string;
}) {
  const pct = income > 0 ? (amount / income) * 100 : 0;
  return (
    <li className="space-y-2">
      <div className="flex justify-between gap-4 text-sm">
        <span className="flex items-center gap-2 font-medium">
          <span
            aria-hidden
            className={cn("inline-block h-2.5 w-2.5 rounded-full", className)}
          />
          {label}{" "}
          <span className="text-muted-foreground">({pct.toFixed(1)}%)</span>
        </span>
        <span className="tabular-nums">{formatINR(amount)}</span>
      </div>
      <div className="h-2.5 overflow-hidden rounded-full bg-muted">
        <div
          className={cn("h-full rounded-full", className)}
          style={{ width: `${Math.min(100, pct)}%` }}
        />
      </div>
      {sub && <p className="text-xs text-muted-foreground">{sub}</p>}
    </li>
  );
}

/** Needs, spending, savings in amounts: what your income is divided into each month. */
export function BudgetSplit({
  budget,
  spending,
}: {
  budget: DashboardSummary["budget"];
  spending: DashboardSummary["spending"];
}) {
  if (budget.income_paise <= 0) return null;
  const used =
    spending.allowance_paise > 0
      ? Math.min(100, (spending.spent_paise / spending.allowance_paise) * 100)
      : 0;
  const share = (paise: number) =>
    Math.max(0, Math.min(100, (paise / budget.income_paise) * 100));
  return (
    <section aria-label="Monthly split" className="space-y-4">
      <div className="space-y-2">
        <h2 className="font-display text-[32px] leading-none tracking-tight">
          Your monthly split
        </h2>
        <p className="text-sm text-muted-foreground">
          Out of {formatINR(budget.income_paise)} a month.
        </p>
      </div>
      {/* The whole month at a glance: one bar split into needs, spending and savings. */}
      <div
        aria-hidden
        className="flex h-4 w-full overflow-hidden rounded-full bg-muted shadow-inner"
      >
        <div
          className="h-full bg-gradient-to-r from-sky to-teal"
          style={{ width: `${share(budget.needs_paise)}%` }}
        />
        <div
          className="h-full bg-gradient-to-r from-sun to-coral"
          style={{ width: `${share(budget.spending_paise)}%` }}
        />
        <div
          className="h-full bg-gradient-to-r from-mint to-teal"
          style={{ width: `${share(budget.savings_paise)}%` }}
        />
      </div>
      <ul className="space-y-5">
        <Bar
          label="Needs"
          amount={budget.needs_paise}
          income={budget.income_paise}
          className="bg-gradient-to-r from-sky to-teal"
          sub="Your listed bills, paid from the reserve"
        />
        <Bar
          label="Spending"
          amount={budget.spending_paise}
          income={budget.income_paise}
          className="bg-gradient-to-r from-sun to-coral"
          sub={
            spending.tracked
              ? `${formatINR(spending.spent_paise)} spent so far this period (${used.toFixed(0)}% of it)`
              : "Not tracked yet: add the unbilled numbers when you update a card"
          }
        />
        <Bar
          label="Savings & investing"
          amount={budget.savings_paise}
          income={budget.income_paise}
          className="bg-gradient-to-r from-mint to-teal"
          sub={`SIP ${formatINR(budget.sip_paise)} + ${formatINR(budget.rest_savings_paise)} to the reserve`}
        />
      </ul>
    </section>
  );
}
