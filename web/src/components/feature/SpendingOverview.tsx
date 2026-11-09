import { Link } from "react-router-dom";

import { CategoryBars } from "@/components/feature/CategoryBars";
import type { SpendMonth, SpendSummary } from "@/services/types";
import { formatINR } from "@/utils/money";
import { monthShort, plural, spendingBreakdown } from "@/utils/spending";

/** Six small bars of total spending, oldest to newest; the current month is highlighted. */
function MonthBars({ months }: { months: SpendMonth[] }) {
  const W = 300;
  const H = 120;
  const top = 12;
  const bottom = 20;
  const max = Math.max(1, ...months.map((m) => m.spent_paise));
  const slot = W / months.length;
  const barW = slot * 0.6;

  return (
    <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Spending over the last six months" className="w-full">
      {months.map((m, i) => {
        const h = ((H - top - bottom) * m.spent_paise) / max;
        const x = i * slot + (slot - barW) / 2;
        const current = i === months.length - 1;
        return (
          <g key={m.month}>
            <title>{`${monthShort(m.month)}: ${formatINR(m.spent_paise)}`}</title>
            <rect
              x={x}
              y={H - bottom - h}
              width={barW}
              height={Math.max(h, 0)}
              rx={3}
              className={current ? "fill-primary" : "fill-primary/40"}
            />
            <text x={x + barW / 2} y={H - 6} textAnchor="middle" fontSize={10} className="fill-muted-foreground">
              {monthShort(m.month)}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

/** This month's spending: total, category split, the last six months and the top merchants. */
export function SpendingOverview({
  summary,
  isLoading,
  isError,
}: {
  summary: SpendSummary | undefined;
  isLoading: boolean;
  isError: boolean;
  /** Kept so existing callers still compile. The review control now links to /spending/sort. */
  onReviewUncategorized?: () => void;
}) {
  if (isLoading) return <div className="h-48 animate-pulse rounded-xl bg-muted/40" />;
  if (isError || !summary) return <p className="text-sm text-destructive">Couldn't load your spending.</p>;

  const current = summary.months[summary.months.length - 1];
  const spent = current?.spent_paise ?? 0;
  const rows = spendingBreakdown(current?.by_category ?? []);
  const uncategorized = summary.uncategorized_count;

  return (
    <section aria-label="This month" className="space-y-6 glass rounded-3xl p-4 md:p-6">
      <div>
        <p className="text-sm text-muted-foreground">Spent this month</p>
        <p className="font-serif-num text-5xl md:text-6xl">{formatINR(spent)}</p>
      </div>

      {uncategorized > 0 && (
        <Link
          to="/spending/sort"
          className="flex w-full items-center justify-between gap-3 rounded-lg border border-warn/40 bg-warn-tint p-3 text-left text-sm font-medium text-warn no-underline transition-colors hover:bg-warn-tint"
        >
          <span>{plural(uncategorized, "transaction needs a category", "transactions need a category")}</span>
          <span aria-hidden="true">Review</span>
        </Link>
      )}

      <div className="grid gap-6 md:grid-cols-2">
        <div className="min-w-0 space-y-3">
          <h3 className="text-sm font-semibold">By category</h3>
          {rows.length === 0 ? (
            <p className="empty-note">No spending yet this month.</p>
          ) : (
            <CategoryBars rows={rows} total={spent} />
          )}
        </div>
        <div className="min-w-0 space-y-3">
          <h3 className="text-sm font-semibold">Last 6 months</h3>
          {summary.months.length === 0 ? (
            <p className="empty-note">Nothing tracked yet.</p>
          ) : (
            <MonthBars months={summary.months} />
          )}
        </div>
      </div>

      {summary.top_merchants.length > 0 && (
        <div className="space-y-3 border-t border-border pt-6">
          <h3 className="text-sm font-semibold">Top merchants</h3>
          <ol className="ledger">
            {summary.top_merchants.map((m) => (
              <li key={m.merchant} className="ledger-row text-sm">
                <div className="min-w-0">
                  <p className="truncate font-medium">{m.merchant}</p>
                  <p className="text-xs text-muted-foreground">{plural(m.count, "payment", "payments")}</p>
                </div>
                <span className="shrink-0 font-semibold tabular-nums">{formatINR(m.amount_paise)}</span>
              </li>
            ))}
          </ol>
        </div>
      )}
    </section>
  );
}
