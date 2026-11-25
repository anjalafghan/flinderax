import type { HistoryMonth } from "@/services/types";
import { LineChart } from "@/components/feature/LineChart";
import { MoneyCounter } from "@/components/feature/MoneyCounter";
import { partyColor, type PartyColor } from "@/utils/categoryColors";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
];

/** "2026-09" -> "Sep '26". Parsed from the string so the viewer's timezone can't shift it. */
function monthLabel(ym: string): string {
  const [y, m] = ym.split("-").map(Number);
  return `${MONTHS[m - 1]} '${String(y).slice(2)}`;
}

function Stat({
  label,
  value,
  color,
  tone,
}: {
  label: string;
  value: number;
  /** Liquid is sky, locked is grape, card debt is coral. */
  color: PartyColor;
  tone?: "bad";
}) {
  return (
    <div
      className="min-w-0 rounded-2xl border p-3"
      style={{
        borderColor: partyColor(color, 0.3),
        background: partyColor(color, 0.1),
      }}
    >
      <dt className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <span
          aria-hidden
          className="inline-block h-2 w-2 shrink-0 rounded-full"
          style={{ background: partyColor(color) }}
        />
        {label}
      </dt>
      <dd
        className={cn(
          "mt-1 truncate font-semibold tabular-nums",
          tone === "bad" && "text-over",
        )}
      >
        {formatINR(value)}
      </dd>
    </div>
  );
}

/** Net worth this month, how it moved since last month, and the trend over time. */
export function NetWorthPanel({ months }: { months: HistoryMonth[] }) {
  return (
    <section
      aria-label="Net worth"
      className="space-y-6 glass rounded-[28px] p-4 md:p-6"
    >
      <h2 className="font-display text-[32px] leading-none tracking-tight">
        Net worth
      </h2>
      {months.length === 0 ? (
        <p className="empty-note">
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
  const change = previous
    ? current.net_worth_paise - previous.net_worth_paise
    : null;

  return (
    <>
      <div>
        <p className="font-serif-num text-5xl text-brand-ink md:text-6xl">
          <MoneyCounter paise={current.net_worth_paise} />
        </p>
        {change === null ? (
          <p className="mt-1 text-sm text-muted-foreground">
            First month tracked
          </p>
        ) : (
          <p
            className={cn(
              "mt-2 inline-flex items-center rounded-full px-3 py-1 text-sm font-semibold tabular-nums",
              change >= 0 ? "bg-good-tint text-good" : "bg-over-tint text-over",
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
            stroke: "stroke-mint",
            area: "hsl(var(--c-mint))",
          },
        ]}
      />

      <dl className="grid grid-cols-1 gap-3 border-t border-border pt-6 text-sm sm:grid-cols-3">
        <Stat label="Liquid" value={current.liquid_paise} color="sky" />
        <Stat
          label="Locked (FD/RD)"
          value={current.locked_paise}
          color="grape"
        />
        <Stat
          label="Card debt"
          color="coral"
          value={current.card_debt_paise}
          tone={current.card_debt_paise > 0 ? "bad" : undefined}
        />
      </dl>
    </>
  );
}
