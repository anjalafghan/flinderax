import type { ReactNode } from "react";
import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import {
  useAnalytics,
  type AnalyticsMonth,
  type AnalyticsOut,
  type AnalyticsPayee,
  type AnalyticsSalary,
} from "@/services/analytics";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

// Colours come from the theme tokens so the chart reads in light and dark.
const IN_COLOR = "hsl(var(--good))";
const OUT_COLOR = "hsl(var(--foreground))";
const INVESTED_COLOR = "hsl(var(--muted-foreground) / 0.35)";

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

/** Whole rupees everywhere in this panel: "₹1,234", never "₹1,234.00". */
const rupees = (paise: number) => formatINR(Math.round(paise / 100) * 100);

/** "2026-09" -> "Sep". */
const monthAbbr = (ym: string) => MONTHS[Number(ym.split("-")[1]) - 1];

/** "2026-09" -> "Sep 2026". */
const monthYear = (ym: string) => {
  const [y, m] = ym.split("-").map(Number);
  return `${MONTHS[m - 1]} ${y}`;
};

function ordinal(n: number): string {
  const suffix =
    n % 100 >= 11 && n % 100 <= 13 ? "th" : n % 10 === 1 ? "st" : n % 10 === 2 ? "nd" : n % 10 === 3 ? "rd" : "th";
  return `${n}${suffix}`;
}

/** "ASHA STORES" -> "Asha Stores". */
const titleCase = (s: string) => s.toLowerCase().replace(/\b\w/g, (c) => c.toUpperCase());

/** The usual salary days: a short list, or the span when there are many. */
function daysLabel(days: number[]): string {
  if (days.length === 0) return "—";
  if (days.length <= 4) return days.map(ordinal).join(", ");
  return `${ordinal(days[0])} to ${ordinal(days[days.length - 1])}`;
}

/** "Person" is a guess (shops take UPI on personal QR codes too), so only the confident kinds get a pill. */
const KIND_LABEL: Record<AnalyticsPayee["kind"], string | null> = {
  merchant: "Merchant",
  shop: "Regular shop",
  person: null,
};

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

function Fact({ label, value, hint }: { label: string; value: string; hint?: string }) {
  return (
    <div className="flex min-w-[120px] flex-1 flex-col gap-1">
      <dt className="text-[13px] text-muted-foreground">{label}</dt>
      <dd className="m-0 font-display text-[28px] leading-none">{value}</dd>
      {hint && <span className="text-[13px] text-muted-foreground">{hint}</span>}
    </div>
  );
}

function Legend({ color, label }: { color: string; label: string }) {
  return (
    <li className="flex items-center gap-2">
      <span aria-hidden className="inline-block h-2.5 w-2.5 rounded-[3px]" style={{ background: color }} />
      {label}
    </li>
  );
}

/** Twelve months of money in (green) against money out (spend, with invested stacked on top, muted). */
function MonthlyChart({ months }: { months: AnalyticsMonth[] }) {
  const peak = Math.max(1, ...months.map((m) => Math.max(m.income_paise, m.spend_paise + m.invested_paise)));
  const height = (paise: number) => `${(paise / peak) * 100}%`;

  return (
    <div className="flex flex-col gap-4">
      <div role="img" aria-label="Money in and out, last 12 months" className="flex h-44 items-stretch gap-2">
        {months.map((m) => {
          const out = m.spend_paise + m.invested_paise;
          return (
            <div
              key={m.month}
              title={`${monthYear(m.month)}: in ${rupees(m.income_paise)}, out ${rupees(out)}`}
              className="flex min-w-0 flex-1 flex-col gap-2"
            >
              <div className="flex min-h-0 flex-1 items-end justify-center gap-[3px]">
                <div className="w-2 rounded-t-[3px]" style={{ height: height(m.income_paise), background: IN_COLOR }} />
                <div
                  className="flex w-2 flex-col overflow-hidden rounded-t-[3px]"
                  style={{ height: height(out) }}
                >
                  <div style={{ flex: m.invested_paise, background: INVESTED_COLOR }} />
                  <div style={{ flex: m.spend_paise, background: OUT_COLOR }} />
                </div>
              </div>
              <span className="text-center text-[10px] leading-none text-muted-foreground">{monthAbbr(m.month)}</span>
            </div>
          );
        })}
      </div>
      <ul className="m-0 flex list-none flex-wrap gap-x-5 gap-y-2 p-0 text-[13px] text-muted-foreground">
        <Legend color={IN_COLOR} label="Money in" />
        <Legend color={OUT_COLOR} label="Spent" />
        <Legend color={INVESTED_COLOR} label="Invested" />
      </ul>
    </div>
  );
}

function SavingsBlock({ bps }: { bps: number | null }) {
  return (
    <div className="flex flex-wrap items-end gap-x-8 gap-y-4">
      <p className="m-0 font-display text-[72px] leading-[0.9] tracking-[-0.03em]">
        {bps === null ? "—" : `${Math.round(bps / 100)}%`}
      </p>
      <p className="m-0 max-w-[30ch] text-sm text-muted-foreground">
        {bps === null
          ? "Savings rate needs a full month of income."
          : "of your income kept over the last three full months."}
      </p>
    </div>
  );
}

function SalarySection({ salary }: { salary: AnalyticsSalary }) {
  const missing = salary.missing_months;
  return (
    <section aria-labelledby="analytics-salary" className="flex flex-col gap-4">
      <h3 id="analytics-salary" className="m-0 font-display text-[24px] leading-tight">
        Salary
      </h3>
      <dl className="m-0 flex flex-wrap gap-x-6 gap-y-5">
        <Fact label="Usually" value={salary.usual || daysLabel(salary.typical_days)} />
        <Fact label="Average" value={rupees(salary.avg_paise)} />
        <Fact label="Last" value={rupees(salary.last_paise)} hint={formatDay(salary.last_on)} />
        <Fact label="Next expected" value={formatDay(salary.next_expected)} />
      </dl>
      <p className="m-0 text-[13px] leading-snug text-muted-foreground">
        {missing.length > 0
          ? `No salary in ${missing.map(monthYear).join(", ")}.`
          : "Every month in your statements has a salary."}
      </p>
    </section>
  );
}

function SipsSection({ sips }: { sips: AnalyticsOut["sips"] }) {
  const monthly = sips.reduce((a, s) => a + s.amount_paise, 0);
  return (
    <section aria-labelledby="analytics-sips" className="flex flex-col gap-4">
      <div className="flex items-baseline justify-between gap-4">
        <h3 id="analytics-sips" className="m-0 font-display text-[24px] leading-tight">
          SIPs
        </h3>
        <span className="font-display text-2xl">{`${rupees(monthly)} a month`}</span>
      </div>
      <ul className="ledger">
        {sips.map((s, i) => (
          <li key={`${s.amount_paise}-${i}`} className="ledger-row">
            <span className="flex min-w-0 flex-col gap-0.5">
              <span className="text-[15px]">{s.label}</span>
              <span className="text-[13px] text-muted-foreground">
                {`debits around the ${ordinal(s.usual_day)} · ${plural(s.months_seen, "month")}`}
              </span>
            </span>
            <span className="text-[13px] text-muted-foreground">{`last ${formatDay(s.last_on)}`}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function AutopaySection({ autopay }: { autopay: AnalyticsOut["autopay"] }) {
  return (
    <section aria-labelledby="analytics-autopay" className="flex flex-col gap-4">
      <h3 id="analytics-autopay" className="m-0 font-display text-[24px] leading-tight">
        AutoPay
      </h3>
      <ul className="ledger">
        {autopay.map((a) => (
          <li key={a.name} className="ledger-row">
            <span className="flex min-w-0 flex-col gap-0.5">
              <span className="truncate text-base font-medium" title={titleCase(a.name)}>
                {titleCase(a.name)}
              </span>
              <span className="text-[13px] text-muted-foreground">
                {`${plural(a.count, "payment")} · last ${formatDay(a.last_on)}`}
              </span>
            </span>
            <span className="font-display text-xl">{rupees(a.amount_paise)}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

function SmallFlowsSection({ data }: { data: AnalyticsOut }) {
  const { own_transfers: own, small_ticket: small } = data;
  return (
    <section aria-labelledby="analytics-small" className="flex flex-col gap-4">
      <h3 id="analytics-small" className="m-0 font-display text-[24px] leading-tight">
        Small payments and transfers
      </h3>
      <ul className="m-0 flex list-none flex-col gap-3 p-0 text-[15px] leading-snug">
        {small.per_month_paise > 0 && (
          <li>
            {`${rupees(small.per_month_paise)} a month goes on ${plural(small.per_month_count, "payment")} under ${rupees(small.threshold_paise)}.`}
          </li>
        )}
        {own && (
          <li>
            {`${plural(own.count, "transfer")} to and from your own accounts: ${rupees(own.out_paise)} out, ${rupees(own.in_paise)} in.`}
          </li>
        )}
      </ul>
    </section>
  );
}

function WeekdaySection({ values }: { values: number[] }) {
  const peak = Math.max(1, ...values);
  return (
    <section aria-labelledby="analytics-weekday" className="flex flex-col gap-4">
      <h3 id="analytics-weekday" className="m-0 font-display text-[24px] leading-tight">
        Weekday spending
      </h3>
      <div role="img" aria-label="Spending by weekday, last 90 days" className="flex h-28 items-stretch gap-2">
        {values.map((v, i) => (
          <div key={WEEKDAYS[i]} title={`${WEEKDAYS[i]}: ${rupees(v)}`} className="flex min-w-0 flex-1 flex-col gap-1.5">
            <div className="flex min-h-0 flex-1 items-end">
              <div
                className="w-full rounded-t-[4px]"
                style={{ height: `${(v / peak) * 100}%`, background: OUT_COLOR }}
              />
            </div>
            <span className="text-center text-[11px] leading-none text-muted-foreground">{WEEKDAYS[i]}</span>
          </div>
        ))}
      </div>
    </section>
  );
}

/** The panel's main content, once there is statement coverage. */
function AnalyticsBody({ data }: { data: AnalyticsOut }) {
  // "Uncategorized" isn't a place money went: show it as a prompt, and the bars without it.
  const unsorted = data.categories.find((c) => c.category === "uncategorized");
  const categories = data.categories.filter((c) => c.category !== "uncategorized").slice(0, 6);
  const hasWeekdays = data.weekday_spend_paise.some((v) => v > 0);
  const hasSmallFlows = data.small_ticket.per_month_paise > 0 || data.own_transfers !== null;

  return (
    <>
      <section aria-labelledby="analytics-flow" className="flex flex-col gap-6">
        <h3 id="analytics-flow" className="sr-only">
          Money in and out
        </h3>
        <MonthlyChart months={data.months} />
        <SavingsBlock bps={data.savings_rate_bps} />
      </section>

      {(categories.length > 0 || unsorted) && (
        <section aria-labelledby="analytics-categories" className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <h3 id="analytics-categories" className="m-0 font-display text-[24px] leading-tight">
              Where the money goes
            </h3>
            <span className="text-[13px] text-muted-foreground">Last 90 days</span>
          </div>
          {unsorted && (
            <p className="tone-warn m-0 rounded-[18px] px-5 py-4 text-sm leading-snug">
              {`${rupees(unsorted.paise)} across ${plural(unsorted.count, "payment")} (${Math.round(unsorted.share_bps / 100)}%) isn't categorised yet. `}
              <Link to="/spending" className="font-medium underline underline-offset-4">
                Categorise a few payees
              </Link>
              {" and joy remembers them, including for past payments."}
            </p>
          )}
          <ul className="m-0 flex list-none flex-col gap-4 p-0">
            {categories.map((c) => (
              <li key={c.category} className="flex flex-col gap-2">
                <div className="flex items-baseline justify-between gap-4 text-[15px]">
                  <span className="capitalize">{c.category}</span>
                  <span className="shrink-0 text-[13px] text-muted-foreground">
                    {`${rupees(c.paise)} · ${Math.round(c.share_bps / 100)}%`}
                  </span>
                </div>
                <div className="h-2 overflow-hidden rounded-full bg-chip" aria-hidden>
                  <div className="h-full rounded-full" style={{ width: `${c.share_bps / 100}%`, background: OUT_COLOR }} />
                </div>
              </li>
            ))}
          </ul>
        </section>
      )}

      {data.top_payees.length > 0 && (
        <section aria-labelledby="analytics-payees" className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <h3 id="analytics-payees" className="m-0 font-display text-[24px] leading-tight">
              Top payees
            </h3>
            <span className="text-[13px] text-muted-foreground">Last 90 days</span>
          </div>
          <ul className="ledger">
            {data.top_payees.map((p) => (
              <li key={p.name} className="ledger-row gap-4">
                <span className="flex min-w-0 flex-auto flex-col gap-1">
                  <span className="truncate text-base font-medium" title={titleCase(p.name)}>
                    {titleCase(p.name)}
                  </span>
                  <span className="flex flex-wrap items-center gap-2">
                    {KIND_LABEL[p.kind] && <Pill className="tone-quiet">{KIND_LABEL[p.kind]}</Pill>}
                    <span className="text-[13px] text-muted-foreground">{plural(p.count, "payment")}</span>
                  </span>
                </span>
                <span className="font-display text-xl">{rupees(p.paise)}</span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {data.salary && <SalarySection salary={data.salary} />}
      {data.sips.length > 0 && <SipsSection sips={data.sips} />}
      {data.autopay.length > 0 && <AutopaySection autopay={data.autopay} />}
      {hasSmallFlows && <SmallFlowsSection data={data} />}
      {hasWeekdays && <WeekdaySection values={data.weekday_spend_paise} />}

      {data.notes.length > 0 && (
        <section aria-labelledby="analytics-notes" className="flex flex-col gap-4">
          <h3 id="analytics-notes" className="m-0 font-display text-[24px] leading-tight">
            Notes
          </h3>
          <ul className="m-0 flex list-none flex-col gap-3 p-0 text-[15px] leading-snug">
            {data.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
        </section>
      )}
    </>
  );
}

/** The "Your money, from your statements" card. Needs statement coverage to show anything. */
export function AnalyticsPanel() {
  const { data, isLoading, isError, refetch } = useAnalytics();

  if (isLoading) {
    return <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your money" />;
  }
  if (isError || !data) {
    return (
      <div className="rounded-[28px] border border-dashed border-border bg-card/30 p-6 text-center text-sm text-muted-foreground">
        Couldn't load this.{" "}
        <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  const coverage = data.coverage;

  return (
    <section
      aria-labelledby="analytics-title"
      className="flex flex-col gap-12 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10"
    >
      <header className="flex flex-col gap-2">
        <h2 id="analytics-title" className="m-0 font-display text-[32px] leading-none">
          Your money, from your statements
        </h2>
        {coverage && (
          <p className="m-0 text-sm text-muted-foreground">
            {`Based on statements from ${formatDay(coverage.first_date)} to ${formatDay(coverage.last_date)}`}
          </p>
        )}
      </header>

      {coverage ? (
        <AnalyticsBody data={data} />
      ) : (
        <div className="flex flex-col items-start gap-4">
          <p className="empty-note m-0 w-full">Upload a statement to see this.</p>
          <Link
            to="/spending"
            className="inline-flex min-h-11 items-center text-sm font-semibold text-foreground no-underline"
          >
            Go to Spending
          </Link>
        </div>
      )}
    </section>
  );
}
