import {
  barWidth,
  categoryLabel,
  formatShare,
  type CategoryAmount,
} from "@/utils/spending";
import { formatINR } from "@/utils/money";
import { categoryColor } from "@/utils/categoryColors";

/**
 * Spending by category as horizontal bars, largest first. Bars are scaled to the largest category;
 * the percentage is each category's share of the total spent.
 */
export function CategoryBars({
  rows,
  total,
}: {
  rows: CategoryAmount[];
  total: number;
}) {
  const max = rows[0]?.amount_paise ?? 0;
  return (
    <ul className="space-y-3">
      {rows.map((r) => (
        <li key={r.category} className="space-y-1">
          <div className="flex items-baseline justify-between gap-3 text-sm">
            <span className="flex min-w-0 items-center gap-2 font-medium">
              <span
                aria-hidden
                className="inline-block h-2.5 w-2.5 shrink-0 rounded-full"
                style={{ background: categoryColor(r.category) }}
              />
              <span className="truncate">{categoryLabel(r.category)}</span>
            </span>
            <span className="shrink-0 tabular-nums">
              {formatINR(r.amount_paise)}
              <span className="ml-2 text-xs text-muted-foreground">
                {formatShare(r.amount_paise, total)}
              </span>
            </span>
          </div>
          <div
            className="h-2.5 w-full overflow-hidden rounded-full"
            style={{ background: categoryColor(r.category, 0.15) }}
          >
            <div
              className="h-full rounded-full transition-[width] duration-700 ease-out"
              style={{
                width: `${barWidth(r.amount_paise, max)}%`,
                background: `linear-gradient(90deg, ${categoryColor(r.category, 0.7)}, ${categoryColor(r.category)})`,
              }}
            />
          </div>
        </li>
      ))}
    </ul>
  );
}
