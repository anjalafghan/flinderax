// What makes an exhibit come alive: the hook that plays its animation (on touch, and once when it is
// first seen), and the name a screen reader hears for it.
import { useCallback, useEffect, useRef } from "react";
import type { Spec } from "./spec";

export interface ExhibitDate {
  title: string;
  place: string;
  /** "9 Nov" */
  day: string;
  /** 0 when not rated. */
  rating: number;
}

const reduced = () => typeof window !== "undefined" && typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
/** Long enough for every animation in exhibits.css and art.css to finish. */
const PLAY_MS = 2800;

/** SVG animations that wait to be started (begin="indefinite"): a car along the ghat road, mist drifting. */
type Startable = Element & { beginElement?: () => void };
const startSvg = (el: Element) =>
  el.querySelectorAll<Startable>('animate[begin="indefinite"], animateMotion[begin="indefinite"], animateTransform[begin="indefinite"]').forEach((a) => a.beginElement?.());

/** Plays an exhibit's animation: restart it on every touch, and once when it is first seen. */
export function usePlay<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const timer = useRef<number | undefined>(undefined);
  const play = useCallback(() => {
    const el = ref.current;
    if (!el || reduced()) return;
    el.removeAttribute("data-playing");
    void el.offsetWidth; // restart the keyframes
    el.setAttribute("data-playing", "");
    startSvg(el);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => el.removeAttribute("data-playing"), PLAY_MS);
  }, []);
  useEffect(() => {
    const el = ref.current;
    if (!el || typeof IntersectionObserver !== "function") return;
    const io = new IntersectionObserver(
      (seen) => {
        if (seen.some((s) => s.isIntersecting)) {
          io.disconnect();
          window.setTimeout(play, 500);
        }
      },
      { threshold: 0.6 },
    );
    io.observe(el);
    return () => {
      io.disconnect();
      window.clearTimeout(timer.current);
    };
  }, [play]);
  return { ref, play };
}

/** What a screen reader hears for an exhibit. */
export const exhibitName = (spec: Spec, d: ExhibitDate) =>
  `${spec.label}: ${d.title}, ${d.day}${d.rating > 0 ? `, ${d.rating} out of 5 stars` : ""}`;

