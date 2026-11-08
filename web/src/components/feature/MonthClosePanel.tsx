import { useState } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";

import type { MonthReview } from "@/services/types";
import { useReviewMonth } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { categoryLabel, monthTitle } from "@/utils/spending";
import { formatINR, formatPercent } from "@/utils/money";
import { cn } from "@/utils/cn";
import { lastCompleteMonth, ratioPercent, signedINR, stepMonth } from "@/utils/review";

function Tile({ title, children, className }: { title: string; children: React.ReactNode; className?: string }) {
  return (
    <Card className={cn("min-w-0", className)}>
      <CardContent className="space-y-2 p-4">
        <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">{title}</p>
        {children}
      </CardContent>
    </Card>
  );
}

function SpentTile({ review }: { review: MonthReview }) {
  const over = review.over_by_paise > 0;
  const pct = ratioPercent(review.spent_paise, review.allowance_paise);
  return (
    <Tile title="Spent vs allowance">
      <p className="text-xl font-bold tabular-nums">{formatINR(review.spent_paise)}</p>
      <div
        role="meter"
        aria-label="Spent against allowance"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(pct)}
        className="h-2 w-full overflow-hidden rounded-full bg-muted"
      >
        <div className={cn("h-full rounded-full", over ? "bg-over" : "bg-primary")} style={{ width: `${pct}%` }} />
      </div>
      <p className={cn("text-xs", over ? "text-over" : "text-muted-foreground")}>
        {over ? `${formatINR(review.over_by_paise)} over` : `of ${formatINR(review.allowance_paise)} allowance`}
      </p>
    </Tile>
  );
}

function SavingsTile({ review }: { review: MonthReview }) {
  return (
    <Tile title="Savings rate">
      <p className="text-xl font-bold tabular-nums">
        {review.savings_rate_bps === null ? "—" : formatPercent(review.savings_rate_bps)}
      </p>
      <p className="text-xs text-muted-foreground">
        {review.income_paise > 0 ? `of ${formatINR(review.income_paise)} income` : "No income recorded"}
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
    <Tile title="Net worth change">
      <p className={cn("text-xl font-bold tabular-nums", tone)}>
        {change === null ? "—" : signedINR(change)}
      </p>
      <p className="text-xs text-muted-foreground">{change === null ? "Not enough history yet" : "over the month"}</p>
    </Tile>
  );
}

function DatesTile({ review }: { review: MonthReview }) {
  const d = review.dates;
  return (
    <Tile title="Dates">
      {d === null ? (
        <p className="text-sm text-muted-foreground">No date spending</p>
      ) : (
        <>
          <p className="text-xl font-bold tabular-nums">
            {formatINR(d.spent_paise)}
            {d.budget_paise !== null && (
              <span className="text-sm font-normal text-muted-foreground"> of {formatINR(d.budget_paise)}</span>
            )}
          </p>
          <p className="text-xs text-muted-foreground">Your share {formatINR(d.your_share_paise)}</p>
        </>
      )}
    </Tile>
  );
}

function TopCategoriesTile({ review }: { review: MonthReview }) {
  const top = review.top_categories.slice(0, 3);
  const max = top[0]?.amount_paise ?? 0;
  return (
    <Tile title="Top categories" className="col-span-2">
      {top.length === 0 ? (
        <p className="text-sm text-muted-foreground">Nothing spent this month</p>
      ) : (
        <ul className="space-y-2.5">
          {top.map((c) => (
            <li key={c.category} className="space-y-1 text-sm">
              <div className="flex items-baseline justify-between gap-2">
                <span className="truncate">{categoryLabel(c.category)}</span>
                <span className="shrink-0 font-semibold tabular-nums">{formatINR(c.amount_paise)}</span>
              </div>
              <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full rounded-full bg-primary"
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

export function MonthClosePanel() {
  // Fixed for the life of the page: the newest month the switcher can reach.
  const [last] = useState(() => lastCompleteMonth());
  const [month, setMonth] = useState(last);
  const prev = stepMonth(month, -1, last) ?? month;
  const next = stepMonth(month, 1, last);
  const { data: review, isLoading, isError, error } = useReviewMonth(month);

  return (
    <section aria-label="Month close" className="space-y-4 glass rounded-3xl p-4 md:p-6">
      <div className="flex items-center justify-between gap-2">
        <div>
          <h2 className="font-display text-[32px] leading-none">Month close</h2>
          <p className="text-sm text-muted-foreground">How the month went, in plain words.</p>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-9 w-9"
            aria-label="Previous month"
            onClick={() => setMonth(prev)}
            disabled={prev === month}
          >
            <ChevronLeft className="h-4 w-4" aria-hidden />
          </Button>
          <p className="min-w-[7.5rem] text-center text-sm font-semibold">{monthTitle(month)}</p>
          <Button
            variant="ghost"
            size="icon"
            className="h-9 w-9"
            aria-label="Next month"
            onClick={() => next && setMonth(next)}
            disabled={next === null}
          >
            <ChevronRight className="h-4 w-4" aria-hidden />
          </Button>
        </div>
      </div>

      {isLoading && <p className="text-sm text-muted-foreground">Loading the month…</p>}
      {isError && (
        <p role="alert" className="text-sm text-destructive">
          {error instanceof Error && error.message ? error.message : "Couldn't load this month."}
        </p>
      )}

      {review && (
        <>
          {review.sentences.length > 0 && (
            <ul className="space-y-2">
              {review.sentences.map((s, i) => (
                <li key={i} className="text-lg leading-relaxed sm:text-xl">
                  {s}
                </li>
              ))}
            </ul>
          )}
          <div className="grid grid-cols-2 gap-3">
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
