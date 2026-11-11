import { useId, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import { NAME_MAX, NOTE, PANEL, SECTION_TITLE, SHORT_NOTE_MAX, errorMessage } from "@/components/feature/people/ui";
import { CIRCLES, createPerson, usePeopleMutation, type Circle } from "@/services/people";

/** Add someone to the circle. Only a name is needed; a catch-up rhythm is optional. */
export function AddPersonForm() {
  const id = useId();
  const add = usePeopleMutation(createPerson);
  const [name, setName] = useState("");
  const [circle, setCircle] = useState<Circle>("friend");
  const [weeks, setWeeks] = useState("");
  const [note, setNote] = useState("");

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (trimmed === "") {
      toast.error("Add a name first.");
      return;
    }
    const every_days = weeks.trim() === "" ? null : Number(weeks) * 7;
    add.mutate(
      { name: trimmed, circle, every_days, note: note.trim() },
      {
        onSuccess: () => {
          toast.success(`${trimmed} is in your circle.`);
          setName("");
          setWeeks("");
          setNote("");
        },
        onError: (err) => toast.error(errorMessage(err)),
      },
    );
  };

  return (
    <section aria-labelledby="people-add-title" className={PANEL}>
      <header className="flex flex-col gap-2">
        <h2 id="people-add-title" className={SECTION_TITLE}>
          Add someone
        </h2>
        <p className={NOTE}>No phone numbers or addresses needed.</p>
      </header>
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
          <Label htmlFor={`${id}-weeks`}>Catch up every (weeks, optional)</Label>
          <Input
            id={`${id}-weeks`}
            type="number"
            inputMode="numeric"
            min={1}
            max={52}
            value={weeks}
            onChange={(e) => setWeeks(e.target.value)}
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor={`${id}-note`}>Note (optional)</Label>
          <Input id={`${id}-note`} maxLength={SHORT_NOTE_MAX} value={note} onChange={(e) => setNote(e.target.value)} />
        </div>
        <div className="sm:col-span-2">
          <Button type="submit" disabled={add.isPending}>
            Add to circle
          </Button>
          <ResetButton
            label="Clear"
            onReset={() => {
              setName("");
              setCircle("friend");
              setWeeks("");
              setNote("");
            }}
          />
        </div>
      </form>
    </section>
  );
}
