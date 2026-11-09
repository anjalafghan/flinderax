import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";
import {
  useAddDietHabit,
  useRemoveDietHabit,
  useUpdateDietHabit,
  type DietOverview,
  type NewHabit,
} from "@/services/diet";

const SECTION = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
const MAX_HABITS = 3;
const DAYS = [1, 2, 3, 4, 5, 6, 7];

/** Presets, custom habits, a days target for each, and a way to stop tracking one. */
export function HabitPicker({ overview }: { overview: DietOverview }) {
  const add = useAddDietHabit();
  const update = useUpdateDietHabit();
  const remove = useRemoveDietHabit();
  const [label, setLabel] = useState("");
  const [target, setTarget] = useState("5");
  const [error, setError] = useState<string | null>(null);

  const active = overview.habits.filter((h) => h.active);
  const presets = overview.presets.filter((p) => !active.some((h) => h.preset_key === p.key));
  const full = active.length >= MAX_HABITS;

  const onError = (e: Error) => toast.error(e.message || "Couldn't save that. Try again.");

  const addHabit = (input: NewHabit) => {
    setError(null);
    add.mutate(input, {
      onSuccess: () => toast.success(`Now tracking ${input.label}.`),
      onError,
    });
  };

  const addCustom = () => {
    const text = label.trim();
    const days = Number(target);
    if (text.length < 1 || text.length > 60) {
      setError("Give the habit a name of up to 60 letters.");
      return;
    }
    addHabit({ label: text, target_days: days });
    setLabel("");
  };

  return (
    <section aria-labelledby="health-habits-title" className={SECTION}>
      <div className="flex flex-col gap-2">
        <h2 id="health-habits-title" className="m-0 font-display text-[32px] leading-none">
          Your habits
        </h2>
        <p className="m-0 text-sm text-muted-foreground">
          {`Up to ${MAX_HABITS} at a time. Each one is a target number of days a week.`}
        </p>
      </div>

      {active.length > 0 && (
        <ul className="ledger">
          {active.map((h) => (
            <li key={h.id} className="ledger-row flex-wrap">
              <span className="text-[15px] font-medium">{h.label}</span>
              <span className="flex flex-wrap items-center gap-3">
                <Label htmlFor={`habit-target-${h.id}`} className="text-[13px] text-muted-foreground">
                  Days a week
                </Label>
                <Select
                  id={`habit-target-${h.id}`}
                  className="h-11 w-20"
                  value={String(h.target_days)}
                  onChange={(e) => update.mutate({ id: h.id, patch: { target_days: Number(e.target.value) } }, { onError })}
                >
                  {DAYS.map((d) => (
                    <option key={d} value={d}>
                      {d}
                    </option>
                  ))}
                </Select>
                <Button
                  variant="ghost"
                  className="h-11 px-3"
                  aria-label={`Stop tracking ${h.label}`}
                  onClick={() =>
                    remove.mutate(h.id, {
                      onSuccess: () => toast.success(`Stopped tracking ${h.label}.`),
                      onError,
                    })
                  }
                >
                  Stop
                </Button>
              </span>
            </li>
          ))}
        </ul>
      )}

      {full ? (
        <p className="empty-note m-0">You can track up to three at a time. Stop one to add another.</p>
      ) : (
        <>
          {presets.length > 0 && (
            <div className="flex flex-col gap-3">
              <h3 className="m-0 text-[15px] font-medium">Start from one of these</h3>
              <div className="flex flex-wrap gap-2">
                {presets.map((p) => (
                  <Button
                    key={p.key}
                    variant="outline"
                    className="h-11"
                    disabled={add.isPending}
                    onClick={() => addHabit({ label: p.label, preset_key: p.key, target_days: p.target_days })}
                  >
                    {`${p.label}, ${p.target_days} days`}
                  </Button>
                ))}
              </div>
            </div>
          )}

          <form
            className="form-grid"
            onSubmit={(e) => {
              e.preventDefault();
              addCustom();
            }}
          >
            <div className="field">
              <Label htmlFor="habit-custom-label">Or your own habit</Label>
              <Input
                id="habit-custom-label"
                value={label}
                maxLength={60}
                placeholder="e.g. Stretch after work"
                onChange={(e) => setLabel(e.target.value)}
              />
            </div>
            <div className="field">
              <Label htmlFor="habit-custom-target">Days a week</Label>
              <Select id="habit-custom-target" value={target} onChange={(e) => setTarget(e.target.value)}>
                {DAYS.map((d) => (
                  <option key={d} value={d}>
                    {d}
                  </option>
                ))}
              </Select>
            </div>
            <div className="form-actions">
              {error && (
                <p role="alert" className="m-0 mr-auto text-sm text-destructive">
                  {error}
                </p>
              )}
              <Button type="submit" disabled={add.isPending}>
                Add habit
              </Button>
            </div>
          </form>
        </>
      )}
    </section>
  );
}
