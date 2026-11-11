import { useEffect, useRef, useState } from "react";

import { prefersReducedMotion } from "@/utils/motion";

/**
 * A number that rolls to its new value when it changes. The first value is shown as it is, so the
 * page never starts from zero (and tests see the real figure); only later changes animate.
 */
export function useCountUp(target: number, ms = 700): number {
  const [value, setValue] = useState(target);
  const from = useRef(target);
  const raf = useRef(0);

  useEffect(() => {
    if (from.current === target) return;
    if (prefersReducedMotion() || typeof requestAnimationFrame === "undefined") {
      from.current = target;
      raf.current = requestAnimationFrame?.(() => setValue(target)) ?? 0;
      return;
    }
    const start = from.current;
    const t0 = performance.now();
    const tick = (now: number) => {
      const p = Math.min(1, (now - t0) / ms);
      const eased = 1 - Math.pow(1 - p, 3);
      setValue(Math.round(start + (target - start) * eased));
      if (p < 1) raf.current = requestAnimationFrame(tick);
      else from.current = target;
    };
    raf.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf.current);
  }, [target, ms]);

  return value;
}
