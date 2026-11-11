import type { ReactNode } from "react";

type Scene = "coins" | "heart" | "sprout";

/** A small floating illustration for empty states, with a friendly line under it. */
export function EmptyScene({ scene = "coins", title, children }: { scene?: Scene; title: string; children?: ReactNode }) {
  return (
    <div className="empty-scene flex flex-col items-center gap-3 rounded-[18px] border border-dashed border-border px-6 py-8 text-center">
      <svg viewBox="0 0 96 72" width="96" height="72" aria-hidden className="overflow-visible">
        <ellipse cx="48" cy="64" rx="26" ry="4" className="empty-shadow" fill="hsl(var(--foreground) / .08)" />
        {scene === "coins" && (
          <g className="empty-float">
            <circle cx="40" cy="40" r="16" fill="hsl(var(--warn) / .25)" stroke="hsl(var(--warn))" strokeWidth="2" />
            <circle cx="58" cy="34" r="12" fill="hsl(var(--good) / .25)" stroke="hsl(var(--good))" strokeWidth="2" />
            <path d="M36 40h8M40 36v8" stroke="hsl(var(--warn))" strokeWidth="2" strokeLinecap="round" />
          </g>
        )}
        {scene === "heart" && (
          <g className="empty-float">
            <path d="M48 56C20 38 26 18 40 20c5 1 8 5 8 8 0-3 3-7 8-8 14-2 20 18-8 36z" fill="hsl(var(--over) / .25)" stroke="hsl(var(--over))" strokeWidth="2" strokeLinejoin="round" />
          </g>
        )}
        {scene === "sprout" && (
          <g className="empty-float">
            <path d="M48 58V34" stroke="hsl(var(--good))" strokeWidth="3" strokeLinecap="round" />
            <path d="M48 38C48 26 36 24 30 26c0 10 8 14 18 12z" fill="hsl(var(--good) / .3)" stroke="hsl(var(--good))" strokeWidth="2" strokeLinejoin="round" />
            <path d="M48 32c0-10 10-14 18-12 0 10-8 14-18 12z" fill="hsl(var(--good) / .3)" stroke="hsl(var(--good))" strokeWidth="2" strokeLinejoin="round" />
          </g>
        )}
      </svg>
      <p className="m-0 font-medium">{title}</p>
      {children && <div className="text-sm text-muted-foreground">{children}</div>}
    </div>
  );
}
