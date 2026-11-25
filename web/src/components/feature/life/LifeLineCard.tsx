import { useQuery } from "@tanstack/react-query";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";

import {
  freeLabel,
  getLifeline,
  lastsLabel,
  lifelineKey,
} from "@/services/lifeline";
import { cn } from "@/utils/cn";

/** The life line in one row on Today: when work becomes optional and how long the money lasts. */
export function LifeLineCard() {
  const { data } = useQuery({ queryKey: lifelineKey, queryFn: getLifeline });
  // Nothing to say until the user has entered their own details.
  if (!data || !data.saved_profile) return null;
  const m = data.line.marks;
  return (
    <Link
      to="/life"
      aria-label="Your life line"
      className="relative mt-6 flex min-h-[72px] flex-wrap items-center justify-between gap-x-6 gap-y-2 overflow-hidden rounded-[28px] border border-[hsl(var(--c-grape)/0.3)] px-6 py-5 no-underline transition-[border-color,box-shadow] hover:border-[hsl(var(--c-grape)/0.55)] hover:shadow-[0_16px_36px_-20px_hsl(var(--c-grape)/0.6)] sm:px-12"
      style={{
        background:
          "linear-gradient(110deg, hsl(var(--c-grape) / 0.14), hsl(var(--c-sky) / 0.1) 55%, hsl(var(--c-mint) / 0.1)), hsl(var(--card))",
      }}
    >
      {/* The long road, as a thin gradient along the bottom edge. */}
      <span
        aria-hidden
        className="absolute inset-x-0 bottom-0 h-1"
        style={{
          background:
            "linear-gradient(90deg, hsl(var(--c-rose)), hsl(var(--c-sun)), hsl(var(--c-mint)), hsl(var(--c-sky)), hsl(var(--c-grape)))",
        }}
      />
      <span className="flex flex-wrap items-baseline gap-x-8 gap-y-1">
        <span className="text-sm text-muted-foreground">
          Work becomes optional{" "}
          <span className="ml-1 font-display text-2xl leading-none text-[hsl(var(--foreground))] dark:text-[hsl(var(--c-grape))]">
            {freeLabel(m).toLowerCase()}
          </span>
        </span>
        <span className="text-sm text-muted-foreground">
          Money lasts{" "}
          <span
            className={cn(
              "ml-1 font-display text-2xl leading-none",
              m.runs_out_at_age === null
                ? "text-[hsl(var(--foreground))] dark:text-[hsl(var(--c-mint))]"
                : "text-over",
            )}
          >
            {lastsLabel(m).toLowerCase()}
          </span>
        </span>
      </span>
      <span className="inline-flex items-center gap-2 text-sm font-medium text-brand-ink">
        Life line <ArrowRight className="h-4 w-4" aria-hidden />
      </span>
    </Link>
  );
}
