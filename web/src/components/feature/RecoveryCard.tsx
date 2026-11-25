import { LifeBuoy } from "lucide-react";

import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";

/** What an overspend costs and how to pay it back in small monthly chunks, without touching an FD. */
export function RecoveryCard({ summary }: { summary: DashboardSummary }) {
  const r = summary.recovery;
  if (!r && summary.spending.overspend_paise === 0) return null;
  return (
    <section
      aria-label="Paying it back"
      className="space-y-3 rounded-[28px] border border-warn/40 bg-[linear-gradient(135deg,hsl(var(--warn-tint)),hsl(var(--c-coral)/0.12))] p-6 text-sm leading-relaxed shadow-[0_18px_40px_-26px_hsl(var(--warn))] md:p-8"
    >
      <h2 className="flex items-center gap-2.5 font-display text-[32px] leading-none tracking-tight">
        <span
          aria-hidden
          className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[linear-gradient(145deg,hsl(var(--c-sun)),hsl(var(--c-coral)))] text-background"
        >
          <LifeBuoy className="h-5 w-5" strokeWidth={1.8} />
        </span>
        Paying it back
      </h2>
      {summary.spending.overspend_paise > 0 && (
        <p>
          You've spent {formatINR(summary.spending.overspend_paise)} more than
          this month's spending allowance of{" "}
          {formatINR(summary.spending.allowance_paise)}.
        </p>
      )}
      {r && (
        <p>
          {formatINR(r.owed_paise)} of savings and spending is missing this
          month. Rebuild it at{" "}
          <span className="rounded-md bg-card/70 px-1.5 py-0.5 font-semibold text-warn">
            {formatINR(r.per_month_paise)} a month for {r.months} months
          </span>
          : spend that much less, or trim your SIP by it, and no FD needs to be
          touched. Change the number of months on the Plan page.
        </p>
      )}
    </section>
  );
}
