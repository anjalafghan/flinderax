import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { rate } from "@/services/checkin";
import { useSaveDietWeek, type DietHabit, type DietOverview, type Week, type WeekInput } from "@/services/diet";
import { formatDay } from "@/utils/dates";
import { cn } from "@/utils/cn";

const SECTION = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
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
function Stepper({ habit, value, onChange }: { habit: DietHabit; value: number; onChange: (next: number) => void }) {
  return (
    <div className="flex items-center justify-between gap-4 border-t border-border py-4 first:border-t-0 first:pt-1">
      <div className="flex min-w-0 flex-col gap-1">
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

  const valueOf = (habit: DietHabit) =>
    days[habit.id] ?? week.entries.find((e) => e.habit_id === habit.id)?.days_hit ?? 0;
  const feelValue = feel === undefined ? week.feel : feel;

  const submit = () => {
    onSave({
      feel: feelValue ?? null,
      note: week.note,
      entries: habits.map((h) => ({ habit_id: h.id, days_hit: valueOf(h) })),
    });
  };

  return (
    <div className="flex flex-col gap-6">
      <div>
        {habits.map((h) => (
          <Stepper
            key={h.id}
            habit={h}
            value={valueOf(h)}
            onChange={(next) => setDays((d) => ({ ...d, [h.id]: next }))}
          />
        ))}
      </div>

      <fieldset className="m-0 flex flex-col gap-3 border-0 p-0">
        <legend className="mb-3 text-[15px] font-medium">How did eating feel?</legend>
        <div className="flex flex-wrap gap-2">
          {[{ value: 0, label: "Skip" }, ...FEEL].map((f) => {
            const selected = (feelValue ?? 0) === f.value;
            return (
              <label
                key={f.value}
                className={cn(
                  "inline-flex min-h-11 cursor-pointer items-center rounded-full border px-4 text-sm",
                  selected ? "border-primary bg-chip font-medium" : "border-border",
                )}
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

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <p role="status" aria-live="polite" className="m-0 text-sm text-muted-foreground">
          {saving ? "Saving..." : ""}
        </p>
        <Button onClick={submit} disabled={saving} className="self-start sm:self-auto">
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
              <span className={cn("rounded-full px-3 py-1", w.logged ? "tone-good" : "tone-quiet")}>
                {w.logged ? "Logged" : "Not logged"}
              </span>
              <span className="w-12 text-right tabular-nums text-muted-foreground">{rate(w.hit_bps)}</span>
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
  const current = overview.week.week_start;
  const [weekStart, setWeekStart] = useState(current);
  const active = overview.habits.filter((h) => h.active);
  // The last eight weeks before this one: the server's recent list starts with the current week.
  const pastWeeks = overview.recent.slice(1, 9);
  const isPast = weekStart !== current;
  // Past weeks come back blank from the server, so their form starts at zero; saving replaces what was there.
  const week: Week = isPast ? { week_start: weekStart, feel: null, note: "", entries: [] } : overview.week;

  const onSave = (input: WeekInput) =>
    save.mutate(
      { weekStart, week: input },
      {
        onSuccess: () => toast.success("Week saved."),
        onError: (e) => toast.error(e.message || "Couldn't save the week. Try again."),
      },
    );

  return (
    <section aria-labelledby="health-week-title" className={SECTION}>
      <div className="flex flex-col gap-2">
        <h2 id="health-week-title" className="m-0 font-display text-[32px] leading-none">
          This week
        </h2>
        <p className="m-0 text-sm text-muted-foreground">How many days you hit each habit. A rough count is fine.</p>
      </div>

      {active.length > 0 ? (
        <>
          {isPast && (
            <p className="m-0 text-sm text-muted-foreground">
              {`You're logging the week of ${formatDay(weekStart)}. Saving replaces what was there.`}
            </p>
          )}
          <WeekForm
            key={weekStart}
            habits={active}
            week={week}
            saving={save.isPending}
            onSave={onSave}
          />
        </>
      ) : (
        <p className="empty-note m-0">Pick up to three habits to track each week.</p>
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
