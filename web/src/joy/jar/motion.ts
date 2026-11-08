// Joy's springs, ported from web/src/motion.ts. Each element gets one Controller holding its pose,
// and we paint it with the individual transform properties (translate / rotate / scale) so it stacks
// on top of the page's own CSS transforms instead of fighting them.
// Unlike Joy this touches no globals: reduced motion is checked per call and the springs jump to rest.
import { Controller, type SpringConfig } from "@react-spring/web";

const QUERY = "(prefers-reduced-motion: reduce)";
export const reduced = (): boolean => typeof matchMedia === "function" && matchMedia(QUERY).matches;

export interface Pose {
  x: number;
  y: number;
  r: number;
  ry: number;
  s: number;
  o: number;
}
export const REST: Pose = { x: 0, y: 0, r: 0, ry: 0, s: 1, o: 1 };
type El = HTMLElement | SVGElement;
const ctrls = new WeakMap<El, Controller<Pose>>();

export const springs = {
  snap: { tension: 380, friction: 18 } as SpringConfig, // back where it was, with a little wobble
  soft: { tension: 220, friction: 20 } as SpringConfig, // things arriving
  quick: { tension: 420, friction: 32 } as SpringConfig, // things leaving, no overshoot
  press: { tension: 650, friction: 30 } as SpringConfig,
  boing: { tension: 520, friction: 11 } as SpringConfig,
};

export function paint(el: El, p: Pose) {
  const st = el.style;
  const near = (a: number, b: number, e = 0.05) => Math.abs(a - b) < e;
  if (near(p.x, 0) && near(p.y, 0)) st.removeProperty("translate");
  else st.translate = `${p.x}px ${p.y}px`;
  if (!near(p.ry, 0, 0.01)) st.rotate = `y ${p.ry}deg`;
  else if (!near(p.r, 0, 0.01)) st.rotate = `${p.r}deg`;
  else st.removeProperty("rotate");
  if (near(p.s, 1, 0.001)) st.removeProperty("scale");
  else st.scale = `${p.s}`;
  if (p.o > 0.999) st.removeProperty("opacity");
  else st.opacity = `${Math.max(0, p.o)}`;
}

export function pose(el: El): Controller<Pose> {
  let c = ctrls.get(el);
  if (!c) {
    c = new Controller<Pose>({ ...REST });
    ctrls.set(el, c);
  }
  return c;
}

/** Jump there now (dragging, setting up an entrance). */
export function set(el: El, p: Partial<Pose>) {
  const c = pose(el);
  c.stop();
  c.set(p);
  paint(el, c.get());
}

type Go = Partial<Pose> & { config?: SpringConfig; delay?: number };
/** Spring there; resolves at rest. Under reduced motion it jumps there instead. */
export function to(el: El, p: Go): Promise<unknown> {
  const c = pose(el);
  const { config = springs.soft, delay, ...vals } = p;
  if (reduced()) {
    c.stop();
    c.set(vals);
    paint(el, c.get());
    return Promise.resolve();
  }
  return c.start({ ...vals, config, delay, onChange: () => paint(el, c.get()) }).then(() => paint(el, c.get()));
}

/** Stop a spring where it is and put the element at `end` (a tap that skips the show). */
export function finish(el: El, end: Partial<Pose> = {}) {
  set(el, { ...REST, ...end });
}

export const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));
export const buzz = (ms: number) => {
  try {
    navigator.vibrate?.(ms);
  } catch {
    /* no buzz */
  }
};
