import { useState } from "react";
import { toast } from "sonner";

import { DateForm } from "@/components/feature/people/DateForm";
import { errorMessage, NOTE } from "@/components/feature/people/ui";
import { Button } from "@/components/ui/button";
import { rupees } from "@/services/checkin";
import {
  dateTitle,
  daysAwayLabel,
  deleteDate,
  usePeopleMutation,
  weekdayDay,
  type PersonDate,
} from "@/services/people";

function DateRow({ date, personName }: { date: PersonDate; personName: string }) {
  const [editing, setEditing] = useState(false);
  const remove = usePeopleMutation(deleteDate);
  if (editing) {
    return (
      <li className="border-t border-border py-5 first:border-t-0 first:pt-1">
        <DateForm personId={date.person_id} initial={date} onDone={() => setEditing(false)} />
      </li>
    );
  }
  const turns = date.turns !== null ? ` · turns ${date.turns}` : "";
  const budget = date.gift_budget_paise !== null ? `Gift budget ${rupees(date.gift_budget_paise)}` : "No gift budget";
  const title = dateTitle(personName, date.kind, date.label);
  return (
    <li className="ledger-row">
      <span className="flex min-w-0 flex-col gap-1">
        <span className="text-[15px] font-medium">{title}</span>
        <span className="text-[13px] text-muted-foreground">{`${weekdayDay(date.next_on)} · ${daysAwayLabel(date.days_away)}${turns}`}</span>
        <span className="text-[13px] text-muted-foreground">{budget}</span>
      </span>
      <span className="flex shrink-0 gap-2">
        <Button variant="outline" className="min-h-11" onClick={() => setEditing(true)}>
          {`Edit ${title}`}
        </Button>
        <Button
          variant="ghost"
          className="min-h-11"
          disabled={remove.isPending}
          onClick={() =>
            remove.mutate(date.id, {
              onSuccess: () => toast.success("Date removed."),
              onError: (err) => toast.error(errorMessage(err)),
            })
          }
        >
          {`Delete ${title}`}
        </Button>
      </span>
    </li>
  );
}

/** A person's dates, soonest first. Each repeats yearly. */
export function PersonDates({ personId, personName, dates }: { personId: string; personName: string; dates: PersonDate[] }) {
  const [adding, setAdding] = useState(false);
  return (
    <section aria-labelledby="person-dates-title" className="flex flex-col gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 id="person-dates-title" className="m-0 font-display text-[32px] leading-none">
          Dates
        </h2>
        {!adding && (
          <Button variant="outline" className="min-h-11" onClick={() => setAdding(true)}>
            Add a date
          </Button>
        )}
      </div>
      {dates.length > 0 ? (
        <ul className="ledger">
          {dates.map((d) => (
            <DateRow key={d.id} date={d} personName={personName} />
          ))}
        </ul>
      ) : (
        <p className={NOTE}>No dates yet. Add a birthday or an anniversary to plan ahead.</p>
      )}
      {adding && <DateForm personId={personId} onDone={() => setAdding(false)} />}
    </section>
  );
}
