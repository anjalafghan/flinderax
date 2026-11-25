import { useEffect, useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  Flame,
  Heart,
  PartyPopper,
  PiggyBank,
  ShoppingBag,
  Sparkles,
  TrendingDown,
  TrendingUp,
  Wallet,
  type LucideIcon,
} from "lucide-react";

import type { MonthReview } from "@/services/types";
import { useReviewMonth } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { Card, CardContent } from "@/components/ui/card";
import { categoryLabel, monthTitle } from "@/utils/spending";
import { formatINR, formatPercent } from "@/utils/money";
import { cn } from "@/utils/cn";
import { celebrate } from "@/utils/motion";
import { useSection } from "@/theme/useSection";
import SloshGauge from "@/components/bits/SloshGauge";
import { PercentOdometer, RupeeOdometer } from "@/components/feature/Odometer";
import {
  lastCompleteMonth,
  ratioPercent,
  signedINR,
  stepMonth,
} from "@/utils/review";

type Accent = "sky" | "mint" | "coral" | "rose" | "grape" | "sun";

// Each tile wears the colour of what it measures: savings mint, net worth sky, dates rose.
const ACCENT: Record<Accent, { card: string; bubble: string }> = {
  sky: {
    card: "border-sky/30 bg-gradient-to-br from-sky/12 via-card to-card",
    bubble: "bg-sky/18 text-sky",
  },
  mint: {
    card: "border-mint/30 bg-gradient-to-br from-mint/12 via-card to-card",
    bubble: "bg-mint/18 text-mint",
  },
  coral: {
    card: "border-coral/35 bg-gradient-to-br from-coral/14 via-card to-card",
    bubble: "bg-coral/18 text-coral",
  },
  rose: {
    card: "border-rose/30 bg-gradient-to-br from-rose/12 via-card to-card",
    bubble: "bg-rose/18 text-rose",
  },
  grape: {
    card: "border-grape/30 bg-gradient-to-br from-grape/12 via-card to-card",
    bubble: "bg-grape/18 text-grape",
  },
  sun: {
    card: "border-sun/35 bg-gradient-to-br from-sun/14 via-card to-card",
    bubble: "bg-sun/20 text-[hsl(var(--foreground))] dark:text-sun",
  },
};

// Bars for the top categories, one party colour each.
const BAR_COLORS = ["bg-coral", "bg-sun", "bg-grape"];

function Tile({
  title,
  icon: Icon,
  accent,
  children,
  className,
}: {
  title: string;
  icon: LucideIcon;
  accent: Accent;
  children: React.ReactNode;
  className?: string;
}) {
  const a = ACCENT[accent];
  return (
    <Card
      className={cn(
        "min-w-0 transition-transform duration-300 hover:-translate-y-0.5",
        a.card,
        className,
      )}
    >
      <CardContent className="space-y-3 p-5 md:p-6">
        <div className="flex items-center gap-2.5">
          <span
            aria-hidden
            className={cn(
              "flex h-8 w-8 shrink-0 items-center justify-center rounded-full",
              a.bubble,
            )}
          >
            <Icon className="h-4 w-4" strokeWidth={1.8} />
          </span>
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {title}
          </p>
        </div>
        {children}
      </CardContent>
    </Card>
  );
}

/** A rupee figure that rolls in, as long as it is a plain amount. */
function Rupees({ paise }: { paise: number }) {
  return paise >= 0 ? <RupeeOdometer paise={paise} /> : <>{formatINR(paise)}</>;
}

function SpentTile({ review }: { review: MonthReview }) {
  const over = review.over_by_paise > 0;
  const pct = ratioPercent(review.spent_paise, review.allowance_paise);
  const { dark } = useSection();
  // The tank fills with how much of the allowance went: mint with room left, sun near the top, coral over.
  const liquid = over
    ? "#a8321f"
    : pct >= 85
      ? "#f59e0b"
      : dark
        ? "#2c6b4d"
        : "#2c6b4d";
  return (
    <Tile
      title="Spent vs allowance"
      icon={Wallet}
      accent={over ? "coral" : "mint"}
    >
      <div className="flex items-end justify-between gap-4">
        <div className="min-w-0 space-y-2">
          <p
            className={cn(
              "text-2xl font-bold tabular-nums",
              over ? "text-over" : "text-good",
            )}
          >
            <Rupees paise={review.spent_paise} />
          </p>
          <p
            className={cn(
              "text-xs",
              over ? "text-over" : "text-muted-foreground",
            )}
          >
            {over
              ? `${formatINR(review.over_by_paise)} over`
              : `of ${formatINR(review.allowance_paise)} allowance`}
          </p>
        </div>
        <SloshGauge
          value={Math.round(pct)}
          ariaLabel="Spent against allowance"
          liquidColor={liquid}
          glassColor={dark ? "#1f2937" : "#eef6f3"}
          width={56}
          height={84}
          radius={16}
          ticks={4}
          className={dark ? "text-background" : "text-slate-800"}
        />
      </div>
    </Tile>
  );
}

function SavingsTile({ review }: { review: MonthReview }) {
  const bps = review.savings_rate_bps;
  return (
    <Tile
      title="Savings rate"
      icon={PiggyBank}
      accent={bps !== null && bps < 0 ? "coral" : "mint"}
    >
      <p
        className={cn(
          "text-2xl font-bold tabular-nums",
          bps === null ? "" : bps < 0 ? "text-over" : "text-good",
        )}
      >
        {bps === null ? (
          "—"
        ) : bps % 100 === 0 ? (
          <PercentOdometer percent={bps / 100} />
        ) : (
          formatPercent(bps)
        )}
      </p>
      <p className="text-xs text-muted-foreground">
        {review.income_paise > 0
          ? `of ${formatINR(review.income_paise)} income`
          : "No income recorded"}
      </p>
    </Tile>
  );
}

function NetWorthTile({ review }: { review: MonthReview }) {
  const change = review.net_worth_change_paise;
  const tone =
    change === null || change === 0
      ? ""
      : change > 0
        ? "text-good"
        : "text-over";
  return (
    <Tile
      title="Net worth change"
      icon={change !== null && change < 0 ? TrendingDown : TrendingUp}
      accent={change !== null && change < 0 ? "coral" : "sky"}
    >
      <p className={cn("text-2xl font-bold tabular-nums", tone)}>
        {change === null ? "—" : signedINR(change)}
      </p>
      <p className="text-xs text-muted-foreground">
        {change === null ? "Not enough history yet" : "over the month"}
      </p>
    </Tile>
  );
}

function DatesTile({ review }: { review: MonthReview }) {
  const d = review.dates;
  return (
    <Tile title="Dates" icon={Heart} accent="rose">
      {d === null ? (
        <p className="text-sm text-muted-foreground">No date spending</p>
      ) : (
        <>
          <p className="text-2xl font-bold tabular-nums">
            <Rupees paise={d.spent_paise} />
            {d.budget_paise !== null && (
              <span className="text-sm font-normal text-muted-foreground">
                {" "}
                of {formatINR(d.budget_paise)}
              </span>
            )}
          </p>
          <p className="text-xs text-muted-foreground">
            Your share {formatINR(d.your_share_paise)}
          </p>
        </>
      )}
    </Tile>
  );
}

function TopCategoriesTile({ review }: { review: MonthReview }) {
  const top = review.top_categories.slice(0, 3);
  const max = top[0]?.amount_paise ?? 0;
  return (
    <Tile
      title="Top categories"
      icon={ShoppingBag}
      accent="grape"
      className="sm:col-span-2"
    >
      {top.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          Nothing spent this month
        </p>
      ) : (
        <ul className="space-y-3">
          {top.map((c, i) => (
            <li key={c.category} className="space-y-1.5 text-sm">
              <div className="flex items-baseline justify-between gap-4">
                <span className="flex min-w-0 items-center gap-2">
                  <span
                    aria-hidden
                    className={cn(
                      "h-2.5 w-2.5 shrink-0 rounded-full",
                      BAR_COLORS[i],
                    )}
                  />
                  <span className="truncate">{categoryLabel(c.category)}</span>
                </span>
                <span className="shrink-0 font-semibold tabular-nums">
                  {formatINR(c.amount_paise)}
                </span>
              </div>
              <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                <div
                  className={cn(
                    "h-full rounded-full transition-[width] duration-700",
                    BAR_COLORS[i],
                  )}
                  style={{ width: `${ratioPercent(c.amount_paise, max)}%` }}
                />
              </div>
            </li>
          ))}
        </ul>
      )}
    </Tile>
  );
}

const SENTENCE_COLORS = ["sky", "grape", "mint", "sun", "rose"];

/** A good month: came in under the allowance and kept some of the income. */
const goodMonth = (r: MonthReview) =>
  r.allowance_paise > 0 &&
  r.over_by_paise <= 0 &&
  r.savings_rate_bps !== null &&
  r.savings_rate_bps > 0;

/** Confetti needs a 2D canvas; skip it where there is none (tests, old browsers). */
function cheer(kind: "burst" | "shower") {
  try {
    if (!document.createElement("canvas").getContext("2d")) return;
  } catch {
    return;
  }
  void celebrate(kind);
}

// Months already cheered this visit, so flipping back and forth does not rain confetti every time.
const cheered = new Set<string>();

function Verdict({ review }: { review: MonthReview }) {
  const good = goodMonth(review);
  const over = review.over_by_paise > 0;
  useEffect(() => {
    if (!good || cheered.has(review.month)) return;
    cheered.add(review.month);
    cheer("burst");
  }, [good, review.month]);
  if (good)
    return (
      <p className="pill pill-mint m-0 self-start px-3 py-1 text-sm">
        <PartyPopper className="h-4 w-4" aria-hidden />A good month
      </p>
    );
  if (over)
    return (
      <p className="pill pill-coral m-0 self-start px-3 py-1 text-sm">
        <Flame className="h-4 w-4" aria-hidden />
        Ran over the allowance
      </p>
    );
  return (
    <p className="pill pill-sky m-0 self-start px-3 py-1 text-sm">
      <Sparkles className="h-4 w-4" aria-hidden />
      Steady month
    </p>
  );
}

export function MonthClosePanel() {
  // Fixed for the life of the page: the newest month the switcher can reach.
  const [last] = useState(() => lastCompleteMonth());
  const [month, setMonth] = useState(last);
  const prev = stepMonth(month, -1, last) ?? month;
  const next = stepMonth(month, 1, last);
  const { data: review, isLoading, isError, error } = useReviewMonth(month);

  return (
    <section
      aria-label="Month close"
      className="stack glass rounded-[28px] p-6 md:p-8"
    >
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="space-y-2">
          <h2 className="brand-text font-display text-[32px] leading-none">
            Month close
          </h2>
          <p className="text-sm text-muted-foreground">
            How the month went, in plain words.
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-11 w-11 rounded-full text-brand-ink hover:bg-brand-tint"
            aria-label="Previous month"
            onClick={() => setMonth(prev)}
            disabled={prev === month}
          >
            <ChevronLeft className="h-4 w-4" aria-hidden />
          </Button>
          <p className="pill pill-sky min-w-[7.5rem] justify-center px-3 py-1 text-center text-sm font-semibold">
            {monthTitle(month)}
          </p>
          <Button
            variant="ghost"
            size="icon"
            className="h-11 w-11 rounded-full text-brand-ink hover:bg-brand-tint"
            aria-label="Next month"
            onClick={() => next && setMonth(next)}
            disabled={next === null}
          >
            <ChevronRight className="h-4 w-4" aria-hidden />
          </Button>
          {month !== last && (
            <ResetButton label="Latest" onReset={() => setMonth(last)} />
          )}
        </div>
      </div>

      {isLoading && (
        <p className="text-sm text-muted-foreground">Loading the month…</p>
      )}
      {isError && (
        <p role="alert" className="text-sm text-destructive">
          {error instanceof Error && error.message
            ? error.message
            : "Couldn't load this month."}
        </p>
      )}

      {review && (
        <>
          <Verdict review={review} />
          {review.sentences.length > 0 && (
            <ul className="space-y-3">
              {review.sentences.map((s, i) => (
                <li
                  key={i}
                  className="border-l-4 pl-4 text-lg leading-relaxed sm:text-xl"
                  style={{
                    borderColor: `hsl(var(--c-${SENTENCE_COLORS[i % SENTENCE_COLORS.length]}))`,
                  }}
                >
                  {s}
                </li>
              ))}
            </ul>
          )}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <SpentTile review={review} />
            <SavingsTile review={review} />
            <NetWorthTile review={review} />
            <DatesTile review={review} />
            <TopCategoriesTile review={review} />
          </div>
        </>
      )}
    </section>
  );
}
