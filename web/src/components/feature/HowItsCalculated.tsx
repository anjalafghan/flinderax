import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

/** Every step from "cash in the bank" to the Safe to Spend number, so nothing is a black box. */
export function HowItsCalculated({ lines }: { lines: DashboardSummary["breakdown"] }) {
  return (
    <section aria-label="How this is calculated" className="space-y-4">
      <h2 className="font-display text-[32px] leading-none tracking-tight">How this is calculated</h2>
      <ul className="glass rounded-3xl px-5 py-2 md:px-6">
        {lines.map((l) => (
          <li
            key={l.label}
            data-total={l.total}
            className={cn(
              "flex items-start justify-between gap-4 py-4 text-sm first:pt-3 last:pb-3",
              l.total ? "border-t border-foreground font-semibold" : "border-t border-border first:border-t-0",
            )}
          >
            <div className="min-w-0">
              <p>{l.label}</p>
              {l.note && <p className="text-[13px] font-normal leading-snug text-muted-foreground">{l.note}</p>}
            </div>
            <span
              className={cn(
                "shrink-0 font-display text-lg tabular-nums",
                l.amount_paise < 0 && !l.total && "text-over",
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
