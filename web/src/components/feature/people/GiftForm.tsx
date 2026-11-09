import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { errorMessage } from "@/components/feature/people/ui";
import { GIFT_LABEL, KIND_LABEL, addGift, localDay, usePeopleMutation, type GiftStatus, type PersonDate } from "@/services/people";
import { formatDay } from "@/utils/dates";
import { toPaise } from "@/utils/money";

const STATUSES: GiftStatus[] = ["idea", "bought", "given"];

/**
 * Add a gift. Picking a date links the gift to that date and its year, which is what clears the
 * "gift for this date" reminder.
 */
export function GiftForm({ personId, dates, onDone }: { personId: string; dates: PersonDate[]; onDone: () => void }) {
  const id = useId();
  const add = usePeopleMutation(addGift);
  const [title, setTitle] = useState("");
  const [dateId, setDateId] = useState("");
  const [status, setStatus] = useState<GiftStatus>("idea");
  const [amount, setAmount] = useState("");
  const [onDay, setOnDay] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (title.trim() === "") {
      setError("Name the gift first.");
      return;
    }
    const input: Parameters<typeof addGift>[0] = { person_id: personId, title: title.trim(), status };
    if (dateId) {
      const date = dates.find((d) => d.id === dateId);
      if (date) {
        input.date_id = date.id;
        input.for_year = Number(date.next_on.slice(0, 4));
      }
    }
    if (amount.trim() !== "") {
      const paise = toPaise(amount);
      if (paise === null || paise <= 0) {
        setError("Enter the amount as rupees, like 1500.");
        return;
      }
      input.amount_paise = paise;
    }
    if (onDay) input.on_day = onDay;
    setError(null);
    add.mutate(input, {
      onSuccess: () => {
        toast.success("Gift added.");
        onDone();
      },
      onError: (err) => toast.error(errorMessage(err)),
    });
  };

  return (
    <form onSubmit={submit} aria-label="Add a gift" className="grid gap-4 sm:grid-cols-2">
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-title`}>What is the gift?</Label>
        <Input id={`${id}-title`} maxLength={80} value={title} onChange={(e) => setTitle(e.target.value)} placeholder="A book they mentioned" />
      </div>
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
      <div className="sm:col-span-2">
        <Button type="submit" disabled={add.isPending}>
          Add gift
        </Button>
      </div>
    </form>
  );
}
