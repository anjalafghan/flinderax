import { Link } from "react-router-dom";

import { Users } from "lucide-react";

import { Avatar } from "@/components/feature/people/Avatar";
import { StatusPill } from "@/components/feature/people/StatusPill";
import {
  CIRCLE_DOT,
  NOTE,
  PANEL,
  SECTION_TITLE,
} from "@/components/feature/people/ui";
import { cn } from "@/utils/cn";
import {
  CIRCLES,
  everyLabel,
  lastCaughtUpLabel,
  plural,
  type PersonRow,
} from "@/services/people";

function PersonLine({ person }: { person: PersonRow }) {
  const rhythm = everyLabel(person.every_days);
  return (
    <li>
      <Link
        to={`/people/${person.id}`}
        className="ledger-row no-underline text-foreground transition-colors hover:bg-brand/10"
      >
        <span className="flex min-w-0 items-center gap-3">
          <Avatar name={person.name} status={person.status} />
          <span className="flex min-w-0 flex-col gap-1">
            <span className="text-[15px] font-medium">{person.name}</span>
          <span className="text-[13px] text-muted-foreground">
            {[rhythm, lastCaughtUpLabel(person.last_contact_on)]
              .filter(Boolean)
              .join(" · ")}
            </span>
          </span>
        </span>
        <StatusPill status={person.status} />
      </Link>
    </li>
  );
}

/** Everyone, grouped by circle. Archived people sit in a collapsed list at the bottom. */
export function CircleList({
  people,
  archived,
}: {
  people: PersonRow[];
  archived: PersonRow[];
}) {
  return (
    <section aria-labelledby="people-circle-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2
          id="people-circle-title"
          className={cn(SECTION_TITLE, "flex items-center gap-3")}
        >
          <span
            aria-hidden
            className="grid h-10 w-10 place-items-center rounded-full bg-rose/15 text-rose"
          >
            <Users className="h-5 w-5" />
          </span>
          Your people
        </h2>
      </header>
      {people.length === 0 ? (
        <p className={NOTE}>No one yet. Add the people who matter below.</p>
      ) : (
        CIRCLES.map((c) => {
          const group = people.filter((p) => p.circle === c.key);
          if (group.length === 0) return null;
          return (
            <div key={c.key} className="flex flex-col gap-2">
              <h3 className="m-0 flex items-center gap-2 text-[15px] font-medium text-muted-foreground">
                <span
                  aria-hidden
                  className={cn("h-2.5 w-2.5 rounded-full", CIRCLE_DOT[c.key])}
                />
                {c.label}
              </h3>
              <ul className="ledger">
                {group.map((p) => (
                  <PersonLine key={p.id} person={p} />
                ))}
              </ul>
            </div>
          );
        })
      )}
      {archived.length > 0 && (
        <details className="flex flex-col gap-3">
          <summary className="flex min-h-11 cursor-pointer items-center text-sm font-medium text-muted-foreground">
            {`Archived (${archived.length})`}
          </summary>
          <p className={NOTE}>
            {plural(archived.length, "person is", "people are")} kept out of
            reminders.
          </p>
          <ul className="ledger">
            {archived.map((p) => (
              <PersonLine key={p.id} person={p} />
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
