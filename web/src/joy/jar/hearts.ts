// The ink hearts that burst off the stamp: at most 5 alive at once, each gone after 2.2 s.
export const MAX_HEARTS = 5;
export const HEART_TTL = 2200;

export interface HeartBit {
  id: number;
  x: number;
  y: number;
  dx: number;
  dy: number;
  rot: number;
  size: number;
  fall: number;
  delay: number;
  born: number;
}

/** Add new hearts, dropping the oldest so no more than MAX_HEARTS are ever alive. */
export function addHearts(list: HeartBit[], fresh: HeartBit[], max = MAX_HEARTS): HeartBit[] {
  return [...list, ...fresh].slice(-max);
}

/** Hearts older than their time to live are cleared. */
export function pruneHearts(list: HeartBit[], now: number, ttl = HEART_TTL): HeartBit[] {
  return list.filter((h) => now - h.born < ttl);
}

/** 2 or 3 hearts flying up and out from (x, y). `rand` is injectable for tests. */
export function makeHearts(x: number, y: number, now: number, firstId: number, rand: () => number = Math.random): HeartBit[] {
  const n = 2 + Math.round(rand());
  return Array.from({ length: n }, (_, i) => {
    const a = -Math.PI / 2 + (rand() - 0.5) * Math.PI * 1.25;
    const d = 150 * (0.45 + rand() * 0.75);
    return {
      id: firstId + i,
      x,
      y,
      dx: Math.cos(a) * d,
      dy: Math.sin(a) * d,
      rot: (rand() - 0.5) * 300,
      size: 12 + rand() * 9,
      fall: 90 * (0.6 + rand() * 0.8),
      delay: i * 40,
      born: now,
    };
  });
}
