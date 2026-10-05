import { CheckCircle2, ArrowRightLeft, Info } from "lucide-react";

import type { DashboardSummary } from "@/services/types";
import { cn } from "@/utils/cn";

const STYLE = {
  urgent: "border-rose-500/40 bg-rose-500/10",
  warn: "border-amber-500/40 bg-amber-500/10",
  info: "border-border bg-muted/30",
} as const;

export function ActionList({ actions }: { actions: DashboardSummary["actions"] }) {
  return (
    <section aria-label="Do this today" className="space-y-3">
      <h2 className="text-lg font-bold tracking-tight">Do this today</h2>
      {actions.length === 0 ? (
        <p className="flex items-center gap-2 rounded-xl border border-dashed p-4 text-sm text-muted-foreground">
          <CheckCircle2 className="h-4 w-4 text-emerald-500" /> Nothing to do. Every payment is covered.
        </p>
      ) : (
        <ul className="space-y-2">
          {actions.map((a) => (
            <li
              key={a.text}
              data-severity={a.severity}
              className={cn("flex items-start gap-3 rounded-xl border p-3 text-sm", STYLE[a.severity])}
            >
              {a.severity === "info" ? (
                <Info className="mt-0.5 h-4 w-4 shrink-0" />
              ) : (
                <ArrowRightLeft className="mt-0.5 h-4 w-4 shrink-0" />
              )}
              <span>{a.text}</span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
