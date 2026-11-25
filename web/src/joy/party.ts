// Confetti for the Dates moments: a pinch of sparks when a slip comes out of the jar, a proper
// burst when it becomes a date. Loaded on first use; skipped under reduced motion.
import { reduced } from "./jar/motion";

const PARTY = ["#8a8073", "#8a8073", "#8a8073", "#c9bfae", "#8a8073", "#8a8073"];

/** Where an element sits, as confetti's 0..1 origin. */
function originOf(el: Element | null | undefined): { x: number; y: number } {
  if (!el || typeof innerWidth === "undefined") return { x: 0.5, y: 0.6 };
  const r = el.getBoundingClientRect();
  return {
    x: (r.left + r.width / 2) / innerWidth,
    y: (r.top + r.height / 2) / innerHeight,
  };
}

export async function confettiFrom(
  el: Element | null | undefined,
  kind: "spark" | "date",
  section: string[] = [],
): Promise<void> {
  if (reduced()) return;
  try {
    const { default: confetti } = await import("canvas-confetti");
    const colors = [...section, ...PARTY];
    const origin = originOf(el);
    if (kind === "spark") {
      confetti({
        particleCount: 16,
        spread: 50,
        startVelocity: 22,
        gravity: 0.9,
        scalar: 0.7,
        ticks: 90,
        shapes: ["circle"],
        origin,
        colors,
        disableForReducedMotion: true,
      });
      return;
    }
    const heart = confetti.shapeFromText({ text: "♥", scalar: 1.4, color: "#ff4f7f" });
    const base = { origin, colors, disableForReducedMotion: true, zIndex: 60 };
    confetti({ ...base, particleCount: 70, spread: 80, startVelocity: 38 });
    confetti({
      ...base,
      particleCount: 18,
      spread: 100,
      startVelocity: 30,
      shapes: [heart],
      scalar: 1.4,
    });
  } catch {
    // Confetti is a nicety: never let it break the page.
  }
}
