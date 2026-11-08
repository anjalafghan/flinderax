import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";

/** What an overspend costs and how to pay it back in small monthly chunks, without touching an FD. */
export function RecoveryCard({ summary }: { summary: DashboardSummary }) {
  const r = summary.recovery;
  if (!r && summary.spending.overspend_paise === 0) return null;
  return (
    <section aria-label="Paying it back" className="space-y-2 rounded-xl border border-warn/40 bg-warn-tint p-4 text-sm">
      <h2 className="font-display text-[32px] leading-none tracking-tight">Paying it back</h2>
      {summary.spending.overspend_paise > 0 && (
        <p>
          You've spent {formatINR(summary.spending.overspend_paise)} more than this month's spending allowance of{" "}
          {formatINR(summary.spending.allowance_paise)}.
        </p>
      )}
      {r && (
        <p>
          {formatINR(r.owed_paise)} of savings and spending is missing this month. Rebuild it at{" "}
          <span className="font-semibold">{formatINR(r.per_month_paise)} a month for {r.months} months</span>: spend that
          much less, or trim your SIP by it, and no FD needs to be touched. Change the number of months on the Plan page.
        </p>
      )}
    </section>
  );
}
