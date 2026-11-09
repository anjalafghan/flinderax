import { useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import {
  cmLabel,
  kgLabel,
  parseTenths,
  signedKg,
  todayIso,
  useAddDietMark,
  useDietMarks,
  useRemoveDietMark,
  type BodySummary,
  type Mark,
  type MarkKind,
} from "@/services/diet";
import { formatDay } from "@/utils/dates";

const SECTION = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
const KINDS: MarkKind[] = ["weight", "waist", "checkup"];
const KIND_LABEL: Record<MarkKind, string> = { weight: "Weight", waist: "Waist", checkup: "Check-up" };

/** "12 Oct 2027" from "2027-10-12". */
const dayYear = (iso: string) => `${formatDay(iso)} ${iso.slice(0, 4)}`;

/** The recent readings across all three kinds, newest first, each with a delete. */
function MarkList() {
  const weight = useDietMarks("weight");
  const waist = useDietMarks("waist");
  const checkup = useDietMarks("checkup");
  const remove = useRemoveDietMark();

  if (weight.isLoading || waist.isLoading || checkup.isLoading) {
    return <p className="m-0 text-sm text-muted-foreground">Loading your readings...</p>;
  }
  const marks: Mark[] = [...(weight.data ?? []), ...(waist.data ?? []), ...(checkup.data ?? [])].sort((a, b) =>
    b.measured_on.localeCompare(a.measured_on),
  );
  if (marks.length === 0) return <p className="empty-note m-0">Your readings will show here.</p>;

  const valueText = (m: Mark) =>
    m.kind === "checkup" || m.value_x10 === null
      ? KIND_LABEL[m.kind]
      : m.kind === "weight"
        ? kgLabel(m.value_x10)
        : cmLabel(m.value_x10);

  return (
    <ul className="ledger">
      {marks.map((m) => (
        <li key={m.id} className="ledger-row">
          <span className="flex min-w-0 flex-col gap-1">
            <span className="text-[15px] font-medium">{valueText(m)}</span>
            <span className="text-[13px] text-muted-foreground">{`${KIND_LABEL[m.kind]} on ${formatDay(m.measured_on)}`}</span>
          </span>
          <Button
            variant="ghost"
            className="h-11 px-3"
            aria-label={`Delete ${KIND_LABEL[m.kind].toLowerCase()} from ${formatDay(m.measured_on)}`}
            onClick={() =>
              remove.mutate(m.id, {
                onSuccess: () => toast.success("Reading deleted."),
                onError: (e) => toast.error(e.message || "Couldn't delete that. Try again."),
              })
            }
          >
            Delete
          </Button>
        </li>
      ))}
    </ul>
  );
}

/** Adds a weight, waist or check-up date. Values in kg or cm to one decimal, a date no later than today. */
function AddMark({ initialKind, onDone }: { initialKind: MarkKind; onDone: () => void }) {
  const add = useAddDietMark();
  const [kind, setKind] = useState<MarkKind>(initialKind);
  const [on, setOn] = useState(todayIso());
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (mark: { kind: MarkKind; measured_on: string; value_x10?: number }) => {
    setError(null);
    add.mutate(mark, {
      onSuccess: () => {
        toast.success("Saved.");
        setValue("");
        onDone();
      },
      onError: (err) => toast.error(err.message || "Couldn't save that. Try again."),
    });
  };

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!on || on > todayIso()) {
      setError("Pick today or an earlier date.");
      return;
    }
    if (kind === "checkup") {
      submit({ kind, measured_on: on });
      return;
    }
    const x10 = kind === "weight" ? parseTenths(value, 200, 3000) : parseTenths(value, 300, 2000);
    if (x10 === null) {
      setError(kind === "weight" ? "Enter a weight from 20 to 300 kg, with one decimal at most." : "Enter a waist from 30 to 200 cm, with one decimal at most.");
      return;
    }
    submit({ kind, measured_on: on, value_x10: x10 });
  };

  return (
    <form className="form-grid" onSubmit={onSubmit} noValidate>
      <div className="field">
        <Label htmlFor="body-kind">What to add</Label>
        <Select id="body-kind" value={kind} onChange={(e) => setKind(e.target.value as MarkKind)}>
          {KINDS.map((k) => (
            <option key={k} value={k}>
              {KIND_LABEL[k]}
            </option>
          ))}
        </Select>
      </div>
      <div className="field">
        <Label htmlFor="body-on">Date</Label>
        <Input id="body-on" type="date" max={todayIso()} value={on} onChange={(e) => setOn(e.target.value)} />
      </div>
      {kind !== "checkup" && (
        <div className="field">
          <Label htmlFor="body-value">{kind === "weight" ? "Weight (kg)" : "Waist (cm)"}</Label>
          <Input
            id="body-value"
            inputMode="decimal"
            placeholder={kind === "weight" ? "70.5" : "84.0"}
            value={value}
            onChange={(e) => setValue(e.target.value)}
          />
        </div>
      )}
      <div className="form-actions">
        {error && (
          <p role="alert" className="m-0 mr-auto text-sm text-destructive">
            {error}
          </p>
        )}
        <Button type="submit" disabled={add.isPending}>
          Add
        </Button>
      </div>
    </form>
  );
}

/**
 * The find-the-change line: the weight mark that matches the previous reading, so the change can say
 * since when. Falls back to a plain phrase when that reading is not in the recent list.
 */
function sinceLabel(prev: number, weightOn: string, marks: Mark[] | undefined): string {
  const match = (marks ?? []).find((m) => m.value_x10 === prev && m.measured_on < weightOn);
  return match ? `since ${formatDay(match.measured_on)}` : "since your last weigh-in";
}

/** Weight, waist and the yearly check-up, with a quiet note that this is private. */
export function BodyCard({ body }: { body: BodySummary }) {
  const weightMarks = useDietMarks("weight");
  const [adding, setAdding] = useState<MarkKind | null>(null);

  const change =
    body.weight_x10 !== null && body.prev_weight_x10 !== null && body.weight_on
      ? `${signedKg(body.weight_x10 - body.prev_weight_x10)} ${sinceLabel(body.prev_weight_x10, body.weight_on, weightMarks.data)}`
      : null;

  return (
    <section aria-labelledby="health-body-title" className={SECTION}>
      <div className="flex flex-col gap-2">
        <h2 id="health-body-title" className="m-0 font-display text-[32px] leading-none">
          Body
        </h2>
        <p className="m-0 text-sm text-muted-foreground">
          Weight, and waist if you like, once a quarter is plenty. Kept private to you. Don&apos;t write medical details here.
        </p>
      </div>

      <dl className="m-0 grid grid-cols-1 gap-x-6 gap-y-5 sm:grid-cols-3">
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Weight</dt>
          <dd className="m-0 font-display text-[26px] leading-none">
            {body.weight_x10 !== null && body.weight_on
              ? `${kgLabel(body.weight_x10)} on ${formatDay(body.weight_on)}`
              : "Not recorded yet"}
          </dd>
          {change && <dd className="m-0 text-[13px] text-muted-foreground">{change}</dd>}
        </div>
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Waist</dt>
          <dd className="m-0 font-display text-[26px] leading-none">
            {body.waist_x10 !== null ? cmLabel(body.waist_x10) : "Not recorded"}
          </dd>
        </div>
        <div className="flex flex-col gap-1">
          <dt className="text-[13px] text-muted-foreground">Yearly check-up</dt>
          <dd className="m-0 text-[15px]">
            {body.last_checkup_on ? `Last ${dayYear(body.last_checkup_on)}` : "No check-up recorded yet."}
          </dd>
          {body.checkup_due_on && (
            <dd className="m-0 text-[13px] text-muted-foreground">{`Next due ${dayYear(body.checkup_due_on)}`}</dd>
          )}
        </div>
      </dl>

      <div className="flex flex-wrap gap-2">
        <Button variant="outline" className="h-11" onClick={() => setAdding(adding === "weight" ? null : "weight")}>
          Add weight
        </Button>
        <Button variant="outline" className="h-11" onClick={() => setAdding(adding === "waist" ? null : "waist")}>
          Add waist
        </Button>
        <Button variant="outline" className="h-11" onClick={() => setAdding(adding === "checkup" ? null : "checkup")}>
          Add check-up date
        </Button>
      </div>

      {adding && <AddMark key={adding} initialKind={adding} onDone={() => setAdding(null)} />}

      <div className="flex flex-col gap-3">
        <h3 className="m-0 text-[15px] font-medium">Recent readings</h3>
        <MarkList />
      </div>
    </section>
  );
}
