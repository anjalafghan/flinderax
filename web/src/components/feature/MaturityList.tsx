import type { HistoryMaturity } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";

function inDays(n: number): string {
  if (n <= 0) return "today";
  if (n === 1) return "in 1 day";
  return `in ${n} days`;
}

/** FDs and RDs that mature soon. Hidden when nothing is maturing. */
export function MaturityList({ items }: { items: HistoryMaturity[] }) {
  if (items.length === 0) return null;
  return (
    <section aria-label="Upcoming maturities" className="space-y-4">
      <h2 className="font-display text-[32px] leading-none tracking-tight">Upcoming maturities</h2>
      <ul className="ledger glass rounded-3xl px-5 md:px-6">
        {items.map((m) => (
          <li key={m.account_id} className="ledger-row text-sm">
            <div className="min-w-0">
              <p className="truncate font-medium">
                {m.name} <span className="text-xs font-normal uppercase text-muted-foreground">{m.kind}</span>
              </p>
              <p className="text-xs text-muted-foreground">
                {formatDay(m.maturity_date)} · {inDays(m.days_left)}
              </p>
            </div>
            <span className="shrink-0 font-semibold tabular-nums">{formatINR(m.balance_paise)}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}
