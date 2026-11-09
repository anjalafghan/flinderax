import { UpcomingRow } from "@/components/feature/people/UpcomingRow";
import { NOTE, PANEL, SECTION_TITLE } from "@/components/feature/people/ui";
import type { Upcoming } from "@/services/people";

/** Every important date in the next 60 days, soonest first, each with its gift state. */
export function PeopleUpcoming({ upcoming }: { upcoming: Upcoming[] }) {
  return (
    <section aria-labelledby="people-upcoming-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="people-upcoming-title" className={SECTION_TITLE}>
          Upcoming
        </h2>
        <p className={NOTE}>The next 60 days.</p>
      </header>
      {upcoming.length > 0 ? (
        <ul className="ledger">
          {upcoming.map((u) => (
            <UpcomingRow key={u.date_id} item={u} />
          ))}
        </ul>
      ) : (
        <p className={NOTE}>No dates in the next 60 days. Add a birthday or anniversary from someone's page.</p>
      )}
    </section>
  );
}
