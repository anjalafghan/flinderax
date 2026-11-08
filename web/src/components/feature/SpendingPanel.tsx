import { Link } from "react-router-dom";

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
  if (current.spent_paise === 0 && current.by_category.length === 0) return null;

  const rows = spendingBreakdown(current.by_category).slice(0, 4);

  return (
    <section aria-label="Spending this month" className="mx-auto w-full max-w-5xl space-y-6 glass rounded-3xl p-4 md:p-6">
      <div className="flex items-end justify-between gap-3">
        <div className="min-w-0">
          <h2 className="font-display text-[32px] leading-none tracking-tight">Spending this month</h2>
          <p className="font-serif-num text-4xl">{formatINR(current.spent_paise)}</p>
        </div>
        <Link to="/spending" className="shrink-0 text-sm font-medium text-primary hover:underline">
          View all
        </Link>
      </div>
      {rows.length > 0 && <CategoryBars rows={rows} total={current.spent_paise} />}
      {data.uncategorized_count > 0 && (
        <Link to="/spending" className="block text-sm text-warn hover:underline">
          {plural(data.uncategorized_count, "transaction needs a category", "transactions need a category")}
        </Link>
      )}
    </section>
  );
}
