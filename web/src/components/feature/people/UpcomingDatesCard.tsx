import { Link } from "react-router-dom";

import { UpcomingRow } from "@/components/feature/people/UpcomingRow";
import { usePeople } from "@/services/people";

const SHOWN = 3;
const WITHIN_DAYS = 14;

/** Dashboard: the next important dates within 14 days. Hidden while loading, on error, or when empty. */
export function UpcomingDatesCard() {
  const { data } = usePeople();
  const soon = (data?.upcoming ?? []).filter((u) => u.days_away <= WITHIN_DAYS).slice(0, SHOWN);
  if (soon.length === 0) return null;

  return (
    <section
      aria-labelledby="upcoming-dates-title"
      className="mt-6 flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10"
    >
      <header className="flex flex-col gap-2">
        <h2 id="upcoming-dates-title" className="m-0 font-display text-[32px] leading-none">
          Coming up
        </h2>
        <p className="m-0 text-sm text-muted-foreground">The people you care about, in the next two weeks.</p>
      </header>
      <ul className="ledger">
        {soon.map((u) => (
          <UpcomingRow key={u.date_id} item={u} />
        ))}
      </ul>
      <Link to="/people" className="self-start text-sm font-medium text-primary">
        All people
      </Link>
    </section>
  );
}
