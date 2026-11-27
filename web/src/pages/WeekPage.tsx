import { Link } from "react-router-dom";
import { Briefcase, ChevronRight, Signpost, Wallet } from "lucide-react";

import { Header } from "@/components/layout/Header";
import { WeekCard } from "@/components/feature/health/WeekCard";
import { CatchUpSection } from "@/components/feature/people/CatchUpSection";
import { useSummary } from "@/hooks/usePlanner";
import { useDiet } from "@/services/diet";
import { useDecisions } from "@/services/decisions";
import { usePeople } from "@/services/people";
import { daysUntil, stepsThisWeek, useWork } from "@/services/work";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { RupeeOdometer } from "@/components/feature/Odometer";
import { cn } from "@/utils/cn";

const PANEL =
  "flex flex-col gap-6 rounded-[28px] border px-6 py-8 sm:px-12 sm:py-10";
// Each panel wears its own colour: the job search grape, decisions sun.
const GRAPE_PANEL =
  "border-grape/30 bg-gradient-to-br from-grape/12 via-card to-card shadow-[0_16px_40px_-28px_hsl(var(--c-grape)/0.7)]";
const SUN_PANEL =
  "border-sun/40 bg-gradient-to-br from-sun/14 via-card to-card shadow-[0_16px_40px_-28px_hsl(var(--c-sun)/0.8)]";
const TITLE =
  "m-0 flex items-center gap-3 font-display text-[32px] leading-none";
const BUBBLE =
  "flex h-10 w-10 shrink-0 items-center justify-center rounded-full";
const NOTE = "m-0 text-sm text-muted-foreground";
const TEXT_LINK =
  "inline-flex min-h-10 items-center self-start text-sm font-medium text-primary";
function WorkSteps() {
  const work = useWork();
  if (!work.data) return null;
  const open = work.data.leads.filter((l) => l.stage !== "closed");
  // No search going on: nothing to show, and no nagging.
  if (open.length === 0) return null;
  const steps = stepsThisWeek(open);
  return (
    <section
      aria-labelledby="week-work-title"
      className={cn(PANEL, GRAPE_PANEL)}
    >
      <header className="flex flex-col gap-2">
        <h2 id="week-work-title" className={TITLE}>
          <span aria-hidden className={cn(BUBBLE, "bg-grape/18 text-grape")}>
            <Briefcase className="h-5 w-5" strokeWidth={1.8} />
          </span>
          Job search
        </h2>
        <p
          className={NOTE}
        >{`${open.length} open ${open.length === 1 ? "lead" : "leads"}. One step a week keeps it moving.`}</p>
      </header>
      {steps.length > 0 ? (
        <ul className="ledger">
          {steps.map((l) => {
            const late = l.next_on !== null && daysUntil(l.next_on) < 0;
            return (
              <li key={l.id} className="ledger-row">
                <span className="flex min-w-0 flex-col gap-1">
                  <span className="text-[15px] font-medium">
                    {l.next_step || "Next step"}
                  </span>
                  <span className="text-[13px] text-muted-foreground">
                    {l.company}
                  </span>
                </span>
                <span
                  className={`shrink-0 rounded-full px-3 py-1 text-[13px] ${late ? "tone-over" : "tone-warn"}`}
                >
                  {l.next_on
                    ? `${late ? "Was due" : "By"} ${formatDay(l.next_on)}`
                    : ""}
                </span>
              </li>
            );
          })}
        </ul>
      ) : (
        <p className={NOTE}>
          No step is due this week. Set one on a lead so the search does not
          stall.
        </p>
      )}
      <Link to="/work" className={TEXT_LINK}>
        Open Work
      </Link>
    </section>
  );
}

function Decisions() {
  const { data } = useDecisions();
  // Nothing needs a decision: nothing to show, and no nagging.
  if (!data || data.length === 0) return null;
  return (
    <section
      aria-labelledby="week-decisions-title"
      className={cn(PANEL, SUN_PANEL)}
    >
      <header className="flex flex-col gap-2">
        <h2 id="week-decisions-title" className={TITLE}>
          <span
            aria-hidden
            className={cn(BUBBLE, "bg-sun/22 text-[hsl(var(--foreground))] dark:text-sun")}
          >
            <Signpost className="h-5 w-5" strokeWidth={1.8} />
          </span>
          Needs a decision
        </h2>
        <p className={NOTE}>Things that will not sort themselves out.</p>
      </header>
      <ul className="ledger">
        {data.map((d) => (
          <li key={d.kind} className="ledger-row">
            <Link
              to={d.link}
              className="flex w-full items-center justify-between gap-4 no-underline"
            >
              <span className="text-[15px] font-medium">{d.text}</span>
              <span className="pill pill-sun shrink-0">
                Open
                <ChevronRight className="h-3.5 w-3.5" aria-hidden />
              </span>
            </Link>
          </li>
        ))}
      </ul>
    </section>
  );
}

function MoneyLine() {
  const { data } = useSummary();
  if (!data) return null;
  return (
    <Link
      to="/"
      className="flex min-h-[72px] flex-wrap items-center justify-between gap-x-6 gap-y-1 rounded-[28px] border border-mint/35 bg-gradient-to-r from-mint/15 via-teal/8 to-sky/12 px-6 py-5 no-underline shadow-[0_16px_40px_-28px_hsl(var(--c-mint)/0.8)] transition-transform duration-300 hover:-translate-y-0.5 sm:px-12"
    >
      <span className="flex items-center gap-3 text-sm text-muted-foreground">
        <span aria-hidden className={cn(BUBBLE, "bg-mint/20 text-mint")}>
          <Wallet className="h-5 w-5" strokeWidth={1.8} />
        </span>
        Safe to spend until {formatDay(data.until)}
      </span>
      <span
        className={cn(
          "font-display text-2xl leading-none",
          data.safe_to_spend_paise < 0 ? "text-over" : "text-good",
        )}
      >
        {data.safe_to_spend_paise >= 0 ? (
          <RupeeOdometer
            paise={Math.round(data.safe_to_spend_paise / 100) * 100}
          />
        ) : (
          formatINR(Math.round(data.safe_to_spend_paise / 100) * 100)
        )}
      </span>
    </Link>
  );
}

/** The week: one screen, about five minutes. Habits, the people to call, the job search, the money. */
export default function WeekPage() {
  const diet = useDiet();
  const people = usePeople();

  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page page-narrow stack">
        <header>
          <h1 className="page-title">This week</h1>
          <p className="page-sub">
            Five minutes, once a week: your habits, the people to call, your job
            search, your money.
          </p>
        </header>

        {diet.isLoading && (
          <div
            className="h-64 animate-pulse rounded-[28px] bg-gradient-to-br from-sky/15 via-grape/10 to-mint/15"
            aria-label="Loading your week"
          />
        )}
        {diet.data && <WeekCard overview={diet.data} />}
        {diet.data && diet.data.habits.every((h) => !h.active) && (
          <Link to="/health" className={TEXT_LINK}>
            Pick your habits on Health
          </Link>
        )}
        {diet.isError && !diet.data && (
          <p className="empty-note m-0">
            Couldn't load your habits. The rest of the week is below.
          </p>
        )}

        {people.data && people.data.catch_up.length > 0 && (
          <CatchUpSection people={people.data.catch_up} />
        )}
        <Decisions />
        <WorkSteps />
        <MoneyLine />
      </main>
    </div>
  );
}
