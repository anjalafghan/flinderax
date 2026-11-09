import { useState } from "react";
import { FileUp, HeartPulse, Shield, Target, Users, Utensils, Wallet, type LucideIcon } from "lucide-react";
import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { useHabits, type Habit } from "@/services/habits";
import { formatDay } from "@/utils/dates";
import { cn } from "@/utils/cn";

const AREA_ICON: Record<Habit["area"], LucideIcon> = {
  data: FileUp,
  money: Wallet,
  food: Utensils,
  goals: Target,
  protection: Shield,
  health: HeartPulse,
  people: Users,
};

const CADENCE_LABEL: Record<Habit["cadence"], string> = {
  once: "Once",
  weekly: "Weekly",
  monthly: "Monthly",
  quarterly: "Every 3 months",
  yearly: "Yearly",
};

const STATUS: Record<Habit["status"], { label: string; tone: string }> = {
  overdue: { label: "Overdue", tone: "tone-over" },
  due_soon: { label: "Due soon", tone: "tone-warn" },
  ok: { label: "On track", tone: "tone-quiet" },
};

const SHOWN_BY_DEFAULT = 5;

function dueLine(h: Habit): string | null {
  if (!h.due_on) return null;
  return h.status === "overdue" ? `Overdue since ${formatDay(h.due_on)}` : `Due ${formatDay(h.due_on)}`;
}

function HabitRow({ habit }: { habit: Habit }) {
  const Icon = AREA_ICON[habit.area];
  const status = STATUS[habit.status];
  const due = dueLine(habit);
  return (
    <li className="ledger-row grid grid-cols-[36px_minmax(0,1fr)] items-start gap-x-4 gap-y-3 sm:grid-cols-[36px_minmax(0,1fr)_auto] sm:items-center">
      <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-chip">
        <Icon className="h-[18px] w-[18px]" strokeWidth={1.5} aria-hidden />
      </span>
      <span className="flex min-w-0 flex-col gap-1">
        <span className="text-[15px] font-medium leading-snug">{habit.title}</span>
        <span className="text-[13px] leading-snug text-muted-foreground">{habit.why}</span>
        <span className="text-[13px] text-muted-foreground">
          {[CADENCE_LABEL[habit.cadence], due].filter(Boolean).join(" · ")}
        </span>
      </span>
      <span className="col-start-2 flex flex-wrap items-center gap-2 sm:col-start-3 sm:flex-col sm:items-end">
        <span
          className={cn(
            "inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold",
            status.tone,
          )}
        >
          {status.label}
        </span>
        {habit.action && (
          <Link
            to={habit.action.to}
            className="inline-flex min-h-9 items-center rounded-full border border-border px-4 text-sm font-medium text-foreground no-underline transition-colors hover:bg-chip"
          >
            {habit.action.label}
          </Link>
        )}
      </span>
    </li>
  );
}

/** "Your habits": what to do and when, from the user's own numbers. Hidden when there is nothing to do. */
export function HabitsCard() {
  const { data, isLoading } = useHabits();
  const [expanded, setExpanded] = useState(false);

  if (!data) {
    // Errors are not critical here: with no data to show, the card simply stays hidden.
    return isLoading ? (
      <div className="mt-6 h-40 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading habits" />
    ) : null;
  }

  const habits = data.habits;
  if (habits.length === 0) return null;

  const hidden = habits.length - SHOWN_BY_DEFAULT;
  const visible = expanded ? habits : habits.slice(0, SHOWN_BY_DEFAULT);

  return (
    <section
      aria-labelledby="habits-title"
      className="mt-6 flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10"
    >
      <header className="flex flex-col gap-2">
        <h2 id="habits-title" className="m-0 font-display text-[32px] leading-none">
          Your habits
        </h2>
        <p className="m-0 text-sm text-muted-foreground">What to do and when, from your own numbers</p>
      </header>

      <ul className="ledger">
        {visible.map((h) => (
          <HabitRow key={h.key} habit={h} />
        ))}
      </ul>

      {hidden > 0 && (
        <Button
          variant="link"
          className="h-auto min-h-11 self-start p-0 text-sm"
          aria-expanded={expanded}
          onClick={() => setExpanded((v) => !v)}
        >
          {expanded ? "Show fewer" : `Show all ${habits.length}`}
        </Button>
      )}
    </section>
  );
}
