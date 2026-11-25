import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import {
  NOTE,
  SECTION_TITLE,
  errorMessage,
} from "@/components/feature/people/ui";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ResetButton } from "@/components/ui/reset-button";
import {
  addFact,
  deleteFact,
  useFacts,
  usePeopleMutation,
} from "@/services/people";

export const FACT_MAX = 120;

/** Short things to remember about one person: allergies, kids' names, what they love. */
export function PersonFacts({ personId }: { personId: string }) {
  const id = useId();
  const all = useFacts();
  const add = usePeopleMutation(addFact);
  const remove = usePeopleMutation(deleteFact);
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const mine = (all.data ?? []).filter((f) => f.person_id === personId);

  const reset = () => {
    setText("");
    setError(null);
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const t = text.trim();
    if (!t) return setError("Write something to remember.");
    if (t.length > FACT_MAX)
      return setError(`Keep it to ${FACT_MAX} characters.`);
    setError(null);
    add.mutate(
      { personId, text: t },
      { onSuccess: reset, onError: (err) => setError(errorMessage(err)) },
    );
  };

  return (
    <section aria-labelledby={`${id}-title`} className="flex flex-col gap-5">
      <h2 id={`${id}-title`} className={SECTION_TITLE}>
        Things to remember
      </h2>
      {mine.length > 0 ? (
        <ul className="ledger">
          {mine.map((f) => (
            <li key={f.id} className="ledger-row">
              <span className="min-w-0 text-[15px] font-medium">{f.text}</span>
              <Button
                variant="ghost"
                className="min-h-11 shrink-0"
                disabled={remove.isPending}
                aria-label={`Remove: ${f.text}`}
                onClick={() => {
                  if (!window.confirm("Remove this?")) return;
                  remove.mutate(f.id, {
                    onError: (err) => toast.error(errorMessage(err)),
                  });
                }}
              >
                Remove
              </Button>
            </li>
          ))}
        </ul>
      ) : (
        <p className={NOTE}>Nothing noted yet.</p>
      )}
      <form
        onSubmit={submit}
        className="flex flex-wrap items-end gap-3"
        noValidate
      >
        <div className="field min-w-0 flex-1">
          <Label htmlFor={`${id}-text`}>Add something to remember</Label>
          <Input
            id={`${id}-text`}
            value={text}
            maxLength={FACT_MAX}
            onChange={(e) => setText(e.target.value)}
            placeholder="e.g. Allergic to peanuts"
          />
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
