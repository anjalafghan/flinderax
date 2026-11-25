import { Minus, Plus } from "lucide-react";

import { cn } from "@/utils/cn";

interface StepperProps {
  label: string;
  value: number;
  onChange: (n: number) => void;
  min: number;
  max: number;
  /** What to show between the buttons, e.g. "Every 3 months". */
  format: (n: number) => string;
  className?: string;
}

/** A number picked with − and +: two 44px taps instead of a twelve-line dropdown. */
export function Stepper({
  label,
  value,
  onChange,
  min,
  max,
  format,
  className,
}: StepperProps) {
  const btn =
    "inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-full border border-brand/30 bg-brand-tint text-brand-ink transition-[transform,background-color] hover:bg-brand hover:text-brand-fg active:scale-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:opacity-40 disabled:hover:bg-brand-tint disabled:hover:text-brand-ink";
  return (
    <div
      role="group"
      aria-label={label}
      className={cn("flex items-center gap-3", className)}
    >
      <button
        type="button"
        className={btn}
        aria-label={`Fewer: ${label}`}
        disabled={value <= min}
        onClick={() => onChange(value - 1)}
      >
        <Minus className="h-4 w-4" aria-hidden />
      </button>
      <output
        aria-live="polite"
        className="min-w-[8.5rem] text-center text-[15px] font-semibold tabular-nums text-brand-ink"
      >
        {format(value)}
      </output>
      <button
        type="button"
        className={btn}
        aria-label={`More: ${label}`}
        disabled={value >= max}
        onClick={() => onChange(value + 1)}
      >
        <Plus className="h-4 w-4" aria-hidden />
      </button>
    </div>
  );
}
