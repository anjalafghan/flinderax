import { Link } from "react-router-dom";

import type { Bill, BillSuggestion, SinkingFund } from "@/services/types";
import { useAddSuggestion, useBills, useBillSuggestions, useDismissSuggestion, useRestoreSuggestions } from "@/hooks/usePlanner";
import { ResetButton } from "@/components/ui/reset-button";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { defaultBucket } from "@/utils/buckets";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { coveredLine, daysBetween, groupBillsByWeek, localIsoDate, nextBills, progressPercent } from "@/utils/planner";

const KIND_LABEL: Record<Bill["kind"], string> = {
  monthly: "Monthly",
  yearly: "Yearly",
  card_bill: "Card bill",
  date: "Date",
};

function BillRow({ bill }: { bill: Bill }) {
  const covered = coveredLine(bill, formatINR);
  const [day, mon] = formatDay(bill.date).split(" ");
  return (
    <li className="ledger-row gap-5 md:gap-6">
      <span className="flex w-14 shrink-0 flex-col items-start">
        <span className="font-serif-num text-4xl leading-[0.9]">{day}</span>
        <span className="eyebrow">{mon}</span>
      </span>
      <div className="min-w-0 flex-1">
        <p className="truncate text-[15px] font-medium" title={bill.label}>{bill.label}</p>
        <p className="text-[13px] text-muted-foreground">
          {KIND_LABEL[bill.kind]}
          {covered && <span className="block text-primary">{covered}</span>}
        </p>
      </div>
      <p className="font-serif-num shrink-0 text-3xl">{formatINR(bill.amount_paise)}</p>
    </li>
  );
}

function SuggestionCard({ s }: { s: BillSuggestion }) {
  const add = useAddSuggestion();
  const dismiss = useDismissSuggestion();
  const every = s.every === "yearly" ? "every year" : "every month";
  const addIt = () =>
    add.mutate({
      label: s.label,
      amount_paise: s.amount_paise,
      day_of_month: s.day,
      account_id: null,
      to_account_id: null,
      category: "subscription",
      bucket: defaultBucket("subscription"),
      frequency: s.every,
      due_month: s.every === "yearly" ? s.month : null,
      active: true,
      start_month: null,
      end_month: null,
    });
  return (
    <li className="space-y-3 border-t border-border py-4 text-sm first:border-t-0 first:pt-1">
      <p className="leading-snug">
        Looks like a subscription: <span className="font-semibold">{s.merchant}</span>{" "}
        <span className="tabular-nums">{formatINR(s.amount_paise)}</span> {every}
      </p>
      {s.was_paise !== null && (
        <p className="text-xs text-amber-700 dark:text-amber-300">went up from {formatINR(s.was_paise)}</p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={addIt} disabled={add.isPending || dismiss.isPending}>
          Add
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={() => dismiss.mutate(s.merchant)}
          disabled={add.isPending || dismiss.isPending}
        >
          Not a subscription
        </Button>
      </div>
    </li>
  );
}

function SinkingFundRow({ f }: { f: SinkingFund }) {
  const pct = progressPercent(f.saved_paise, f.amount_paise);
  return (
    <li className="space-y-2 border-t border-border py-4 text-sm first:border-t-0 first:pt-1">
      <div className="flex items-baseline justify-between gap-4">
        <p className="min-w-0 truncate font-medium" title={`${f.label} ${formatINR(f.amount_paise)} due ${formatDay(f.due)}`}>
          {f.label} {formatINR(f.amount_paise)} due {formatDay(f.due)}
        </p>
        <p className="shrink-0 text-xs text-muted-foreground tabular-nums">{pct}%</p>
      </div>
      <div
        role="progressbar"
        aria-label={`${f.label} saved`}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
        className="h-2 w-full overflow-hidden rounded-full bg-muted"
      >
        <div className="h-full rounded-full bg-emerald-500" style={{ width: `${pct}%` }} />
      </div>
      <p className="text-xs text-muted-foreground">{formatINR(f.saved_paise)} set aside</p>
    </li>
  );
}

/** Plan page: every bill in the next 30 days, weekly, plus suggestions and sinking funds. */
export function BillsPanel() {
  const { data, isLoading, isError } = useBills(30);
  const { data: suggestions = [] } = useBillSuggestions();
  const restoreDismissed = useRestoreSuggestions();
  const today = localIsoDate();
  const weeks = data ? groupBillsByWeek(data.bills, today) : [];

  return (
    <Card id="bills" className="scroll-mt-24">
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">Bills</CardTitle>
        {data && (
          <p className="font-serif-num text-4xl">
            {formatINR(data.total_paise)} <span className="text-sm font-medium text-muted-foreground">due in the next 30 days</span>
          </p>
        )}
      </CardHeader>
      <CardContent className="space-y-8">
        {isLoading && <p className="text-sm text-muted-foreground">Loading bills...</p>}
        {isError && <p className="text-sm text-destructive">Could not load bills. Try again in a moment.</p>}
        {data && weeks.length === 0 && <p className="empty-note">Nothing due in the next 30 days.</p>}
        {weeks.map((w) => (
          <section key={w.key} aria-label={w.label} className="space-y-2">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{w.label}</h3>
            <ul className="ledger">{w.bills.map((b) => <BillRow key={`${b.date}-${b.label}-${b.kind}`} bill={b} />)}</ul>
          </section>
        ))}

        {suggestions.length > 0 && (
          <section aria-label="Subscription suggestions" className="space-y-2">
            <h3 className="text-sm font-semibold">Possible subscriptions</h3>
            <ul className="ledger">
              {suggestions.map((s) => <SuggestionCard key={s.merchant} s={s} />)}
            </ul>
          </section>
        )}
        <div>
          <ResetButton
            label="Bring back dismissed suggestions"
            disabled={restoreDismissed.isPending}
            onReset={() => restoreDismissed.mutate()}
          />
        </div>

        {data && data.sinking_funds.length > 0 && (
          <section aria-label="Saving up for" className="space-y-2">
            <h3 className="text-sm font-semibold">Saving up for</h3>
            <ul className="ledger">
              {data.sinking_funds.map((f) => <SinkingFundRow key={`${f.label}-${f.due}`} f={f} />)}
            </ul>
          </section>
        )}
      </CardContent>
    </Card>
  );
}

/** Today page: the next three bills due within a week. Renders nothing when there are none. */
/** "in 2 days" / "today" / "tomorrow". */
function inDays(n: number): string {
  return n <= 0 ? "today" : n === 1 ? "tomorrow" : `in ${n} days`;
}

/** Home-screen ledger: the next week's bills, one hairline row each, then the total. */
export function DueSoonCard() {
  const { data } = useBills(30);
  const today = localIsoDate();
  const soon = data ? nextBills(data.bills, today, 7, 5) : [];
  if (soon.length === 0) return null;
  const total = soon.reduce((a, b) => a + b.amount_paise, 0);
  return (
    <section aria-labelledby="due-title" className="flex w-full flex-col">
      <div className="flex items-baseline justify-between gap-4 pb-3.5">
        <h2 id="due-title" className="m-0 font-display text-[32px] leading-none">
          Due soon
        </h2>
        <Link to="/plan#bills" className="inline-flex min-h-11 items-center text-sm font-medium text-foreground">
          All bills
        </Link>
      </div>
      <ul className="m-0 list-none p-0">
        {soon.map((b) => {
          const covered = coveredLine(b, formatINR);
          return (
            <li key={`${b.date}-${b.label}-${b.kind}`} className="flex items-center gap-5 border-t border-border py-[18px]">
              <span className="w-16 shrink-0 text-sm text-muted-foreground">{formatDay(b.date)}</span>
              <span className="flex min-w-0 flex-auto flex-col gap-0.5">
                <span className="truncate text-base font-medium">{b.label}</span>
                <span className="text-[13px] text-muted-foreground">
                  {KIND_LABEL[b.kind]} · {inDays(daysBetween(today, b.date))}
                  {covered && <> · {covered}</>}
                </span>
              </span>
              <span className="shrink-0 font-display text-2xl">{formatINR(b.amount_paise)}</span>
            </li>
          );
        })}
      </ul>
      <div className="flex items-center justify-between gap-5 border-t border-foreground pt-[18px]">
        <span className="text-sm text-muted-foreground">Total this week</span>
        <span className="font-display text-2xl">{formatINR(total)}</span>
      </div>
    </section>
  );
}
