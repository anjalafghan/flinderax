import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { WeekCard } from "@/components/feature/health/WeekCard";
import { CatchUpSection } from "@/components/feature/people/CatchUpSection";
import { useSummary } from "@/hooks/usePlanner";
import { useDiet } from "@/services/diet";
import { usePeople } from "@/services/people";
import { useWork, type Lead } from "@/services/work";
import { formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";

const PANEL = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
const TITLE = "m-0 font-display text-[32px] leading-none";
const NOTE = "m-0 text-sm text-muted-foreground";
const TEXT_LINK = "self-start text-sm font-medium text-primary";
/** A next step this many days away, or already late, belongs on this week's list. */
const SOON_DAYS = 7;

function daysUntil(day: string): number {
  const [y, m, d] = day.split("-").map(Number);
  const today = new Date();
  const from = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
  return Math.round((Date.UTC(y ?? 0, (m ?? 1) - 1, d ?? 1) - from) / 86_400_000);
}

/** Open leads whose next step is late or falls within the week, soonest first. */
export function stepsThisWeek(leads: Lead[]): Lead[] {
  return leads
    .filter((l) => l.stage !== "closed" && l.next_on !== null && daysUntil(l.next_on) <= SOON_DAYS)
    .sort((a, b) => (a.next_on ?? "").localeCompare(b.next_on ?? ""));
}

function WorkSteps() {
  const work = useWork();
  if (!work.data) return null;
  const open = work.data.leads.filter((l) => l.stage !== "closed");
  // No search going on: nothing to show, and no nagging.
  if (open.length === 0) return null;
  const steps = stepsThisWeek(open);
  return (
    <section aria-labelledby="week-work-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="week-work-title" className={TITLE}>
          Job search
        </h2>
        <p className={NOTE}>{`${open.length} open ${open.length === 1 ? "lead" : "leads"}. One step a week keeps it moving.`}</p>
      </header>
      {steps.length > 0 ? (
        <ul className="ledger">
          {steps.map((l) => {
            const late = l.next_on !== null && daysUntil(l.next_on) < 0;
            return (
              <li key={l.id} className="ledger-row">
                <span className="flex min-w-0 flex-col gap-1">
                  <span className="text-[15px] font-medium">{l.next_step || "Next step"}</span>
                  <span className="text-[13px] text-muted-foreground">{l.company}</span>
                </span>
                <span className={`shrink-0 rounded-full px-3 py-1 text-[13px] ${late ? "tone-over" : "tone-warn"}`}>
                  {l.next_on ? `${late ? "Was due" : "By"} ${formatDay(l.next_on)}` : ""}
                </span>
              </li>
            );
          })}
        </ul>
      ) : (
        <p className={NOTE}>No step is due this week. Set one on a lead so the search does not stall.</p>
      )}
      <Link to="/work" className={TEXT_LINK}>
        Open Work
      </Link>
    </section>
  );
}

function MoneyLine() {
  const { data } = useSummary();
  if (!data) return null;
  return (
    <Link to="/" className="flex min-h-[72px] flex-wrap items-center justify-between gap-x-6 gap-y-1 rounded-[28px] border border-border bg-card px-6 py-5 no-underline hover:bg-[hsl(var(--chip))] sm:px-12">
      <span className="text-sm text-muted-foreground">Safe to spend until {formatDay(data.until)}</span>
      <span className="font-display text-2xl leading-none">{formatINR(Math.round(data.safe_to_spend_paise / 100) * 100)}</span>
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
      <main className="page max-w-3xl stack">
        <header>
          <h1 className="page-title">This week</h1>
          <p className="page-sub">Five minutes, once a week: your habits, the people to call, your job search, your money.</p>
        </header>

        {diet.isLoading && <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your week" />}
        {diet.data && <WeekCard overview={diet.data} />}
        {diet.data && diet.data.habits.every((h) => !h.active) && (
          <Link to="/health" className={TEXT_LINK}>
            Pick your habits on Health
          </Link>
        )}
        {diet.isError && !diet.data && <p className="empty-note m-0">Couldn't load your habits. The rest of the week is below.</p>}

        {people.data && people.data.catch_up.length > 0 && <CatchUpSection people={people.data.catch_up} />}
        <WorkSteps />
        <MoneyLine />
      </main>
    </div>
  );
}
