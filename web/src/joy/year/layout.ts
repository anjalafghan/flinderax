// The share picture's maths, kept out of the canvas code so it can be tested: sizes, seeded
// randomness, text wrapping, what the picture may contain, and where the polaroids and bars go.

export const W = 1080;
export const H = 1920;
export const M = 96;
export const MAX_PHOTOS = 9;

/** Seeded randomness, so the same year always draws the same paper. */
export function rng(seed: number): () => number {
  let s = seed;
  return () => ((s = (s * 16807) % 2147483647) - 1) / 2147483646;
}

export interface SharePhoto {
  id: number;
  url: string;
}

/** Exactly what the server lets the picture hold (no names, notes or homes). */
export interface ShareData {
  year: number;
  dates: number;
  months: number[];
  vibe: string;
  tags: string[];
  best_title: string;
  best_rating: number;
  best_more: number;
  vibe_key: string;
  avg_rating: string;
  streak_weeks: number;
  places: string[];
  spent: string;
  photos: SharePhoto[];
}

export interface ShareChoices {
  places: boolean;
  spent: boolean;
  /** Photo ids ticked on the wall, in the order ticked. */
  photos: number[];
}

/** The ticked photo ids that really are in the share data, once each, at most nine. */
export function pickPhotos(ticked: readonly number[], available: readonly SharePhoto[], max = MAX_PHOTOS): SharePhoto[] {
  const byId = new Map(available.map((p) => [p.id, p]));
  const out: SharePhoto[] = [];
  const seen = new Set<number>();
  for (const id of ticked) {
    const p = byId.get(id);
    if (p && !seen.has(id)) {
      seen.add(id);
      out.push(p);
    }
    if (out.length >= max) break;
  }
  return out;
}

/** The data the picture is drawn from: places and spend only when ticked, de-duplicated places. */
export function drawData(data: ShareData, c: ShareChoices): ShareData {
  return {
    ...data,
    places: c.places ? uniquePlaces(data.places) : [],
    spent: c.spent ? data.spent : "",
    photos: pickPhotos(c.photos, data.photos),
  };
}

export const uniquePlaces = (places: readonly string[]): string[] => [...new Set(places.filter(Boolean))];

/** Which controls the page shows: a box with nothing behind it stays hidden. */
export function shareControls(data: ShareData) {
  return { places: uniquePlaces(data.places).length > 0, spent: !!data.spent, photos: data.photos.length > 0 };
}

export type Measure = (text: string) => number;

/** Word-wrap to `max` px in at most `lines` lines; the last line ends in "…" when text is cut. */
export function wrapText(measure: Measure, text: string, max: number, lines: number): string[] {
  const words = String(text).split(/\s+/).filter(Boolean);
  const out: string[] = [];
  let line = "";
  for (const w of words) {
    const t = line ? `${line} ${w}` : w;
    if (measure(t) <= max || !line) line = t;
    else {
      out.push(line);
      line = w;
    }
  }
  if (line) out.push(line);
  if (out.length > lines) {
    out.length = lines;
    let last = out[lines - 1] ?? "";
    while (last && measure(`${last}…`) > max) last = last.replace(/\s*\S+$/, "");
    out[lines - 1] = (last || out[lines - 1] || "").replace(/[\s·,]+$/, "") + "…";
  }
  // a single word too wide for the line never overflows
  return out.map((l) => {
    while (measure(l) > max && l.length > 2) l = `${l.slice(0, -2)}…`;
    return l.replace(/^[\s·]+|[\s·]+$/g, "");
  });
}

/** Roomy first; if the extras don't fit above the sign-off (and there are no photos), lay out again, tighter. */
export const needsDense = (roomyEnd: number, hasPhotos: boolean, h = H): boolean => !hasPhotos && roomyEnd > h - 150;

/** Extra height to spread between sections (only without photos), capped at 380. */
export function slackFor(end: number, hasPhotos: boolean, h = H): number {
  const slack = h - 200 - end;
  return !hasPhotos && slack > 24 ? Math.min(slack, 380) : 0;
}

export interface PolaroidSlot {
  x: number;
  y: number;
  /** Frame width and height. */
  pw: number;
  fh: number;
  /** Photo window inside the frame. */
  pad: number;
  iw: number;
  ih: number;
  rot: number;
}

/** A loose pile of polaroids in the room left above the sign-off: one or two rows, overlapping when many. */
export function polaroidPile(n: number, y: number, h = H, w = W, m = M): PolaroidSlot[] {
  if (n <= 0) return [];
  const rows = n <= 3 ? 1 : 2;
  const per = Math.ceil(n / rows);
  const areaTop = y + 6;
  const areaH = h - 200 - areaTop;
  const pw = Math.max(120, Math.min(n === 1 ? 420 : n <= 3 ? 300 : 340, areaH / (rows * 1.45 - (rows - 1) * 0.3)));
  const fh = pw * 1.45;
  const rowStep = fh * 0.7;
  const usedH = fh + (rows - 1) * rowStep;
  const y0 = areaTop + Math.max(0, (areaH - usedH) / 2);
  const slots: PolaroidSlot[] = [];
  for (let i = 0; i < n; i++) {
    const r = Math.floor(i / per);
    const c = i % per;
    const inRow = Math.min(per, n - r * per);
    const step = inRow > 1 ? Math.min(pw + 28, (w - 2 * m - pw) / (inRow - 1)) : 0;
    const rowW = pw + step * (inRow - 1);
    const x = (w - rowW) / 2 + c * step + (r % 2 ? step * 0.18 : 0) * (inRow < per ? 1 : 0);
    const pad = pw * 0.06;
    const iw = pw - 2 * pad;
    slots.push({ x, y: y0 + r * rowStep + ((c * 53) % 3) * 8, pw, fh, pad, iw, ih: iw * 1.25, rot: (((i * 37) % 5) - 2) * 0.03 });
  }
  return slots;
}

/** Cover-crop: the middle of a photo at the tile's aspect ratio. */
export function coverCrop(bw: number, bh: number, tw: number, th: number) {
  const sr = bw / bh;
  const tr = tw / th;
  let sw = bw;
  let sh = bh;
  let sx = 0;
  let sy = 0;
  if (sr > tr) {
    sw = bh * tr;
    sx = (bw - sw) / 2;
  } else {
    sh = bw / tr;
    sy = (bh - sh) / 2;
  }
  return { sx, sy, sw, sh };
}

export interface Bar {
  cx: number;
  /** 0 = nothing that month (a hollow ring). */
  h: number;
  n: number;
  busiest: boolean;
}

/** The months ledger: one column per month, bar heights relative to the busiest. */
export function monthBars(months: readonly number[], barH: number, w = W, m = M): Bar[] {
  const colW = (w - 2 * m) / 12;
  const max = Math.max(1, ...months);
  const top = Math.max(0, ...months);
  return months.map((n, i) => ({
    cx: m + colW * i + colW / 2,
    h: n === 0 ? 0 : Math.max(8, (n / max) * barH),
    n,
    busiest: n > 0 && n === top,
  }));
}

/** The big number shrinks to stay clear of the postmark. */
export function fitNumberSize(size: number, measuredWidth: number, maxWidth: number): number {
  return measuredWidth > maxWidth ? Math.floor((size * maxWidth) / measuredWidth) : size;
}

/** The paper tint behind "MOSTLY", by the server's kind-of-night key. */
export const TINT: Record<string, string> = {
  home: "#f3e2b3",
  apart: "#ddd3ee",
  local: "#f2cfc6",
  day_trip: "#d3e1c6",
  travel: "#cbdbea",
};

/** The file the picture is saved as: a JPEG when it carries photos, a PNG when it doesn't. */
export const shareFile = (year: number, hasPhotos: boolean) => ({
  name: `our-${year}.${hasPhotos ? "jpg" : "png"}`,
  type: hasPhotos ? "image/jpeg" : "image/png",
});

/** Where each polaroid on the page is dealt from: a seeded angle and delay, so a year always lands the same. */
export function dealParams(seed: number, i: number): { rot: number; dx: number; delay: number } {
  const r = rng(seed * 31 + i * 7 + 1);
  const a = r();
  const b = r();
  return { rot: (a - 0.5) * 36, dx: (b - 0.5) * 80, delay: 120 + i * 70 };
}
