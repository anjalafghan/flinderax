import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { errorMessage } from "@/components/feature/people/ui";
import {
  GIFT_LABEL,
  KIND_LABEL,
  addGift,
  localDay,
  updateGift,
  usePeopleMutation,
  type Gift,
  type GiftInput,
  type GiftPatch,
  type GiftStatus,
  type PersonDate,
} from "@/services/people";
import { formatDay } from "@/utils/dates";
import { paiseToInput, toPaise } from "@/utils/money";

const STATUSES: GiftStatus[] = ["idea", "bought", "given"];

/** The shared success path: say so, then close the form. */
const saved = (onDone: () => void, message: string) => ({
  onSuccess: () => {
    toast.success(message);
    onDone();
  },
  onError: (err: unknown) => toast.error(errorMessage(err)),
});

/**
 * Add a gift, or edit one. Picking a date links a new gift to that date and its year, which is what
 * clears the "gift for this date" reminder. A date can't be changed on an existing gift.
 */
export function GiftForm({
  personId,
  dates,
  initial,
  onDone,
}: {
  personId: string;
  dates: PersonDate[];
  initial?: Gift;
  onDone: () => void;
}) {
  const id = useId();
  const save = usePeopleMutation((v: { giftId?: string; body: GiftInput | GiftPatch }) =>
    v.giftId ? updateGift(v.giftId, v.body as GiftPatch) : addGift(v.body as GiftInput),
  );
  const [title, setTitle] = useState(initial?.title ?? "");
  const [dateId, setDateId] = useState("");
  const [status, setStatus] = useState<GiftStatus>(initial?.status ?? "idea");
  const [amount, setAmount] = useState(initial?.amount_paise != null ? paiseToInput(initial.amount_paise) : "");
  const [onDay, setOnDay] = useState(initial?.on_day ?? "");
  const [error, setError] = useState<string | null>(null);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (title.trim() === "") {
      setError("Name the gift first.");
      return;
    }
    let amountPaise: number | null = null;
    if (amount.trim() !== "") {
      const paise = toPaise(amount);
      if (paise === null || paise <= 0) {
        setError("Enter the amount as rupees, like 1500.");
        return;
      }
      amountPaise = paise;
    }
    setError(null);

    if (initial) {
      // Editing: an empty amount or day is sent as null, so a cleared value really clears.
      const patch: GiftPatch = {
        title: title.trim(),
        status,
        amount_paise: amountPaise,
        on_day: onDay || null,
      };
      save.mutate({ giftId: initial.id, body: patch }, saved(onDone, "Gift saved."));
      return;
    }

    const input: GiftInput = { person_id: personId, title: title.trim(), status };
    if (dateId) {
      const date = dates.find((d) => d.id === dateId);
      if (date) {
        input.date_id = date.id;
        input.for_year = Number(date.next_on.slice(0, 4));
      }
    }
    if (amountPaise !== null) input.amount_paise = amountPaise;
    if (onDay) input.on_day = onDay;
    save.mutate({ body: input }, saved(onDone, "Gift added."));
  };

  return (
    <form
      onSubmit={submit}
      aria-label={initial ? "Edit gift" : "Add a gift"}
      className="grid gap-4 sm:grid-cols-2"
    >
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-title`}>What is the gift?</Label>
        <Input
          id={`${id}-title`}
          maxLength={80}
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="A book they mentioned"
        />
      </div>
      {!initial && (
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-date`}>For which date? (optional)</Label>
          <Select id={`${id}-date`} value={dateId} onChange={(e) => setDateId(e.target.value)}>
            <option value="">No date</option>
            {dates.map((d) => (
              <option key={d.id} value={d.id}>
                {`${d.label.trim() || KIND_LABEL[d.kind]} · ${formatDay(d.next_on)}`}
              </option>
            ))}
          </Select>
        </div>
      )}
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-status`}>Status</Label>
        <Select id={`${id}-status`} value={status} onChange={(e) => setStatus(e.target.value as GiftStatus)}>
          {STATUSES.map((s) => (
            <option key={s} value={s}>
              {GIFT_LABEL[s]}
            </option>
          ))}
        </Select>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-amount`}>Amount (₹, optional)</Label>
        <MoneyInput
          id={`${id}-amount`}
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          aria-invalid={error !== null}
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-on-day`}>Given on (optional)</Label>
        <Input id={`${id}-on-day`} type="date" max={localDay()} value={onDay} onChange={(e) => setOnDay(e.target.value)} />
      </div>
      {error && (
        <p role="alert" className="m-0 text-[13px] text-destructive sm:col-span-2">
          {error}
        </p>
      )}
      <div className="flex flex-wrap gap-2 sm:col-span-2">
        <Button type="submit" disabled={save.isPending}>
          {initial ? "Save gift" : "Add gift"}
        </Button>
        {initial && (
          <Button type="button" variant="ghost" onClick={onDone}>
            Cancel
          </Button>
        )}
      </div>
    </form>
  );
}
