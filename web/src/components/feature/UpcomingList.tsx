import type { DashboardSummary } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";

export function UpcomingList({ items, limit = 5 }: { items: DashboardSummary["upcoming"]; limit?: number }) {
  const shown = items.slice(0, limit);
  return (
    <section aria-label="Upcoming payments" className="space-y-3">
      <h2 className="text-lg font-bold tracking-tight">Upcoming payments</h2>
      {shown.length === 0 ? (
        <p className="rounded-xl border border-dashed p-4 text-sm text-muted-foreground">
          No payments before your next income.
        </p>
      ) : (
        <ul className="divide-y rounded-xl border">
          {shown.map((u, i) => (
            <li key={`${u.date}-${u.label}-${i}`} className="flex items-center justify-between gap-3 p-3 text-sm">
              <div className="min-w-0">
                <p className="truncate font-medium">{u.label}</p>
                <p className="text-xs text-muted-foreground">{formatDay(u.date)}</p>
              </div>
              <span className="font-semibold tabular-nums">{formatINR(u.amount_paise)}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
