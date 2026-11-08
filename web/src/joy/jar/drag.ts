// One pointer, one drag at a time, scoped to a root element (Joy's drag.ts was document-wide).
// Sideways swipes only: vertical moves are left to the page so it still scrolls. Under reduced
// motion nothing can be dragged.
import { reduced } from "./motion";
import { axisDecision } from "./swipe";

export interface Drag {
  el: HTMLElement;
  dx: number;
  dy: number;
  vx: number;
  vy: number;
}

export interface DragOpts {
  sel: string;
  ok?: (el: HTMLElement) => boolean;
  start?: (d: Drag) => void;
  move: (d: Drag) => void;
  end: (d: Drag, cancelled: boolean) => void;
}

// Starting on these is a tap on them, never a drag. (A link is fine: a slip's face may be one.)
const FIELDS = "button, input, select, textarea, label, summary, [contenteditable], audio, video";

export function attachDrag(root: HTMLElement, o: DragOpts): () => void {
  let p: { el: HTMLElement; id: number; x0: number; y0: number; started: boolean; d: Drag; lx: number; ly: number; lt: number } | null = null;
  let swallowUntil = 0;

  const begin = () => {
    if (!p || p.started) return;
    p.started = true;
    root.classList.add("m-dragging");
    try {
      p.el.setPointerCapture(p.id);
    } catch {
      /* already gone */
    }
    o.start?.(p.d);
  };

  const finish = (cancelled: boolean) => {
    if (!p) return;
    if (p.started) {
      root.classList.remove("m-dragging");
      swallowUntil = performance.now() + 350;
      o.end(p.d, cancelled);
    }
    p = null;
  };

  const down = (e: PointerEvent) => {
    if (p || e.button !== 0 || !e.isPrimary || reduced()) return;
    const t = e.target as Element;
    const el = t.closest<HTMLElement>(o.sel);
    if (!el || !root.contains(el)) return;
    const hit = t.closest(FIELDS);
    if (hit && el.contains(hit)) return;
    if (o.ok && !o.ok(el)) return;
    const now = performance.now();
    p = { el, id: e.pointerId, x0: e.clientX, y0: e.clientY, started: false, d: { el, dx: 0, dy: 0, vx: 0, vy: 0 }, lx: e.clientX, ly: e.clientY, lt: now };
  };

  const move = (e: PointerEvent) => {
    if (!p || e.pointerId !== p.id) return;
    const now = performance.now();
    const dt = Math.max(1, now - p.lt);
    const d = p.d;
    d.vx = 0.7 * ((e.clientX - p.lx) / dt) + 0.3 * d.vx;
    d.vy = 0.7 * ((e.clientY - p.ly) / dt) + 0.3 * d.vy;
    p.lx = e.clientX;
    p.ly = e.clientY;
    p.lt = now;
    d.dx = e.clientX - p.x0;
    d.dy = e.clientY - p.y0;
    if (!p.started) {
      const verdict = axisDecision(d.dx, d.dy);
      if (verdict === "start") begin();
      else if (verdict === "abort") {
        p = null;
        return;
      }
      if (!p?.started) return;
    }
    o.move(d);
  };

  const up = (e: PointerEvent) => {
    if (p && e.pointerId === p.id) finish(false);
  };
  const cancel = (e: PointerEvent) => {
    if (p && e.pointerId === p.id) finish(true);
  };
  // Once something is picked up, the finger moves it, not the page.
  const touchmove = (e: TouchEvent) => {
    if (p?.started && e.cancelable) e.preventDefault();
  };
  // A drag that ends over a link or button is not a click on it.
  const click = (e: MouseEvent) => {
    if (e.isTrusted && performance.now() < swallowUntil) {
      e.preventDefault();
      e.stopPropagation();
      swallowUntil = 0;
    }
  };
  const dragstart = (e: Event) => {
    if (p) e.preventDefault();
  };

  root.addEventListener("pointerdown", down, { passive: true });
  addEventListener("pointermove", move);
  addEventListener("pointerup", up);
  addEventListener("pointercancel", cancel);
  addEventListener("touchmove", touchmove, { passive: false });
  root.addEventListener("click", click, true);
  root.addEventListener("dragstart", dragstart);
  return () => {
    root.removeEventListener("pointerdown", down);
    removeEventListener("pointermove", move);
    removeEventListener("pointerup", up);
    removeEventListener("pointercancel", cancel);
    removeEventListener("touchmove", touchmove);
    root.removeEventListener("click", click, true);
    root.removeEventListener("dragstart", dragstart);
    root.classList.remove("m-dragging");
    p = null;
  };
}

/** Buttons and chips squash to 0.93 when pressed and bounce back when let go (Joy's press.ts). */
export function attachPress(root: HTMLElement, sel: string, squash: (el: HTMLElement, down: boolean) => void): () => void {
  let held: HTMLElement | null = null;
  const up = () => {
    if (!held) return;
    squash(held, false);
    held = null;
  };
  const down = (e: PointerEvent) => {
    const el = (e.target as Element).closest?.<HTMLElement>(sel);
    if (!el || !root.contains(el) || (el as HTMLButtonElement).disabled) return;
    held = el;
    squash(el, true);
  };
  const out = (e: PointerEvent) => {
    if (held && e.target === held && !held.contains(e.relatedTarget as Node)) up();
  };
  root.addEventListener("pointerdown", down, { passive: true });
  addEventListener("pointerup", up);
  addEventListener("pointercancel", up);
  root.addEventListener("pointerout", out);
  return () => {
    root.removeEventListener("pointerdown", down);
    removeEventListener("pointerup", up);
    removeEventListener("pointercancel", up);
    root.removeEventListener("pointerout", out);
    held = null;
  };
}
