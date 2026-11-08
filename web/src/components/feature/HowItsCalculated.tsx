import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

/** Every step from "cash in the bank" to the Safe to Spend number, so nothing is a black box. */
export function HowItsCalculated({ lines }: { lines: DashboardSummary["breakdown"] }) {
  return (
    <section aria-label="How this is calculated" className="space-y-3">
      <h2 className="text-lg font-bold tracking-tight">How this is calculated</h2>
      <ul className="divide-y rounded-xl border">
        {lines.map((l) => (
          <li
            key={l.label}
            data-total={l.total}
            className={cn("flex items-start justify-between gap-3 p-3 text-sm", l.total && "bg-muted/40 font-semibold")}
          >
            <div className="min-w-0">
              <p>{l.label}</p>
              {l.note && <p className="text-xs font-normal text-muted-foreground">{l.note}</p>}
            </div>
            <span
              className={cn(
                "shrink-0 tabular-nums",
                l.amount_paise < 0 && !l.total && "text-rose-600 dark:text-rose-400",
              )}
            >
              {l.amount_paise < 0 && !l.total ? "−" : ""}
              {formatINR(l.total ? l.amount_paise : Math.abs(l.amount_paise))}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
