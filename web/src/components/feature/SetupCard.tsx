import { ArrowRight, Check } from "lucide-react";
import { Link } from "react-router-dom";

import type { DashboardSummary } from "@/services/types";
import { useAccounts, useIncomes } from "@/hooks/usePlanner";
import { cn } from "@/utils/cn";

/** Three steps, each a single tap away, ticking off as the data arrives. */
export function SetupCard({ summary }: { summary: DashboardSummary }) {
  const { data: incomes } = useIncomes();
  const { data: accounts } = useAccounts();

  const steps = [
    {
      to: "/plan#income",
      title: "Add your salary",
      hint: "Your next payday decides how far today's number reaches.",
      done: (incomes?.length ?? 0) > 0,
    },
    {
      to: "/accounts",
      title: "Add an account",
      hint: "One balance from your bank app is enough to begin.",
      done: (accounts?.length ?? 0) > 0,
    },
    {
      to: "/spending",
      title: "Log your first spend",
      hint: "Paste a bank text, upload a statement, or type it.",
      done: summary.spending.tracked,
    },
  ];
  const doneCount = steps.filter((s) => s.done).length;
  const next = steps.findIndex((s) => !s.done);
  return (
    <section
      aria-labelledby="setup-title"
      className="glass flex flex-col gap-6 rounded-[28px] px-6 pb-7 pt-7 sm:px-10 sm:pb-9 sm:pt-10"
    >
      <div className="flex flex-col gap-2">
        <p className="eyebrow m-0">
          Getting started · {doneCount} of {steps.length}
        </p>
        <h1
          id="setup-title"
          className="brand-text m-0 font-display text-[40px] font-light leading-[1.02] tracking-[-0.02em] sm:text-[56px]"
        >
          Let's work out what's safe to spend.
        </h1>
        <p className="m-0 text-base text-muted-foreground">
          Three small steps. Until then there's nothing to judge, so joy won't
          guess.
        </p>
      </div>

      <div
        role="progressbar"
        aria-label="Setup progress"
        aria-valuemin={0}
        aria-valuemax={steps.length}
        aria-valuenow={doneCount}
        className="h-1.5 overflow-hidden rounded-full bg-secondary"
      >
        <span
          className="block h-full rounded-full bg-foreground transition-[width] duration-700 ease-out"
          style={{
            width: `${Math.max(doneCount === 0 ? 4 : 0, (doneCount / steps.length) * 100)}%`,
          }}
        />
      </div>

      <ol className="m-0 flex list-none flex-col gap-2.5 p-0">
        {steps.map((s, i) => {
          const body = (
            <>
              <span
                aria-hidden
                className={cn(
                  "inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full border text-sm font-semibold",
                  s.done
                    ? "border-good/30 bg-good/15 text-good"
                    : "border-border bg-secondary text-muted-foreground",
                )}
              >
                {s.done ? <Check className="h-4 w-4" strokeWidth={2.5} /> : i + 1}
              </span>
              <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span
                  className={cn(
                    "text-[15px] font-medium",
                    s.done && "line-through decoration-muted-foreground/50",
                  )}
                >
                  {s.title}
                </span>
                <span className="text-[13px] leading-snug text-muted-foreground">
                  {s.hint}
                </span>
              </span>
              {!s.done && (
                <ArrowRight
                  className={cn(
                    "h-4 w-4 shrink-0",
                    i === next ? "text-brand-ink" : "text-muted-foreground",
                  )}
                  strokeWidth={1.5}
                  aria-hidden
                />
              )}
            </>
          );
          const label = `${s.title}${s.done ? " (done)" : ""}`;
          return (
            <li key={s.to}>
              <Link
                to={s.to}
                viewTransition
                aria-label={label}
                className={cn(
                  "flex min-h-[68px] items-center gap-4 rounded-[18px] border bg-card px-4 no-underline transition-colors hover:bg-secondary/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  i === next && "border-foreground/35 bg-secondary/30",
                  s.done && "border-good/30 opacity-70",
                )}
              >
                {body}
              </Link>
            </li>
          );
        })}
      </ol>
    </section>
  );
}
