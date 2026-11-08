import type { HistoryAutoPosting } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

/** The most recent balance changes the server made on your behalf. Shown only when auto-update is on. */
export function AutoPostings({ items, enabled }: { items: HistoryAutoPosting[]; enabled: boolean }) {
  if (!enabled || items.length === 0) return null;
  return (
    <section aria-label="Auto-updated balances" className="space-y-3">
      <div className="space-y-1">
        <h2 className="font-display text-3xl leading-none tracking-tight">Auto-updated balances</h2>
        <p className="text-sm text-muted-foreground">
          Salary and fixed payments are added to your balances on their dates. Turn this off in Plan settings.
        </p>
      </div>
      <ul className="divide-y glass rounded-3xl">
        {items.map((p, i) => (
          <li key={`${p.occurs_on}-${p.label}-${i}`} className="flex items-center justify-between gap-3 p-3 text-sm">
            <div className="min-w-0">
              <p className="truncate font-medium">{p.label}</p>
              <p className="text-xs text-muted-foreground">
                {formatDay(p.occurs_on)} · {p.account_name}
              </p>
            </div>
            <span
              className={cn(
                "shrink-0 font-semibold tabular-nums",
                p.amount_paise >= 0 ? "text-emerald-600 dark:text-emerald-400" : "text-foreground",
              )}
            >
              {p.amount_paise >= 0 ? "+" : "−"}
              {formatINR(Math.abs(p.amount_paise))}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
