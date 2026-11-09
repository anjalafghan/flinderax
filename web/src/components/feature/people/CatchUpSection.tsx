import { Link } from "react-router-dom";

import { CatchUpActions } from "@/components/feature/people/CatchUpActions";
import { NOTE, PANEL, SECTION_TITLE } from "@/components/feature/people/ui";
import { StatusPill } from "@/components/feature/people/StatusPill";
import { everyLabel, lastCaughtUpLabel, weekdayDay, type PersonRow } from "@/services/people";

function dueText(p: PersonRow): string {
  if (!p.next_due_on) return "";
  return p.status === "overdue" ? `Was due ${weekdayDay(p.next_due_on)}` : `Due ${weekdayDay(p.next_due_on)}`;
}

function CatchUpRow({ person }: { person: PersonRow }) {
  const details = [everyLabel(person.every_days), lastCaughtUpLabel(person.last_contact_on), dueText(person)]
    .filter(Boolean)
    .join(" · ");

  return (
    <li className="flex flex-col gap-4 border-t border-border py-5 first:border-t-0 first:pt-1">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-1">
          <div className="flex flex-wrap items-center gap-2">
            <Link to={`/people/${person.id}`} className="text-[15px] font-medium text-foreground">
              {person.name}
            </Link>
            <StatusPill status={person.status} />
          </div>
          <span className="text-[13px] text-muted-foreground">{details}</span>
        </div>
      </div>
      <CatchUpActions personId={person.id} name={person.name} />
    </li>
  );
}

/** People past, or close to, their catch-up date. The most overdue come first. */
export function CatchUpSection({ people }: { people: PersonRow[] }) {
  return (
    <section aria-labelledby="people-catch-up-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="people-catch-up-title" className={SECTION_TITLE}>
          Catch up
        </h2>
        <p className={NOTE}>A nudge when it's been a while.</p>
      </header>
      {people.length > 0 ? (
        <ul className="m-0 list-none p-0">
          {people.map((p) => (
            <CatchUpRow key={p.id} person={p} />
          ))}
        </ul>
      ) : (
        <p className={NOTE}>Nobody needs a nudge right now.</p>
      )}
    </section>
  );
}
