import autoAnimate from "@formkit/auto-animate";
import { useCallback } from "react";

import { prefersReducedMotion } from "@/utils/motion";

/** A ref for a list's container: rows slide into place when added, removed or reordered. */
export function useAnimatedList<T extends HTMLElement>() {
  return useCallback((el: T | null) => {
    if (!el || prefersReducedMotion()) return;
    autoAnimate(el, { duration: 250, easing: "ease-out" });
  }, []);
}
