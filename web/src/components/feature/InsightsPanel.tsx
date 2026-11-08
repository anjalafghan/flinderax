import type { ReactNode } from "react";
import { Copy, Receipt, TrendingUp, Undo2, Wallet, Clock } from "lucide-react";
import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { useInsights, type InsightRefund, type InsightsOut } from "@/services/insights";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** "2026-03" -> "Mar 2026". */
function monthYear(ym: string): string {
  const [y, m] = ym.split("-").map(Number);
  return `${MONTHS[m - 1]} ${y}`;
}

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

/** Whole rupees for the sentences: "₹1,234", not "₹1,234.00". */
const rupees = (paise: number) => formatINR(Math.round(paise / 100) * 100);

const RAIL_LABEL: Record<InsightRefund["rail"], string> = {
  p2p: "to a person",
  p2m: "to a merchant",
  unknown: "UPI payment",
};

const IMPACT_TONE: Record<InsightsOut["health"]["actions"][number]["impact"], string> = {
  high: "tone-over",
  medium: "tone-warn",
  low: "tone-quiet",
};

/** Score bands from the spec: 75+ good, 50 to 74 warn, under 50 over. */
function healthTone(score: number): "good" | "warn" | "over" {
  if (score >= 75) return "good";
  if (score >= 50) return "warn";
  return "over";
}

const HEALTH_LABEL = { good: "Healthy", warn: "Watch", over: "Needs attention" } as const;

/** No signal at all: nothing flagged and no health inputs. */
function isEmpty(d: InsightsOut): boolean {
  return (
    d.refunds.length === 0 &&
    d.charges.last_12m_paise === 0 &&
    d.duplicates.length === 0 &&
    d.unusual.length === 0 &&
    d.spend_to_payday === null &&
    d.health.savings_rate_bps === null &&
    d.health.emergency_months_tenths === null &&
    d.health.income_cv_bps === null
  );
}

function Pill({ className, children }: { className?: string; children: ReactNode }) {
  return (
    <span
      className={cn(
        "inline-flex min-h-7 shrink-0 items-center rounded-full px-3 text-[13px] font-semibold whitespace-nowrap",
        className,
      )}
    >
      {children}
    </span>
  );
}

function Stat({ label, value, unit }: { label: string; value: string; unit?: string }) {
  return (
    <div className="flex min-w-[120px] flex-1 flex-col gap-1">
      <span className="text-[13px] text-muted-foreground">{label}</span>
      <span className="font-display text-[32px] leading-none">
        {value}
        {unit && <span className="ml-1 text-sm text-muted-foreground">{unit}</span>}
      </span>
    </div>
  );
}

function HealthSection({ health }: { health: InsightsOut["health"] }) {
  const tone = healthTone(health.score);
  const cv = health.income_cv_bps;
  return (
    <section aria-labelledby="insights-health" className="flex flex-col gap-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-baseline gap-3">
          <h3 id="insights-health" className="m-0 font-display text-[28px] leading-none">
            Health
          </h3>
        </div>
        <span className={cn("inline-flex min-h-8 items-center rounded-full px-3.5 text-sm font-semibold", `tone-${tone}`)}>
          {HEALTH_LABEL[tone]}
        </span>
      </div>

      <div className="flex flex-wrap items-end gap-x-8 gap-y-5">
        <p className="m-0 flex items-baseline gap-2 font-display text-[96px] leading-[0.9] tracking-[-0.03em]">
          {health.score}
          <span className="text-2xl text-muted-foreground">/100</span>
        </p>
        <div className="flex flex-1 flex-wrap gap-6 pb-1">
          <Stat
            label="Savings rate"
            value={health.savings_rate_bps === null ? "—" : `${Math.round(health.savings_rate_bps / 100)}%`}
          />
          <Stat
            label="Emergency months"
            value={health.emergency_months_tenths === null ? "—" : (health.emergency_months_tenths / 10).toFixed(1)}
          />
          <Stat
            label="Income"
            value={cv === null ? "—" : cv < 1500 ? "steady" : "variable"}
          />
        </div>
      </div>

      {health.actions.length > 0 && (
        <ol className="m-0 list-none p-0">
          {health.actions.map((a) => (
            <li key={a.text} className="flex items-start justify-between gap-4 border-t border-border py-4">
              <span className="text-[15px] leading-snug">{a.text}</span>
              <Pill className={IMPACT_TONE[a.impact]}>{a.impact} impact</Pill>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

function RefundsSection({ refunds }: { refunds: InsightRefund[] }) {
  return (
    <section aria-labelledby="insights-refunds" className="flex flex-col">
      <h3 id="insights-refunds" className="m-0 flex items-center gap-2.5 font-display text-[28px] leading-none">
        <Undo2 className="h-5 w-5" strokeWidth={1.5} aria-hidden />
        Failed UPI refunds owed
      </h3>
      <p className="m-0 pb-2 pt-3 text-[13px] text-muted-foreground">
        RBI requires failed UPI debits to be reversed within a day (to a person) or five days (to a merchant); after that
        the bank owes ₹100 a day.
      </p>
      <ul className="m-0 list-none p-0">
        {refunds.map((r) => (
          <li key={r.txn_id} className="flex flex-wrap items-center justify-between gap-x-5 gap-y-2 border-t border-border py-4">
            <span className="flex min-w-0 flex-auto flex-col gap-0.5">
              <span className="truncate text-base font-medium">{r.merchant ?? "Unknown payee"}</span>
              <span className="text-[13px] text-muted-foreground">
                {formatINR(r.amount_paise)} · {RAIL_LABEL[r.rail]} · debited {formatDay(r.occurred_on)}
              </span>
            </span>
            {r.status === "overdue" ? (
              <Pill className="tone-over">
                {plural(r.days_overdue, "day")} late · bank owes {rupees(r.compensation_paise)}
              </Pill>
            ) : (
              <Pill className="tone-quiet">Due {formatDay(r.due_by)}</Pill>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}

function ChargesSection({ charges }: { charges: InsightsOut["charges"] }) {
  return (
    <section aria-labelledby="insights-charges" className="flex flex-col">
      <div className="flex items-baseline justify-between gap-4 pb-3">
        <h3 id="insights-charges" className="m-0 flex items-center gap-2.5 font-display text-[28px] leading-none">
          <Receipt className="h-5 w-5" strokeWidth={1.5} aria-hidden />
          Charges &amp; fees
        </h3>
        <span className="font-display text-2xl">{formatINR(charges.last_12m_paise)}</span>
      </div>
      <p className="m-0 pb-1 text-[13px] text-muted-foreground">Bank charges and fees in the last 12 months.</p>
      <ul className="m-0 list-none p-0">
        {charges.items.map((c) => (
          <li key={c.label} className="flex items-center justify-between gap-4 border-t border-border py-3.5">
            <span className="text-[15px]">
              {c.label}
              {c.count > 1 && <span className="ml-2 text-[13px] text-muted-foreground">×{c.count}</span>}
            </span>
            <span className="font-display text-xl">{formatINR(c.amount_paise)}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function DuplicatesSection({ duplicates }: { duplicates: InsightsOut["duplicates"] }) {
  return (
    <section aria-labelledby="insights-duplicates" className="flex flex-col">
      <h3 id="insights-duplicates" className="m-0 flex items-center gap-2.5 font-display text-[28px] leading-none">
        <Copy className="h-5 w-5" strokeWidth={1.5} aria-hidden />
        Possible duplicate charges
      </h3>
      <ul className="mt-3 list-none p-0">
        {duplicates.map((d) => (
          <li key={`${d.merchant}-${d.occurred_on}-${d.amount_paise}`} className="flex items-center justify-between gap-5 border-t border-border py-4">
            <span className="flex min-w-0 flex-auto flex-col gap-0.5">
              <span className="truncate text-base font-medium">{d.merchant}</span>
              <span className="text-[13px] text-muted-foreground">
                {plural(d.count, "charge")} on {formatDay(d.occurred_on)}
              </span>
            </span>
            <span className="font-display text-2xl">{formatINR(d.amount_paise)}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function UnusualSection({ unusual }: { unusual: InsightsOut["unusual"] }) {
  return (
    <section aria-labelledby="insights-unusual" className="flex flex-col">
      <h3 id="insights-unusual" className="m-0 flex items-center gap-2.5 font-display text-[28px] leading-none">
        <TrendingUp className="h-5 w-5" strokeWidth={1.5} aria-hidden />
        Unusual months
      </h3>
      <ul className="mt-3 list-none p-0">
        {unusual.map((u) => (
          <li key={`${u.category}-${u.month}`} className="flex items-center justify-between gap-5 border-t border-border py-4">
            <span className="text-[15px]">
              {`${u.category} in ${monthYear(u.month)}: ${rupees(u.amount_paise)} vs usual ${rupees(u.typical_paise)}`}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function PaydaySection({ p }: { p: NonNullable<InsightsOut["spend_to_payday"]> }) {
  return (
    <section aria-labelledby="insights-payday" className="flex flex-col">
      <h3 id="insights-payday" className="m-0 flex items-center gap-2.5 font-display text-[28px] leading-none">
        <Wallet className="h-5 w-5" strokeWidth={1.5} aria-hidden />
        Spending to payday
      </h3>
      <p className="m-0 pt-3 text-[15px] leading-snug">
        {`Between ${rupees(p.p10_paise)} and ${rupees(p.p90_paise)} likely to go before payday in ${plural(p.days, "day")} (most likely ${rupees(p.p50_paise)}).`}
      </p>
    </section>
  );
}

/** The Insights card: what the transactions say. Empty sections are hidden. */
export function InsightsPanel() {
  const { data, isLoading, isError, refetch } = useInsights();

  if (isLoading) {
    return <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading insights" />;
  }
  if (isError || !data) {
    return (
      <div className="rounded-[28px] border border-dashed border-border bg-card/30 p-6 text-center text-sm text-muted-foreground">
        Couldn't load your insights.{" "}
        <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <section
      aria-labelledby="insights-title"
      className="flex flex-col gap-12 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10"
    >
      <header className="flex flex-col gap-2">
        <h2 id="insights-title" className="m-0 font-display text-[32px] leading-none">
          Insights
        </h2>
        <p className="m-0 text-sm text-muted-foreground">What your transactions say</p>
      </header>

      {isEmpty(data) ? (
        <p className="m-0 text-sm text-muted-foreground">
          Not enough transactions yet. Import a statement or forward bank alerts to see insights.
        </p>
      ) : (
        <>
          <HealthSection health={data.health} />
          {data.refunds.length > 0 && <RefundsSection refunds={data.refunds} />}
          {data.charges.last_12m_paise > 0 && <ChargesSection charges={data.charges} />}
          {data.duplicates.length > 0 && <DuplicatesSection duplicates={data.duplicates} />}
          {data.unusual.length > 0 && <UnusualSection unusual={data.unusual} />}
          {data.spend_to_payday && <PaydaySection p={data.spend_to_payday} />}
        </>
      )}
    </section>
  );
}

/**
 * A one-line nudge for the dashboard: an overdue failed-UPI refund if there is one, otherwise the
 * spend-to-payday range. Renders nothing when there is neither (or while loading / on error).
 */
export function InsightsNudge() {
  const { data } = useInsights();
  if (!data) return null;

  const overdue = data.refunds.filter((r) => r.status === "overdue");
  if (overdue.length > 0) {
    const owed = overdue.reduce((a, r) => a + r.compensation_paise, 0);
    return (
      <div className="tone-over mt-4 flex flex-wrap items-center justify-between gap-3 rounded-[18px] px-5 py-3.5">
        <span className="flex items-center gap-2.5 text-sm font-medium">
          <Clock className="h-[18px] w-[18px] shrink-0" strokeWidth={1.6} aria-hidden />
          {`${plural(overdue.length, "failed UPI refund")} overdue · bank owes ${rupees(owed)}`}
        </span>
        <Link to="/money" className="inline-flex min-h-11 items-center text-sm font-semibold text-foreground no-underline">
          See insights
        </Link>
      </div>
    );
  }

  const p = data.spend_to_payday;
  if (p) {
    return (
      <p className="m-0 mt-4 flex flex-wrap items-center justify-between gap-3 text-sm text-muted-foreground">
        <span>{`Between ${rupees(p.p10_paise)} and ${rupees(p.p90_paise)} likely to go before payday in ${plural(p.days, "day")}.`}</span>
        <Link to="/money" className="inline-flex min-h-11 items-center font-medium text-foreground no-underline">
          Insights
        </Link>
      </p>
    );
  }
  return null;
}
