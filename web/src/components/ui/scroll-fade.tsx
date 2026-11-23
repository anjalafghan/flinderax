import { useEffect, useRef, useState, type ReactNode } from "react";

import { cn } from "@/utils/cn";

/**
 * A row that scrolls sideways and says so: a soft fade appears on whichever edge has more to see,
 * and the current item is kept in view. Without it a clipped last tab reads as a layout bug.
 */
export function ScrollFade({ children, className, watch }: { children: ReactNode; className?: string; watch?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const [edge, setEdge] = useState({ left: false, right: false });

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const measure = () =>
      setEdge({ left: el.scrollLeft > 4, right: el.scrollLeft + el.clientWidth < el.scrollWidth - 4 });
    measure();
    el.addEventListener("scroll", measure, { passive: true });
    window.addEventListener("resize", measure);
    el.querySelector<HTMLElement>("[aria-current]")?.scrollIntoView?.({ inline: "nearest", block: "nearest" });
    return () => {
      el.removeEventListener("scroll", measure);
      window.removeEventListener("resize", measure);
    };
  }, [watch]);

  const mask = `linear-gradient(to right, ${edge.left ? "transparent, black 28px" : "black, black"}, ${edge.right ? "black calc(100% - 28px), transparent" : "black, black"})`;
  return (
    <div
      ref={ref}
      style={{ maskImage: mask, WebkitMaskImage: mask }}
      className={cn("flex gap-1 overflow-x-auto py-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden", className)}
    >
      {children}
    </div>
  );
}
