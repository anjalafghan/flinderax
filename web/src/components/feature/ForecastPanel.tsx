import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { useForecast, type Basis, type Forecast, type GoalForecast, type YearBand } from "@/services/forecast";
import { formatDay } from "@/utils/dates";
import { cn } from "@/utils/cn";
import { formatCompactINR, formatINR } from "@/utils/money";

const SECTION = "flex flex-col gap-10 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";

// Chart geometry, in SVG user units.
const W = 600;
const H = 220;
const PAD_X = 12;
const PAD_TOP = 24;
const PAD_BOTTOM = 30;

/** Whole rupees with Indian grouping: round paise first, then format. */
const rupees = (paise: number) => formatINR(Math.round(paise / 100) * 100);

/** Whole percent from basis points: 8249 -> 82. */
const percent = (bps: number) => Math.round(bps / 100);

type Tone = "good" | "warn" | "over";

/** Likely at 80 and up, possible from 50, unlikely below. */
const toneFor = (pct: number): Tone => (pct >= 80 ? "good" : pct >= 50 ? "warn" : "over");

const BAR_FILL: Record<Tone, string> = { good: "bg-good", warn: "bg-warn", over: "bg-over" };

/** "2028-02-10" -> "10 Feb 2028", or "Now" for goals with no date. */
const whenLabel = (when: string | null): string => (when ? `${formatDay(when)} ${when.slice(0, 4)}` : "Now");

function Pill({ tone, children }: { tone: Tone; children: ReactNode }) {
  return (
    <span
      className={cn(
        "inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold",
        `tone-${tone}`,
      )}
    >
      {children}
    </span>
  );
}

/** The basis line: where the numbers come from, and a note when the plan and the statements disagree. */
function BasisLine({ basis }: { basis: Basis }) {
  const text =
    basis.source === "statements"
      ? `Based on ${basis.months_used} months of statements: about ${rupees(basis.income_paise)} in, ${rupees(basis.spend_paise)} out, ${rupees(basis.surplus_paise)} left each month.`
      : "Based on your plan's numbers. Upload statements for a forecast from what really happens.";
  return (
    <div className="flex flex-col gap-2 text-sm text-muted-foreground">
      <p className="m-0">{text}</p>
      {basis.note && <p className="m-0 text-[13px] leading-snug">{basis.note}</p>}
      {basis.calibration_note && <p className="m-0 text-[13px] leading-snug">{basis.calibration_note}</p>}
    </div>
  );
}

function LikelihoodMeter({ label, bps }: { label: string; bps: number }) {
  const pct = percent(bps);
  const tone = toneFor(pct);
  return (
    <div className="flex items-center gap-3">
      <div
        role="meter"
        aria-label={`Chance of reaching ${label} on time`}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
        aria-valuetext={`${pct}% likely`}
        className="h-2 min-w-0 flex-1 overflow-hidden rounded-full bg-chip"
      >
        <div className={cn("h-full rounded-full", BAR_FILL[tone])} style={{ width: `${pct}%` }} />
      </div>
      <Pill tone={tone}>{`${pct}% likely`}</Pill>
    </div>
  );
}

function GoalRow({ goal }: { goal: GoalForecast }) {
  return (
    <li className="flex flex-col gap-4 border-t border-border py-6 first:border-t-0 first:pt-1">
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
        <span className="text-[17px] font-medium">{goal.label}</span>
        <span className="text-[13px] text-muted-foreground">{whenLabel(goal.when)}</span>
      </div>

      <LikelihoodMeter label={goal.label} bps={goal.likelihood_bps} />

      <div className="flex flex-col gap-1 text-[13px] text-muted-foreground">
        <span>{`Likely between ${rupees(goal.p10_paise)} and ${rupees(goal.p90_paise)} (target ${rupees(goal.target_paise)}).`}</span>
        <span>{fundingLine(goal)}</span>
      </div>

      {goal.levers.length > 0 && (
        <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px] leading-snug">
          {goal.levers.map((l) => (
            <li key={l.text} className="flex gap-3">
              <span aria-hidden className="mt-2.5 h-1.5 w-1.5 shrink-0 rounded-full bg-muted-foreground" />
              <span>{l.text}</span>
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}

/** Ticks every few years, always with the last year, and never two labels so close they touch. */
function yearTicks(path: YearBand[]): number[] {
  const n = path.length;
  const step = Math.max(1, Math.ceil(n / 5));
  const ticks: number[] = [];
  for (let i = 0; i < n; i += step) ticks.push(i);
  const last = n - 1;
  if (ticks[ticks.length - 1] !== last) {
    if (last - ticks[ticks.length - 1] < step / 2) ticks.pop();
    ticks.push(last);
  }
  return ticks;
}

/** The fan: the middle 80% of simulated pots as a shaded band, the middle case as a line. */
function FanChart({ path }: { path: YearBand[] }) {
  const n = path.length;
  const first = path[0];
  const last = path[n - 1];
  const top = Math.max(1, ...path.map((p) => p.p90_paise));
  const plotH = H - PAD_TOP - PAD_BOTTOM;

  const x = (i: number) => (n === 1 ? W / 2 : PAD_X + (i / (n - 1)) * (W - 2 * PAD_X));
  const y = (paise: number) => PAD_TOP + (1 - paise / top) * plotH;

  const upper = path.map((p, i) => `${x(i)},${y(p.p90_paise)}`);
  const lower = path.map((p, i) => ({ p, i })).reverse().map(({ p, i }) => `${x(i)},${y(p.p10_paise)}`);
  const band = [...upper, ...lower].join(" ");
  const median = path.map((p, i) => `${x(i)},${y(p.p50_paise)}`).join(" ");

  const label =
    `Forecast of your goal pots from ${first.year} to ${last.year}. ` +
    `In ${last.year} the middle case is ${rupees(last.p50_paise)}, ` +
    `with the middle 80% of outcomes between ${rupees(last.p10_paise)} and ${rupees(last.p90_paise)}.`;

  return (
    <figure className="m-0 flex flex-col gap-3">
      <svg
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label={label}
        className="block h-auto w-full overflow-visible"
      >
        <text x={PAD_X} y={PAD_TOP - 8} className="text-[12px]" style={{ fill: "hsl(var(--muted-foreground))" }}>
          {formatCompactINR(top)}
        </text>
        <line
          x1={PAD_X}
          x2={W - PAD_X}
          y1={H - PAD_BOTTOM}
          y2={H - PAD_BOTTOM}
          style={{ stroke: "hsl(var(--border))" }}
        />
        <polygon points={band} style={{ fill: "hsl(var(--good) / 0.16)" }} />
        <polyline
          points={median}
          fill="none"
          strokeWidth={2.5}
          strokeLinejoin="round"
          strokeLinecap="round"
          style={{ stroke: "hsl(var(--good))" }}
        />
        {yearTicks(path).map((i) => {
          const anchor = i === 0 ? "start" : i === n - 1 ? "end" : "middle";
          return (
            <text
              key={path[i].year}
              x={x(i)}
              y={H - 10}
              textAnchor={anchor}
              className="text-[12px]"
              style={{ fill: "hsl(var(--muted-foreground))" }}
            >
              {path[i].year}
            </text>
          );
        })}
      </svg>
      <figcaption className="text-[13px] text-muted-foreground">
        The shaded band is the middle 80% of simulated outcomes. The line is the middle case.
      </figcaption>
    </figure>
  );
}

function ForecastBody({ data }: { data: Forecast }) {
  return (
    <section aria-labelledby="forecast-title" className={SECTION}>
      <header className="flex flex-col gap-3">
        <h2 id="forecast-title" className="m-0 font-display text-[32px] leading-none">
          Will you get there?
        </h2>
        <p className="m-0 text-base">{data.summary}</p>
        <BasisLine basis={data.basis} />
        {!data.saved_profile && (
          <p className="empty-note m-0">Add your goals below for a forecast of them.</p>
        )}
      </header>

      {data.goals.length > 0 && (
        <ul className="m-0 flex list-none flex-col p-0">
          {data.goals.map((g) => (
            <GoalRow key={g.key} goal={g} />
          ))}
        </ul>
      )}

      {data.path.length > 0 && (
        <div className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <h3 className="m-0 font-display text-[24px] leading-tight">Your goal pots over time</h3>
            <span className="text-[13px] text-muted-foreground">All goals together, in each year</span>
          </div>
          <FanChart path={data.path} />
        </div>
      )}
    </section>
  );
}

/** "Will you get there?": the goal forecast from the statements (or the plan), on the Life page. */
/** How this goal's monthly share compares with what it needs. */
function fundingLine(goal: GoalForecast): string {
  const funded = rupees(goal.funded_monthly_paise);
  const needed = rupees(goal.needed_monthly_paise);
  if (goal.funded_monthly_paise > goal.needed_monthly_paise) return `Gets ${funded} a month, more than the ${needed} it needs.`;
  return `Getting ${funded} of the ${needed} a month it needs.`;
}

export function ForecastPanel() {
  const { data, isLoading, isError, refetch } = useForecast();

  if (isLoading) {
    return <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your forecast" />;
  }
  if (isError || !data) {
    return (
      <div className="rounded-[28px] border border-dashed border-border bg-card/30 p-6 text-center text-sm text-muted-foreground">
        Couldn't load your forecast.{" "}
        <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  return <ForecastBody data={data} />;
}
