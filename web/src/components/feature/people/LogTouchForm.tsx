import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { NOTE_MAX, errorMessage } from "@/components/feature/people/ui";
import { useAddTouch } from "@/components/feature/people/mutations";
import { localDay, TOUCH_LABEL, type TouchKind } from "@/services/people";
import { toPaise } from "@/utils/money";

const KINDS: TouchKind[] = ["call", "visit", "message", "met"];

/** Log a catch-up that isn't a quick call: another kind, an earlier day, a visit's cost, or a note. */
export function LogTouchForm({ personId, onDone }: { personId: string; onDone?: () => void }) {
  const id = useId();
  const add = useAddTouch();
  const [kind, setKind] = useState<TouchKind>("call");
  const [onDay, setOnDay] = useState("");
  const [cost, setCost] = useState("");
  const [note, setNote] = useState("");
  const [costError, setCostError] = useState<string | null>(null);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const input: { kind: TouchKind; on_day?: string; cost_paise?: number; note?: string } = { kind };
    if (onDay) input.on_day = onDay;
    if (kind === "visit" && cost.trim() !== "") {
      const paise = toPaise(cost);
      if (paise === null || paise <= 0) {
        setCostError("Enter the cost as rupees, like 450 or 450.50.");
        return;
      }
      input.cost_paise = paise;
    }
    if (note.trim() !== "") input.note = note.trim();
    setCostError(null);
    add.mutate(
      { personId, input },
      {
        onSuccess: () => {
          toast.success("Catch-up logged.");
          setOnDay("");
          setCost("");
          setNote("");
          onDone?.();
        },
        onError: (err) => toast.error(errorMessage(err)),
      },
    );
  };

  return (
    <form onSubmit={submit} aria-label="Log a catch-up" className="grid gap-4 sm:grid-cols-2">
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-kind`}>How did you catch up?</Label>
        <Select id={`${id}-kind`} value={kind} onChange={(e) => setKind(e.target.value as TouchKind)}>
          {KINDS.map((k) => (
            <option key={k} value={k}>
              {TOUCH_LABEL[k]}
            </option>
          ))}
        </Select>
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor={`${id}-day`}>Day</Label>
        <Input id={`${id}-day`} type="date" max={localDay()} value={onDay} onChange={(e) => setOnDay(e.target.value)} />
        <span className="text-[13px] text-muted-foreground">Leave blank for today.</span>
      </div>
      {kind === "visit" && (
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-cost`}>What did the visit cost? (₹, optional)</Label>
          <MoneyInput
            id={`${id}-cost`}
            value={cost}
            onChange={(e) => setCost(e.target.value)}
            aria-invalid={costError !== null}
            aria-describedby={costError ? `${id}-cost-error` : undefined}
          />
          {costError && (
            <span id={`${id}-cost-error`} role="alert" className="text-[13px] text-destructive">
              {costError}
            </span>
          )}
        </div>
      )}
      <div className="flex flex-col gap-2 sm:col-span-2">
        <Label htmlFor={`${id}-note`}>Note (optional)</Label>
        <Input
          id={`${id}-note`}
          maxLength={NOTE_MAX}
          value={note}
          onChange={(e) => setNote(e.target.value)}
          placeholder="What you talked about"
        />
      </div>
      <div className="sm:col-span-2">
        <Button type="submit" disabled={add.isPending}>
          Log catch-up
        </Button>
      </div>
    </form>
  );
}
