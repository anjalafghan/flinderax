import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { rate } from "@/services/checkin";
import { ResetButton } from "@/components/ui/reset-button";
import {
  useClearDietWeek,
  useDietWeek,
  useSaveDietWeek,
  type DietHabit,
  type DietOverview,
  type Week,
  type WeekInput,
} from "@/services/diet";
import { formatDay } from "@/utils/dates";
import { cn } from "@/utils/cn";
import { hue, ink, type Hue } from "@/components/feature/life/hue";
import { Ring } from "@/components/feature/life/flair";

/** Each habit in turn gets a colour, so the rings are easy to tell apart. */
const HABIT_HUES: Hue[] = ["sun", "mint", "grape", "sky", "coral"];

/** How eating felt, from rough (coral) to great (grape). */
const FEEL_HUE: Record<number, Hue> = {
  1: "coral",
  2: "sun",
  3: "sky",
  4: "mint",
  5: "grape",
};

const SECTION =
  "relative flex flex-col gap-6 overflow-hidden rounded-[28px] border border-[hsl(var(--c-sun)/0.35)] bg-[linear-gradient(160deg,hsl(var(--c-sun)/0.13),transparent_55%),hsl(var(--card))] px-6 py-8 sm:px-12 sm:py-10";
const STEP_BTN =
  "inline-flex h-11 w-11 items-center justify-center rounded-full border border-border text-lg leading-none hover:bg-chip disabled:pointer-events-none disabled:opacity-40";

// 0 is "skip": the feel answer is optional and saves as null.
const FEEL: { value: number; label: string }[] = [
  { value: 1, label: "Rough" },
  { value: 2, label: "Meh" },
  { value: 3, label: "Okay" },
  { value: 4, label: "Good" },
  { value: 5, label: "Great" },
];

/** One habit: a minus and plus stepper for the days hit this week, 0 to 7. */
function Stepper({
  habit,
  value,
  color,
  onChange,
}: {
  habit: DietHabit;
  value: number;
  color: Hue;
  onChange: (next: number) => void;
}) {
  const pct =
    habit.target_days > 0 ? (value / habit.target_days) * 100 : 0;
  return (
    <div className="flex items-center justify-between gap-4 border-t border-border py-4 first:border-t-0 first:pt-1">
      <Ring pct={pct} color={hue(color)} />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <span className="text-[15px] font-medium">{habit.label}</span>
        <span className="text-[13px] text-muted-foreground">{`${value} of ${habit.target_days} days`}</span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <button
          type="button"
          aria-label={`Fewer days for ${habit.label}`}
          className={STEP_BTN}
          disabled={value <= 0}
          onClick={() => onChange(value - 1)}
        >
          −
        </button>
        <button
          type="button"
          aria-label={`More days for ${habit.label}`}
          className={STEP_BTN}
          disabled={value >= 7}
          onClick={() => onChange(value + 1)}
        >
          +
        </button>
      </div>
    </div>
  );
}

interface FormProps {
  habits: DietHabit[];
  week: Week;
  saving: boolean;
  onSave: (input: WeekInput) => void;
}

/**
 * The editable week. It is keyed by its week start in the parent, so switching weeks starts a fresh draft.
 * Local state holds only what the user has changed; everything else comes from the week the server sent.
 */
function WeekForm({ habits, week, saving, onSave }: FormProps) {
  const [days, setDays] = useState<Record<string, number>>({});
  const [feel, setFeel] = useState<number | null | undefined>(undefined);
  const [prep, setPrep] = useState<string | undefined>(undefined);
  const prepText =
    prep ??
    (week.prep_hours_x10 == null ? "" : String(week.prep_hours_x10 / 10));
  const prepHours = prepText.trim() === "" ? null : Number(prepText);
  const prepBad =
    prepHours !== null &&
    !(Number.isFinite(prepHours) && prepHours >= 0 && prepHours <= 168);

  const valueOf = (habit: DietHabit) =>
    days[habit.id] ??
    week.entries.find((e) => e.habit_id === habit.id)?.days_hit ??
    0;
  const feelValue = feel === undefined ? week.feel : feel;

  const submit = () => {
    onSave({
      feel: feelValue ?? null,
      note: week.note,
      prep_hours_x10: prepHours === null ? null : Math.round(prepHours * 10),
      entries: habits.map((h) => ({ habit_id: h.id, days_hit: valueOf(h) })),
    });
  };

  return (
    <div className="flex flex-col gap-6">
      <div>
        {habits.map((h, i) => (
          <Stepper
            key={h.id}
            habit={h}
            color={HABIT_HUES[i % HABIT_HUES.length]}
            value={valueOf(h)}
            onChange={(next) => setDays((d) => ({ ...d, [h.id]: next }))}
          />
        ))}
      </div>

      <fieldset className="m-0 flex flex-col gap-3 border-0 p-0">
        <legend className="mb-3 text-[15px] font-medium">
          How did eating feel?
        </legend>
        <div className="flex flex-wrap gap-2">
          {[{ value: 0, label: "Skip" }, ...FEEL].map((f) => {
            const selected = (feelValue ?? 0) === f.value;
            const fh = FEEL_HUE[f.value];
            return (
              <label
                key={f.value}
                className={cn(
                  "inline-flex min-h-11 cursor-pointer items-center rounded-full border px-4 text-sm transition-colors",
                  selected ? "font-semibold" : "border-border",
                )}
                style={
                  selected
                    ? fh
                      ? {
                          borderColor: hue(fh),
                          background: hue(fh, 0.2),
                          color: ink(fh),
                        }
                      : { borderColor: "hsl(var(--primary))" }
                    : undefined
                }
              >
                <input
                  type="radio"
                  name="feel"
                  value={f.value}
                  checked={selected}
                  onChange={() => setFeel(f.value === 0 ? null : f.value)}
                  className="sr-only"
                />
                {f.label}
              </label>
            );
          })}
        </div>
      </fieldset>

      <div className="flex flex-col gap-2">
        <label htmlFor="week-prep-hours" className="text-[15px] font-medium">
          Hours on your job search or learning
        </label>
        <Input
          id="week-prep-hours"
          inputMode="decimal"
          placeholder="0"
          value={prepText}
          onChange={(e) => setPrep(e.target.value)}
          aria-invalid={prepBad}
          className="sm:max-w-[10rem]"
        />
        <p
          className={cn(
            "m-0 text-[13px]",
            prepBad ? "text-destructive" : "text-muted-foreground",
          )}
        >
          {prepBad
            ? "Enter hours as a number, like 3 or 4.5."
            : "Leave blank if you are not looking."}
        </p>
      </div>

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <p
          role="status"
          aria-live="polite"
          className="m-0 text-sm text-muted-foreground"
        >
          {saving ? "Saving..." : ""}
        </p>
        <Button
          onClick={submit}
          disabled={saving || prepBad}
          className="self-start sm:self-auto"
        >
          Save week
        </Button>
      </div>
    </div>
  );
}

/** Weeks already logged, newest first, with the share of target days hit. */
function RecentWeeks({ recent }: { recent: DietOverview["recent"] }) {
  return (
    <div className="flex flex-col gap-3">
      <h3 className="m-0 text-[15px] font-medium">Last 12 weeks</h3>
      <ol className="ledger">
        {recent.map((w) => (
          <li key={w.week_start} className="ledger-row">
            <span className="text-[15px]">{`Week of ${formatDay(w.week_start)}`}</span>
            <span className="flex shrink-0 items-center gap-3 text-[13px]">
              <span
                className={cn(
                  "rounded-full px-3 py-1",
                  w.logged ? "tone-good" : "tone-quiet",
                )}
              >
                {w.logged ? "Saved" : "No entry"}
              </span>
              <span
                aria-hidden
                className="hidden h-2 w-20 overflow-hidden rounded-full bg-chip sm:block"
              >
                <span
                  className="block h-full rounded-full"
                  style={{
                    width: `${Math.min(100, (w.hit_bps ?? 0) / 100)}%`,
                    background: `linear-gradient(90deg, ${hue("sun")}, ${hue("mint")})`,
                  }}
                />
              </span>
              <span className="w-12 text-right tabular-nums text-muted-foreground">
                {rate(w.hit_bps)}
              </span>
            </span>
          </li>
        ))}
      </ol>
    </div>
  );
}

/** This week's habit days and feel, plus a way to log a past week. */
export function WeekCard({ overview }: { overview: DietOverview }) {
  const save = useSaveDietWeek();
  const clearWeek = useClearDietWeek();
  const current = overview.week.week_start;
  const [weekStart, setWeekStart] = useState(current);
  const active = overview.habits.filter((h) => h.active);
  // The last eight weeks before this one: the server's recent list starts with the current week.
  const pastWeeks = overview.recent.slice(1, 9);
  const isPast = weekStart !== current;
  const selectedWeekLogged =
    overview.recent.find((w) => w.week_start === weekStart)?.logged ?? false;
  // A past week loads what was logged then, so filling it in starts from that.
  const past = useDietWeek(weekStart, isPast);
  const week: Week = isPast
    ? (past.data ?? {
        week_start: weekStart,
        feel: null,
        note: "",
        entries: [],
      })
    : overview.week;

  const onSave = (input: WeekInput) =>
    save.mutate(
      { weekStart, week: input },
      {
        onSuccess: () => toast.success("Week saved."),
        onError: (e) =>
          toast.error(e.message || "Couldn't save the week. Try again."),
      },
    );

  return (
    <section aria-labelledby="health-week-title" className={SECTION}>
      <div className="flex flex-col gap-2">
        <h2
          id="health-week-title"
          className="m-0 flex items-center gap-3 font-display text-[32px] leading-none"
        >
          <span
            aria-hidden
            className="h-3.5 w-3.5 shrink-0 rounded-full"
            style={{
              background: "hsl(var(--c-sun))",
              boxShadow: "0 0 0 4px hsl(var(--c-sun) / 0.22)",
            }}
          />
          {isPast ? `Week of ${formatDay(weekStart)}` : "This week"}
        </h2>
        <p className="m-0 text-sm text-muted-foreground">
          Add a rough count when it suits you. You can edit this week later; an
          empty week simply means there is no entry.
        </p>
        <span
          className={cn(
            "self-start rounded-full px-3 py-1 text-[13px]",
            selectedWeekLogged ? "tone-good" : "tone-quiet",
          )}
        >
          {selectedWeekLogged ? "Saved" : "No entry yet"}
        </span>
      </div>

      {active.length > 0 ? (
        <>
          {isPast && past.isLoading && (
            <p className="m-0 text-sm text-muted-foreground" role="status">
              Loading this week…
            </p>
          )}
          <WeekForm
            key={`${weekStart}:${past.dataUpdatedAt}`}
            habits={active}
            week={week}
            saving={save.isPending}
            onSave={onSave}
          />
        </>
      ) : (
        <p className="empty-note m-0">
          Pick up to five habits to track each week.
        </p>
      )}

      {active.length > 0 && (
        <div>
          <ResetButton
            label={
              isPast ? "Reset: clear this week" : "Reset: clear this week's log"
            }
            disabled={clearWeek.isPending}
            confirm="Clear everything logged for this week (habit days, how it felt, the note)? It will no longer count as logged."
            onReset={() =>
              clearWeek.mutate(weekStart, {
                onSuccess: () => toast.success("Week cleared."),
                onError: (e) =>
                  toast.error(
                    e.message || "Couldn't clear the week. Try again.",
                  ),
              })
            }
          />
        </div>
      )}

      <div className="flex flex-col gap-3">
        <label htmlFor="health-week-pick" className="text-[15px] font-medium">
          Log a past week
        </label>
        <Select
          id="health-week-pick"
          value={weekStart}
          onChange={(e) => setWeekStart(e.target.value)}
          className="sm:max-w-xs"
        >
          <option value={current}>This week</option>
          {pastWeeks.map((w) => (
            <option key={w.week_start} value={w.week_start}>
              {`Week of ${formatDay(w.week_start)}`}
            </option>
          ))}
        </Select>
      </div>

      <RecentWeeks recent={overview.recent} />
    </section>
  );
}
