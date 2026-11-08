// Small browser-side helpers. Every one is guarded: none of it is needed for the page to work.
import { useEffect, useLayoutEffect, type RefObject } from "react";
import { dealPose } from "./helpers";

export const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

/** A tiny buzz where the device has one (a heart, a reveal). */
export function buzz(ms: number) {
  try {
    if (typeof navigator !== "undefined" && typeof navigator.vibrate === "function") navigator.vibrate(ms);
  } catch {
    /* some browsers throw without a gesture */
  }
}

export const behavior = (): ScrollBehavior => (reduced() ? "auto" : "smooth");

export const nextFrame = () => new Promise<void>((r) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => r()) : setTimeout(r, 16)));

/** Notes grow with their words where `field-sizing` isn't available. */
export function useFit(ref: RefObject<HTMLTextAreaElement | null>, value: string) {
  useLayoutEffect(() => {
    const ta = ref.current;
    if (!ta || (typeof CSS !== "undefined" && CSS.supports?.("field-sizing", "content"))) return;
    ta.style.height = "auto";
    ta.style.height = `${ta.scrollHeight}px`;
  }, [ref, value]);
}

/**
 * reveal.ts, in plain WAAPI: slips wait out of sight and are dealt onto the page as they scroll
 * into view, a few at a time, each from its own slight angle. `dep` changes when slips do.
 */
export function useDealIn(root: RefObject<HTMLElement | null>, dep: unknown): boolean {
  const on = !reduced() && typeof IntersectionObserver !== "undefined" && typeof Element !== "undefined" && "animate" in Element.prototype;
  useEffect(() => {
    const book = root.current;
    if (!on || !book) return;
    const fresh = Array.from(book.querySelectorAll<HTMLElement>(".k-slip:not([data-dealt])"));
    if (!fresh.length) return;
    let queue: HTMLElement[] = [];
    let raf = 0;
    const flush = () => {
      raf = 0;
      queue.sort((a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top);
      queue.forEach((el, i) => {
        const { y, deg } = dealPose(Number(el.dataset.dealt));
        el.animate(
          [
            { opacity: 0, transform: `translateY(${y}px) rotate(calc(var(--rot) + ${deg}deg)) scale(.96)` },
            { opacity: 1, transform: "rotate(var(--rot))" },
          ],
          { duration: 520, delay: Math.min(i, 8) * 50, easing: "cubic-bezier(.34,1.4,.64,1)", fill: "backwards" },
        );
        el.style.opacity = "";
        el.dataset.dealt = "done";
      });
      queue = [];
    };
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (!e.isIntersecting) continue;
          io.unobserve(e.target);
          queue.push(e.target as HTMLElement);
        }
        if (queue.length && !raf) raf = requestAnimationFrame(flush);
      },
      { rootMargin: "0px 0px -5% 0px" },
    );
    fresh.forEach((el, i) => {
      el.dataset.dealt = String(i);
      el.style.opacity = "0";
      io.observe(el);
    });
    return () => {
      io.disconnect();
      if (raf) cancelAnimationFrame(raf);
      // Anything still waiting must not stay invisible (a re-run will pick it up again).
      book.querySelectorAll<HTMLElement>(".k-slip[data-dealt]:not([data-dealt='done'])").forEach((el) => {
        el.style.opacity = "";
        delete el.dataset.dealt;
      });
    };
  }, [root, dep, on]);
  return on;
}
