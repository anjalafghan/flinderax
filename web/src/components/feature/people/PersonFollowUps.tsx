import { useId, useState, type FormEvent } from "react";

import { FollowUpRow } from "@/components/feature/people/FollowUpRow";
import { NOTE, SECTION_TITLE, errorMessage } from "@/components/feature/people/ui";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ResetButton } from "@/components/ui/reset-button";
import { addFollowUp, useFollowUps, usePeopleMutation } from "@/services/people";

export const FOLLOWUP_MAX = 120;

/** The to-do list for one person: things to ask, send or remember, with an optional due day. */
export function PersonFollowUps({ personId }: { personId: string }) {
  const id = useId();
  const all = useFollowUps();
  const add = usePeopleMutation(addFollowUp);
  const [text, setText] = useState("");
  const [due, setDue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const mine = (all.data?.open ?? []).filter((f) => f.person_id === personId);
  const done = (all.data?.done ?? []).filter((f) => f.person_id === personId);
  const today = new Date().toLocaleDateString("en-CA");

  const reset = () => {
    setText("");
    setDue("");
    setError(null);
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const t = text.trim();
    if (!t) return setError("Write what to follow up on.");
    if (t.length > FOLLOWUP_MAX) return setError(`Keep it to ${FOLLOWUP_MAX} characters.`);
    setError(null);
    add.mutate(
      { personId, text: t, due_on: due || undefined },
      { onSuccess: reset, onError: (err) => setError(errorMessage(err)) },
    );
  };

  return (
    <section aria-labelledby={`${id}-title`} className="flex flex-col gap-5">
      <h2 id={`${id}-title`} className={SECTION_TITLE}>
        Follow-ups
      </h2>
      {mine.length > 0 ? (
        <ul className="ledger">
          {mine.map((f) => (
            <FollowUpRow key={f.id} f={f} />
          ))}
        </ul>
      ) : (
        <p className={NOTE}>Nothing to follow up on.</p>
      )}
      {done.length > 0 && (
        <details className="text-sm">
          <summary className="cursor-pointer font-medium">Done recently ({done.length})</summary>
          <ul className="ledger mt-2">
            {done.map((f) => (
              <FollowUpRow key={f.id} f={f} />
            ))}
          </ul>
        </details>
      )}
      <form onSubmit={submit} className="flex flex-wrap items-end gap-3" noValidate>
        <div className="field min-w-0 flex-1">
          <Label htmlFor={`${id}-text`}>Add a follow-up</Label>
          <Input id={`${id}-text`} value={text} maxLength={FOLLOWUP_MAX} onChange={(e) => setText(e.target.value)} placeholder="e.g. Ask how the surgery went" />
        </div>
        <div className="field">
          <Label htmlFor={`${id}-due`}>By (optional)</Label>
          <Input id={`${id}-due`} type="date" min={today} value={due} onChange={(e) => setDue(e.target.value)} />
        </div>
        <div className="flex gap-2">
          <ResetButton onReset={reset} />
          <Button type="submit" disabled={add.isPending}>
            Add
          </Button>
        </div>
        {error && (
          <p role="alert" className="w-full text-sm text-destructive">
            {error}
          </p>
        )}
      </form>
    </section>
  );
}
