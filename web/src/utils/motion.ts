/** True when the person has asked their system for less motion. */
export function prefersReducedMotion(): boolean {
  return typeof window !== "undefined" && typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

const COLORS = ["#7c9a6d", "#e8a87c", "#f2c14e", "#c38d9e", "#85b8cb"];

/** A burst of confetti for a win. Loaded on first use, and skipped under reduced motion. */
export async function celebrate(kind: "burst" | "shower" = "burst"): Promise<void> {
  if (prefersReducedMotion()) return;
  try {
    const { default: confetti } = await import("canvas-confetti");
    if (kind === "burst") {
      confetti({ particleCount: 90, spread: 70, origin: { y: 0.65 }, colors: COLORS, disableForReducedMotion: true });
      return;
    }
    const end = Date.now() + 1200;
    const frame = () => {
      confetti({ particleCount: 3, angle: 60, spread: 55, origin: { x: 0 }, colors: COLORS, disableForReducedMotion: true });
      confetti({ particleCount: 3, angle: 120, spread: 55, origin: { x: 1 }, colors: COLORS, disableForReducedMotion: true });
      if (Date.now() < end) requestAnimationFrame(frame);
    };
    frame();
  } catch {
    // Confetti is a nicety: never let it break the page.
  }
}
