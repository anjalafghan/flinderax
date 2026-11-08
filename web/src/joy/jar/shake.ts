// Shake the phone to draw: three hard shakes inside 700 ms (Joy's jar.js), two seconds apart at most.

export const SHAKE_MAG = 14; // m/s^2 for a hit
export const JIGGLE_MAG = 5; // the jar rattles at gentler motion
export const WINDOW_MS = 700;
export const HITS = 3;
export const COOLDOWN_MS = 2000;

export interface ShakeState {
  hits: number[];
  lastDraw: number;
  lastJig: number;
}

export const newShake = (): ShakeState => ({ hits: [], lastDraw: -Infinity, lastJig: -Infinity });

/** Magnitude of a devicemotion reading, with or without gravity removed. */
export function magnitude(ev: {
  acceleration?: { x: number | null; y: number | null; z: number | null } | null;
  accelerationIncludingGravity?: { x: number | null; y: number | null; z: number | null } | null;
}): number {
  const a = ev.acceleration && ev.acceleration.x != null ? ev.acceleration : null;
  const g = ev.accelerationIncludingGravity;
  if (a) return Math.hypot(a.x || 0, a.y || 0, a.z || 0);
  if (g) return Math.abs(Math.hypot(g.x || 0, g.y || 0, g.z || 0) - 9.81);
  return 0;
}

/** Feed one reading; says whether to rattle the jar and whether that was the third shake. */
export function shakeStep(s: ShakeState, mag: number, now: number, busy: boolean): { state: ShakeState; jiggle: number; draw: boolean } {
  let { hits, lastDraw, lastJig } = s;
  let jiggle = 0;
  let draw = false;
  if (mag > JIGGLE_MAG && now - lastJig > 140) {
    lastJig = now;
    jiggle = Math.min(1, mag / 20);
  }
  if (mag > SHAKE_MAG) {
    hits = hits.filter((t) => now - t < WINDOW_MS);
    hits.push(now);
    if (hits.length >= HITS && !busy && now - lastDraw > COOLDOWN_MS) {
      hits = [];
      lastDraw = now;
      draw = true;
    }
  }
  return { state: { hits, lastDraw, lastJig }, jiggle, draw };
}
