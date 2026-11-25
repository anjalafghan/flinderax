import { Link } from "react-router-dom";

import { GiftChip } from "@/components/feature/people/GiftChip";
import { KIND_TONE, MONTH_NAMES } from "@/components/feature/people/ui";
import { cn } from "@/utils/cn";
import {
  dateTitle,
  daysAwayLabel,
  weekdayDay,
  type Upcoming,
} from "@/services/people";

/** One upcoming date: the whole row opens the person, the gift chip shows the gift state. */
export function UpcomingRow({ item }: { item: Upcoming }) {
  const turns = item.turns !== null ? ` · turns ${item.turns}` : "";
  // a little calendar leaf in the date's colour (the text below says the same thing)
  const [, mm, dd] = item.on.split("-");
  const month = MONTH_NAMES[Number(mm) - 1]?.slice(0, 3) ?? "";
  const soon = item.days_away <= 7;
  return (
    <li>
      <Link
        to={`/people/${item.person_id}`}
        className="ledger-row no-underline text-foreground transition-colors hover:bg-brand/10"
      >
        <span className="flex min-w-0 items-center gap-3">
          <span
            aria-hidden
            className={cn(
              "flex h-12 w-12 shrink-0 flex-col items-center justify-center rounded-2xl border leading-none",
              KIND_TONE[item.kind] ?? KIND_TONE.other,
            )}
          >
            <span className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
              {month}
            </span>
            <span className="font-display text-[20px] font-semibold">
              {Number(dd)}
            </span>
          </span>
          <span className="flex min-w-0 flex-col gap-1">
            <span className="text-[15px] font-medium">
              {dateTitle(item.name, item.kind, item.label)}
            </span>
            <span
              className={cn(
                "text-[13px]",
                soon ? "font-medium text-brand-ink" : "text-muted-foreground",
              )}
            >
              {`${weekdayDay(item.on)} · ${daysAwayLabel(item.days_away)}${turns}`}
            </span>
          </span>
        </span>
        <GiftChip status={item.gift_status} />
      </Link>
    </li>
  );
}
