import type { HistoryMonth } from "@/services/types";
import { LineChart } from "@/components/feature/LineChart";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** "2026-09" -> "Sep '26". Parsed from the string so the viewer's timezone can't shift it. */
function monthLabel(ym: string): string {
  const [y, m] = ym.split("-").map(Number);
  return `${MONTHS[m - 1]} '${String(y).slice(2)}`;
}

function Stat({ label, value, tone }: { label: string; value: number; tone?: "bad" }) {
  return (
    <div className="min-w-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className={cn("font-semibold tabular-nums", tone === "bad" && "text-rose-600 dark:text-rose-400")}>
        {formatINR(value)}
      </dd>
    </div>
  );
}

/** Net worth this month, how it moved since last month, and the trend over time. */
export function NetWorthPanel({ months }: { months: HistoryMonth[] }) {
  return (
    <section aria-label="Net worth" className="space-y-3 glass rounded-3xl p-4 md:p-6">
      <h2 className="font-display text-3xl leading-none tracking-tight">Net worth</h2>
      {months.length === 0 ? (
        <p className="rounded-3xl border border-dashed border-primary/25 bg-card/30 p-4 text-sm text-muted-foreground">
          Net worth history starts building as you update balances.
        </p>
      ) : (
        <NetWorthBody months={months} />
      )}
    </section>
  );
}

function NetWorthBody({ months }: { months: HistoryMonth[] }) {
  const current = months[months.length - 1];
  const previous = months.length > 1 ? months[months.length - 2] : null;
  const change = previous ? current.net_worth_paise - previous.net_worth_paise : null;

  return (
    <>
      <div>
        <p className="font-serif-num text-5xl md:text-6xl">
          {formatINR(current.net_worth_paise)}
        </p>
        {change === null ? (
          <p className="mt-1 text-sm text-muted-foreground">First month tracked</p>
        ) : (
          <p
            className={cn(
              "mt-1 text-sm font-semibold tabular-nums",
              change >= 0 ? "text-emerald-600 dark:text-emerald-400" : "text-rose-600 dark:text-rose-400",
            )}
          >
            {change >= 0 ? "+" : "−"}
            {formatINR(Math.abs(change))} vs last month
          </p>
        )}
      </div>

      <LineChart
        ariaLabel="Net worth over time"
        labels={months.map((m) => monthLabel(m.month))}
        series={[
          {
            label: "Net worth",
            values: months.map((m) => m.net_worth_paise),
            stroke: "stroke-emerald-500",
          },
        ]}
      />

      <dl className="grid grid-cols-3 gap-3 border-t pt-4 text-sm">
        <Stat label="Liquid" value={current.liquid_paise} />
        <Stat label="Locked (FD/RD)" value={current.locked_paise} />
        <Stat label="Card debt" value={current.card_debt_paise} tone={current.card_debt_paise > 0 ? "bad" : undefined} />
      </dl>
    </>
  );
}
