import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { MONTH_NAMES, errorMessage } from "@/components/feature/people/ui";
import { addDate, localDay, updateDate, usePeopleMutation, KIND_LABEL, type DateKind, type PersonDate } from "@/services/people";
import { paiseToInput, toPaise } from "@/utils/money";

const KINDS: DateKind[] = ["birthday", "anniversary", "remembrance", "other"];
const DAYS = Array.from({ length: 31 }, (_, i) => i + 1);

/** Add a date to a person, or edit one. Day and month are required; the year is optional. */
export function DateForm({
  personId,
  initial,
  onDone,
}: {
  personId: string;
  initial?: PersonDate;
  onDone: () => void;
}) {
  const id = useId();
  const save = usePeopleMutation((v: { dateId?: string; input: Parameters<typeof addDate>[1] }) =>
    v.dateId ? updateDate(v.dateId, v.input) : addDate(personId, v.input),
  );
  const today = localDay().split("-");
  const [kind, setKind] = useState<DateKind>(initial?.kind ?? "birthday");
  const [label, setLabel] = useState(initial?.label ?? "");
  const [month, setMonth] = useState(initial?.month ?? Number(today[1]));
  const [day, setDay] = useState(initial?.day ?? Number(today[2]));
  const [year, setYear] = useState(initial?.year ? String(initial.year) : "");
  const [remind, setRemind] = useState(String(initial?.remind_days ?? 7));
  const [budget, setBudget] = useState(initial?.gift_budget_paise != null ? paiseToInput(initial.gift_budget_paise) : "");

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const remindDays = Number(remind);
    if (!Number.isInteger(remindDays) || remindDays < 0 || remindDays > 60) {
      toast.error("Remind me up to 60 days ahead.");
      return;
    }
    const input: Parameters<typeof addDate>[1] = { kind, label: label.trim(), month, day, remind_days: remindDays };
    if (year.trim() !== "") input.year = Number(year);
    if (budget.trim() === "") {
      input.gift_budget_paise = null;
    } else {
      const paise = toPaise(budget);
      if (paise === null || paise <= 0) {
        toast.error("Enter a gift budget in rupees, like 2000.");
        return;
      }
      input.gift_budget_paise = paise;
    }
    save.mutate(
      { dateId: initial?.id, input },
      {
        onSuccess: () => {
          toast.success(initial ? "Date updated." : "Date added.");
          onDone();
        },
        onError: (err) => toast.error(errorMessage(err)),
      },
    );
  };

  return (
    <form onSubmit={submit} aria-label={initial ? "Edit date" : "Add a date"} className="grid gap-4 sm:grid-cols-2">
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-kind`}>What is it?</Label>
        <Select id={`${id}-kind`} value={kind} onChange={(e) => setKind(e.target.value as DateKind)}>
          {KINDS.map((k) => (
            <option key={k} value={k}>
              {KIND_LABEL[k]}
            </option>
          ))}
        </Select>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-label`}>Name it (optional)</Label>
        <Input id={`${id}-label`} maxLength={40} value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Diwali" />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-month`}>Month</Label>
        <Select id={`${id}-month`} value={month} onChange={(e) => setMonth(Number(e.target.value))}>
          {MONTH_NAMES.map((name, i) => (
            <option key={name} value={i + 1}>
              {name}
            </option>
          ))}
        </Select>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-day`}>Day</Label>
        <Select id={`${id}-day`} value={day} onChange={(e) => setDay(Number(e.target.value))}>
          {DAYS.map((d) => (
            <option key={d} value={d}>
              {d}
            </option>
          ))}
        </Select>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-year`}>Year it began (optional)</Label>
        <Input
          id={`${id}-year`}
          type="number"
          inputMode="numeric"
          min={1900}
          max={Number(today[0])}
          value={year}
          onChange={(e) => setYear(e.target.value)}
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-remind`}>Remind me how many days before</Label>
        <Input
          id={`${id}-remind`}
          type="number"
          inputMode="numeric"
          min={0}
          max={60}
          value={remind}
          onChange={(e) => setRemind(e.target.value)}
        />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-budget`}>Gift budget (₹, optional)</Label>
        <MoneyInput id={`${id}-budget`} value={budget} onChange={(e) => setBudget(e.target.value)} />
      </div>
      <div className="flex flex-wrap gap-2 sm:col-span-2">
        <Button type="submit" disabled={save.isPending}>
          {initial ? "Save date" : "Add date"}
        </Button>
        <Button type="button" variant="ghost" onClick={onDone}>
          Cancel
        </Button>
      </div>
    </form>
  );
}
