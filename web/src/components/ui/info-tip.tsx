import { useId, useState } from "react";
import { Info } from "lucide-react";
import { cn } from "@/utils/cn";

/** A small ⓘ that explains a field in plain English. Hover, focus or tap to open. */
export function InfoTip({ text, className }: { text: string; className?: string }) {
  const [open, setOpen] = useState(false);
  const id = useId();
  return (
    <span className={cn("relative inline-flex", className)}>
      <button
        type="button"
        aria-label="What is this?"
        aria-describedby={open ? id : undefined}
        aria-expanded={open}
        onClick={() => setOpen((o) => !o)}
        onBlur={() => setOpen(false)}
        onMouseEnter={() => setOpen(true)}
        onMouseLeave={() => setOpen(false)}
        className="rounded-full text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <Info className="h-4 w-4" />
      </button>
      {open && (
        <span
          role="tooltip"
          id={id}
          className="absolute left-1/2 top-full z-50 mt-2 w-60 -translate-x-1/2 rounded-md border bg-popover p-3 text-xs font-normal leading-relaxed text-popover-foreground shadow-md"
        >
          {text}
        </span>
      )}
    </span>
  );
}
