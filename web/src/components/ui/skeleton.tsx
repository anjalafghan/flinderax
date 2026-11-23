import { cn } from "@/utils/cn";

/** A shimmering block (the shimmer is in index.css for anything with animate-pulse and bg-muted). */
export function Skeleton({ className, width }: { className?: string; width?: number }) {
  return <div aria-hidden style={width ? { width } : undefined} className={cn("animate-pulse rounded-2xl bg-muted/40", className)} />;
}

/**
 * What a page looks like before it arrives: the wordmark, a row of tabs, a title and two cards, in the
 * same places the real page puts them, so nothing jumps when it loads. It waits 180ms before showing,
 * so a page that loads at once never flashes it.
 */
export function PageSkeleton() {
  return (
    <div role="status" aria-busy="true" aria-label="Loading" className="min-h-screen [animation:skeleton-in_.25s_ease_.18s_both]">
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
