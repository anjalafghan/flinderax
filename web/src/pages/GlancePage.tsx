import { useQuery } from "@tanstack/react-query";
import { ArrowRight } from "lucide-react";
import type { ReactNode } from "react";
import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { Skeleton } from "@/components/ui/skeleton";
import { CategoryBars } from "@/components/feature/CategoryBars";
import { Bars, Legend, type BarSeries } from "@/components/feature/glance/Bars";
import { RoadAhead } from "@/components/feature/glance/RoadAhead";
import { chartMonths, datesStats, dayShort, getaways, jarBudget, monthShort, outflowMonths, pct } from "@/components/feature/glance/glance";
import { useGoals, useSummary } from "@/hooks/usePlanner";
import { getCouple, keys as joyKeys } from "@/joy/api";
import { getKept, keptKey } from "@/joy/kept/api";
import { useAnalytics } from "@/services/analytics";
import { useRewardFund } from "@/services/rewards";
import { useRoad } from "@/services/road";
import { useYear } from "@/services/year";
import type { DashboardSummary } from "@/services/types";
import { formatINR } from "@/utils/money";

/** Whole rupees: this page is about the shape of things, not the paise. */
const rupees = (p: number) => formatINR(Math.round(p / 100) * 100);

const MONEY_SERIES: BarSeries[] = [
  { name: "Spent", color: "hsl(var(--primary))" },
  { name: "Invested", color: "hsl(var(--good))" },
];
const OUTFLOW_SERIES: BarSeries[] = [{ name: "Promised", color: "hsl(var(--primary))" }];

function Section({ id, kicker, children }: { id: string; kicker: string; children: ReactNode }) {
  return (
    <section id={id} aria-labelledby={`${id}-k`} className="flex flex-col gap-4">
      <p id={`${id}-k`} className="text-xs font-medium uppercase tracking-[0.08em] text-muted-foreground">
        {kicker}
      </p>
      {children}
    </section>
  );
}

function Panel({ title, sub, children, to }: { title: string; sub?: string; children: ReactNode; to?: string }) {
  return (
    <div className="glass flex min-w-0 flex-col gap-3 rounded-[24px] p-5 md:p-6">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h2 className="text-[17px] font-medium">{title}</h2>
          {sub && <p className="mt-0.5 text-[13px] text-muted-foreground">{sub}</p>}
        </div>
        {to && (
          <Link to={to} viewTransition className="shrink-0 text-muted-foreground" aria-label={`Open ${title}`}>
            <ArrowRight className="h-4 w-4" strokeWidth={1.5} />
          </Link>
        )}
      </div>
      {children}
    </div>
  );
}

function Tile({ label, value, note, tone }: { label: string; value: ReactNode; note?: ReactNode; tone?: "over" | "good" }) {
  return (
    <div className="glass flex min-w-0 flex-col gap-1 rounded-[20px] p-4 md:p-5">
      <span className="text-[13px] text-muted-foreground">{label}</span>
      <span
        className="font-display text-[1.9rem] leading-tight tabular-nums"
        style={tone ? { color: `hsl(var(--${tone}))` } : undefined}
      >
        {value}
      </span>
      {note && <span className="text-[12px] text-muted-foreground">{note}</span>}
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
  return <p className="text-sm text-muted-foreground">This part didn't load. Pull to refresh, or try again in a moment.</p>;
}

/** The sentence under the title: where spending stands, and how the dates are going. */
function lede(s: DashboardSummary | undefined, dates: { kept: number; average: number | null } | null): string | null {
  if (!s) return null;
  const sp = s.spending;
  const days = `${s.days_left} day${s.days_left === 1 ? "" : "s"} to go`;
  const money = !sp.tracked
    ? "Spending isn't tracked yet"
    : sp.overspend_paise > 0
      ? `Spending is ${rupees(sp.overspend_paise)} past its ${rupees(sp.allowance_paise)} allowance with ${days}`
      : `Spending is ${rupees(sp.left_paise)} inside its ${rupees(sp.allowance_paise)} allowance with ${days}`;
  if (!dates || dates.kept === 0) return `${money}.`;
  const avg = dates.average !== null ? `, rated ${dates.average} out of 5 on average` : "";
  return `${money}, and ${dates.kept} date${dates.kept === 1 ? "" : "s"} kept from the jar${avg}.`;
}

export default function GlancePage() {
  const summary = useSummary();
  const analytics = useAnalytics();
  const year = useYear();
  const goals = useGoals();
  const fund = useRewardFund();
  const road = useRoad();
  const couple = useQuery({ queryKey: joyKeys.couple, queryFn: getCouple, retry: 1 });
  const kept = useQuery({ queryKey: keptKey, queryFn: getKept, retry: 1, enabled: !!couple.data?.in_couple });

  const s = summary.data;
  const stats = kept.data ? datesStats(kept.data) : null;
  const months = analytics.data ? chartMonths(analytics.data.months) : [];
  const outflows = year.data ? outflowMonths(year.data) : [];
  const median = outflows.length ? [...outflows].sort((a, b) => a.total_paise - b.total_paise)[Math.floor(outflows.length / 2)].total_paise : 0;
  const cats = (analytics.data?.categories ?? []).map((c) => ({ category: c.category, amount_paise: c.paise })).slice(0, 8);
  const catTotal = (analytics.data?.categories ?? []).reduce((t, c) => t + c.paise, 0);
  const score = fund.data?.this_month.score;
  const jar = kept.data ? jarBudget(kept.data) : null;
  const trips = kept.data ? getaways(kept.data) : [];
  const wedding = road.data?.milestones.find((m) => m.source === "life" && m.area === "family" && /wedding/i.test(m.title));
  const line = lede(s, stats);

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page flex flex-col gap-10">
        <div>
          <h1 className="page-title">At a glance</h1>
          {line ? <p className="page-sub max-w-[62ch]">{line}</p> : <Skeleton className="mt-3 h-5 w-2/3" />}
          <nav aria-label="Sections" className="mt-4 flex flex-wrap gap-2">
            {[
              ["#now", "Now"],
              ["#so-far", "Money so far"],
              ["#ahead", "Ahead"],
              ["#us", "Us"],
              ["#road", "The road"],
            ].map(([href, label]) => (
              <a
                key={href}
                href={href}
                className="rounded-full border border-border px-3 py-1 text-[13px] no-underline transition-colors hover:bg-[hsl(var(--chip))]"
              >
                {label}
              </a>
            ))}
          </nav>
        </div>

        <Section id="now" kicker="Now">
          {summary.isLoading ? (
            <Loading h={110} />
          ) : !s ? (
            <Failed />
          ) : (
            <div className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,160px),1fr))] gap-3">
              <Tile label={`${s.pots.spending.account_name ?? "Spending account"}, today`} value={rupees(s.pots.spending.balance_paise)} note="Your spending pot" />
              <Tile
                label="Locked away"
                value={rupees(s.pots.locked.total_paise)}
                note={`${s.pots.locked.count} deposit${s.pots.locked.count === 1 ? "" : "s"}`}
              />
              <Tile
                label="Spent this period"
                value={rupees(s.spending.spent_paise)}
                tone={s.spending.overspend_paise > 0 ? "over" : undefined}
                note={`Allowance ${rupees(s.spending.allowance_paise)}`}
              />
              {s.next_period && (
                <Tile
                  label={`${s.next_period.label}, after what's promised`}
                  value={rupees(s.next_period.left_paise)}
                  tone={s.next_period.left_paise < 0 ? "over" : undefined}
                  note={`Of ${rupees(s.next_period.income_paise)} on ${dayShort(s.next_period.arrives)}`}
                />
              )}
              {score && (
                <Tile
                  label="Reward score"
                  value={`${Math.round(score.score_bps / 100)}%`}
                  note={`Earned ${rupees(score.earned_paise)}, paid ${rupees(score.payout_paise)}`}
                />
              )}
            </div>
          )}
        </Section>

        <Section id="so-far" kicker="Money so far">
          <div className="grid items-start gap-4 lg:grid-cols-2">
            <Panel title="Spent and invested, month by month" sub="Card bills count as transfers, so card purchases show as one bill." to="/spending">
              <Legend series={MONEY_SERIES} />
              {analytics.isLoading ? (
                <Loading h={220} />
              ) : analytics.isError ? (
                <Failed />
              ) : months.length === 0 ? (
                <p className="text-sm text-muted-foreground">Import a statement and the months fill in.</p>
              ) : (
                <Bars
                  label="Bar chart of money spent and invested each month"
                  series={MONEY_SERIES}
                  rows={months.map((m) => ({ key: m.month, label: monthShort(m.month), values: [m.spend_paise, m.invested_paise] }))}
                />
              )}
            </Panel>
            <Panel title="Where spending went" sub="Last 90 days, biggest first." to="/spending/sort">
              {analytics.isLoading ? (
                <Loading />
              ) : analytics.isError ? (
                <Failed />
              ) : cats.length === 0 ? (
                <p className="text-sm text-muted-foreground">No spending in the last 90 days yet.</p>
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
                  notes: [`Fixed ${rupees(o.fixed_paise)}`, ...(o.lumpy_paise > 0 ? [`One-offs ${rupees(o.lumpy_paise)}: ${o.items.join(", ")}`] : [])],
                }))}
              />
            )}
          </Panel>
          <div className="grid items-start gap-4 lg:grid-cols-2">
            <Panel title="Goals" to="/plan">
              {goals.isLoading ? (
                <Loading />
              ) : !goals.data || goals.data.goals.length === 0 ? (
                <p className="text-sm text-muted-foreground">No savings goals yet.</p>
              ) : (
                <ul className="flex flex-col divide-y divide-border">
                  {goals.data.goals.map((g) => (
                    <li key={g.id} className="flex flex-col gap-1.5 py-3 first:pt-0 last:pb-0">
                      <div className="flex items-baseline justify-between gap-3 text-[15px]">
                        <span className="min-w-0 truncate">{g.name}</span>
                        <span
                          className="shrink-0 rounded-full border px-2 text-[11px]"
                          style={{ color: `hsl(var(--${g.status === "behind" ? "over" : "good"}))` }}
                        >
                          {g.status === "behind" ? "Behind" : g.status === "done" ? "Done" : g.status === "open" ? "No date" : "On track"}
                        </span>
                      </div>
                      <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                        <div className="h-full rounded-full bg-primary" style={{ width: `${pct(g.saved_paise, g.target_paise)}%` }} />
                      </div>
                      <span className="text-[13px] text-muted-foreground tabular-nums">
                        {rupees(g.saved_paise)} of {rupees(g.target_paise)}
                        {g.target_date && <> by {dayShort(g.target_date)} {g.target_date.slice(0, 4)}</>}
                        {g.needs_monthly_paise > 0 && <> · needs {rupees(g.needs_monthly_paise)} a month</>}
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
                <p className="text-sm text-muted-foreground">Nothing due in the next few weeks.</p>
              ) : (
                <ul className="flex flex-col divide-y divide-border text-[14px]">
                  {s.upcoming.map((u) => (
                    <li key={`${u.date}-${u.label}`} className="flex items-baseline justify-between gap-3 py-2 first:pt-0 last:pb-0">
                      <span className="w-14 shrink-0 text-[13px] text-muted-foreground">{dayShort(u.date)}</span>
                      <span className="min-w-0 flex-1 truncate">{u.label}</span>
                      <span className="shrink-0 tabular-nums">{rupees(u.amount_paise)}</span>
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
                sub={jar ? `${rupees(jar.spent_paise)} of a ${rupees(jar.budget_paise)} dates budget this month.` : undefined}
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
                            background: jar.spent_paise > jar.budget_paise ? "hsl(var(--over))" : "hsl(var(--good))",
                          }}
                        />
                      </div>
                    )}
                    <ul className="flex flex-col divide-y divide-border text-[14px]">
                      {kept.data.months
                        .flatMap((m) => m.items)
                        .slice(0, 6)
                        .map((d) => (
                          <li key={d.id} className="flex items-baseline justify-between gap-3 py-2 first:pt-0 last:pb-0">
                            <span className="w-14 shrink-0 text-[13px] text-muted-foreground">{dayShort(d.day)}</span>
                            <span className="min-w-0 flex-1 truncate">
                              {d.title}
                              {d.place && <span className="text-muted-foreground"> · {d.place}</span>}
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
                        {stats.kept} kept · {rupees(stats.spent_paise)} spent on them · {kept.data.skipped.length} slips put back for another day
                      </p>
                    )}
                  </>
                )}
              </Panel>
            ) : (
              <Panel title="The date jar" to="/joy">
                <p className="text-sm text-muted-foreground">Start the jar with your partner and your dates show up here.</p>
              </Panel>
            )}
            <Panel title="This month's reward score" sub="Money counts 40%; health, relationships and the future 20% each." to="/">
              {fund.isLoading ? (
                <Loading />
              ) : !score ? (
                <Failed />
              ) : (
                <>
                  <div className="grid grid-cols-[repeat(auto-fit,minmax(min(100%,130px),1fr))] gap-4">
                    {score.areas.map((a) => {
                      const full = a.score_bps >= 10_000;
                      return (
                        <div key={a.area} className="flex flex-col gap-1">
                          <div className="flex items-baseline justify-between text-[14px]">
                            <span>{a.label}</span>
                            <span className="tabular-nums" style={{ color: `hsl(var(--${full ? "good" : "over"}))` }}>
                              {Math.round(a.score_bps / 100)}%
                            </span>
                          </div>
                          <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                            <div
                              className="h-full rounded-full"
                              style={{ width: `${Math.max(2, Math.min(100, a.score_bps / 100))}%`, background: full ? "hsl(var(--good))" : "hsl(var(--over))" }}
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
                    <p className="border-l-2 border-border pl-3 text-[14px]">
                      Weakest: <b className="font-medium">{score.areas.find((a) => a.area === score.focus)?.label}</b>.{" "}
                      {score.areas.find((a) => a.area === score.focus)?.tip}
                    </p>
                  )}
                </>
              )}
            </Panel>
          </div>
          {trips.length > 0 && (
            <Panel title="Getaways the jar could save toward" sub="The jar's priced trip ideas, biggest first. Pick one as the jar's savings goal on Kept." to="/joy/kept">
              <ul className="flex flex-col gap-2.5">
                {trips.map((t) => (
                  <li key={t.id} className="grid grid-cols-[minmax(0,40%)_1fr_auto] items-center gap-3 text-[13px]">
                    <span className="truncate" title={t.title}>
                      {t.title}
                    </span>
                    <span className="h-2 overflow-hidden rounded-full bg-muted">
                      <span className="block h-full rounded-full bg-[hsl(var(--warn))]" style={{ width: `${pct(t.cost_paise, trips[0].cost_paise)}%` }} />
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
                  <p className="border-l-2 border-border pl-3 text-[14px] leading-relaxed">
                    Saving for everything on it at once takes about <b className="font-medium">{rupees(road.data.pace_all_paise)}</b> a month
                    {wedding && wedding.per_month_paise > 0 && (
                      <>
                        ; the wedding alone needs <b className="font-medium">{rupees(wedding.per_month_paise)}</b>
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
