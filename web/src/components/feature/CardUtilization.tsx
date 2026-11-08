import type { HistoryCard } from "@/services/types";
import { formatINR, formatPercent } from "@/utils/money";
import { cn } from "@/utils/cn";

/** Under 30% reads as healthy, 30-70% as a warning, over 70% as a risk. */
function barTone(bps: number): string {
  if (bps < 3000) return "bg-emerald-500";
  if (bps <= 7000) return "bg-amber-500";
  return "bg-rose-500";
}

/** How much of each card's credit limit is in use right now. Hidden when there are no cards. */
export function CardUtilization({ cards }: { cards: HistoryCard[] }) {
  if (cards.length === 0) return null;
  return (
    <section aria-label="Card utilization" className="space-y-3 rounded-xl border p-4 md:p-6">
      <h2 className="text-lg font-bold tracking-tight">Card usage</h2>
      <ul className="space-y-4">
        {cards.map((c) => {
          const hasLimit = c.limit_paise != null && c.limit_paise > 0 && c.utilization_bps != null;
          return (
            <li key={c.card_id} className="space-y-1.5">
              <div className="flex items-baseline justify-between gap-3 text-sm">
                <p className="min-w-0 truncate font-medium">{c.name}</p>
                {hasLimit ? (
                  <p className="shrink-0 tabular-nums text-muted-foreground">
                    {formatINR(c.outstanding_paise)} of {formatINR(c.limit_paise!)}
                    <span className="ml-2 font-semibold text-foreground">{formatPercent(c.utilization_bps!)}</span>
                  </p>
                ) : (
                  <p className="shrink-0 tabular-nums text-muted-foreground">{formatINR(c.outstanding_paise)} owed</p>
                )}
              </div>
              {hasLimit ? (
                <div
                  role="progressbar"
                  aria-label={`${c.name} utilization`}
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-valuenow={Math.round(c.utilization_bps! / 100)}
                  className="h-2 w-full overflow-hidden rounded-full bg-muted"
                >
                  <div
                    className={cn("h-full rounded-full transition-all", barTone(c.utilization_bps!))}
                    style={{ width: `${Math.min(100, Math.max(0, c.utilization_bps! / 100))}%` }}
                  />
                </div>
              ) : (
                <p className="text-xs text-muted-foreground">add a credit limit to see usage</p>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
