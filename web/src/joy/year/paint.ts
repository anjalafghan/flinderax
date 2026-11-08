// The share picture, drawn on a 1080x1920 canvas. A near line-for-line port of Joy's static/year.js: one fixed
// palette (the jar's paper by day), the same seeded grain, and only the ShareData it is handed: no names, notes, homes
// or app address.
import { coverCrop, fitNumberSize, H, M, monthBars, needsDense, polaroidPile, rng, slackFor, TINT, W, wrapText, type ShareData } from "./layout";

export const C = {
  paper: "#f5eee3",
  slip: "#fffaf2",
  ink: "#2b2433",
  soft: "#675d6e",
  rule: "#d8cab8",
  stamp: "#b9301b",
  tape: "rgba(242,207,198,.85)",
};
const STAR =
  "M12.3 2.4c.9 2 1.7 4 2.6 6.1 2.1.1 4.3.3 6.4.6-1.6 1.4-3.3 2.8-4.9 4.1.5 2.1 1 4.2 1.4 6.3-1.8-1.1-3.7-2.3-5.6-3.4-1.8 1.2-3.6 2.4-5.5 3.5.5-2.2 1-4.3 1.5-6.4C6.6 11.8 5 10.4 3.3 9c2.2-.2 4.4-.4 6.5-.5.8-2 1.6-4 2.5-6.1z";
const MONTHS = "JFMAMJJASOND";
const HEART =
  "M16.2 27.4C11.6 24.1 3.4 18.3 3.1 10.9 2.9 6.6 6 3.3 9.8 3.4c2.9.1 5 2.2 6.2 4.6 1.3-2.6 3.5-4.8 6.6-4.7 3.9.2 6.5 3.6 6.2 7.7-.5 7.2-8.4 12.6-12.6 16.4z";
export const FD = '"Fraunces", Georgia, serif';
export const FU = '"Instrument Sans", system-ui, sans-serif';
export const FH = '"Caveat", cursive';

type Ctx = CanvasRenderingContext2D;
const newCanvas = (w: number, h: number) => {
  const c = document.createElement("canvas");
  c.width = w;
  c.height = h;
  return c;
};

const font = (ctx: Ctx, f: string, spacing = 0) => {
  ctx.font = f;
  if ("letterSpacing" in ctx) ctx.letterSpacing = `${spacing}px`;
};
const wrap = (ctx: Ctx, text: string, max: number, lines: number) => wrapText((t) => ctx.measureText(t).width, text, max, lines);

const inkLine = (ctx: Ctx, x1: number, y: number, x2: number, w: number, color: string) => {
  ctx.save();
  ctx.strokeStyle = color;
  ctx.lineWidth = w;
  ctx.lineCap = "round";
  ctx.beginPath();
  ctx.moveTo(x1, y + w * 0.5);
  ctx.bezierCurveTo(x1 + (x2 - x1) * 0.3, y - w * 0.9, x1 + (x2 - x1) * 0.62, y - w * 0.4, x2, y - w * 1.1);
  ctx.stroke();
  ctx.restore();
};

const star = (ctx: Ctx, x: number, y: number, size: number, on: boolean, rot: number) => {
  ctx.save();
  ctx.translate(x + size / 2, y + size / 2);
  ctx.rotate(rot);
  ctx.translate(-size / 2, -size / 2);
  ctx.scale(size / 24, size / 24);
  const p = new Path2D(STAR);
  ctx.lineJoin = "round";
  if (on) {
    ctx.fillStyle = C.stamp;
    ctx.fill(p);
  }
  ctx.strokeStyle = on ? C.stamp : C.soft;
  ctx.lineWidth = 1.5;
  ctx.stroke(p);
  ctx.restore();
};

const tape = (ctx: Ctx, x: number, y: number, w: number, h: number, rot: number) => {
  ctx.save();
  ctx.translate(x, y);
  ctx.rotate(rot);
  ctx.fillStyle = C.tape;
  ctx.beginPath();
  const teeth = 7;
  ctx.moveTo(0, 0);
  ctx.lineTo(w, 0);
  for (let i = 1; i <= teeth; i++) ctx.lineTo(w - (i % 2 ? 7 : 0), (h * i) / teeth);
  ctx.lineTo(0, h);
  for (let i = teeth - 1; i >= 0; i--) ctx.lineTo(i % 2 ? 7 : 0, (h * i) / teeth);
  ctx.closePath();
  ctx.fill();
  ctx.globalAlpha = 0.25;
  ctx.fillStyle = C.slip;
  for (let i = 14; i < w - 10; i += 18) ctx.fillRect(i, 0, 4, h);
  ctx.restore();
};

/** The postmark, drawn on its own small canvas so its ink can be made uneven. */
const postmark = (year: number, r: number) => {
  const s = Math.ceil(r * 2 + 40);
  const c = newCanvas(s, s);
  const g = c.getContext("2d") as Ctx;
  const cx = s / 2;
  g.strokeStyle = g.fillStyle = C.stamp;
  g.lineWidth = r * 0.045;
  g.beginPath();
  g.arc(cx, cx, r, 0, Math.PI * 2);
  g.stroke();
  g.lineWidth = r * 0.022;
  g.setLineDash([r * 0.05, r * 0.055]);
  g.beginPath();
  g.arc(cx, cx, r * 0.53, 0, Math.PI * 2);
  g.stroke();
  g.setLineDash([]);
  const text = "KEPT IN THE JAR · JUST US · ";
  font(g, `700 ${Math.round(r * 0.17)}px ${FU}`);
  const rr = r * 0.77;
  const step = (Math.PI * 2) / text.length;
  g.textAlign = "center";
  g.textBaseline = "middle";
  [...text].forEach((ch, i) => {
    g.save();
    g.translate(cx, cx);
    g.rotate(-Math.PI / 2 + i * step);
    g.translate(0, -rr);
    g.fillText(ch, 0, 0);
    g.restore();
  });
  font(g, `650 ${Math.round(r * 0.4)}px ${FD}`);
  g.fillText(String(year), cx, cx + r * 0.03);
  // worn ink: knock out small specks
  const rnd = rng(year * 31 + 7);
  g.globalCompositeOperation = "destination-out";
  for (let i = 0; i < 420; i++) {
    g.globalAlpha = 0.35 + rnd() * 0.6;
    g.beginPath();
    g.arc(rnd() * s, rnd() * s, rnd() * r * 0.028 + 0.6, 0, Math.PI * 2);
    g.fill();
  }
  return c;
};

/** A little "BEST NIGHT" rubber stamp with a double border. */
const rubber = (ctx: Ctx, cx: number, cy: number, k: number) => {
  const w = 190;
  const h = 104;
  const c = newCanvas(w, h);
  const g = c.getContext("2d") as Ctx;
  g.strokeStyle = g.fillStyle = C.stamp;
  g.lineWidth = 5;
  g.strokeRect(6, 6, w - 12, h - 12);
  g.lineWidth = 2;
  g.strokeRect(14, 14, w - 28, h - 28);
  font(g, `700 26px ${FU}`, 5);
  g.textAlign = "center";
  g.fillText("BEST", w / 2 + 2, 48);
  g.fillText("NIGHT", w / 2 + 2, 78);
  const rnd = rng(77);
  g.globalCompositeOperation = "destination-out";
  for (let i = 0; i < 160; i++) {
    g.globalAlpha = 0.3 + rnd() * 0.6;
    g.beginPath();
    g.arc(rnd() * w, rnd() * h, rnd() * 2.6 + 0.5, 0, Math.PI * 2);
    g.fill();
  }
  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(0.14);
  ctx.scale(k, k);
  ctx.globalAlpha = 0.92;
  ctx.drawImage(c, -w / 2, -h / 2);
  ctx.restore();
  c.width = c.height = 0;
};

type Photo = ImageBitmap;

/** Lay out roomy first; if the extras don't fit above the sign-off, lay out again, tighter. */
export function paint(ctx: Ctx, d: ShareData, photos: Photo[]) {
  let dense = photos.length > 0;
  let end = draw(ctx, d, photos, dense, 0);
  if (needsDense(end, photos.length > 0)) {
    dense = true;
    end = draw(ctx, d, photos, true, 0);
  }
  // without photos, share any spare height out between the sections so the page sits balanced
  const extra = slackFor(end, photos.length > 0);
  if (extra > 0) draw(ctx, d, photos, dense, extra);
}

function draw(ctx: Ctx, d: ShareData, photos: Photo[], dense: boolean, extra: number): number {
  const gx = extra / 4;
  const rnd = rng(d.year * 97 + d.dates);

  // paper + a little grain
  ctx.fillStyle = C.paper;
  ctx.fillRect(0, 0, W, H);
  ctx.fillStyle = C.ink;
  for (let i = 0; i < 2600; i++) {
    ctx.globalAlpha = rnd() * 0.05;
    ctx.fillRect(rnd() * W, rnd() * H, 2, 2);
  }
  ctx.globalAlpha = 1;
  ctx.textBaseline = "alphabetic";
  ctx.textAlign = "left";

  // postmark + cancellation waves, top right
  const pr = dense ? 92 : 122;
  const pcx = dense ? 868 : 846;
  const pcy = dense ? 178 : 236;
  ctx.save();
  ctx.translate(pcx, pcy);
  ctx.rotate(-0.19);
  ctx.globalAlpha = 0.9;
  ctx.strokeStyle = C.stamp;
  ctx.lineWidth = 5;
  ctx.lineCap = "round";
  for (let k = -1; k <= 1; k++) {
    ctx.beginPath();
    const y = k * pr * 0.42;
    ctx.moveTo(-pr - 190, y);
    for (let x = -pr - 190; x < -pr - 10; x += 40) {
      ctx.quadraticCurveTo(x + 10, y - 12, x + 20, y);
      ctx.quadraticCurveTo(x + 30, y + 12, x + 40, y);
    }
    ctx.stroke();
  }
  const pm = postmark(d.year, pr);
  ctx.drawImage(pm, -pm.width / 2, -pm.height / 2);
  pm.width = pm.height = 0;
  ctx.restore();

  // eyebrow
  let y = dense ? 136 : 186;
  font(ctx, `600 30px ${FU}`, 5);
  ctx.fillStyle = C.soft;
  ctx.fillText("OUR YEAR IN THE JAR", M, y);

  // the big number, shrunk to keep clear of the postmark
  let size = dense ? 220 : 380;
  font(ctx, `640 ${size}px ${FD}`, -size * 0.04);
  const maxNum = dense ? 600 : 560;
  const resized = fitNumberSize(size, ctx.measureText(String(d.dates)).width, maxNum);
  if (resized !== size) {
    size = resized;
    font(ctx, `640 ${size}px ${FD}`, -size * 0.04);
  }
  y += Math.round(size * 0.78) + (dense ? 18 : 30);
  ctx.fillStyle = C.ink;
  ctx.fillText(String(d.dates), M - size * 0.03, y);
  const nw = ctx.measureText(String(d.dates)).width;
  inkLine(ctx, M - 8, y + size * 0.07, M + nw * 0.97, size * 0.045, C.stamp);

  y += dense ? 84 : 130;
  font(ctx, `italic 420 ${dense ? 62 : 88}px ${FD}`, -1);
  ctx.fillText(`${d.dates === 1 ? "date" : "dates"} kept in ${d.year}`, M, y);

  // months ledger
  y += (dense ? 50 : 96) + gx;
  const barH = dense ? 96 : 190;
  const base = y + barH + 34;
  monthBars(d.months, barH).forEach((b, i) => {
    if (b.n === 0) {
      ctx.strokeStyle = C.soft;
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.arc(b.cx, base - 10, 6, 0, Math.PI * 2);
      ctx.stroke();
    } else {
      const bw = dense ? 24 : 30;
      ctx.fillStyle = b.busiest ? C.stamp : C.ink;
      ctx.beginPath();
      if (ctx.roundRect) ctx.roundRect(b.cx - bw / 2, base - b.h, bw, b.h, [bw * 0.3, bw * 0.38, 2, 2]);
      else ctx.rect(b.cx - bw / 2, base - b.h, bw, b.h);
      ctx.fill();
      font(ctx, `600 ${dense ? 26 : 30}px ${FU}`);
      ctx.fillStyle = b.busiest ? C.stamp : C.soft;
      ctx.textAlign = "center";
      ctx.fillText(String(b.n), b.cx, base - b.h - 14);
    }
    font(ctx, `600 ${dense ? 26 : 30}px ${FU}`);
    ctx.fillStyle = C.soft;
    ctx.textAlign = "center";
    ctx.fillText(MONTHS[i] ?? "", b.cx, base + (dense ? 40 : 48));
    ctx.textAlign = "left";
  });
  ctx.strokeStyle = C.ink;
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.moveTo(M, base + 1.5);
  ctx.lineTo(W - M, base + 1.5);
  ctx.stroke();
  y = base + (dense ? 86 : 120) + gx;

  // kind of night, streak, stars: three little columns
  const cols: Array<[string, string, string]> = [];
  if (d.vibe) cols.push(["MOSTLY", d.vibe, d.tags && d.tags.length ? d.tags.slice(0, 3).join(" · ") : ""]);
  if (d.streak_weeks > 1) cols.push(["LONGEST RUN", `${d.streak_weeks} weeks`, "in a row"]);
  if (d.avg_rating) cols.push(["AVERAGE NIGHT", `${d.avg_rating} stars`, "out of five"]);
  if (cols.length) {
    const gap = 36;
    const wt = cols.map((_, i) => (i === 0 && d.vibe ? 1.3 : 1));
    const tot = wt.reduce((a, b) => a + b, 0);
    const inner = W - 2 * M - gap * (cols.length - 1);
    let x = M;
    cols.forEach(([k, v, s], i) => {
      const cw = (inner * (wt[i] ?? 1)) / tot;
      if (i) x += (inner * (wt[i - 1] ?? 1)) / tot + gap;
      if (i === 0 && d.vibe) {
        ctx.fillStyle = TINT[d.vibe_key] || C.rule;
        ctx.fillRect(x, y - 30, 10, dense ? 150 : 180);
      }
      const tx = i === 0 && d.vibe ? x + 28 : x;
      const tw = cw - (tx - x);
      font(ctx, `600 ${dense ? 22 : 24}px ${FU}`, 3.5);
      ctx.fillStyle = C.soft;
      ctx.fillText(wrap(ctx, k, tw, 1)[0] ?? "", tx, y);
      let vs = dense ? 50 : 60;
      font(ctx, `600 ${vs}px ${FD}`, -1);
      while (ctx.measureText(v).width > tw && vs > 34) {
        vs -= 2;
        font(ctx, `600 ${vs}px ${FD}`, -1);
      }
      ctx.fillStyle = C.ink;
      ctx.fillText(wrap(ctx, v, tw, 1)[0] ?? "", tx, y + (dense ? 62 : 74));
      if (s) {
        const ss = dense ? 25 : 28;
        font(ctx, `400 ${ss}px ${FU}`);
        ctx.fillStyle = C.soft;
        wrap(ctx, s, tw, 2).forEach((l, j) => ctx.fillText(l, tx, y + (dense ? 104 : 122) + j * ss * 1.3));
      }
    });
    y += (dense ? 172 : 240) + gx;
  }

  // the best night, a slip taped onto the page
  if (d.best_title) {
    const ts = dense ? 52 : 64;
    font(ctx, `600 ${ts}px ${FD}`, -1);
    const lines = wrap(ctx, d.best_title, W - 2 * M - 280, 2);
    const sh = (dense ? 140 : 196) + lines.length * ts * 1.12;
    ctx.save();
    ctx.translate(W / 2, y + sh / 2);
    ctx.rotate(0.012);
    ctx.translate(-W / 2, -(y + sh / 2));
    ctx.shadowColor = "rgba(43,36,51,.22)";
    ctx.shadowBlur = 30;
    ctx.shadowOffsetY = 12;
    ctx.fillStyle = C.slip;
    ctx.fillRect(M - 8, y, W - 2 * M + 16, sh);
    ctx.shadowColor = "transparent";
    tape(ctx, M + 40, y - 18, 170, 40, -0.07);
    const px = M + 36;
    let ty = y + (dense ? 64 : 80);
    font(ctx, `600 ${dense ? 22 : 24}px ${FU}`, 3.5);
    ctx.fillStyle = C.soft;
    ctx.fillText("OUR BEST NIGHT", px, ty);
    font(ctx, `600 ${ts}px ${FD}`, -1);
    ctx.fillStyle = C.ink;
    lines.forEach((l) => {
      ty += ts * 1.12;
      ctx.fillText(l, px, ty);
    });
    ty += dense ? 26 : 34;
    const ss = dense ? 40 : 48;
    for (let i = 1; i <= 5; i++) star(ctx, px + (i - 1) * (ss + 8), ty, ss, i <= d.best_rating, (i % 2 ? -1 : 1) * 0.07);
    rubber(ctx, W - M - (dense ? 104 : 124), y + (dense ? 96 : 112), dense ? 0.85 : 1);
    ctx.restore();
    y += sh + (dense ? 56 : 70) + gx;
  }

  // opt-in extras: where we went, what we spent
  if (d.spent) {
    font(ctx, `600 ${dense ? 22 : 24}px ${FU}`, 3.5);
    ctx.fillStyle = C.soft;
    ctx.fillText("WHAT IT CAME TO", M, y);
    if (dense) {
      const lw = ctx.measureText("WHAT IT CAME TO").width;
      font(ctx, `600 40px ${FD}`, -1);
      ctx.fillStyle = C.ink;
      ctx.fillText(`${d.spent} on dates`, M + lw + 22, y + 4);
      y += 64;
    } else {
      font(ctx, `600 52px ${FD}`, -1);
      ctx.fillStyle = C.ink;
      ctx.fillText(`${d.spent} on dates`, M, y + 64);
      y += 124;
    }
  }
  if (d.places && d.places.length) {
    font(ctx, `600 ${dense ? 22 : 24}px ${FU}`, 3.5);
    ctx.fillStyle = C.soft;
    ctx.fillText("WHERE WE WENT", M, y);
    const ps = dense ? 30 : 34;
    font(ctx, `400 ${ps}px ${FU}`);
    ctx.fillStyle = C.ink;
    const room = Math.max(1, Math.floor((H - 250 - y - (dense ? 440 : 0)) / (ps * 1.4)));
    const lines = wrap(ctx, d.places.join("  ·  "), W - 2 * M, Math.min(dense ? 2 : 4, room));
    lines.forEach((l, i) => ctx.fillText(l, M, y + 50 + i * ps * 1.4));
    y += 50 + lines.length * ps * 1.4 + 30;
  }

  // photos, as polaroids, in whatever room is left above the sign-off
  const end = y;
  polaroidPile(photos.length, y).forEach((s, i) => {
    const b = photos[i] as Photo;
    ctx.save();
    ctx.translate(s.x + s.pw / 2, s.y + s.fh / 2);
    ctx.rotate(s.rot);
    ctx.translate(-s.pw / 2, -s.fh / 2);
    ctx.shadowColor = "rgba(43,36,51,.28)";
    ctx.shadowBlur = 26;
    ctx.shadowOffsetY = 10;
    ctx.fillStyle = C.slip;
    ctx.fillRect(0, 0, s.pw, s.fh);
    ctx.shadowColor = "transparent";
    const { sx, sy, sw, sh } = coverCrop(b.width, b.height, s.iw, s.ih);
    ctx.drawImage(b, sx, sy, sw, sh, s.pad, s.pad, s.iw, s.ih);
    ctx.restore();
  });

  // sign-off: no names, no address
  font(ctx, `600 ${dense ? 50 : 58}px ${FH}`);
  ctx.fillStyle = C.soft;
  ctx.fillText("one jar, just us", M, H - 104);
  ctx.save();
  ctx.translate(W - M - 52, H - 150);
  ctx.rotate(0.12);
  ctx.scale(1.7, 1.7);
  ctx.fillStyle = C.stamp;
  ctx.fill(new Path2D(HEART));
  ctx.restore();
  return end;
}

// document.fonts.ready only waits for faces the page already asked for; ask for each one, then check they are in.
const FACES = ['640 96px "Fraunces"', 'italic 400 48px "Fraunces"', '600 32px "Instrument Sans"', '400 32px "Instrument Sans"', '600 40px "Caveat"'];
export async function loadFonts(): Promise<void> {
  if (!document.fonts || !document.fonts.load) return;
  await Promise.all(FACES.map((f) => document.fonts.load(f))).catch(() => {});
}

/** Fetches a signed photo and decodes it at most 720 px wide. null when it can't be had. */
export async function decodePhoto(src: string): Promise<ImageBitmap | null> {
  try {
    const r = await fetch(src, { credentials: "same-origin" });
    if (!r.ok) return null;
    const blob = await r.blob();
    try {
      return await createImageBitmap(blob, { resizeWidth: 720, resizeQuality: "high" });
    } catch {
      return await createImageBitmap(blob);
    }
  } catch {
    return null;
  }
}

/** Draws the whole picture and encodes it. Resolves with the file; the canvas is released either way. */
export async function makePicture(d: ShareData): Promise<File> {
  const canvas = document.createElement("canvas");
  let bitmaps: ImageBitmap[] = [];
  try {
    await loadFonts();
    bitmaps = (await Promise.all(d.photos.map((p) => decodePhoto(p.url)))).filter((b): b is ImageBitmap => !!b);
    canvas.width = W; // never devicePixelRatio: 1080 wide is the story size
    canvas.height = H;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("canvas");
    paint(ctx, d, bitmaps);
    const type = bitmaps.length ? "image/jpeg" : "image/png";
    const blob = await new Promise<Blob>((ok, no) => canvas.toBlob((b) => (b ? ok(b) : no(new Error("toBlob"))), type, 0.9));
    return new File([blob], `our-${d.year}.${bitmaps.length ? "jpg" : "png"}`, { type });
  } finally {
    canvas.width = canvas.height = 0;
    bitmaps.forEach((b) => b.close());
  }
}
