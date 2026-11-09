import { Link } from "react-router-dom";

import { GiftChip } from "@/components/feature/people/GiftChip";
import { dateTitle, daysAwayLabel, weekdayDay, type Upcoming } from "@/services/people";

/** One upcoming date: the whole row opens the person, the gift chip shows the gift state. */
export function UpcomingRow({ item }: { item: Upcoming }) {
  const turns = item.turns !== null ? ` · turns ${item.turns}` : "";
  return (
    <li>
      <Link
        to={`/people/${item.person_id}`}
        className="ledger-row no-underline text-foreground transition-colors hover:bg-chip/40"
      >
        <span className="flex min-w-0 flex-col gap-1">
          <span className="text-[15px] font-medium">{dateTitle(item.name, item.kind, item.label)}</span>
          <span className="text-[13px] text-muted-foreground">
            {`${weekdayDay(item.on)} · ${daysAwayLabel(item.days_away)}${turns}`}
          </span>
        </span>
        <GiftChip status={item.gift_status} />
      </Link>
    </li>
  );
}
