import { useQuery } from "@tanstack/react-query";
import { ArrowRight } from "lucide-react";
import { Link } from "react-router-dom";

import { freeLabel, getLifeline, lastsLabel, lifelineKey } from "@/services/lifeline";
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
      className="mt-6 flex min-h-[72px] flex-wrap items-center justify-between gap-x-6 gap-y-2 rounded-[28px] border border-border bg-card px-6 py-5 no-underline transition-colors hover:bg-[hsl(var(--chip))] sm:px-12"
    >
      <span className="flex flex-wrap items-baseline gap-x-8 gap-y-1">
        <span className="text-sm text-muted-foreground">
          Work becomes optional <span className="ml-1 font-display text-2xl leading-none text-foreground">{freeLabel(m).toLowerCase()}</span>
        </span>
        <span className="text-sm text-muted-foreground">
          Money lasts{" "}
          <span className={cn("ml-1 font-display text-2xl leading-none", m.runs_out_at_age === null ? "text-foreground" : "text-over")}>
            {lastsLabel(m).toLowerCase()}
          </span>
        </span>
      </span>
      <span className="inline-flex items-center gap-2 text-sm font-medium">
        Life line <ArrowRight className="h-4 w-4" aria-hidden />
      </span>
    </Link>
  );
}
