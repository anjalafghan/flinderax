import { cn } from "@/utils/cn";

/** The section-tinted shimmer: a wash of the tab's colour with a brighter band sweeping across. */
const SHIMMER =
  "rounded-2xl [animation:shimmer_1.6s_linear_infinite] [background-color:hsl(var(--brand-tint))] [background-image:linear-gradient(100deg,transparent_25%,hsl(var(--brand)/0.16)_45%,hsl(var(--brand-2)/0.16)_55%,transparent_75%)] [background-size:200%_100%]";

/**
 * A shimmering block in the section's colour. Give it a `label` when it stands in for a whole card,
 * so it is announced as loading; without one it is hidden from assistive tech.
 */
export function Skeleton({
  className,
  width,
  label,
}: {
  className?: string;
  width?: number;
  label?: string;
}) {
  return (
    <div
      aria-hidden={label ? undefined : true}
      aria-label={label}
      aria-busy={label ? true : undefined}
      style={width ? { width } : undefined}
      className={cn(SHIMMER, className)}
    />
  );
}

/**
 * What a page looks like before it arrives: the wordmark, a row of tabs, a title and two cards, in the
 * same places the real page puts them, so nothing jumps when it loads. It waits 180ms before showing,
 * so a page that loads at once never flashes it.
 */
export function PageSkeleton() {
  return (
    <div
      role="status"
      aria-busy="true"
      aria-label="Loading"
      className="min-h-screen [animation:skeleton-in_.25s_ease_.18s_both]"
    >
      <div className="mx-auto w-full max-w-[1040px] px-4 pt-7 md:px-6">
        <div className="flex items-center justify-between">
          <Skeleton className="h-9 w-16 rounded-xl" />
          <Skeleton className="h-11 w-11 rounded-full" />
        </div>
        <div className="mt-6 flex gap-6 border-b border-border pb-3">
          {[64, 72, 48, 76, 56].map((w) => (
            <Skeleton key={w} width={w} className="h-4 rounded-full" />
          ))}
        </div>
      </div>
      <div className="mx-auto flex w-full max-w-[1040px] flex-col gap-5 px-4 py-8 md:px-6 md:py-12">
        <Skeleton className="h-12 w-48 rounded-xl" />
        <Skeleton className="h-4 w-2/3 max-w-md rounded-full" />
        <Skeleton className="h-52 w-full rounded-[28px]" />
        <Skeleton className="h-28 w-full rounded-[18px]" />
        <span className="sr-only">Loading…</span>
      </div>
    </div>
  );
}
