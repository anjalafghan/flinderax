import { CalendarHeart } from "lucide-react";

import { UpcomingRow } from "@/components/feature/people/UpcomingRow";
import { cn } from "@/utils/cn";
import { NOTE, PANEL, SECTION_TITLE } from "@/components/feature/people/ui";
import type { Upcoming } from "@/services/people";

/** Every important date in the next 60 days, soonest first, each with its gift state. */
export function PeopleUpcoming({ upcoming }: { upcoming: Upcoming[] }) {
  return (
    <section aria-labelledby="people-upcoming-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2
          id="people-upcoming-title"
          className={cn(SECTION_TITLE, "flex items-center gap-3")}
        >
          <span
            aria-hidden
            className="grid h-10 w-10 place-items-center rounded-full bg-sun/25 text-foreground"
          >
            <CalendarHeart className="h-5 w-5" />
          </span>
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
        <p className={NOTE}>
          No dates in the next 60 days. Add a birthday or anniversary from
          someone's page.
        </p>
      )}
    </section>
  );
}
