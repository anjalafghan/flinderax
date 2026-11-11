import type { DashboardSummary } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { useAnimatedList } from "@/hooks/useAnimatedList";

export function UpcomingList({ items, limit = 5 }: { items: DashboardSummary["upcoming"]; limit?: number }) {
  const shown = items.slice(0, limit);
  const listRef = useAnimatedList<HTMLUListElement>();
  return (
    <section aria-label="Upcoming payments" className="space-y-4">
      <h2 className="font-display text-[32px] leading-none tracking-tight">Upcoming payments</h2>
      {shown.length === 0 ? (
        <p className="empty-note">No payments before your next income.</p>
      ) : (
        <ul ref={listRef} className="ledger glass rounded-3xl px-5 py-2 md:px-6">
          {shown.map((u, i) => (
            <li key={`${u.date}-${u.label}-${i}`} className="ledger-row text-sm">
              <div className="min-w-0">
                <p className="truncate font-medium" title={u.label}>{u.label}</p>
                <p className="text-[13px] text-muted-foreground">{formatDay(u.date)}</p>
              </div>
              <span className="shrink-0 font-display text-lg tabular-nums">{formatINR(u.amount_paise)}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
