// Headless checks for the keepsake contact sheet (run by build.py): `bun tools/keepsakes/check.ts out/sheet.html`.
// For every scene, closed and then open: text must sit on its object and not be covered, the closed
// state must show most of the object, the object must not be a plain rectangle, and opening must
// move something. Also measures each drawing's crop and saves the two contact-sheet screenshots.
import { chromium } from "@playwright/test";
import { writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const sheet = resolve(process.argv[2] ?? "tools/keepsakes/out/sheet.html");
const out = dirname(sheet);

type Box = [number, number, number, number];
type State = {
  box: Box | null;
  text: string[];
  moving: number;
  rect: string | null;
};

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: 1600, height: 900 },
  deviceScaleFactor: 1,
});
await page.goto("file://" + sheet);
await page.evaluate(() => document.fonts.ready);
// hit-testing only works on screen: make the window as tall as the whole sheet
await page.setViewportSize({
  width: 1600,
  height: await page.evaluate(() => document.documentElement.scrollHeight),
});

/** Runs in the page: measures every tile in its current state. */
const measure = () =>
  page.evaluate(() => {
    const SHAPES = "path,rect,circle,ellipse,polygon,polyline,line,use,text";
    const visible = (el: Element, root: Element) => {
      let o = 1;
      for (let e: Element | null = el; e && e !== root; e = e.parentElement) {
        const cs = getComputedStyle(e);
        if (cs.display === "none" || cs.visibility === "hidden") return 0;
        o *= Number(cs.opacity);
      }
      return o;
    };
    const inText = (e: Element) => !!e.closest("text");
    // paper-grain overlays (a filtered copy of a shape on top of it) are texture, not something on top
    // so are see-through gradient washes (a sheen, a spotlight, a glow)
    const seeThrough = (e: Element) => {
      const m = /url\("?#([^")]+)"?\)/.exec(getComputedStyle(e).fill);
      const grad = m ? document.getElementById(m[1]!) : null;
      if (!grad) return false;
      const stops = [...grad.querySelectorAll("stop")].map((s) =>
        Number(getComputedStyle(s).stopOpacity),
      );
      return stops.length > 0 && Math.max(...stops) < 0.6;
    };
    const solid = (e: Element, svg: Element) =>
      e !== svg &&
      svg.contains(e) &&
      !inText(e) &&
      !e.hasAttribute("filter") &&
      !seeThrough(e) &&
      visible(e, svg) >= 0.6;
    const res: Record<
      string,
      {
        box: [number, number, number, number] | null;
        text: string[];
        moving: number;
        rect: string | null;
      }
    > = {};
    for (const tile of document.querySelectorAll<HTMLElement>(".tile")) {
      const id = tile.dataset.id!;
      const svg = tile.querySelector("svg")!;
      const R = svg.getBoundingClientRect();
      const k = 600 / R.width;
      let box: [number, number, number, number] | null = null;
      let biggest: { el: Element; area: number } | null = null;
      const text: string[] = [];
      for (const el of svg.querySelectorAll(SHAPES)) {
        if (
          el.closest("defs,clipPath,pattern,mask,symbol") ||
          (el.tagName !== "text" && inText(el))
        )
          continue;
        if (visible(el, svg) < 0.05) continue;
        const r = el.getBoundingClientRect();
        if (r.width < 1 || r.height < 1) continue;
        const b: [number, number, number, number] = [
          (r.left - R.left) * k,
          (r.top - R.top) * k,
          (r.right - R.left) * k,
          (r.bottom - R.top) * k,
        ];
        box = box
          ? [
              Math.min(box[0], b[0]),
              Math.min(box[1], b[1]),
              Math.max(box[2], b[2]),
              Math.max(box[3], b[3]),
            ]
          : b;
        if (el.tagName === "text") {
          // what the text sits on: the first thing under it that isn't text
          // the text's own box, in its own (possibly rotated) frame, mapped to the screen
          const t = el as SVGTextElement,
            bb = t.getBBox(),
            m = t.getScreenCTM()!;
          const at = (fx: number, fy: number) => {
            const p = new DOMPoint(
              bb.x + bb.width * fx,
              bb.y + bb.height * fy,
            ).matrixTransform(m);
            return [p.x, p.y] as const;
          };
          const under = (x: number, y: number) =>
            document.elementsFromPoint(x, y).find((e) => solid(e, svg)) ?? null;
          const [cx, cy] = at(0.5, 0.5);
          const words = (el.textContent ?? "").trim().slice(0, 24);
          const top = document
            .elementsFromPoint(cx, cy)
            .find((e) => e === el || el.contains(e) || solid(e, svg));
          if (top && top !== el && !el.contains(top))
            text.push(`“${words}” is covered`);
          if (under(cx, cy)) {
            const corners = [
              at(0.1, 0.3),
              at(0.9, 0.3),
              at(0.1, 0.75),
              at(0.9, 0.75),
            ];
            if (corners.some(([x, y]) => !under(x, y)))
              text.push(`“${words}” runs off its object`);
          }
        } else if (
          !el.closest("[clip-path]") &&
          Number(getComputedStyle(el).fillOpacity) > 0.5 &&
          getComputedStyle(el).fill !== "none"
        ) {
          const area = r.width * r.height;
          if (!biggest || area > biggest.area) biggest = { el, area };
        }
      }
      let rect: string | null = null;
      if (biggest) {
        const el = biggest.el;
        const plain =
          (el.tagName === "rect" && Number(el.getAttribute("rx") ?? 0) < 10) ||
          (el.tagName === "path" &&
            /^M[-\d. ]+(?:[HV][-\d. ]+){3,4}Z$/i.test(
              (el.getAttribute("d") ?? "").trim(),
            ));
        if (plain) rect = "the object is a plain rectangle";
      }
      res[id] = {
        box,
        text,
        moving: tile.getAnimations({ subtree: true }).length,
        rect,
      };
    }
    return res;
  });

const settle = async () => {
  for (let i = 0; i < 3; i++) {
    await page.waitForTimeout(120);
    await page.evaluate(() =>
      document.getAnimations().forEach((a) => {
        try {
          if (a.effect?.getTiming().iterations !== Infinity) a.finish();
        } catch {
          /* still pending */
        }
      }),
    );
  }
};

const closed: Record<string, State> = await measure();
await page.evaluate(() =>
  document.querySelectorAll(".ksc").forEach((e) => e.classList.add("open")),
);
await page.waitForTimeout(60);
const started = await page.evaluate(() =>
  Object.fromEntries(
    [...document.querySelectorAll<HTMLElement>(".tile")].map((t) => [
      t.dataset.id!,
      t.getAnimations({ subtree: true }).length,
    ]),
  ),
);
await settle();
const opened: Record<string, State> = await measure();

const area = (b: Box | null) =>
  b ? Math.max(0, b[2] - b[0]) * Math.max(0, b[3] - b[1]) : 0;
const report: Record<string, { crop: Box; issues: string[] }> = {};
for (const id of Object.keys(closed)) {
  const c = closed[id]!,
    o = opened[id]!;
  const u: Box | null =
    c.box && o.box
      ? [
          Math.min(c.box[0], o.box[0]),
          Math.min(c.box[1], o.box[1]),
          Math.max(c.box[2], o.box[2]),
          Math.max(c.box[3], o.box[3]),
        ]
      : (c.box ?? o.box);
  const pad = 12;
  const crop: Box = u
    ? [
        Math.floor(u[0] - pad),
        Math.floor(u[1] - pad),
        Math.ceil(u[2] - u[0] + 2 * pad),
        Math.ceil(u[3] - u[1] + 2 * pad),
      ]
    : [0, 0, 600, 600];
  const issues = [
    ...new Set([
      ...c.text
        .filter((t) => !t.endsWith("is covered"))
        .map((t) => t + " (closed)"),
      ...o.text.map((t) => t + " (open)"),
    ]),
  ];
  if (c.rect) issues.push(c.rect);
  if (!started[id]) issues.push("opening it moves nothing");
  if (area(c.box) < 0.3 * area(u))
    issues.push("the closed state shows little of the object");
  report[id] = { crop, issues };
}

await page.evaluate(
  (bad) =>
    bad.forEach((id) =>
      document.querySelector(`.tile[data-id="${id}"]`)?.classList.add("bad"),
    ),
  Object.keys(report).filter((id) => report[id]!.issues.length),
);
await page.screenshot({ path: join(out, "sheet-open.png"), fullPage: true });
await page.evaluate(() =>
  document.querySelectorAll(".ksc").forEach((e) => e.classList.remove("open")),
);
await settle();
await page.screenshot({ path: join(out, "sheet-closed.png"), fullPage: true });
writeFileSync(join(out, "report.json"), JSON.stringify(report, null, 1));

// Silhouettes: what each closed keepsake looks like as a grey ghost in the museum's jar grid. Each one's
// opaque shape (cut to its own box, on a 20×20 grid) and a coarse 6×6 map of how light or dark it is.
// Two keepsakes read as the same object when their shapes overlap closely, at about the same
// proportions, with light and dark in the same places. Clusters go to silhouettes.json; build.py lists them.
await page.addStyleTag({
  content:
    "body,.tile{background:transparent!important;outline:none!important}figcaption{visibility:hidden}",
});
const shot = await page.screenshot({ fullPage: true, omitBackground: true });
type Sil = { mask: number[]; aspect: number; luma: number[] };
const sils: Record<string, Sil> = await page.evaluate(
  async ({ png, crops }) => {
    const img = new Image();
    img.src = "data:image/png;base64," + png;
    await img.decode();
    const cv = document.createElement("canvas");
    cv.width = img.width;
    cv.height = img.height;
    const cx = cv.getContext("2d", { willReadFrequently: true })!;
    cx.drawImage(img, 0, 0);
    const res: Record<string, { mask: number[]; aspect: number; luma: number[] }> = {};
    const N = 20,
      L = 6;
    for (const tile of document.querySelectorAll<HTMLElement>(".tile")) {
      const id = tile.dataset.id!;
      const R = tile.querySelector("svg")!.getBoundingClientRect();
      const k = R.width / 600,
        c = crops[id]!;
      const x0 = Math.max(0, Math.floor(R.left + window.scrollX + c[0] * k)),
        y0 = Math.max(0, Math.floor(R.top + window.scrollY + c[1] * k));
      const w = Math.max(1, Math.ceil(c[2] * k)),
        h = Math.max(1, Math.ceil(c[3] * k));
      const d = cx.getImageData(x0, y0, w, h).data;
      let bx0 = w,
        by0 = h,
        bx1 = -1,
        by1 = -1;
      for (let y = 0; y < h; y++)
        for (let x = 0; x < w; x++)
          if (d[(y * w + x) * 4 + 3]! > 160) {
            if (x < bx0) bx0 = x;
            if (x > bx1) bx1 = x;
            if (y < by0) by0 = y;
            if (y > by1) by1 = y;
          }
      if (bx1 < 0) {
        res[id] = { mask: [], aspect: 1, luma: [] };
        continue;
      }
      const bw = bx1 - bx0 + 1,
        bh = by1 - by0 + 1;
      const cell = (n: number, fx: number, fy: number) => {
        let on = 0,
          all = 0,
          lum = 0;
        const xa = bx0 + Math.floor((fx * bw) / n),
          xb = bx0 + Math.floor(((fx + 1) * bw) / n),
          ya = by0 + Math.floor((fy * bh) / n),
          yb = by0 + Math.floor(((fy + 1) * bh) / n);
        for (let y = ya; y < Math.max(yb, ya + 1); y++)
          for (let x = xa; x < Math.max(xb, xa + 1); x++) {
            const i = (y * w + x) * 4;
            all++;
            if (d[i + 3]! > 160) {
              on++;
              lum += 0.3 * d[i]! + 0.59 * d[i + 1]! + 0.11 * d[i + 2]!;
            }
          }
        return { on: on / all, lum: on ? lum / on / 255 : -1 };
      };
      const mask: number[] = [],
        luma: number[] = [];
      for (let fy = 0; fy < N; fy++)
        for (let fx = 0; fx < N; fx++) mask.push(cell(N, fx, fy).on > 0.5 ? 1 : 0);
      for (let fy = 0; fy < L; fy++)
        for (let fx = 0; fx < L; fx++) luma.push(cell(L, fx, fy).lum);
      res[id] = { mask, aspect: bw / bh, luma };
    }
    return res;
  },
  {
    png: shot.toString("base64"),
    crops: Object.fromEntries(
      Object.entries(report).map(([id, r]) => [id, r.crop]),
    ),
  },
);
writeFileSync(join(out, "silhouettes-raw.json"), JSON.stringify(sils));
// the jar grid as the museum shows it: grey ghosts about 96px tall, cut to each crop, with their ids
await page.evaluate((crops) => {
  document.body.style.cssText =
    "display:grid;grid-template-columns:repeat(14,104px);gap:6px;padding:10px;background:#efe7d8!important";
  for (const t of document.querySelectorAll<HTMLElement>(".tile")) {
    const c = crops[t.dataset.id!]!;
    const svg = t.querySelector("svg")!;
    svg.setAttribute("viewBox", c.join(" "));
    svg.style.cssText = `width:${Math.min(100, (96 * c[2]) / c[3])}px;height:96px;overflow:hidden;filter:grayscale(1) sepia(.25) contrast(.85) brightness(1.02);opacity:.72`;
    t.style.cssText = "background:transparent;padding:0;display:grid;justify-items:center";
    const f = t.querySelector("figcaption")! as HTMLElement;
    f.style.cssText = "visibility:visible;color:#3a3328;font:10px system-ui;padding:0";
  }
}, Object.fromEntries(Object.entries(report).map(([id, r]) => [id, r.crop])));
await page.setViewportSize({ width: 1600, height: 400 });
await page.screenshot({ path: join(out, "sheet-ghost.png"), fullPage: true });
const same = (a: Sil, b: Sil) => {
  if (!a.mask.length || !b.mask.length) return false;
  let both = 0,
    any = 0;
  a.mask.forEach((v, i) => {
    both += v & b.mask[i]!;
    any += v | b.mask[i]!;
  });
  const iou = any ? both / any : 0;
  const prop = Math.min(a.aspect, b.aspect) / Math.max(a.aspect, b.aspect);
  // light and dark: mean difference where both have something
  let diff = 0,
    n = 0;
  a.luma.forEach((v, i) => {
    const u = b.luma[i]!;
    if (v >= 0 && u >= 0) {
      diff += Math.abs(v - u);
      n++;
    }
  });
  // calibrated on the kit: two stubs, two coins or two postcards score 0.93+; different die-cut things score under 0.85
  return iou >= 0.86 && prop >= 0.82 && (n ? diff / n : 1) <= 0.16;
};
// complete linkage: a keepsake joins a cluster only when it looks like every one already in it
const clusters: string[][] = [];
for (const id of Object.keys(sils).sort()) {
  const c = clusters.find((c) => c.every((o) => same(sils[o]!, sils[id]!)));
  if (c) c.push(id);
  else clusters.push([id]);
}
const groups = Object.fromEntries(clusters.map((c) => [c[0]!, c]));
writeFileSync(
  join(out, "silhouettes.json"),
  JSON.stringify(
    Object.values(groups)
      .filter((g) => g.length > 1)
      .sort((a, b) => b.length - a.length),
    null,
    1,
  ),
);
await browser.close();
