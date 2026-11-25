import { Link } from "react-router-dom";
import { ArrowRight, ShoppingBag, Tags } from "lucide-react";

import { CategoryBars } from "@/components/feature/CategoryBars";
import { useSpendSummary } from "@/hooks/usePlanner";
import { formatINR } from "@/utils/money";
import { plural, spendingBreakdown } from "@/utils/spending";

/** Compact dashboard card: this month's spending and the top categories. Hidden if it can't load or nothing is tracked. */
export function SpendingPanel() {
  const { data, isError } = useSpendSummary(6);
  if (isError || !data) return null;

  const current = data.months[data.months.length - 1];
  if (!current) return null;
  if (current.spent_paise === 0 && current.by_category.length === 0)
    return null;

  const rows = spendingBreakdown(current.by_category).slice(0, 4);

  return (
    <section
      aria-label="Spending this month"
      className="mx-auto w-full max-w-5xl space-y-6 glass rounded-[28px] p-4 md:p-6"
    >
      <div className="flex items-end justify-between gap-3">
        <div className="min-w-0">
          <h2 className="flex items-center gap-2.5 font-display text-[32px] leading-none tracking-tight">
            <span
              aria-hidden
              className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-coral text-background shadow-[0_8px_18px_-8px_hsl(var(--c-coral))]"
            >
              <ShoppingBag className="h-[18px] w-[18px]" strokeWidth={1.8} />
            </span>
            Spending this month
          </h2>
          {/* Spending is coral everywhere, so the month's total wears it too. */}
          <p className="font-serif-num text-4xl text-[hsl(var(--foreground))] dark:text-coral">
            {formatINR(current.spent_paise)}
          </p>
        </div>
        <Link
          to="/spending"
          className="group inline-flex min-h-11 shrink-0 items-center gap-1 text-sm font-semibold text-brand-ink hover:underline"
        >
          View all
          <ArrowRight
            className="h-4 w-4 transition-transform group-hover:translate-x-0.5"
            aria-hidden
          />
        </Link>
      </div>
      {rows.length > 0 && (
        <CategoryBars rows={rows} total={current.spent_paise} />
      )}
      {data.uncategorized_count > 0 && (
        <Link
          to="/spending"
          className="flex min-h-11 items-center gap-2 rounded-2xl bg-warn-tint px-4 text-sm font-medium text-warn hover:underline"
        >
          <Tags className="h-4 w-4 shrink-0" aria-hidden />
          {plural(
            data.uncategorized_count,
            "transaction needs a category",
            "transactions need a category",
          )}
        </Link>
      )}
    </section>
  );
}
