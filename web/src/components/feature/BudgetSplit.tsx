import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

function Bar({ label, amount, income, sub, className }: { label: string; amount: number; income: number; sub?: string; className: string }) {
  const pct = income > 0 ? (amount / income) * 100 : 0;
  return (
    <li className="space-y-1">
      <div className="flex justify-between text-sm">
        <span className="font-medium">
          {label} <span className="text-muted-foreground">({pct.toFixed(1)}%)</span>
        </span>
        <span className="tabular-nums">{formatINR(amount)}</span>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-muted">
        <div className={cn("h-full rounded-full", className)} style={{ width: `${Math.min(100, pct)}%` }} />
      </div>
      {sub && <p className="text-xs text-muted-foreground">{sub}</p>}
    </li>
  );
}

/** Needs, spending, savings in amounts: what your income is divided into each month. */
export function BudgetSplit({ budget, spending }: { budget: DashboardSummary["budget"]; spending: DashboardSummary["spending"] }) {
  if (budget.income_paise <= 0) return null;
  const used = spending.allowance_paise > 0 ? Math.min(100, ((spending.spent_paise) / spending.allowance_paise) * 100) : 0;
  return (
    <section aria-label="Monthly split" className="space-y-3">
      <div>
        <h2 className="font-display text-[32px] leading-none tracking-tight">Your monthly split</h2>
        <p className="text-xs text-muted-foreground">Out of {formatINR(budget.income_paise)} a month.</p>
      </div>
      <ul className="space-y-3">
        <Bar label="Needs" amount={budget.needs_paise} income={budget.income_paise} className="bg-primary" sub="Your listed bills, paid from the reserve" />
        <Bar
          label="Spending"
          amount={budget.spending_paise}
          income={budget.income_paise}
          className="bg-muted-foreground"
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
          className="bg-good"
          sub={`SIP ${formatINR(budget.sip_paise)} + ${formatINR(budget.rest_savings_paise)} to the reserve`}
        />
      </ul>
    </section>
  );
}
