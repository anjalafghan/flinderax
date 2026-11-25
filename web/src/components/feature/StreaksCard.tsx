import { useEffect } from "react";
import {
  CalendarCheck,
  Flame,
  Heart,
  PiggyBank,
  Upload,
  type LucideIcon,
} from "lucide-react";

import { useStreaks, type Streak } from "@/services/streaks";
import { celebrate } from "@/utils/motion";
import {
  milestoneFor,
  milestoneKey,
  streakHeadline,
  streakLine,
  unitLabel,
  type StreakKind,
} from "@/utils/streaks";
import { cn } from "@/utils/cn";
import { RewardChips } from "@/components/feature/RewardChips";
import { useRewards } from "@/services/rewards";
import { forStreak } from "@/utils/rewards";
import ShinyText from "@/components/bits/ShinyText";
import { useTheme } from "@/context/ThemeContext";

/**
 * Each run has its own colour: keeping up is fire, saving mint, showing up sky, dates rose.
 * `ink` is the headline colour (deep on paper, bright at night) and `shine` the glint that runs over it.
 */
const META: Record<
  StreakKind,
  {
    title: string;
    icon: LucideIcon;
    c: string;
    ink: [string, string];
    shine: string;
  }
> = {
  logging: {
    title: "Keeping up",
    icon: Flame,
    c: "coral",
    ink: ["#c2410c", "#ffb37a"],
    shine: "#c9bfae",
  },
  saved: {
    title: "Saving",
    icon: PiggyBank,
    c: "mint",
    ink: ["#047857", "#6ee7b7"],
    shine: "#d1fae5",
  },
  dates: {
    title: "Dates",
    icon: Heart,
    c: "rose",
    ink: ["#8a8073", "#c9bfae"],
    shine: "#ffe4e6",
  },
  activity: {
    title: "Showing up",
    icon: Upload,
    c: "sky",
    ink: ["#8a8073", "#c9bfae"],
    shine: "#e0f2fe",
  },
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
  const { title, icon: Icon, c, ink, shine } = META[kind];
  const alive = streak.current > 0;
  const { data: rewards } = useRewards();
  const { theme } = useTheme();
  const fire = alive && kind === "logging";
  return (
    <li
      className={cn(
        "relative flex min-w-0 flex-col gap-2 overflow-hidden rounded-[18px] border px-4 py-4",
        !alive && "border-dashed",
      )}
      style={{
        borderColor: `hsl(var(--c-${c}) / ${alive ? 0.45 : 0.3})`,
        background: fire
          ? "linear-gradient(160deg, hsl(var(--c-sun) / 0.22), hsl(var(--c-coral) / 0.16) 45%, hsl(var(--c-rose) / 0.12))"
          : `linear-gradient(160deg, hsl(var(--c-${c}) / ${alive ? 0.16 : 0.06}), transparent 70%)`,
        boxShadow: alive
          ? `0 14px 30px -20px hsl(var(--c-${c}))`
          : undefined,
      }}
    >
      <div className="flex items-center gap-2 text-[13px] font-medium text-muted-foreground">
        <span
          aria-hidden
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-full",
            alive ? "text-background" : "",
          )}
          style={{
            background: alive
              ? fire
                ? "linear-gradient(160deg, hsl(var(--c-sun)), hsl(var(--c-coral)) 55%, hsl(var(--c-rose)))"
                : `hsl(var(--c-${c}))`
              : `hsl(var(--c-${c}) / 0.15)`,
            color: alive ? undefined : `hsl(var(--c-${c}))`,
          }}
        >
          <Icon
            className={cn("h-4 w-4", alive && "streak-flame")}
            strokeWidth={1.8}
          />
        </span>
        {title}
      </div>
      <p className="m-0 font-display text-[26px] leading-none">
        {alive ? (
          <ShinyText
            text={streakHeadline(kind, streak)}
            color={ink[theme === "dark" ? 1 : 0]}
            shineColor={shine}
            speed={3}
            delay={1.5}
          />
        ) : (
          streakHeadline(kind, streak)
        )}
      </p>
      <p className="m-0 text-[13px] leading-snug text-muted-foreground">
        {streakLine(kind, streak)}
      </p>
      {streak.best > streak.current && streak.current > 0 && (
        <p className="m-0 text-xs text-muted-foreground">
          Best: {unitLabel(kind, streak.best)}
        </p>
      )}
      <RewardChips rewards={forStreak(rewards?.rewards, kind)} />
    </li>
  );
}

/** Runs worth cheering for. Hidden until there is something to cheer. */
export function StreaksCard() {
  const { data } = useStreaks();

  // The first time a run lands on a milestone, celebrate it once.
  useEffect(() => {
    if (!data) return;
    const kinds: [StreakKind, Streak | null][] = [
      ["logging", data.logging],
      ["saved", data.saved],
      ["dates", data.dates],
      ["activity", data.activity],
    ];
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
  const tiles: [StreakKind, Streak][] = [
    ["logging", data.logging],
    ["saved", data.saved],
    ["activity", data.activity],
  ];
  if (data.dates) tiles.push(["dates", data.dates]);
  if (tiles.every(([, s]) => s.current === 0 && s.best === 0)) return null;

  return (
    <section aria-label="Your streaks" className="my-6 w-full">
      <div className="mb-3 flex items-center gap-2">
        <CalendarCheck
          className="h-6 w-6 text-brand"
          strokeWidth={1.6}
          aria-hidden
        />
        <h2 className="brand-text m-0 font-display text-[32px] leading-none">
          Your streaks
        </h2>
      </div>
      <ul className="m-0 grid list-none grid-cols-[repeat(auto-fit,minmax(220px,1fr))] gap-3 p-0">
        {tiles.map(([kind, s]) => (
          <Tile key={kind} kind={kind} streak={s} />
        ))}
      </ul>
    </section>
  );
}
