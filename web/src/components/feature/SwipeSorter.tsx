import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, Check, X } from "lucide-react";

import type { Txn } from "@/services/types";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { formatDay } from "@/utils/dates";
import { cn } from "@/utils/cn";
import { categoryLabel, CATEGORY_IDS, signedAmount, suggestCategory, topCategories } from "@/utils/spending";
import { celebrate, tick } from "@/utils/motion";

/** How far a card has to travel before letting go counts as a decision. */
const SWIPE_PX = 90;
const FLY_MS = 220;

type Leaving = "left" | "right" | null;

interface Props {
  /** The transactions to sort, in the order shown. */
  queue: Txn[];
  /** Everything loaded, so suggestions can learn from what is already sorted. */
  all: Txn[];
  /** Saves a category. Call `undo` if it fails so the card comes back. */
  onCategorise: (txn: Txn, category: string, remember: boolean, undo: () => void) => void;
  onClose: () => void;
}

/**
 * Sort uncategorised transactions one card at a time. Swipe right to accept the suggestion (from
 * what you chose for the same merchant before), left to skip, or tap a category. Everything a
 * swipe does is also a button or an arrow key.
 */
export function SwipeSorter({ queue, all, onCategorise, onClose }: Props) {
  const [done, setDone] = useState<Set<string>>(new Set());
  const [skipped, setSkipped] = useState<string[]>([]);
  const [sorted, setSorted] = useState(0);
  const [dx, setDx] = useState(0);
  const [dragging, setDragging] = useState(false);
  const [leaving, setLeaving] = useState<Leaving>(null);
  const [stamp, setStamp] = useState<string | null>(null);
  const [nudge, setNudge] = useState(false);
  const start = useRef<{ x: number; id: number } | null>(null);
  const celebrated = useRef(false);

  // Skipped cards go to the back of the line; sorted ones are gone at once, before the refetch.
  const line = useMemo(() => {
    const rest = queue.filter((t) => !done.has(t.txn_id));
    const later = new Set(skipped);
    return [...rest.filter((t) => !later.has(t.txn_id)), ...skipped.map((id) => rest.find((t) => t.txn_id === id)).filter((t): t is Txn => !!t)];
  }, [queue, done, skipped]);

  const current = line[0];
  const upNext = line[1];
  const suggestion = current ? suggestCategory(current, all) : null;
  const chips = current ? topCategories(all, current.direction, 6) : [];

  const reset = useCallback(() => {
    setDx(0);
    setLeaving(null);
    setStamp(null);
    setNudge(false);
  }, []);

  const fly = useCallback((dir: Exclude<Leaving, null>, then: () => void) => {
    setLeaving(dir);
    setDragging(false);
    window.setTimeout(() => {
      then();
      reset();
    }, FLY_MS);
  }, [reset]);

  const choose = useCallback(
    (category: string) => {
      if (!current || leaving) return;
      const txn = current;
      tick();
      setStamp(categoryLabel(category));
      fly("right", () => {
        setDone((d) => new Set(d).add(txn.txn_id));
        setSorted((n) => n + 1);
        onCategorise(txn, category, txn.merchant !== null, () => {
          setDone((d) => {
            const next = new Set(d);
            next.delete(txn.txn_id);
            return next;
          });
          setSorted((n) => Math.max(0, n - 1));
        });
      });
    },
    [current, leaving, fly, onCategorise],
  );

  const skip = useCallback(() => {
    if (!current || leaving) return;
    const id = current.txn_id;
    setStamp("Later");
    fly("left", () => setSkipped((s) => [...s.filter((x) => x !== id), id]));
  }, [current, leaving, fly]);

  const accept = useCallback(() => {
    if (suggestion) choose(suggestion);
    else {
      // Nothing to accept: say so, and spring back instead of silently doing nothing.
      setNudge(true);
      setDx(0);
    }
  }, [suggestion, choose]);

  useEffect(() => {
    if (!current && sorted > 0 && !celebrated.current) {
      celebrated.current = true;
      void celebrate("shower");
    }
  }, [current, sorted]);

  const onPointerDown = (e: React.PointerEvent) => {
    if (leaving || (e.target as HTMLElement).closest("button, select")) return;
    start.current = { x: e.clientX, id: e.pointerId };
    setDragging(true);
    setNudge(false);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  };
  const onPointerMove = (e: React.PointerEvent) => {
    if (!start.current || start.current.id !== e.pointerId) return;
    setDx(e.clientX - start.current.x);
  };
  const onPointerUp = (e: React.PointerEvent) => {
    if (!start.current || start.current.id !== e.pointerId) return;
    const moved = e.clientX - start.current.x;
    start.current = null;
    setDragging(false);
    if (moved >= SWIPE_PX) accept();
    else if (moved <= -SWIPE_PX) skip();
    else setDx(0);
  };
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowRight") {
      e.preventDefault();
      accept();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      skip();
    }
  };

  const position = leaving === "right" ? 520 : leaving === "left" ? -520 : dx;
  const tilt = Math.max(-14, Math.min(14, position / 14));
  const rightStrength = Math.max(0, Math.min(1, dx / SWIPE_PX));
  const leftStrength = Math.max(0, Math.min(1, -dx / SWIPE_PX));
  const remaining = line.length;

  if (!current) {
    return (
      <section aria-label="Sort transactions" className="glass space-y-3 rounded-3xl p-6 text-center">
        <div className="sorter-pop mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-[hsl(var(--good-tint))] text-good">
          <Check className="h-7 w-7" aria-hidden />
        </div>
        <h3 className="font-display text-[28px] leading-none">{sorted > 0 ? "All sorted!" : "Nothing to sort"}</h3>
        <p className="m-0 text-sm text-muted-foreground">
          {sorted > 0 ? `You sorted ${sorted} ${sorted === 1 ? "transaction" : "transactions"}. Your spending picture just got clearer.` : "Everything here already has a category."}
        </p>
        <Button onClick={onClose}>Done</Button>
      </section>
    );
  }

  return (
    <section aria-label="Sort transactions" className="glass space-y-4 rounded-3xl p-4 md:p-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h3 className="font-display text-[26px] leading-none">Sort one by one</h3>
          <p className="m-0 mt-1 text-[13px] text-muted-foreground" aria-live="polite">
            {sorted} sorted · {remaining} to go
          </p>
        </div>
        <Button variant="ghost" size="icon" aria-label="Close sorting" onClick={onClose}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      <div className="relative mx-auto h-[190px] w-full max-w-sm select-none">
        {upNext && (
          <div aria-hidden className="absolute inset-x-3 top-3 h-full rounded-[22px] border border-border bg-card opacity-70 [transform:scale(.96)]" />
        )}
        <div
          role="group"
          tabIndex={0}
          aria-label={`${current.merchant ?? "Unknown"}, ${signedAmount(current)}. Arrow right ${suggestion ? `uses ${categoryLabel(suggestion)}` : "needs a category"}; arrow left skips.`}
          onKeyDown={onKeyDown}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={() => {
            start.current = null;
            setDragging(false);
            setDx(0);
          }}
          className={cn(
            "absolute inset-0 flex touch-pan-y flex-col justify-between rounded-[22px] border border-border bg-card p-5 outline-none focus-visible:ring-2 focus-visible:ring-ring",
            dragging ? "cursor-grabbing" : "cursor-grab",
            !dragging && "transition-transform duration-200 ease-out",
          )}
          style={{ transform: `translateX(${position}px) rotate(${tilt}deg)`, opacity: leaving ? 0 : 1 }}
        >
          <div className="min-w-0">
            <p className="m-0 truncate text-lg font-semibold">{current.merchant ?? "Unknown"}</p>
            <p className="m-0 text-xs text-muted-foreground">{formatDay(current.occurred_on)}</p>
          </div>
          <p className={cn("m-0 font-display text-[40px] leading-none tabular-nums", current.direction === "credit" && "text-good")}>{signedAmount(current)}</p>

          <span
            aria-hidden
            className="sorter-stamp absolute left-4 top-4 rotate-[-8deg] rounded-lg border-2 border-good px-2 py-0.5 text-xs font-extrabold uppercase tracking-wider text-good"
            style={{ opacity: stamp && leaving === "right" ? 1 : rightStrength }}
          >
            {stamp && leaving === "right" ? stamp : suggestion ? categoryLabel(suggestion) : "Pick below"}
          </span>
          <span
            aria-hidden
            className="absolute right-4 top-4 rotate-[8deg] rounded-lg border-2 border-muted-foreground px-2 py-0.5 text-xs font-extrabold uppercase tracking-wider text-muted-foreground"
            style={{ opacity: leaving === "left" ? 1 : leftStrength }}
          >
            Later
          </span>
        </div>
      </div>

      {nudge && (
        <p role="status" className="m-0 text-center text-sm text-warn">
          Nothing to suggest for this one yet. Pick a category below.
        </p>
      )}

      <div className="flex flex-wrap justify-center gap-2" role="group" aria-label="Pick a category">
        {chips.map((c) => (
          <button
            key={c}
            type="button"
            onClick={() => choose(c)}
            className={cn(
              "min-h-10 rounded-full border px-4 text-sm transition-colors",
              c === suggestion ? "border-good bg-[hsl(var(--good-tint))] font-medium text-good" : "border-border hover:bg-[hsl(var(--chip))]",
            )}
          >
            {categoryLabel(c)}
          </button>
        ))}
      </div>
      <Select
        aria-label="Another category"
        className="mx-auto h-10 w-full max-w-sm text-sm"
        value=""
        onChange={(e) => e.target.value && choose(e.target.value)}
      >
        <option value="">More categories…</option>
        {CATEGORY_IDS.filter((id) => id !== "uncategorized").map((id) => (
          <option key={id} value={id}>
            {categoryLabel(id)}
          </option>
        ))}
      </Select>

      <div className="flex items-center justify-between gap-3">
        <Button variant="outline" onClick={skip} className="gap-2">
          <ArrowLeft className="h-4 w-4" aria-hidden /> Later
        </Button>
        <p className="m-0 hidden text-xs text-muted-foreground sm:block">Swipe or use ← →</p>
        <Button onClick={accept} disabled={!suggestion} className="gap-2" aria-label={suggestion ? `Use ${categoryLabel(suggestion)}` : "No suggestion"}>
          {suggestion ? categoryLabel(suggestion) : "No suggestion"} <ArrowRight className="h-4 w-4" aria-hidden />
        </Button>
      </div>
    </section>
  );
}
