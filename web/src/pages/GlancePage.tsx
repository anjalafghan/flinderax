import { useQuery } from "@tanstack/react-query";
import { ArrowRight } from "lucide-react";
import type { ReactNode } from "react";
import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { Skeleton } from "@/components/ui/skeleton";
import { CategoryBars } from "@/components/feature/CategoryBars";
import { Bars, Legend, type BarSeries } from "@/components/feature/glance/Bars";
import { RoadAhead } from "@/components/feature/glance/RoadAhead";
import { hue, type Hue } from "@/components/feature/life/hue";
import { RupeeOdometer } from "@/components/feature/life/flair";
import {
  chartMonths,
  datesStats,
  dayShort,
  getaways,
  jarBudget,
  monthShort,
  outflowMonths,
  pct,
} from "@/components/feature/glance/glance";
import { useGoals, useSummary } from "@/hooks/usePlanner";
import { getCouple, keys as joyKeys } from "@/joy/api";
import { getKept, keptKey } from "@/joy/kept/api";
import { useAnalytics } from "@/services/analytics";
import { useRewardFund } from "@/services/rewards";
import { useRoad } from "@/services/road";
import { useYear } from "@/services/year";
import type { DashboardSummary } from "@/services/types";
import { formatWholeINR } from "@/utils/money";

/** Whole rupees: this page is about the shape of things, not the paise. */
const rupees = formatWholeINR;

// Spending coral, investing mint, what is promised sky: the same colours as the rest of the app.
const MONEY_SERIES: BarSeries[] = [
  { name: "Spent", color: hue("coral") },
  { name: "Invested", color: hue("mint") },
];
const OUTFLOW_SERIES: BarSeries[] = [{ name: "Promised", color: hue("sky") }];

/** The colour of each part of the page, used by its kicker, its jump link and its tiles. */
const SECTION_HUE: Record<string, Hue> = {
  now: "sky",
  "so-far": "coral",
  ahead: "grape",
  us: "rose",
  road: "mint",
};

/** Reward areas in order: money, health, relationships, the future. */
const AREA_HUES: Hue[] = ["mint", "coral", "rose", "sky"];

/** Goal bars cycle through the party colours so each goal is easy to tell apart. */
const GOAL_HUES: Hue[] = ["grape", "sky", "mint", "sun", "rose", "teal"];

function Section({
  id,
  kicker,
  children,
}: {
  id: string;
  kicker: string;
  children: ReactNode;
}) {
  return (
    <section
      id={id}
      aria-labelledby={`${id}-k`}
      className="flex flex-col gap-4"
    >
      <p
        id={`${id}-k`}
        className="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.08em] text-foreground"
      >
        <span
          aria-hidden
          className="h-2.5 w-2.5 rounded-full"
          style={{
            background: hue(SECTION_HUE[id] ?? "grape"),
            boxShadow: `0 0 0 4px ${hue(SECTION_HUE[id] ?? "grape", 0.2)}`,
          }}
        />
        {kicker}
        <span
          aria-hidden
          className="h-px flex-1"
          style={{
            background: `linear-gradient(90deg, ${hue(SECTION_HUE[id] ?? "grape", 0.5)}, transparent)`,
          }}
        />
      </p>
      {children}
    </section>
  );
}

function Panel({
  title,
  sub,
  children,
  to,
}: {
  title: string;
  sub?: string;
  children: ReactNode;
  to?: string;
}) {
  return (
    <div className="glass flex min-w-0 flex-col gap-3 rounded-[24px] p-5 md:p-6">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h2 className="text-[17px] font-medium">{title}</h2>
          {sub && (
            <p className="mt-0.5 text-[13px] text-muted-foreground">{sub}</p>
          )}
        </div>
        {to && (
          <Link
            to={to}
            viewTransition
            className="-m-3 grid h-11 w-11 shrink-0 place-items-center rounded-full text-brand-ink transition-colors hover:bg-brand-tint"
            aria-label={`Open ${title}`}
          >
            <ArrowRight className="h-4 w-4" strokeWidth={1.5} />
          </Link>
        )}
      </div>
      {children}
    </div>
  );
}

function Tile({
  label,
  value,
  note,
  tone,
  color,
}: {
  label: string;
  value: ReactNode;
  note?: ReactNode;
  tone?: "over" | "good";
  /** The tile's own colour: a wash and a top edge. */
  color: Hue;
}) {
  return (
    <div
      className="relative flex min-w-0 flex-col gap-1 overflow-hidden rounded-[20px] border p-4 md:p-5"
      style={{
        borderColor: hue(color, 0.35),
        background: `linear-gradient(160deg, ${hue(color, 0.18)}, ${hue(color, 0.04)} 70%), hsl(var(--card))`,
      }}
    >
      <span
        aria-hidden
        className="absolute inset-x-0 top-0 h-1"
        style={{ background: hue(color) }}
      />
      <span className="text-[13px] text-muted-foreground">{label}</span>
      <span
        className="font-display text-[1.9rem] leading-tight tabular-nums"
        style={tone ? { color: `hsl(var(--${tone}))` } : undefined}
      >
        {value}
      </span>
      {note && (
        <span className="text-[12px] text-muted-foreground">{note}</span>
      )}
    </div>
  );
}

function Loading({ h = 160 }: { h?: number }) {
  return (
    <div style={{ height: h }} aria-busy="true">
      <Skeleton className="h-full w-full rounded-xl" />
    </div>
  );
}

function Failed() {
  return (
    <p className="text-sm text-muted-foreground">
      This part didn't load. Pull to refresh, or try again in a moment.
    </p>
  );
}

/** The sentence under the title: where spending stands, and how the dates are going. */
function lede(
  s: DashboardSummary | undefined,
  dates: { kept: number; average: number | null } | null,
): string | null {
  if (!s) return null;
  const sp = s.spending;
  const days = `${s.days_left} day${s.days_left === 1 ? "" : "s"} to go`;
  const money = !sp.tracked
    ? "Spending isn't tracked yet"
    : sp.overspend_paise > 0
      ? `Spending is ${rupees(sp.overspend_paise)} past its ${rupees(sp.allowance_paise)} allowance with ${days}`
      : `Spending is ${rupees(sp.left_paise)} inside its ${rupees(sp.allowance_paise)} allowance with ${days}`;
  if (!dates || dates.kept === 0) return `${money}.`;
  const avg =
    dates.average !== null
      ? `, rated ${dates.average} out of 5 on average`
      : "";
  return `${money}, and ${dates.kept} date${dates.kept === 1 ? "" : "s"} kept from the jar${avg}.`;
}

export default function GlancePage() {
  const summary = useSummary();
  const analytics = useAnalytics();
  const year = useYear();
  const goals = useGoals();
  const fund = useRewardFund();
  const road = useRoad();
  const couple = useQuery({
    queryKey: joyKeys.couple,
    queryFn: getCouple,
    retry: 1,
  });
  const kept = useQuery({
    queryKey: keptKey,
    queryFn: getKept,
    retry: 1,
    enabled: !!couple.data?.in_couple,
  });

  const s = summary.data;
  const stats = kept.data ? datesStats(kept.data) : null;
  const months = analytics.data ? chartMonths(analytics.data.months) : [];
  const outflows = year.data ? outflowMonths(year.data) : [];
  const median = outflows.length
    ? [...outflows].sort((a, b) => a.total_paise - b.total_paise)[
        Math.floor(outflows.length / 2)
      ].total_paise
    : 0;
  const cats = (analytics.data?.categories ?? [])
    .map((c) => ({ category: c.category, amount_paise: c.paise }))
    .slice(0, 8);
  const catTotal = (analytics.data?.categories ?? []).reduce(
    (t, c) => t + c.paise,
    0,
  );
  const score = fund.data?.this_month.score;
  const jar = kept.data ? jarBudget(kept.data) : null;
  const trips = kept.data ? getaways(kept.data) : [];
  const wedding = road.data?.milestones.find(
    (m) =>
      m.source === "life" && m.area === "family" && /wedding/i.test(m.title),
  );
  const line = lede(s, stats);

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page flex flex-col gap-10">
        <div>
          <h1 className="page-title">At a glance</h1>
          {line ? (
            <p className="page-sub max-w-[62ch]">{line}</p>
          ) : (
            <Skeleton className="mt-3 h-5 w-2/3" />
          )}
          <nav aria-label="Sections" className="mt-4 flex flex-wrap gap-2">
            {[
              ["#now", "Now"],
              ["#so-far", "Money so far"],
              ["#ahead", "Ahead"],
              ["#us", "Us"],
              ["#road", "The road"],
            ].map(([href, label]) => {
              const h = SECTION_HUE[href.slice(1)] ?? "grape";
              return (
                <a
                  key={href}
                  href={href}
                  className="inline-flex min-h-11 items-center gap-2 rounded-full border px-4 text-[13px] font-medium no-underline transition-colors"
                  style={{
                    borderColor: hue(h, 0.4),
                    background: hue(h, 0.12),
                  }}
                >
                  <span
                    aria-hidden
                    className="h-2 w-2 rounded-full"
                    style={{ background: hue(h) }}
                  />
                  {label}
                </a>
              );
            })}
          </nav>
        </div>

        <Section id="now" kicker="Now">
          {summary.isLoading ? (
            <Loading h={110} />
          ) : !s ? (
            <Failed />
          ) : (
            <div className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,160px),1fr))] gap-3">
              <Tile
                label={`${s.pots.spending.account_name ?? "Spending account"}, today`}
                value={
                  <RupeeOdometer
                    paise={s.pots.spending.balance_paise}
                    text={rupees(s.pots.spending.balance_paise)}
                  />
                }
                note="Your spending pot"
                color="sky"
              />
              <Tile
                label="Locked away"
                value={
                  <RupeeOdometer
                    paise={s.pots.locked.total_paise}
                    text={rupees(s.pots.locked.total_paise)}
                  />
                }
                note={`${s.pots.locked.count} deposit${s.pots.locked.count === 1 ? "" : "s"}`}
                color="grape"
              />
              <Tile
                label="Spent this period"
                value={
                  <RupeeOdometer
                    paise={s.spending.spent_paise}
                    text={rupees(s.spending.spent_paise)}
                  />
                }
                tone={s.spending.overspend_paise > 0 ? "over" : undefined}
                note={`Allowance ${rupees(s.spending.allowance_paise)}`}
                color="coral"
              />
              {s.next_period && (
                <Tile
                  label={`${s.next_period.label}, after what's promised`}
                  value={
                    <RupeeOdometer
                      paise={s.next_period.left_paise}
                      text={rupees(s.next_period.left_paise)}
                    />
                  }
                  tone={s.next_period.left_paise < 0 ? "over" : undefined}
                  note={`Of ${rupees(s.next_period.income_paise)} on ${dayShort(s.next_period.arrives)}`}
                  color="mint"
                />
              )}
              {score && (
                <Tile
                  label="Reward score"
                  value={`${Math.round(score.score_bps / 100)}%`}
                  note={`Earned ${rupees(score.earned_paise)}, paid ${rupees(score.payout_paise)}`}
                  color="sun"
                />
              )}
            </div>
          )}
        </Section>

        <Section id="so-far" kicker="Money so far">
          <div className="grid items-start gap-4 lg:grid-cols-2">
            <Panel
              title="Spent and invested, month by month"
              sub="Card bills count as transfers, so card purchases show as one bill."
              to="/spending"
            >
              <Legend series={MONEY_SERIES} />
              {analytics.isLoading ? (
                <Loading h={220} />
              ) : analytics.isError ? (
                <Failed />
              ) : months.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  Import a statement and the months fill in.
                </p>
              ) : (
                <Bars
                  label="Bar chart of money spent and invested each month"
                  series={MONEY_SERIES}
                  rows={months.map((m) => ({
                    key: m.month,
                    label: monthShort(m.month),
                    values: [m.spend_paise, m.invested_paise],
                  }))}
                />
              )}
            </Panel>
            <Panel
              title="Where spending went"
              sub="Last 90 days, biggest first."
              to="/spending/sort"
            >
              {analytics.isLoading ? (
                <Loading />
              ) : analytics.isError ? (
                <Failed />
              ) : cats.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  No spending in the last 90 days yet.
                </p>
              ) : (
                <CategoryBars rows={cats} total={catTotal} />
              )}
            </Panel>
          </div>
        </Section>

        <Section id="ahead" kicker="Ahead">
          <Panel
            title="What's already promised, the next 12 months"
            sub="Every commitment in the month it falls due, plus the year's one-off payments. Heavier months are marked."
            to="/year"
          >
            {year.isLoading ? (
              <Loading h={220} />
            ) : year.isError ? (
              <Failed />
            ) : (
              <Bars
                label="Bar chart of committed outflows in each of the next twelve months"
                series={OUTFLOW_SERIES}
                rows={outflows.map((o) => ({
                  key: o.month,
                  label: monthShort(o.month),
                  values: [o.total_paise],
                  flag: median > 0 && o.total_paise > median * 1.25,
                  notes: [
                    `Fixed ${rupees(o.fixed_paise)}`,
                    ...(o.lumpy_paise > 0
                      ? [
                          `One-offs ${rupees(o.lumpy_paise)}: ${o.items.join(", ")}`,
                        ]
                      : []),
                  ],
                }))}
              />
            )}
          </Panel>
          <div className="grid items-start gap-4 lg:grid-cols-2">
            <Panel title="Goals" to="/plan">
              {goals.isLoading ? (
                <Loading />
              ) : !goals.data || goals.data.goals.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  No savings goals yet.
                </p>
              ) : (
                <ul className="flex flex-col divide-y divide-border">
                  {goals.data.goals.map((g, gi) => (
                    <li
                      key={g.id}
                      className="flex flex-col gap-1.5 py-3 first:pt-0 last:pb-0"
                    >
                      <div className="flex items-baseline justify-between gap-3 text-[15px]">
                        <span className="min-w-0 truncate">{g.name}</span>
                        <span
                          className={`shrink-0 rounded-full px-2 text-[11px] font-semibold ${g.status === "behind" ? "tone-over" : "tone-good"}`}
                        >
                          {g.status === "behind"
                            ? "Behind"
                            : g.status === "done"
                              ? "Done"
                              : g.status === "open"
                                ? "No date"
                                : "On track"}
                        </span>
                      </div>
                      <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                        <div
                          className="h-full rounded-full"
                          style={{
                            width: `${pct(g.saved_paise, g.target_paise)}%`,
                            background: `linear-gradient(90deg, ${hue(GOAL_HUES[gi % GOAL_HUES.length], 0.65)}, ${hue(GOAL_HUES[gi % GOAL_HUES.length])})`,
                          }}
                        />
                      </div>
                      <span className="text-[13px] text-muted-foreground tabular-nums">
                        {rupees(g.saved_paise)} of {rupees(g.target_paise)}
                        {g.target_date && (
                          <>
                            {" "}
                            by {dayShort(g.target_date)}{" "}
                            {g.target_date.slice(0, 4)}
                          </>
                        )}
                        {g.needs_monthly_paise > 0 && (
                          <> · needs {rupees(g.needs_monthly_paise)} a month</>
                        )}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </Panel>
            <Panel title="Coming up next" to="/money">
              {!s ? (
                <Loading />
              ) : s.upcoming.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  Nothing due in the next few weeks.
                </p>
              ) : (
                <ul className="flex flex-col divide-y divide-border text-[14px]">
                  {s.upcoming.map((u) => (
                    <li
                      key={`${u.date}-${u.label}`}
                      className="flex items-baseline justify-between gap-3 py-2 first:pt-0 last:pb-0"
                    >
                      <span className="w-14 shrink-0 text-[13px] text-muted-foreground">
                        {dayShort(u.date)}
                      </span>
                      <span className="min-w-0 flex-1 truncate">{u.label}</span>
                      <span className="shrink-0 tabular-nums">
                        {rupees(u.amount_paise)}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </Panel>
          </div>
        </Section>

        <Section id="us" kicker="Us">
          <div className="grid items-start gap-4 lg:grid-cols-2">
            {couple.data?.in_couple ? (
              <Panel
                title="The date jar"
                sub={
                  jar
                    ? `${rupees(jar.spent_paise)} of a ${rupees(jar.budget_paise)} dates budget this month.`
                    : undefined
                }
                to="/joy/kept"
              >
                {kept.isLoading ? (
                  <Loading />
                ) : !kept.data ? (
                  <Failed />
                ) : (
                  <>
                    {jar && jar.budget_paise > 0 && (
                      <div className="h-2.5 w-full overflow-hidden rounded-full bg-muted">
                        <div
                          className="h-full rounded-full"
                          style={{
                            width: `${pct(jar.spent_paise, jar.budget_paise)}%`,
                            background:
                              jar.spent_paise > jar.budget_paise
                                ? "hsl(var(--over))"
                                : `linear-gradient(90deg, ${hue("rose")}, ${hue("berry")})`,
                          }}
                        />
                      </div>
                    )}
                    <ul className="flex flex-col divide-y divide-border text-[14px]">
                      {kept.data.months
                        .flatMap((m) => m.items)
                        .slice(0, 6)
                        .map((d) => (
                          <li
                            key={d.id}
                            className="flex items-baseline justify-between gap-3 py-2 first:pt-0 last:pb-0"
                          >
                            <span className="w-14 shrink-0 text-[13px] text-muted-foreground">
                              {dayShort(d.day)}
                            </span>
                            <span className="min-w-0 flex-1 truncate">
                              {d.title}
                              {d.place && (
                                <span className="text-muted-foreground">
                                  {" "}
                                  · {d.place}
                                </span>
                              )}
                            </span>
                            <span className="shrink-0 tabular-nums text-muted-foreground">
                              {d.rating ? `${d.rating} ★` : ""}
                              {d.cost_actual ? ` · ${d.cost_actual}` : ""}
                            </span>
                          </li>
                        ))}
                    </ul>
                    {stats && (
                      <p className="text-[13px] text-muted-foreground">
                        {stats.kept} kept · {rupees(stats.spent_paise)} spent on
                        them · {kept.data.skipped.length} slips put back for
                        another day
                      </p>
                    )}
                  </>
                )}
              </Panel>
            ) : (
              <Panel title="The date jar" to="/joy">
                <p className="text-sm text-muted-foreground">
                  Start the jar with your partner and your dates show up here.
                </p>
              </Panel>
            )}
            <Panel
              title="This month's reward score"
              sub="Money counts 40%; health, relationships and the future 20% each."
              to="/"
            >
              {fund.isLoading ? (
                <Loading />
              ) : !score ? (
                <Failed />
              ) : (
                <>
                  <div className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,130px),1fr))] gap-4">
                    {score.areas.map((a, ai) => {
                      const full = a.score_bps >= 10_000;
                      const ah = AREA_HUES[ai % AREA_HUES.length];
                      return (
                        <div key={a.area} className="flex flex-col gap-1">
                          <div className="flex items-baseline justify-between text-[14px]">
                            <span>{a.label}</span>
                            <span
                              className="tabular-nums"
                              style={{
                                color: full ? "hsl(var(--good))" : hue(ah),
                              }}
                            >
                              {Math.round(a.score_bps / 100)}%
                            </span>
                          </div>
                          <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                            <div
                              className="h-full rounded-full"
                              style={{
                                width: `${Math.max(2, Math.min(100, a.score_bps / 100))}%`,
                                background: full
                                  ? "hsl(var(--good))"
                                  : `linear-gradient(90deg, ${hue(ah, 0.6)}, ${hue(ah)})`,
                              }}
                            />
                          </div>
                          <span className="text-[12px] text-muted-foreground">
                            {a.raw} of {a.target} points
                          </span>
                        </div>
                      );
                    })}
                  </div>
                  {score.focus && (
                    <p
                      className="rounded-r-xl border-l-4 py-2 pl-3 pr-2 text-[14px]"
                      style={{
                        borderColor: hue("sun"),
                        background: hue("sun", 0.12),
                      }}
                    >
                      Weakest:{" "}
                      <b className="font-medium">
                        {score.areas.find((a) => a.area === score.focus)?.label}
                      </b>
                      . {score.areas.find((a) => a.area === score.focus)?.tip}
                    </p>
                  )}
                </>
              )}
            </Panel>
          </div>
          {trips.length > 0 && (
            <Panel
              title="Getaways the jar could save toward"
              sub="The jar's priced trip ideas, biggest first. Pick one as the jar's savings goal on Kept."
              to="/joy/kept"
            >
              <ul className="flex flex-col gap-2.5">
                {trips.map((t) => (
                  <li
                    key={t.id}
                    className="grid grid-cols-[minmax(0,40%)_1fr_auto] items-center gap-3 text-[13px]"
                  >
                    <span className="truncate" title={t.title}>
                      {t.title}
                    </span>
                    <span className="h-2 overflow-hidden rounded-full bg-muted">
                      <span
                        className="block h-full rounded-full"
                        style={{
                          width: `${pct(t.cost_paise, trips[0].cost_paise)}%`,
                          background: `linear-gradient(90deg, ${hue("sun")}, ${hue("coral")})`,
                        }}
                      />
                    </span>
                    <span className="tabular-nums">{rupees(t.cost_paise)}</span>
                  </li>
                ))}
              </ul>
            </Panel>
          )}
        </Section>

        <Section id="road" kicker="The road">
          <Panel
            title="The road ahead"
            sub="Built from the life plan, your goals and the year's payments. Each line says what it costs then and what it needs a month from now."
            to="/life"
          >
            {road.isLoading ? (
              <Loading h={240} />
            ) : !road.data ? (
              <Failed />
            ) : (
              <>
                {road.data.pace_all_paise > 0 && (
                  <p
                    className="rounded-r-xl border-l-4 py-2 pl-3 pr-2 text-[14px] leading-relaxed"
                    style={{
                      borderColor: hue("mint"),
                      background: hue("mint", 0.1),
                    }}
                  >
                    Saving for everything on it at once takes about{" "}
                    <b className="font-medium">
                      {rupees(road.data.pace_all_paise)}
                    </b>{" "}
                    a month
                    {wedding && wedding.per_month_paise > 0 && (
                      <>
                        ; the wedding alone needs{" "}
                        <b className="font-medium">
                          {rupees(wedding.per_month_paise)}
                        </b>
                      </>
                    )}
                    .
                  </p>
                )}
                <RoadAhead road={road.data} />
              </>
            )}
          </Panel>
        </Section>
      </main>
    </div>
  );
}
