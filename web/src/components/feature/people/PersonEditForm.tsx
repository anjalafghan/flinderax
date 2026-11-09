import { useId, useState, type FormEvent } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";

import { ConfirmationModal } from "@/components/ui/ConfirmationModal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { NAME_MAX, NOTE, SHORT_NOTE_MAX, errorMessage } from "@/components/feature/people/ui";
import {
  CIRCLES,
  deletePerson,
  everyLabel,
  personKey,
  updatePerson,
  usePeopleMutation,
  type Circle,
  type PersonRow,
} from "@/services/people";

/** Edit a person's name, circle, rhythm and note; archive them; or delete them after a confirm. */
export function PersonEditForm({ person }: { person: PersonRow }) {
  const id = useId();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const save = usePeopleMutation((p: { id: string; patch: Parameters<typeof updatePerson>[1] }) =>
    updatePerson(p.id, p.patch),
  );
  const remove = usePeopleMutation(deletePerson);
  const [name, setName] = useState(person.name);
  const [circle, setCircle] = useState<Circle>(person.circle);
  // A rhythm in whole weeks prefills; one in days stays blank, with the current rhythm shown beside it.
  const wholeWeeks = person.every_days !== null && person.every_days % 7 === 0;
  const [weeks, setWeeks] = useState(wholeWeeks ? String(person.every_days! / 7) : "");
  // Only a changed rhythm is sent, so a rhythm set in days (not a whole number of weeks) survives an unrelated save.
  const [weeksChanged, setWeeksChanged] = useState(false);
  const [note, setNote] = useState(person.note);
  const [confirming, setConfirming] = useState(false);

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (name.trim() === "") {
      toast.error("Add a name first.");
      return;
    }
    const every_days = !weeksChanged ? person.every_days : weeks.trim() === "" ? null : Number(weeks) * 7;
    save.mutate(
      { id: person.id, patch: { name: name.trim(), circle, every_days, note: note.trim() } },
      {
        onSuccess: () => toast.success("Saved."),
        onError: (err) => toast.error(errorMessage(err)),
      },
    );
  };

  const toggleArchive = () =>
    save.mutate(
      { id: person.id, patch: { archived: !person.archived } },
      {
        onSuccess: () => toast.success(person.archived ? `${name} is back in your circle.` : `${name} is archived.`),
        onError: (err) => toast.error(errorMessage(err)),
      },
    );

  const doDelete = () =>
    remove.mutate(person.id, {
      onSuccess: () => {
        toast.success("Removed.");
        // The person's page is gone, so drop its query rather than let it refetch a 404.
        qc.removeQueries({ queryKey: personKey(person.id) });
        navigate("/people");
      },
      onError: (err) => {
        setConfirming(false);
        toast.error(errorMessage(err));
      },
    });

  return (
    <section aria-labelledby="person-edit-title" className="flex flex-col gap-5 border-t border-border pt-8">
      <h2 id="person-edit-title" className="m-0 font-display text-[32px] leading-none">
        Edit
      </h2>
      <form onSubmit={submit} className="grid gap-4 sm:grid-cols-2">
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-name`}>Name</Label>
          <Input id={`${id}-name`} maxLength={NAME_MAX} value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-circle`}>Circle</Label>
          <Select id={`${id}-circle`} value={circle} onChange={(e) => setCircle(e.target.value as Circle)}>
            {CIRCLES.map((c) => (
              <option key={c.key} value={c.key}>
                {c.label}
              </option>
            ))}
          </Select>
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-weeks`}>Catch up every (weeks, blank for none)</Label>
          {person.every_days !== null && !wholeWeeks && (
            <span className="text-[13px] text-muted-foreground">{`Currently ${everyLabel(person.every_days)}. Enter weeks to change it.`}</span>
          )}
          <Input
            id={`${id}-weeks`}
            type="number"
            inputMode="numeric"
            min={1}
            max={52}
            value={weeks}
            onChange={(e) => {
              setWeeks(e.target.value);
              setWeeksChanged(true);
            }}
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-note`}>Note</Label>
          <Input id={`${id}-note`} maxLength={SHORT_NOTE_MAX} value={note} onChange={(e) => setNote(e.target.value)} />
        </div>
        <p className={`${NOTE} sm:col-span-2`}>No phone numbers or addresses needed.</p>
        <div className="flex flex-wrap gap-2 sm:col-span-2">
          <Button type="submit" disabled={save.isPending}>
            Save
          </Button>
          <Button type="button" variant="outline" disabled={save.isPending} onClick={toggleArchive}>
            {person.archived ? "Bring back to circle" : "Archive"}
          </Button>
          <Button type="button" variant="destructive" onClick={() => setConfirming(true)}>
            Delete {person.name}
          </Button>
        </div>
      </form>

      <ConfirmationModal
        isOpen={confirming}
        onClose={() => setConfirming(false)}
        onConfirm={doDelete}
        title={`Delete ${person.name}?`}
        description="Their dates, gifts and catch-ups are removed too. This can't be undone."
        confirmText="Delete"
        variant="destructive"
        isPending={remove.isPending}
      />
    </section>
  );
}
