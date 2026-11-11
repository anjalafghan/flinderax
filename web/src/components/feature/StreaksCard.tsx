import { useEffect } from "react";
import { CalendarCheck, Flame, Heart, PiggyBank, type LucideIcon } from "lucide-react";

import { useStreaks, type Streak } from "@/services/streaks";
import { celebrate } from "@/utils/motion";
import { milestoneFor, milestoneKey, streakHeadline, streakLine, unitLabel, type StreakKind } from "@/utils/streaks";
import { cn } from "@/utils/cn";

const META: Record<StreakKind, { title: string; icon: LucideIcon }> = {
  logging: { title: "Keeping up", icon: Flame },
  saved: { title: "Saving", icon: PiggyBank },
  dates: { title: "Dates", icon: Heart },
};

function seen(key: string): boolean {
  try {
    return localStorage.getItem(key) !== null;
  } catch {
    return false;
  }
}

function remember(key: string): void {
  try {
    localStorage.setItem(key, "1");
  } catch {
    // Without storage the celebration may repeat; that is harmless.
  }
}

function Tile({ kind, streak }: { kind: StreakKind; streak: Streak }) {
  const { title, icon: Icon } = META[kind];
  const alive = streak.current > 0;
  return (
    <li className="flex min-w-0 flex-col gap-2 rounded-[18px] border border-border px-4 py-4">
      <div className="flex items-center gap-2 text-[13px] text-muted-foreground">
        <Icon className={cn("h-[18px] w-[18px]", alive && "streak-flame text-warn")} strokeWidth={1.6} aria-hidden />
        {title}
      </div>
      <p className="m-0 font-display text-[26px] leading-none">{streakHeadline(kind, streak)}</p>
      <p className="m-0 text-[13px] leading-snug text-muted-foreground">{streakLine(kind, streak)}</p>
      {streak.best > streak.current && streak.current > 0 && (
        <p className="m-0 text-xs text-muted-foreground">Best: {unitLabel(kind, streak.best)}</p>
      )}
    </li>
  );
}

/** Runs worth cheering for. Hidden until there is something to cheer. */
export function StreaksCard() {
  const { data } = useStreaks();

  // The first time a run lands on a milestone, celebrate it once.
  useEffect(() => {
    if (!data) return;
    const kinds: [StreakKind, Streak | null][] = [["logging", data.logging], ["saved", data.saved], ["dates", data.dates]];
    for (const [kind, s] of kinds) {
      const n = s ? milestoneFor(kind, s.current) : null;
      if (n === null) continue;
      const key = milestoneKey(kind, n);
      if (seen(key)) continue;
      remember(key);
      void celebrate("burst");
      break;
    }
  }, [data]);

  if (!data) return null;
  const tiles: [StreakKind, Streak][] = [["logging", data.logging], ["saved", data.saved]];
  if (data.dates) tiles.push(["dates", data.dates]);
  if (tiles.every(([, s]) => s.current === 0 && s.best === 0)) return null;

  return (
    <section aria-label="Your streaks" className="mb-6 w-full">
      <div className="mb-3 flex items-center gap-2">
        <CalendarCheck className="h-5 w-5 text-muted-foreground" strokeWidth={1.6} aria-hidden />
        <h2 className="m-0 font-display text-[26px] leading-none">Your streaks</h2>
      </div>
      <ul className="m-0 grid list-none grid-cols-[repeat(auto-fit,minmax(220px,1fr))] gap-3 p-0">
        {tiles.map(([kind, s]) => (
          <Tile key={kind} kind={kind} streak={s} />
        ))}
      </ul>
    </section>
  );
}
