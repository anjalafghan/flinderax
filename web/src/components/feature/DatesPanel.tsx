import { useJoySummary } from "@/hooks/usePlanner";
import { cn } from "@/utils/cn";
import { formatINR } from "@/utils/money";
import { barPercent, budgetTone, dayLabel, syncedLabel } from "@/utils/joy";

/** Dashboard card: this month's date budget from Joy, and the user's share of it. Hidden unless Joy is connected. */
export function DatesPanel() {
  const { data, isError } = useJoySummary();
  if (isError || !data || !data.connected || !data.month) return null;

  const m = data.month;
  const tone = budgetTone(m.spent_paise, m.budget_paise);
  const width = barPercent(m.spent_paise, m.budget_paise);
  const recent = data.recent.slice(0, 4);

  return (
    <section aria-label="Dates this month" className="space-y-4 rounded-xl border p-4 md:p-6">
      <div>
        <h2 className="text-lg font-bold tracking-tight">Dates</h2>
        <p className="text-2xl font-extrabold tabular-nums">
          {formatINR(m.spent_paise)}
          {m.budget_paise !== null && <span className="text-base font-semibold text-muted-foreground"> of {formatINR(m.budget_paise)}</span>}
          <span className="text-base font-semibold text-muted-foreground"> spent</span>
        </p>
        {m.budget_paise !== null && (
          <div
            role="progressbar"
            aria-label="Date budget used"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={width}
            className="mt-2 h-2 w-full overflow-hidden rounded-full bg-muted"
          >
            <div
              className={cn(
                "h-full rounded-full transition-[width] duration-300",
                tone === "over" ? "bg-rose-500" : tone === "warn" ? "bg-amber-500" : "bg-primary",
              )}
              style={{ width: `${width}%` }}
            />
          </div>
        )}
        <p className="mt-2 text-sm text-muted-foreground tabular-nums">
          Your share {formatINR(m.your_share_paise)}
          {m.your_budget_paise !== null && <> of {formatINR(m.your_budget_paise)}</>}
        </p>
        {m.carry_paise > 0 && (
          <p className="mt-1 text-xs text-muted-foreground">includes {formatINR(m.carry_paise)} rolled over</p>
        )}
      </div>

      {recent.length > 0 && (
        <ul className="divide-y border-t text-sm">
          {recent.map((r, i) => (
            <li key={`${r.day}-${i}`} className="flex items-baseline justify-between gap-3 py-2">
              <span className="shrink-0 text-xs text-muted-foreground">{dayLabel(r.day)}</span>
              <span className="min-w-0 flex-1 truncate">{r.title}</span>
              <span className="shrink-0 text-right tabular-nums">
                {formatINR(r.amount_paise)}
                <span className="block text-xs text-muted-foreground">you {formatINR(r.your_share_paise)}</span>
              </span>
            </li>
          ))}
        </ul>
      )}

      {data.stale && (
        <p className="text-xs text-muted-foreground">
          Joy couldn't be reached — showing figures from {syncedLabel(data.synced_at)}.
        </p>
      )}
    </section>
  );
}
