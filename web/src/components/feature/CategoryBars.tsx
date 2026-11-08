import { barWidth, categoryLabel, formatShare, type CategoryAmount } from "@/utils/spending";
import { formatINR } from "@/utils/money";

/**
 * Spending by category as horizontal bars, largest first. Bars are scaled to the largest category;
 * the percentage is each category's share of the total spent.
 */
export function CategoryBars({ rows, total }: { rows: CategoryAmount[]; total: number }) {
  const max = rows[0]?.amount_paise ?? 0;
  return (
    <ul className="space-y-3">
      {rows.map((r) => (
        <li key={r.category} className="space-y-1">
          <div className="flex items-baseline justify-between gap-3 text-sm">
            <span className="min-w-0 truncate font-medium">{categoryLabel(r.category)}</span>
            <span className="shrink-0 tabular-nums">
              {formatINR(r.amount_paise)}
              <span className="ml-2 text-xs text-muted-foreground">{formatShare(r.amount_paise, total)}</span>
            </span>
          </div>
          <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
            <div className="h-full rounded-full bg-primary" style={{ width: `${barWidth(r.amount_paise, max)}%` }} />
          </div>
        </li>
      ))}
    </ul>
  );
}
