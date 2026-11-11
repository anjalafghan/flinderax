import { useJoySummary } from "@/hooks/usePlanner";
import { cn } from "@/utils/cn";
import { formatINR } from "@/utils/money";
import { barPercent, budgetTone, dayLabel } from "@/utils/joy";
import { DatesNudge } from "@/joy/DatesNudge";

/** Dashboard card: this month's date budget from Joy, and the user's share of it. Hidden unless Joy is connected. */
export function DatesPanel() {
  const { data, isError } = useJoySummary();
  if (isError || !data || !data.connected || !data.month) return null;

  const m = data.month;
  const tone = budgetTone(m.spent_paise, m.budget_paise);
  const width = barPercent(m.spent_paise, m.budget_paise);
  const recent = data.recent.slice(0, 4);

  return (
    <section aria-label="Dates this month" className="space-y-6 glass rounded-3xl p-4 md:p-6">
      <div>
        <h2 className="font-display text-[32px] leading-none tracking-tight">Dates</h2>
        <DatesNudge budget_paise={m.budget_paise} spent_paise={m.spent_paise} />
        <p className="font-serif-num text-4xl">
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
                tone === "over" ? "bg-over" : tone === "warn" ? "bg-warn" : "bg-primary",
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
        <ul className="ledger text-sm">
          {recent.map((r, i) => (
            <li key={`${r.day}-${i}`} className="ledger-row items-baseline">
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

    </section>
  );
}
