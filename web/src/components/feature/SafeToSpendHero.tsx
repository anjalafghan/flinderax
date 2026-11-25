import { lazy, Suspense } from "react";
import { Clock } from "lucide-react";
import { Link } from "react-router-dom";

import Counter from "@/components/bits/Counter";
import StarBorder from "@/components/bits/StarBorder";
import { useTheme } from "@/context/ThemeContext";

import type { DashboardSummary, Status } from "@/services/types";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { daysBetween, localIsoDate } from "@/utils/planner";
import { cn } from "@/utils/cn";
import { useCountUp } from "@/hooks/useCountUp";

// The one WebGL background on Today, loaded after the number so it never holds it up.
const SoftAurora = lazy(() => import("@/components/bits/SoftAurora"));

/** Aurora colours per status: mint when fine, sun when snug, coral when over. */
const AURORA: Record<Tone["tone"] | "quiet", [string, string]> = {
  good: ["#8a8073", "#8a8073"],
  warn: ["#8a8073", "#8a8073"],
  over: ["#8a8073", "#8a8073"],
  quiet: ["#8a8073", "#8a8073"],
};

/**
 * Rupees as an odometer. The formatted text (same as formatINR) sits in a screen-reader copy inside
 * Counter, so tests and assistive tech read the real value; the rolling digits are hidden from them.
 */
export function MoneyCounter({
  paise,
  prefix,
  className,
}: {
  paise: number;
  prefix?: string;
  className?: string;
}) {
  const fraction = paise % 100 === 0 ? 0 : 2;
  return (
    <Counter
      value={paise / 100}
      locale="en-IN"
      format={{
        style: "currency",
        currency: "INR",
        minimumFractionDigits: fraction,
        maximumFractionDigits: fraction,
      }}
      prefix={prefix || undefined}
      fontSize="1em"
      className={className}
    />
  );
}

/** Balances this old or older make the number a guess, not just a little behind. */
const STALE_GUESS_DAYS = 21;

interface Tone {
  label: string;
  /** Shown instead of `label` when the data behind the number is old. */
  unsure: string;
  tone: "good" | "warn" | "over";
  line: (until: string) => string;
}

const TONES: Record<Status, Tone> = {
  green: {
    label: "You’re doing well",
    unsure: "Probably fine",
    tone: "good",
    line: () => "On track. Nicely done.",
  },
  yellow: {
    label: "Getting snug",
    unsure: "Probably tight",
    tone: "warn",
    line: () => "A little care this week keeps you comfortable.",
  },
  red: {
    label: "A bit over plan",
    unsure: "Possibly over",
    tone: "over",
    line: (u) => `Pause non-essentials until ${u}. You can get back on track.`,
  },
};

/** Whole rupees: the hero is a reading, not a ledger, so paise are dropped. */
function rupees(paise: number): string {
  return formatINR(Math.round(paise / 100) * 100);
}

/** "2026-11-08" minus `days` days, as an ISO date. */
function isoMinusDays(iso: string, days: number): string {
  const [y, m, d] = iso.split("-").map(Number);
  const t = new Date(y, m - 1, d - days);
  return localIsoDate(t);
}

/** One bar per day of the pay period: past days muted, today in the status colour, the rest hairline. */
function PeriodStrip({
  start,
  until,
  today,
  dot,
}: {
  start: string;
  until: string;
  today: string;
  dot: string;
}) {
  const total = Math.max(1, Math.min(62, daysBetween(start, until)));
  const elapsed = Math.max(0, Math.min(total - 1, daysBetween(start, today)));
  return (
    <div className="flex flex-col gap-2.5">
      <div
        role="img"
        aria-label={`Day ${elapsed + 1} of ${total} in this pay period`}
        className="grid gap-[3px]"
        style={{ gridTemplateColumns: `repeat(${total}, minmax(0, 1fr))` }}
      >
        {Array.from({ length: total }, (_, i) => (
          <span
            key={i}
            className={cn(
              "h-1.5 rounded-[3px]",
              i === elapsed &&
                "h-2 [animation:today-bounce_1.8s_ease-in-out_infinite_.6s]",
            )}
            style={
              i < elapsed
                ? {
                    ["--i" as string]: i,
                    // Days gone are slices of one sunrise gradient across the whole period.
                    backgroundImage:
                      "linear-gradient(90deg, hsl(var(--brand-3)), hsl(var(--brand)), hsl(var(--brand-2)))",
                    backgroundSize: `${total * 100}% 100%`,
                    backgroundPosition: `${total > 1 ? (i / (total - 1)) * 100 : 0}% 0`,
                  }
                : i === elapsed
                  ? {
                      ["--i" as string]: i,
                      background: dot,
                      boxShadow: `0 0 10px ${dot}`,
                    }
                  : {
                      ["--i" as string]: i,
                      background: "hsl(var(--border))",
                    }
            }
          />
        ))}
      </div>
      <div className="flex justify-between text-[13px] text-muted-foreground">
        <span>{formatDay(start)}</span>
        <span className="hidden font-medium text-brand-ink sm:inline">
          Today, {formatDay(today)}
        </span>
        <span>{formatDay(until)}</span>
      </div>
    </div>
  );
}

/** The one number: what is safe to spend from now until the next payday. */
export function SafeToSpendHero({ summary }: { summary: DashboardSummary }) {
  const today = localIsoDate();
  const until = formatDay(summary.until);
  const t = TONES[summary.status];

  // Freshness comes from the oldest stale input. Over a week: "~" and a softer number; three weeks: a guess.
  const oldest = summary.stale.reduce<DashboardSummary["stale"][number] | null>(
    (a, s) => (!a || s.days_old > a.days_old ? s : a),
    null,
  );
  const freshness: "fresh" | "week" | "stale" = !oldest
    ? "fresh"
    : oldest.days_old >= STALE_GUESS_DAYS
      ? "stale"
      : "week";
  const tilde = freshness === "fresh" ? "" : "~";
  const since = oldest ? formatDay(isoMinusDays(today, oldest.days_old)) : "";

  // Warnings keep their colour when unsure; only "fine" softens, because we can't promise it.
  const pillClass =
    freshness !== "fresh" && t.tone === "good"
      ? "tone-quiet"
      : `tone-${t.tone}`;
  const pillLabel = freshness === "fresh" ? t.label : t.unsure;
  const dot =
    pillClass === "tone-quiet"
      ? "hsl(var(--muted-foreground))"
      : `hsl(var(--${t.tone}))`;

  const line =
    freshness === "fresh"
      ? t.line(until)
      : freshness === "week"
        ? `Could be a little lower: spending since ${since} isn’t counted.`
        : `Could be lower: spending since ${since} isn’t counted.`;

  const { theme } = useTheme();
  const shownWeek = useCountUp(summary.per_week_paise);
  const wholeSafe = Math.round(summary.safe_to_spend_paise / 100) * 100;
  const amount = rupees(summary.safe_to_spend_paise);
  // The status colour the card glows in; "quiet" when a fine number can't be promised.
  const glow = pillClass === "tone-quiet" ? "quiet" : t.tone;
  const glowVar = glow === "quiet" ? "--brand" : `--${glow}`;
  const aria =
    freshness === "fresh"
      ? `Safe to spend: ${amount}`
      : `Safe to spend: roughly ${amount}, from a ${oldest!.days_old}-day-old balance`;

  return (
    <section
      aria-labelledby="safe-title"
      data-status={summary.status}
      className="relative isolate flex flex-col gap-7 overflow-hidden rounded-[28px] border px-6 pb-8 pt-8 transition-[border-color] duration-500 sm:px-12 sm:pb-10 sm:pt-12"
      style={{
        borderColor: `hsl(var(${glowVar}) / 0.35)`,
        background: `radial-gradient(110% 80% at 100% 0%, hsl(var(${glowVar}) / 0.14) 0%, transparent 60%), radial-gradient(90% 70% at 0% 100%, hsl(var(--brand-tint-2)) 0%, transparent 65%), hsl(var(--card))`,
        boxShadow: `0 1px 0 hsl(var(${glowVar}) / 0.08), 0 24px 60px -30px hsl(var(${glowVar}) / 0.55)`,
      }}
    >
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[75%] opacity-60 [mask-image:linear-gradient(to_bottom,#000_30%,transparent)]"
      >
        <Suspense fallback={null}>
          <SoftAurora
            color1={AURORA[glow][0]}
            color2={AURORA[glow][1]}
            lightMode={theme !== "dark"}
            speed={0.4}
            brightness={theme === "dark" ? 0.9 : 1.1}
            enableMouseInteraction={false}
          />
        </Suspense>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1
          id="safe-title"
          className="m-0 text-[15px] font-medium text-muted-foreground"
        >
          Safe to spend until {until}
        </h1>
        <span
          className={cn(
            "inline-flex min-h-8 items-center gap-2 rounded-full px-3.5 text-sm font-semibold transition-colors duration-500",
            pillClass,
          )}
        >
          <span
            className="h-2 w-2 rounded-full [animation:soft-pulse_2.4s_ease-in-out_infinite]"
            style={{ background: dot, boxShadow: `0 0 8px ${dot}` }}
            aria-hidden
          />
          {pillLabel}
        </span>
      </div>

      <div className="flex flex-col gap-3.5">
        <p
          aria-label={aria}
          className={cn(
            "relative isolate m-0 self-start font-display text-[72px] font-light leading-[0.95] tracking-[-0.03em] transition-colors duration-500 sm:text-[104px] lg:text-[136px]",
            freshness === "fresh" && "text-foreground",
            freshness === "week" && "text-[hsl(var(--soft))]",
            freshness === "stale" &&
              "text-muted-foreground underline decoration-border decoration-dotted decoration-[3px] underline-offset-[18px]",
          )}
        >
          <span
            aria-hidden
            className="pointer-events-none absolute -inset-x-8 inset-y-0 -z-10 blur-2xl transition-[background] duration-500"
            style={{
              background: `radial-gradient(closest-side, hsl(var(${glowVar}) / 0.4), transparent)`,
            }}
          />
          <MoneyCounter paise={wholeSafe} prefix={tilde} />
        </p>
        <p className="m-0 text-base text-muted-foreground sm:text-lg">
          About{" "}
          <span className="font-semibold text-brand-ink">
            {tilde}
            {rupees(shownWeek)} a week
          </span>{" "}
          for the next {summary.days_left}{" "}
          {summary.days_left === 1 ? "day" : "days"}. {line}
        </p>
        {summary.short_by_paise > 0 && (
          <p className="m-0 text-sm text-[hsl(var(--warn))]">
            {summary.pots.spending.account_name ?? "Your spending account"} is
            short by {formatINR(summary.short_by_paise)} for the moves below.
            Mark what you’ve already dealt with, or skip this month’s savings
            top-up.
          </p>
        )}
        {summary.spending.tracked && summary.spending.overspend_paise > 0 && (
          <p className="m-0 text-sm text-[hsl(var(--over))]">
            You’ve gone {formatINR(summary.spending.overspend_paise)} over this
            month’s spending allowance.
          </p>
        )}
      </div>

      <PeriodStrip
        start={summary.period_start}
        until={summary.until}
        today={today}
        dot={dot}
      />

      {freshness === "stale" && oldest && (
        <div className="tone-warn flex flex-wrap items-center justify-between gap-4 rounded-[18px] px-5 py-[18px]">
          <div className="flex min-w-0 flex-[1_1_360px] items-start gap-3">
            <Clock
              className="mt-px h-5 w-5 shrink-0"
              strokeWidth={1.6}
              aria-hidden
            />
            <span className="flex flex-col gap-0.5">
              <span className="text-[15px] font-semibold">
                This is a guess, based on {oldest.what} from {since}
              </span>
              <span className="text-sm text-muted-foreground">
                {oldest.days_old} days old. Update it from your bank app and the
                number becomes real again.
              </span>
            </span>
          </div>
          <StarBorder
            as={Link}
            to="/money"
            radius={999}
            color={theme === "dark" ? "#c9bfae" : "#8a8073"}
            backgroundColor="hsl(var(--foreground))"
            textColor="hsl(var(--background))"
            borderColor="hsl(var(--warn) / 0.5)"
            theme={theme === "dark" ? "dark" : "light"}
            glow={0.8}
            className="min-h-11 !px-5 !py-0 text-sm font-semibold"
          >
            Update balance
          </StarBorder>
        </div>
      )}

      {freshness === "week" && oldest && (
        <div className="flex flex-wrap items-center justify-between gap-4 rounded-[18px] border border-warn/30 bg-warn-tint/60 px-5 py-3.5">
          <span className="inline-flex items-center gap-2.5 text-sm text-muted-foreground">
            <Clock
              className="h-[18px] w-[18px]"
              strokeWidth={1.6}
              aria-hidden
            />
            {oldest.what} from {since} · {oldest.days_old} days old
          </span>
          <Link
            to="/money"
            className="inline-flex min-h-11 items-center rounded-full border border-warn/40 bg-card px-[18px] text-sm font-semibold text-foreground no-underline transition-colors hover:bg-warn-tint"
          >
            Update
          </Link>
        </div>
      )}

      <div className="flex flex-wrap justify-between gap-3 text-sm text-muted-foreground">
        <span>
          {summary.next_income
            ? `${summary.next_income.label} arrives ${formatDay(summary.next_income.arrives)} and isn’t counted here.`
            : summary.spending.tracked
              ? `${formatINR(summary.spending.spent_paise)} spent so far of a ${formatINR(summary.spending.allowance_paise)} allowance.`
              : "Spending isn’t tracked yet. Update a card from your bank app to start."}
        </span>
        <Link
          to="/working"
          className="min-h-6 font-semibold text-brand-ink no-underline hover:underline"
        >
          See the working →
        </Link>
      </div>
    </section>
  );
}
