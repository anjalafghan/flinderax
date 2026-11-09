import { useState } from "react";
import { toast } from "sonner";

import { GiftForm } from "@/components/feature/people/GiftForm";
import { errorMessage, NOTE } from "@/components/feature/people/ui";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { rupees } from "@/services/checkin";
import {
  GIFT_LABEL,
  KIND_LABEL,
  deleteGift,
  updateGift,
  usePeopleMutation,
  type Gift,
  type GiftStatus,
  type PersonDate,
} from "@/services/people";
import { formatDay } from "@/utils/dates";

const STATUSES: GiftStatus[] = ["idea", "bought", "given"];

function GiftRow({ gift, dates }: { gift: Gift; dates: PersonDate[] }) {
  const change = usePeopleMutation((s: GiftStatus) => updateGift(gift.id, { status: s }));
  const remove = usePeopleMutation(deleteGift);
  const date = dates.find((d) => d.id === gift.date_id);
  const forDate = date ? ` for ${date.label.trim() || KIND_LABEL[date.kind]}` : "";
  const when = gift.on_day ? ` · ${formatDay(gift.on_day)}` : "";
  const amount = gift.amount_paise !== null ? ` · ${rupees(gift.amount_paise)}` : "";
  return (
    <li className="flex flex-col gap-3 border-t border-border py-5 first:border-t-0 first:pt-1 sm:flex-row sm:items-center sm:justify-between">
      <span className="flex min-w-0 flex-col gap-1">
        <span className="text-[15px] font-medium">{gift.title}</span>
        <span className="text-[13px] text-muted-foreground">{`${GIFT_LABEL[gift.status]}${forDate}${amount}${when}`}</span>
      </span>
      <span className="flex flex-wrap items-center gap-2">
        <Select
          aria-label={`Status for ${gift.title}`}
          className="h-11 w-auto min-w-36"
          value={gift.status}
          disabled={change.isPending}
          onChange={(e) => {
            // Read the value now: the select is controlled, so React may reset it before the save returns.
            const next = e.target.value as GiftStatus;
            change.mutate(next, {
              onSuccess: () => toast.success(`${gift.title} is now ${GIFT_LABEL[next].toLowerCase()}.`),
              onError: (err) => toast.error(errorMessage(err)),
            });
          }}
        >
          {STATUSES.map((s) => (
            <option key={s} value={s}>
              {GIFT_LABEL[s]}
            </option>
          ))}
        </Select>
        <Button
          variant="ghost"
          className="min-h-11"
          disabled={remove.isPending}
          onClick={() =>
            remove.mutate(gift.id, {
              onSuccess: () => toast.success("Gift removed."),
              onError: (err) => toast.error(errorMessage(err)),
            })
          }
        >
          {`Delete ${gift.title}`}
        </Button>
      </span>
    </li>
  );
}

/** Gifts for this person, newest first, with a status that moves from idea to bought to given. */
export function PersonGifts({ personId, gifts, dates }: { personId: string; gifts: Gift[]; dates: PersonDate[] }) {
  const [adding, setAdding] = useState(false);
  return (
    <section aria-labelledby="person-gifts-title" className="flex flex-col gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 id="person-gifts-title" className="m-0 font-display text-[32px] leading-none">
          Gifts
        </h2>
        {!adding && (
          <Button variant="outline" className="min-h-11" onClick={() => setAdding(true)}>
            Add a gift
          </Button>
        )}
      </div>
      {gifts.length > 0 ? (
        <ul className="m-0 list-none p-0">
          {gifts.map((g) => (
            <GiftRow key={g.id} gift={g} dates={dates} />
          ))}
        </ul>
      ) : (
        <p className={NOTE}>No gifts yet. Note an idea as soon as it comes up.</p>
      )}
      {adding && <GiftForm personId={personId} dates={dates} onDone={() => setAdding(false)} />}
    </section>
  );
}
