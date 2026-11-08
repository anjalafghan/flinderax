// The slip simulation from Joy's jar.js, split so the step function is pure: no canvas, no DOM,
// no timers. JarCanvas owns the loop and the drawing; everything here only moves numbers.

/* ── jar geometry (SVG viewBox units, 240 x 300) ───────────── */
export const W = 240;
export const H = 300;
export const WALL_L = 27;
export const WALL_R = 213;
export const FLOOR = 275;
export const CORNER_R = 24;
export const CORNER_Y = FLOOR - CORNER_R;
export const MOUTH = { x: 120, y: 50 };
export const SW = 30; // a folded slip
export const SH = 12;
export const CAP = 80; // slips we simulate; the rest are represented by the counter

export interface SlipInfo {
  id: number;
  setting: string;
  outdoor: boolean;
}

export interface Part {
  id: number;
  setting: string;
  outdoor: boolean;
  rank: number;
  x: number;
  y: number;
  a: number;
  vx: number;
  vy: number;
  va: number;
  tx: number;
  ty: number;
  ta: number;
  /** 0 = lit (fits), 1 = greyed out (doesn't). */
  dim: number;
  dimT: number;
  delay: number;
  k: number;
  c: number;
  out: boolean;
  falling: boolean;
  scribble: number;
  subFor: number | null;
}

export interface Slot {
  x: number;
  y: number;
  a: number;
}

export type Rand = () => number;

export const insetAt = (y: number) =>
  y <= CORNER_Y ? 0 : CORNER_R - Math.sqrt(Math.max(0, CORNER_R * CORNER_R - (y - CORNER_Y) ** 2));

/** Deterministic RNG so the pile looks the same every visit. */
export const rng = (seed: number): Rand => () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) / 4294967296;

export const hash = (n: number) => {
  n = (n ^ 61) ^ (n >>> 16);
  n = (n + (n << 3)) >>> 0;
  n ^= n >>> 4;
  n = Math.imul(n, 0x27d4eb2d) >>> 0;
  return (n ^ (n >>> 15)) >>> 0;
};

export function makeSlots(n: number): Slot[] {
  const r = rng(20261014);
  const slots: Slot[] = [];
  for (let row = 0; slots.length < n; row++) {
    let y = 268 - row * 9.4;
    if (y < 128) y = 128 + r() * 30; // overflow: pile up loosely near the shoulder
    const inset = insetAt(y + 6);
    const left = WALL_L + inset + SW / 2 - 2;
    const right = WALL_R - inset - SW / 2 + 2;
    const count = Math.max(1, Math.floor((right - left) / 27) + 1);
    const step = count > 1 ? (right - left) / (count - 1) : 0;
    const rowSlots: Slot[] = [];
    for (let i = 0; i < count; i++) {
      const shift = row % 2 ? step * 0.45 : 0;
      let x = (count > 1 ? left + i * step + shift - step * 0.22 : (left + right) / 2) + (r() - 0.5) * 7;
      x = Math.min(right, Math.max(left, x));
      const steep = r() < 0.18;
      rowSlots.push({ x, y: y + (r() - 0.5) * 4, a: (r() - 0.5) * (steep ? 1.6 : 0.7) });
    }
    for (let i = rowSlots.length - 1; i > 0; i--) {
      const j = (r() * (i + 1)) | 0;
      [rowSlots[i], rowSlots[j]] = [rowSlots[j], rowSlots[i]];
    }
    slots.push(...rowSlots);
  }
  return slots.slice(0, n);
}

/** The slips we simulate: all of them, or a stable sample of CAP when the jar is big. */
export function sample(slips: SlipInfo[]): SlipInfo[] {
  return slips.length > CAP ? [...slips].sort((a, b) => hash(a.id) - hash(b.id)).slice(0, CAP) : slips;
}

export function makeParts(shown: SlipInfo[]): Part[] {
  return shown.map((s) => ({
    id: s.id,
    setting: s.setting,
    outdoor: !!s.outdoor,
    rank: hash(s.id * 7 + 3),
    x: 0, y: 0, a: 0, vx: 0, vy: 0, va: 0, tx: 0, ty: 0, ta: 0,
    dim: 0, dimT: 0, delay: 0, k: 120, c: 14, out: false, falling: false,
    scribble: hash(s.id) % 3,
    subFor: null,
  }));
}

export interface AssignOpts {
  /** null = everything fits. */
  matchSet: Set<number> | null;
  rainy: boolean;
  animate: boolean;
  reduced: boolean;
  rand?: Rand;
}

/**
 * Hand every slip in the jar its slot: the ones that fit go on top, and (when rain is likely) the
 * cozy ones float up while the outdoor ones sink. Slips that move get a spring kick unless
 * motion is off.
 */
export function assign(parts: Part[], slots: Slot[], o: AssignOpts) {
  const rand = o.rand ?? Math.random;
  const inJar = parts.filter((p) => !p.out);
  const fits = (p: Part) => !o.matchSet || o.matchSet.has(p.id) || (p.subFor != null && o.matchSet.has(p.subFor));
  const cozy = (p: Part) => (!o.rainy ? 0 : p.setting === "home" || p.setting === "apart" ? 1 : p.outdoor ? -1 : 0);
  inJar.sort((a, b) => Number(fits(a)) - Number(fits(b)) || cozy(a) - cozy(b) || a.rank - b.rank);
  inJar.forEach((p, i) => {
    const s = slots[i];
    const was = p.dimT;
    p.dimT = fits(p) ? 0 : 1;
    const moved = Math.abs(p.tx - s.x) + Math.abs(p.ty - s.y) > 0.5;
    p.tx = s.x;
    p.ty = s.y;
    p.ta = s.a;
    if (!o.animate || o.reduced) {
      if (!p.falling) {
        p.x = s.x;
        p.y = s.y;
        p.a = s.a;
        p.vx = p.vy = p.va = 0;
      }
      if (!o.animate) p.dim = p.dimT;
      return;
    }
    if (moved || was !== p.dimT) {
      const sinking = p.dimT > was;
      p.k = sinking ? 70 : 130;
      p.c = sinking ? 12 : 13;
      p.delay = rand() * 0.16 + (sinking ? 0 : 0.06);
      if (!sinking && moved) {
        p.vy -= 40 + rand() * 40;
        p.va += (rand() - 0.5) * 4;
      }
    }
  });
}

/** Rattle the pile: the top of it moves most. */
export function jostle(parts: Part[], strength = 1, rand: Rand = Math.random) {
  for (const p of parts) {
    if (p.out) continue;
    const lift = 1 - (p.y - 120) / 160;
    p.vx += (rand() - 0.5) * 90 * strength;
    p.vy -= (30 + rand() * 110 * lift) * strength;
    p.va += (rand() - 0.5) * 6 * strength;
    p.delay = 0;
  }
}

/** Lift a slip out of the jar: the drawn one if we simulate it, else the top slip that fits. */
export function takeOut(parts: Part[], id: number, setting?: string): { x: number; y: number; a: number; setting: string } | null {
  let p = parts.find((q) => !q.out && (q.id === id || q.subFor === id));
  if (!p) {
    const fit = parts.filter((q) => !q.out && q.dimT === 0);
    p = (fit.length ? fit : parts.filter((q) => !q.out)).sort((a, b) => a.y - b.y)[0];
    if (p) {
      p.subFor = id;
      p.setting = setting || p.setting;
    }
  }
  if (!p) return null;
  p.out = true;
  return { x: p.x, y: p.y, a: p.a, setting: p.setting };
}

/** Drop a slip back in through the mouth. `shown` restores the colour of a stand-in. */
export function dropIn(parts: Part[], id: number, shown: SlipInfo[], reducedMotion: boolean, rand: Rand = Math.random): boolean {
  const p = parts.find((q) => q.out && (q.id === id || q.subFor === id)) || parts.find((q) => q.out);
  if (!p) return false;
  p.out = false;
  if (p.subFor != null && p.id !== id) {
    const orig = shown.find((s) => s.id === p.id);
    if (orig) p.setting = orig.setting;
  }
  p.subFor = null;
  p.x = MOUTH.x + (rand() - 0.5) * 20;
  p.y = MOUTH.y + 6;
  p.vx = (rand() - 0.5) * 30;
  p.vy = 40;
  p.a = rand() - 0.5;
  p.va = (rand() - 0.5) * 8;
  p.falling = !reducedMotion;
  return true;
}

/**
 * One physics tick. Returns true while anything is still moving, so the caller can stop its
 * animation loop when the jar has settled.
 */
export function step(parts: Part[], dt: number, reducedMotion: boolean): boolean {
  let busy = false;
  const dimRate = reducedMotion ? dt / 0.15 : dt * 5;
  for (const p of parts) {
    if (p.out) continue;
    if (Math.abs(p.dimT - p.dim) > 0.002) {
      p.dim += Math.sign(p.dimT - p.dim) * Math.min(Math.abs(p.dimT - p.dim), dimRate);
      busy = true;
    } else p.dim = p.dimT;
    if (reducedMotion) {
      p.x = p.tx;
      p.y = p.ty;
      p.a = p.ta;
      p.falling = false;
      continue;
    }
    if (p.delay > 0) {
      p.delay -= dt;
      busy = true;
      continue;
    }
    if (p.falling) {
      p.vy += 900 * dt;
      p.x += (p.tx - p.x) * Math.min(1, dt * 7);
      p.y += p.vy * dt;
      p.a += p.va * dt;
      if (p.y >= p.ty) {
        p.y = p.ty;
        p.vy *= -0.28;
        if (Math.abs(p.vy) < 40) p.falling = false;
      }
      busy = true;
      continue;
    }
    const ax = p.k * (p.tx - p.x) - p.c * p.vx;
    const ay = p.k * (p.ty - p.y) - p.c * p.vy;
    const aa = 90 * (p.ta - p.a) - 10 * p.va;
    p.vx += ax * dt;
    p.vy += ay * dt;
    p.va += aa * dt;
    p.x += p.vx * dt;
    p.y += p.vy * dt;
    p.a += p.va * dt;
    if (p.y > FLOOR - 4) {
      p.y = FLOOR - 4;
      p.vy = Math.min(0, p.vy);
    }
    if (Math.abs(p.vx) + Math.abs(p.vy) > 1.5 || Math.abs(p.va) > 0.05 || Math.abs(p.tx - p.x) + Math.abs(p.ty - p.y) > 0.3) busy = true;
    else {
      p.x = p.tx;
      p.y = p.ty;
      p.vx = p.vy = 0;
    }
  }
  return busy;
}

/* ── colours (canvas can't read CSS variables, so we do) ───── */
export type RGB = [number, number, number];
export type RGBA = [number, number, number, number];

export function parseColor(s: string): RGBA {
  s = (s || "").trim();
  if (s[0] === "#") {
    let h = s.slice(1);
    if (h.length === 3) h = h.split("").map((c) => c + c).join("");
    return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16), 1];
  }
  const m = s.match(/[\d.]+/g);
  return m && m.length >= 3 ? [+m[0], +m[1], +m[2], m[3] != null ? +m[3] : 1] : [128, 128, 128, 1];
}
export const mix = (a: number[], b: number[], t: number): RGB => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
export const rgba = (c: number[], a = 1) => `rgba(${c[0] | 0},${c[1] | 0},${c[2] | 0},${a})`;
