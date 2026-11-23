import { useRef } from "react";

import { cn } from "@/utils/cn";

interface ChipGroupProps<T extends string> {
  /** Names the group for screen readers; the visible label sits above it. */
  label: string;
  value: T;
  onChange: (v: T) => void;
  options: { value: T; label: string }[];
  className?: string;
}

/**
 * A short, single-choice list shown as tappable pills: every option visible, one tap, no dropdown.
 * A radio group underneath, so the arrow keys move between pills and Tab leaves the group.
 */
export function ChipGroup<T extends string>({ label, value, onChange, options, className }: ChipGroupProps<T>) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const move = (from: number, step: number) => {
    const i = (from + step + options.length) % options.length;
    onChange(options[i].value);
    refs.current[i]?.focus();
  };
  return (
    <div role="radiogroup" aria-label={label} className={cn("flex flex-wrap gap-2", className)}>
      {options.map((o, i) => {
        const on = o.value === value;
        return (
          <button
            key={o.value}
            ref={(el) => {
              refs.current[i] = el;
            }}
            type="button"
            role="radio"
            aria-checked={on}
            tabIndex={on ? 0 : -1}
            onClick={() => onChange(o.value)}
            onKeyDown={(e) => {
              if (e.key === "ArrowRight" || e.key === "ArrowDown") {
                e.preventDefault();
                move(i, 1);
              } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
                e.preventDefault();
                move(i, -1);
              }
            }}
            className={cn(
              "inline-flex min-h-11 items-center rounded-full border px-4 text-sm capitalize transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              on ? "border-foreground bg-foreground font-medium text-background" : "border-border text-muted-foreground hover:text-foreground",
            )}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}
