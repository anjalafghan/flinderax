import { useEffect, type ReactNode } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { toast } from "sonner";

import { Header } from "@/components/layout/Header";
import { StatementImport } from "@/components/feature/StatementImport";
import { Button } from "@/components/ui/button";
import {
  checkinHistoryKey,
  checkinKey,
  checkinReportKey,
  isSaved,
  rate,
  rupees,
  signedChange,
  useCheckinHistory,
  useCheckinReport,
  useCheckinStatus,
  useCompleteCheckin,
  type CheckinStatus,
  type Step,
  type Totals,
} from "@/services/checkin";
import { formatDay } from "@/utils/dates";

const SECTION = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
const PILL = "inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold";
const OUTLINE_LINK =
  "inline-flex min-h-11 items-center rounded-full border border-border px-5 text-sm font-medium text-foreground no-underline transition-colors hover:bg-chip";
const TOP_CATEGORIES_NOTE = "Spending by category, with the change from the quarter before.";

const cap = (s: string) => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);

/** A numbered step's heading, with its one-line detail and a done or to-do pill. */
function StepHead({ n, step, id }: { n: number; step: Step; id: string }) {
  const pill = step.done ? (
    <span className={`${PILL} tone-good`}>Done</span>
  ) : step.count > 0 ? (
    <span className={`${PILL} tone-warn`}>{step.count} to do</span>
  ) : (
    <span className={`${PILL} tone-quiet`}>To do</span>
  );
  return (
    <header className="flex items-start gap-4">
      <span
        aria-hidden
        className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-chip text-[15px] font-medium"
      >
        {n}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <h2 id={id} className="m-0 font-display text-[32px] leading-none">
            {step.title}
          </h2>
          {pill}
        </div>
        <p className="m-0 text-sm text-muted-foreground">{step.detail}</p>
      </div>
    </header>
  );
}

function Retry({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="rounded-[28px] border border-dashed border-border px-6 py-8 text-center text-sm text-muted-foreground">
      {message}{" "}
      <Button variant="link" className="h-auto p-0" onClick={onRetry}>
        Try again
      </Button>
    </div>
  );
}

function TotalsRow({ now, before }: { now: Totals; before: Totals | null }) {
  const cells: { label: string; value: string; was: string | null }[] = [
    { label: "Income", value: rupees(now.income_paise), was: before ? rupees(before.income_paise) : null },
    { label: "Spent", value: rupees(now.spend_paise), was: before ? rupees(before.spend_paise) : null },
    { label: "Invested", value: rupees(now.invested_paise), was: before ? rupees(before.invested_paise) : null },
    {
      label: "Kept",
      value: rate(now.savings_rate_bps),
      was: before && before.savings_rate_bps !== null ? rate(before.savings_rate_bps) : null,
    },
  ];
  return (
    <dl className="m-0 grid grid-cols-2 gap-x-6 gap-y-5 sm:grid-cols-4">
      {cells.map((c) => (
        <div key={c.label} className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">{c.label}</dt>
          <dd className="m-0 font-display text-[26px] leading-none">{c.value}</dd>
          {c.was !== null && <dd className="m-0 text-[13px] text-muted-foreground">Before: {c.was}</dd>}
        </div>
      ))}
    </dl>
  );
}

function StatementsStep({ status, n }: { status: CheckinStatus; n: number }) {
  const step = status.steps.find((s) => s.key === "statements");
  if (!step) return null;
  return (
    <section aria-labelledby="checkin-step-statements" className={SECTION}>
      <StepHead n={n} step={step} id="checkin-step-statements" />
      {status.uploads.length > 0 ? (
        <ul className="ledger">
          {status.uploads.map((u) => (
            <li key={`${u.kind}:${u.id}`} className="ledger-row">
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">{u.name}</span>
                <span className="text-[13px] text-muted-foreground">
                  {u.covered_to ? `covered to ${formatDay(u.covered_to)}` : "no statement yet"}
                </span>
              </span>
              {u.needed && <span className={`${PILL} tone-warn`}>Upload a newer one</span>}
            </li>
          ))}
        </ul>
      ) : (
        <p className="empty-note m-0">
          You have no accounts or cards yet. <Link to="/accounts">Add one</Link> to start.
        </p>
      )}
      <StatementImport />
    </section>
  );
}

function BalancesStep({ status, n }: { status: CheckinStatus; n: number }) {
  const step = status.steps.find((s) => s.key === "balances");
  if (!step) return null;
  return (
    <section aria-labelledby="checkin-step-balances" className={SECTION}>
      <StepHead n={n} step={step} id="checkin-step-balances" />
      <div>
        <Link to="/accounts" className={OUTLINE_LINK}>
          Update balances
        </Link>
      </div>
      {status.estimates.length > 0 && (
        <div className="flex flex-col gap-3">
          <h3 className="m-0 text-[15px] font-medium">Estimated balances</h3>
          <p className="m-0 text-[13px] text-muted-foreground">
            These are estimates from your usual flow, not bank figures. They stay estimates until you set the balance.
          </p>
          <ul className="ledger">
            {status.estimates.map((e) => (
              <li key={e.account_id} className="ledger-row items-start">
                <span className="flex min-w-0 flex-col gap-1">
                  <span className="text-[15px] font-medium">{e.name}</span>
                  <span className="text-[13px] text-muted-foreground">{e.basis}</span>
                  <span className="text-[13px] text-muted-foreground">
                    Last set to {rupees(e.known_paise)} on {formatDay(e.known_on)}
                  </span>
                </span>
                <span className="flex shrink-0 flex-col items-end gap-1 text-right">
                  <span className="font-display text-[26px] leading-none">{rupees(e.estimate_paise)}</span>
                  <span className="text-[13px] text-muted-foreground">estimate</span>
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}

function PayeesStep({ status, n }: { status: CheckinStatus; n: number }) {
  const step = status.steps.find((s) => s.key === "payees");
  if (!step) return null;
  return (
    <section aria-labelledby="checkin-step-payees" className={SECTION}>
      <StepHead n={n} step={step} id="checkin-step-payees" />
      <div>
        <Link to="/spending/sort" className={OUTLINE_LINK}>
          Sort payees
        </Link>
      </div>
    </section>
  );
}

function QuarterStep({ status, n }: { status: CheckinStatus; n: number }) {
  const step = status.steps.find((s) => s.key === "report");
  const report = useCheckinReport(true);
  if (!step) return null;
  const data = report.data;

  let body: ReactNode;
  if (report.isLoading) {
    body = <div className="h-48 animate-pulse rounded-[18px] bg-muted/40" aria-label="Loading your quarter" />;
  } else if (report.isError && !data) {
    body = <Retry message="Couldn't load your quarter." onRetry={() => void report.refetch()} />;
  } else if (data) {
    // Unsorted spend is step 3's job, so it stays out of the category list.
    const sorted = data.categories.filter((c) => c.category !== "uncategorized");
    body = (
      <div className="flex flex-col gap-8">
        <TotalsRow now={data.totals} before={data.prev} />

        {sorted.length > 0 && (
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">Where it went</h3>
            <p className="m-0 text-[13px] text-muted-foreground">{TOP_CATEGORIES_NOTE}</p>
            <ul className="ledger">
              {sorted.map((c) => (
                <li key={c.category} className="ledger-row">
                  <span className="text-[15px] font-medium">{cap(c.category)}</span>
                  <span className="flex flex-col items-end gap-1 text-right">
                    <span className="font-display text-[20px] leading-none">{rupees(c.paise)}</span>
                    <span className="text-[13px] text-muted-foreground">{signedChange(c.change_bps)}</span>
                  </span>
                </li>
              ))}
            </ul>
          </div>
        )}

        <div className="grid gap-8 md:grid-cols-2">
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">Wins</h3>
            {data.wins.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
                {data.wins.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">Nothing standing out yet.</p>
            )}
          </div>
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">Watch</h3>
            {data.watch.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
                {data.watch.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">Nothing to flag this time.</p>
            )}
          </div>
        </div>

        {data.calibration && data.calibration.lines.length > 0 && (
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">How the last forecast did</h3>
            <p className="m-0 text-sm text-muted-foreground">
              Since your check-in on {formatDay(data.calibration.since)}, about {data.calibration.months} months ago.
            </p>
            <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
              {data.calibration.lines.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
          </div>
        )}

        {data.habits.length > 0 && (
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">Next up</h3>
            <ul className="ledger">
              {data.habits.slice(0, 3).map((h) => (
                <li key={h.key} className="ledger-row">
                  <span className="text-[15px] font-medium">{h.title}</span>
                  {h.action && (
                    <Link to={h.action.to} className="text-sm font-medium text-primary">
                      {h.action.label}
                    </Link>
                  )}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>
    );
  }

  return (
    <section aria-labelledby="checkin-step-report" className={SECTION}>
      <StepHead n={n} step={step} id="checkin-step-report" />
      {body}
    </section>
  );
}

/** Past check-ins, newest first, from the server's saved summaries. */
function History() {
  const history = useCheckinHistory();
  const rows = history.data?.checkins ?? [];
  return (
    <section aria-labelledby="checkin-history-title" className={SECTION}>
      <h2 id="checkin-history-title" className="m-0 font-display text-[32px] leading-none">
        Past check-ins
      </h2>
      {rows.length > 0 ? (
        <ul className="ledger">
          {rows.map((c) => (
            <li key={c.done_on} className="ledger-row items-start">
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">Saved {formatDay(c.done_on)}</span>
                <span className="text-[13px] text-muted-foreground">
                  Covers {formatDay(c.period.start)} to {formatDay(c.period.end)}
                </span>
              </span>
              <span className="shrink-0 text-right text-[13px] text-muted-foreground">
                In {rupees(c.totals.income_paise)}
                <br />
                Spent {rupees(c.totals.spend_paise)}
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="empty-note m-0">Your finished check-ins will show here.</p>
      )}
    </section>
  );
}

/** The quarterly check-in: upload, confirm, sort, read, finish. */
export default function CheckInPage() {
  const qc = useQueryClient();
  const status = useCheckinStatus();
  const finish = useCompleteCheckin();
  const s = status.data;
  const saved = s ? isSaved(s) : false;

  // StatementImport has no callback, so any successful write (an upload included) refreshes this check-in.
  useEffect(
    () =>
      qc.getMutationCache().subscribe((event) => {
        if (event.type === "updated" && event.action.type === "success") {
          void qc.invalidateQueries({ queryKey: checkinKey });
        }
      }),
    [qc],
  );

  const onFinish = () => {
    finish.mutate(undefined, {
      onSuccess: (next) => {
        qc.setQueryData(checkinKey, next);
        void qc.invalidateQueries({ queryKey: checkinHistoryKey });
        void qc.invalidateQueries({ queryKey: checkinReportKey });
        toast.success(`Check-in saved. Next one ${formatDay(next.next_due_on)}.`);
      },
      onError: () => toast.error("Couldn't save the check-in. Try again."),
    });
  };

  const sub = !s
    ? ""
    : saved
      ? `Saved on ${formatDay(s.last_done_on ?? s.period.end)}. Covers ${formatDay(s.period.start)} to ${formatDay(s.period.end)}.`
      : `Covers ${formatDay(s.period.start)} to ${formatDay(s.period.end)}.`;

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl stack">
        <div>
          <h1 className="page-title">Quarterly check-in</h1>
          {sub && <p className="page-sub">{sub}</p>}
        </div>

        {status.isLoading && <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your check-in" />}

        {status.isError && !s && <Retry message="Couldn't load your check-in." onRetry={() => void status.refetch()} />}

        {s && (
          <>
            {saved && (
              <p className="m-0 rounded-[18px] px-5 py-4 text-sm tone-good">
                This check-in is saved. Next one {formatDay(s.next_due_on)}.
              </p>
            )}

            <StatementsStep status={s} n={1} />
            <BalancesStep status={s} n={2} />
            <PayeesStep status={s} n={3} />
            <QuarterStep status={s} n={4} />

            {!saved && (
              <footer className="flex flex-col gap-3 border-t border-border pt-6 sm:flex-row sm:items-center sm:justify-between">
                <p className="m-0 text-sm text-muted-foreground">
                  Finishing saves this quarter&apos;s report and sets the next check-in.
                </p>
                <Button onClick={onFinish} disabled={finish.isPending} className="self-start sm:self-auto">
                  {finish.isPending ? "Saving..." : "Finish check-in"}
                </Button>
              </footer>
            )}
          </>
        )}

        <History />
      </main>
    </div>
  );
}
