import { useEffect, type ReactNode } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { toast } from "sonner";
import {
  Check,
  Eye,
  FileText,
  Landmark,
  PartyPopper,
  Sparkles,
  Tags,
  Telescope,
  type LucideIcon,
} from "lucide-react";

import { Header } from "@/components/layout/Header";
import { StatementImport } from "@/components/feature/StatementImport";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import {
  checkinHistoryKey,
  checkinKey,
  checkinReportKey,
  isSaved,
  rate,
  resetCalibration,
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
import { DietReportSection } from "@/components/feature/health/DietReportSection";
import { PeopleReportSection } from "@/components/feature/people/PeopleReportSection";
import { Retry } from "@/components/ui/retry";
import Magnet from "@/components/bits/Magnet";
import { PercentOdometer, RupeeOdometer } from "@/components/feature/Odometer";
import { celebrate } from "@/utils/motion";
import { cn } from "@/utils/cn";

const SECTION =
  "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";

// Each step of the check-in has its own colour and picture, so the four read as a path.
const STEP_LOOK: Record<
  Step["key"],
  { icon: LucideIcon; card: string; bubble: string; bar: string }
> = {
  statements: {
    icon: FileText,
    card: "border-sky/30 bg-gradient-to-br from-sky/10 via-card to-card",
    bubble: "bg-sky/18 text-sky",
    bar: "bg-sky",
  },
  balances: {
    icon: Landmark,
    card: "border-mint/30 bg-gradient-to-br from-mint/10 via-card to-card",
    bubble: "bg-mint/18 text-mint",
    bar: "bg-mint",
  },
  payees: {
    icon: Tags,
    card: "border-sun/35 bg-gradient-to-br from-sun/12 via-card to-card",
    bubble: "bg-sun/22 text-[hsl(var(--foreground))] dark:text-sun",
    bar: "bg-sun",
  },
  body: {
    icon: Sparkles,
    card: "border-rose/25 bg-gradient-to-br from-rose/8 via-card to-card",
    bubble: "bg-rose/18 text-rose",
    bar: "bg-rose",
  },
  report: {
    icon: Telescope,
    card: "border-grape/30 bg-gradient-to-br from-grape/10 via-card to-card",
    bubble: "bg-grape/18 text-grape",
    bar: "bg-grape",
  },
};

const sectionFor = (key: Step["key"]) => cn(SECTION, STEP_LOOK[key].card);
const PILL =
  "inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold";
const OUTLINE_LINK =
  "inline-flex min-h-11 items-center rounded-full border border-border px-5 text-sm font-medium text-foreground no-underline transition-colors hover:bg-chip";
const TOP_CATEGORIES_NOTE =
  "Spending by category, with the change from the quarter before.";

const DOTS = [
  "bg-coral",
  "bg-sun",
  "bg-mint",
  "bg-sky",
  "bg-grape",
  "bg-rose",
  "bg-teal",
  "bg-berry",
];

/** Confetti needs a 2D canvas; skip it where there is none (tests, old browsers). */
function cheer(kind: "burst" | "shower") {
  try {
    if (!document.createElement("canvas").getContext("2d")) return;
  } catch {
    return;
  }
  void celebrate(kind);
}

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
  const look = STEP_LOOK[step.key];
  return (
    <header className="flex items-start gap-4">
      <span
        aria-hidden
        className={cn(
          "relative flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-[15px] font-semibold",
          step.done ? "bg-mint text-background" : look.bubble,
        )}
      >
        {step.done ? (
          <Check className="h-5 w-5" strokeWidth={2.4} />
        ) : (
          <look.icon className="h-5 w-5" strokeWidth={1.8} />
        )}
        <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full border border-border bg-card text-[11px] text-foreground">
          {n}
        </span>
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

function TotalsRow({ now, before }: { now: Totals; before: Totals | null }) {
  // Income mint, spending coral, investing sky, what you kept grape.
  const cells: {
    label: string;
    value: ReactNode;
    was: string | null;
    tint: string;
    ink: string;
  }[] = [
    {
      label: "Income",
      value: <RupeeOdometer paise={Math.round(now.income_paise / 100) * 100} />,
      was: before ? rupees(before.income_paise) : null,
      tint: "border-mint/30 bg-mint/10",
      ink: "text-good",
    },
    {
      label: "Spent",
      value: <RupeeOdometer paise={Math.round(now.spend_paise / 100) * 100} />,
      was: before ? rupees(before.spend_paise) : null,
      tint: "border-coral/30 bg-coral/10",
      ink: "text-coral",
    },
    {
      label: "Invested",
      value: (
        <RupeeOdometer paise={Math.round(now.invested_paise / 100) * 100} />
      ),
      was: before ? rupees(before.invested_paise) : null,
      tint: "border-sky/30 bg-sky/10",
      ink: "text-sky",
    },
    {
      label: "Kept",
      value:
        now.savings_rate_bps === null ? (
          rate(null)
        ) : (
          <PercentOdometer percent={now.savings_rate_bps / 100} />
        ),
      was:
        before && before.savings_rate_bps !== null
          ? rate(before.savings_rate_bps)
          : null,
      tint: "border-grape/30 bg-grape/10",
      ink: "text-grape",
    },
  ];
  return (
    <dl className="m-0 grid grid-cols-2 gap-3 sm:grid-cols-4">
      {cells.map((c) => (
        <div
          key={c.label}
          className={cn("flex flex-col gap-1 rounded-[18px] border p-4", c.tint)}
        >
          <dt className="text-[13px] text-muted-foreground">{c.label}</dt>
          <dd className={cn("m-0 font-display text-[26px] leading-none", c.ink)}>
            {c.value}
          </dd>
          {c.was !== null && (
            <dd className="m-0 text-[13px] text-muted-foreground">
              Before: {c.was}
            </dd>
          )}
        </div>
      ))}
    </dl>
  );
}

function StatementsStep({ status, n }: { status: CheckinStatus; n: number }) {
  const step = status.steps.find((s) => s.key === "statements");
  if (!step) return null;
  return (
    <section
      aria-labelledby="checkin-step-statements"
      className={sectionFor("statements")}
    >
      <StepHead n={n} step={step} id="checkin-step-statements" />
      {status.uploads.length > 0 ? (
        <ul className="ledger">
          {status.uploads.map((u) => (
            <li key={`${u.kind}:${u.id}`} className="ledger-row">
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">{u.name}</span>
                <span className="text-[13px] text-muted-foreground">
                  {u.covered_to
                    ? `covered to ${formatDay(u.covered_to)}`
                    : "no statement yet"}
                </span>
              </span>
              {u.needed && (
                <span className={`${PILL} tone-warn`}>Upload a newer one</span>
              )}
            </li>
          ))}
        </ul>
      ) : (
        <p className="empty-note m-0">
          You have no accounts or cards yet. <Link to="/accounts">Add one</Link>{" "}
          to start.
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
    <section
      aria-labelledby="checkin-step-balances"
      className={sectionFor("balances")}
    >
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
            These are estimates from your usual flow, not bank figures. They
            stay estimates until you set the balance.
          </p>
          <ul className="ledger">
            {status.estimates.map((e) => (
              <li key={e.account_id} className="ledger-row items-start">
                <span className="flex min-w-0 flex-col gap-1">
                  <span className="text-[15px] font-medium">{e.name}</span>
                  <span className="text-[13px] text-muted-foreground">
                    {e.basis}
                  </span>
                  <span className="text-[13px] text-muted-foreground">
                    Last set to {rupees(e.known_paise)} on{" "}
                    {formatDay(e.known_on)}
                  </span>
                </span>
                <span className="flex shrink-0 flex-col items-end gap-1 text-right">
                  <span className="font-display text-[26px] leading-none">
                    {rupees(e.estimate_paise)}
                  </span>
                  <span className="text-[13px] text-muted-foreground">
                    estimate
                  </span>
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
    <section
      aria-labelledby="checkin-step-payees"
      className={sectionFor("payees")}
    >
      <StepHead n={n} step={step} id="checkin-step-payees" />
      <div>
        <Link to="/spending/sort" className={OUTLINE_LINK}>
          Sort payees
        </Link>
      </div>
    </section>
  );
}

/** Optional: this quarter's weight, added on the Health page. */
function BodyStep({ status }: { status: CheckinStatus }) {
  const step = status.steps.find((s) => s.key === "body");
  if (!step) return null;
  return (
    <section
      aria-labelledby="checkin-step-body"
      className={sectionFor("body")}
    >
      <div className="flex flex-col gap-1">
        <h2 id="checkin-step-body" className="m-0 text-[17px] font-medium">
          {step.title}{" "}
          <span className="text-sm font-normal text-muted-foreground">
            (optional)
          </span>
        </h2>
        <p className="m-0 text-sm text-muted-foreground">{step.detail}</p>
      </div>
      {!step.done && (
        <Link
          to="/health"
          className="self-start text-sm font-medium text-primary"
        >
          Add your weight
        </Link>
      )}
    </section>
  );
}

function QuarterStep({ status, n }: { status: CheckinStatus; n: number }) {
  const qc = useQueryClient();
  const step = status.steps.find((s) => s.key === "report");
  const report = useCheckinReport(true);
  if (!step) return null;
  const data = report.data;

  let body: ReactNode;
  if (report.isLoading) {
    body = (
      <div
        className="h-48 animate-pulse rounded-[18px] bg-muted/40"
        aria-label="Loading your quarter"
      />
    );
  } else if (report.isError && !data) {
    body = (
      <Retry
        message="Couldn't load your quarter."
        onRetry={() => void report.refetch()}
      />
    );
  } else if (data) {
    // Unsorted spend is step 3's job, so it stays out of the category list.
    const sorted = data.categories.filter(
      (c) => c.category !== "uncategorized",
    );
    body = (
      <div className="flex flex-col gap-8">
        <div className="flex flex-col gap-1">
          <h3 className="m-0 text-[15px] font-medium">This quarter</h3>
          <p className="m-0 text-sm text-muted-foreground">
            {`${formatDay(data.period.start)} to ${formatDay(data.period.end)}`}
            {` · compared with ${formatDay(data.previous.start)} to ${formatDay(data.previous.end)}`}
          </p>
          {!data.prev && (
            <p className="m-0 text-sm text-muted-foreground">
              There isn’t enough information from the previous period for a
              financial comparison yet.
            </p>
          )}
        </div>
        <TotalsRow now={data.totals} before={data.prev} />

        {sorted.length > 0 && (
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">Where it went</h3>
            <p className="m-0 text-[13px] text-muted-foreground">
              {TOP_CATEGORIES_NOTE}
            </p>
            <ul className="ledger">
              {sorted.map((c, i) => (
                <li key={c.category} className="ledger-row">
                  <span className="flex items-center gap-2.5 text-[15px] font-medium">
                    <span
                      aria-hidden
                      className={cn(
                        "h-2.5 w-2.5 shrink-0 rounded-full",
                        DOTS[i % DOTS.length],
                      )}
                    />
                    {cap(c.category)}
                  </span>
                  <span className="flex flex-col items-end gap-1 text-right">
                    <span className="font-display text-[20px] leading-none">
                      {rupees(c.paise)}
                    </span>
                    <span
                      className={cn(
                        "text-[13px]",
                        c.change_bps !== null && c.change_bps > 0
                          ? "text-over"
                          : c.change_bps !== null && c.change_bps < 0
                            ? "text-good"
                            : "text-muted-foreground",
                      )}
                    >
                      {signedChange(c.change_bps)}
                    </span>
                  </span>
                </li>
              ))}
            </ul>
          </div>
        )}

        <div className="grid gap-8 md:grid-cols-2">
          <div className="flex flex-col gap-3 rounded-[22px] border border-mint/30 bg-mint/8 p-5">
            <h3 className="m-0 flex items-center gap-2 text-[15px] font-semibold text-good">
              <PartyPopper className="h-4 w-4" aria-hidden />
              Wins
            </h3>
            {data.wins.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
                {data.wins.map((w) => (
                  <li key={w} className="flex gap-2">
                    <Check
                      className="mt-1 h-4 w-4 shrink-0 text-mint"
                      strokeWidth={2.4}
                      aria-hidden
                    />
                    <span>{w}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">
                Nothing standing out yet.
              </p>
            )}
          </div>
          <div className="flex flex-col gap-3 rounded-[22px] border border-coral/30 bg-coral/8 p-5">
            <h3 className="m-0 flex items-center gap-2 text-[15px] font-semibold text-over">
              <Eye className="h-4 w-4" aria-hidden />
              Watch
            </h3>
            {data.watch.length > 0 ? (
              <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
                {data.watch.map((w) => (
                  <li key={w} className="flex gap-2">
                    <span
                      aria-hidden
                      className="mt-2 h-2 w-2 shrink-0 rounded-full bg-coral"
                    />
                    <span>{w}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="m-0 text-sm text-muted-foreground">
                Nothing to flag this time.
              </p>
            )}
          </div>
        </div>

        {data.diet && <DietReportSection report={data.diet} />}
        {data.people && <PeopleReportSection report={data.people} />}

        {data.lifeline && (
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">
              Life line since your last check-in
            </h3>
            <p className="m-0 text-[15px]">{data.lifeline.line}</p>
            <Link
              to="/life"
              className="self-start text-sm font-medium text-primary"
            >
              Open the life line
            </Link>
          </div>
        )}

        {data.calibration && data.calibration.lines.length > 0 && (
          <div className="flex flex-col gap-3">
            <h3 className="m-0 text-[15px] font-medium">
              How the last forecast did
            </h3>
            <p className="m-0 text-sm text-muted-foreground">
              Since your check-in on {formatDay(data.calibration.since)}, about{" "}
              {data.calibration.months} months ago.
            </p>
            <ul className="m-0 flex list-none flex-col gap-2 p-0 text-[15px]">
              {data.calibration.lines.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
            <div>
              <ResetButton
                label="Reset: forget how the forecast did"
                confirm="Forget how the last forecast did? Your forecast stops being adjusted by it. Your check-ins stay."
                onReset={() =>
                  void resetCalibration().then(() =>
                    qc.invalidateQueries({ queryKey: checkinKey }),
                  )
                }
              />
            </div>
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
                    <Link
                      to={h.action.to}
                      className="text-sm font-medium text-primary"
                    >
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
    <section
      aria-labelledby="checkin-step-report"
      className={sectionFor("report")}
    >
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
      <h2
        id="checkin-history-title"
        className="brand-text m-0 font-display text-[32px] leading-none"
      >
        Past check-ins
      </h2>
      {rows.length > 0 ? (
        <ul className="ledger">
          {rows.map((c) => (
            <li key={c.done_on} className="ledger-row items-start">
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">
                  Saved {formatDay(c.done_on)}
                </span>
                <span className="text-[13px] text-muted-foreground">
                  Covers {formatDay(c.period.start)} to{" "}
                  {formatDay(c.period.end)}
                </span>
              </span>
              <span className="shrink-0 text-right text-[13px]">
                <span className="text-good">
                  In {rupees(c.totals.income_paise)}
                </span>
                <br />
                <span className="text-coral">
                  Spent {rupees(c.totals.spend_paise)}
                </span>
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="empty-note m-0">
          Your finished check-ins will show here.
        </p>
      )}
    </section>
  );
}

/** The four steps as a coloured path: a filled segment for each one done. */
function Progress({ status }: { status: CheckinStatus }) {
  const steps = status.steps.filter((st) => !st.optional && st.key !== "body");
  const done = steps.filter((st) => st.done).length;
  return (
    <div className="flex flex-col gap-2">
      <p className="m-0 text-[13px] font-medium text-brand-ink">
        {done} of {steps.length} steps done
      </p>
      <ol aria-hidden className="m-0 flex list-none gap-1.5 p-0">
        {steps.map((st) => (
          <li
            key={st.key}
            className={cn(
              "h-2 flex-1 rounded-full transition-colors duration-500",
              st.done ? STEP_LOOK[st.key].bar : "bg-muted",
            )}
          />
        ))}
      </ol>
    </div>
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
        cheer("shower");
        toast.success(
          `Check-in saved. Next one ${formatDay(next.next_due_on)}.`,
        );
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
      <main className="page page-narrow stack">
        <div>
          <h1 className="page-title">Quarterly check-in</h1>
          {sub && <p className="page-sub">{sub}</p>}
        </div>

        {status.isLoading && (
          <div
            className="h-64 animate-pulse rounded-[28px] bg-gradient-to-br from-sky/15 via-grape/10 to-mint/15"
            aria-label="Loading your check-in"
          />
        )}

        {status.isError && !s && (
          <Retry
            message="Couldn't load your check-in."
            onRetry={() => void status.refetch()}
          />
        )}

        {s && (
          <>
            {saved && (
              <p className="m-0 flex items-center gap-3 rounded-[18px] border border-mint/30 px-5 py-4 text-sm tone-good">
                <PartyPopper className="h-5 w-5 shrink-0" aria-hidden />
                <span>
                  This check-in is saved. Next one {formatDay(s.next_due_on)}.
                </span>
              </p>
            )}

            <Progress status={s} />
            <StatementsStep status={s} n={1} />
            <BalancesStep status={s} n={2} />
            <PayeesStep status={s} n={3} />
            <BodyStep status={s} />
            <QuarterStep status={s} n={4} />

            {!saved && (
              <footer className="flex flex-col gap-3 border-t border-border pt-6 sm:flex-row sm:items-center sm:justify-between">
                <p className="m-0 text-sm text-muted-foreground">
                  Finishing saves this quarter&apos;s report and sets the next
                  check-in.
                </p>
                <Magnet
                  padding={40}
                  magnetStrength={4}
                  disabled={finish.isPending}
                  wrapperClassName="self-start sm:self-auto"
                >
                  <Button
                    onClick={onFinish}
                    disabled={finish.isPending}
                    className="h-12 px-7 text-base shadow-[0_12px_30px_-12px_hsl(var(--brand)/0.8)]"
                  >
                    <PartyPopper className="mr-2 h-4 w-4" aria-hidden />
                    {finish.isPending ? "Saving..." : "Finish check-in"}
                  </Button>
                </Magnet>
              </footer>
            )}
          </>
        )}

        <History />
      </main>
    </div>
  );
}
