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
type State = { box: Box | null; text: string[]; moving: number; rect: string | null };

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
await page.goto("file://" + sheet);
await page.evaluate(() => document.fonts.ready);
// hit-testing only works on screen: make the window as tall as the whole sheet
await page.setViewportSize({ width: 1600, height: await page.evaluate(() => document.documentElement.scrollHeight) });

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
      const stops = [...grad.querySelectorAll("stop")].map((s) => Number(getComputedStyle(s).stopOpacity));
      return stops.length > 0 && Math.max(...stops) < 0.6;
    };
    const solid = (e: Element, svg: Element) => e !== svg && svg.contains(e) && !inText(e) && !e.hasAttribute("filter") && !seeThrough(e) && visible(e, svg) >= 0.6;
    const res: Record<string, { box: [number, number, number, number] | null; text: string[]; moving: number; rect: string | null }> = {};
    for (const tile of document.querySelectorAll<HTMLElement>(".tile")) {
      const id = tile.dataset.id!;
      const svg = tile.querySelector("svg")!;
      const R = svg.getBoundingClientRect();
      const k = 600 / R.width;
      let box: [number, number, number, number] | null = null;
      let biggest: { el: Element; area: number } | null = null;
      const text: string[] = [];
      for (const el of svg.querySelectorAll(SHAPES)) {
        if (el.closest("defs,clipPath,pattern,mask,symbol") || (el.tagName !== "text" && inText(el))) continue;
        if (visible(el, svg) < 0.05) continue;
        const r = el.getBoundingClientRect();
        if (r.width < 1 || r.height < 1) continue;
        const b: [number, number, number, number] = [(r.left - R.left) * k, (r.top - R.top) * k, (r.right - R.left) * k, (r.bottom - R.top) * k];
        box = box ? [Math.min(box[0], b[0]), Math.min(box[1], b[1]), Math.max(box[2], b[2]), Math.max(box[3], b[3])] : b;
        if (el.tagName === "text") {
          // what the text sits on: the first thing under it that isn't text
          // the text's own box, in its own (possibly rotated) frame, mapped to the screen
          const t = el as SVGTextElement, bb = t.getBBox(), m = t.getScreenCTM()!;
          const at = (fx: number, fy: number) => { const p = new DOMPoint(bb.x + bb.width * fx, bb.y + bb.height * fy).matrixTransform(m); return [p.x, p.y] as const; };
          const under = (x: number, y: number) => document.elementsFromPoint(x, y).find((e) => solid(e, svg)) ?? null;
          const [cx, cy] = at(0.5, 0.5);
          const words = (el.textContent ?? "").trim().slice(0, 24);
          const top = document.elementsFromPoint(cx, cy).find((e) => e === el || el.contains(e) || solid(e, svg));
          if (top && top !== el && !el.contains(top)) text.push(`“${words}” is covered`);
          if (under(cx, cy)) {
            const corners = [at(0.1, 0.3), at(0.9, 0.3), at(0.1, 0.75), at(0.9, 0.75)];
            if (corners.some(([x, y]) => !under(x, y))) text.push(`“${words}” runs off its object`);
          }
        } else if (!el.closest("[clip-path]") && Number(getComputedStyle(el).fillOpacity) > 0.5 && getComputedStyle(el).fill !== "none") {
          const area = r.width * r.height;
          if (!biggest || area > biggest.area) biggest = { el, area };
        }
      }
      let rect: string | null = null;
      if (biggest) {
        const el = biggest.el;
        const plain = (el.tagName === "rect" && Number(el.getAttribute("rx") ?? 0) < 10) || (el.tagName === "path" && /^M[-\d. ]+(?:[HV][-\d. ]+){3,4}Z$/i.test((el.getAttribute("d") ?? "").trim()));
        if (plain) rect = "the object is a plain rectangle";
      }
      res[id] = { box, text, moving: tile.getAnimations({ subtree: true }).length, rect };
    }
    return res;
  });

const settle = async () => {
  for (let i = 0; i < 3; i++) {
    await page.waitForTimeout(120);
    await page.evaluate(() => document.getAnimations().forEach((a) => { try { if (a.effect?.getTiming().iterations !== Infinity) a.finish(); } catch { /* still pending */ } }));
  }
};

const closed: Record<string, State> = await measure();
await page.evaluate(() => document.querySelectorAll(".ksc").forEach((e) => e.classList.add("open")));
await page.waitForTimeout(60);
const started = await page.evaluate(() => Object.fromEntries([...document.querySelectorAll<HTMLElement>(".tile")].map((t) => [t.dataset.id!, t.getAnimations({ subtree: true }).length])));
await settle();
const opened: Record<string, State> = await measure();

const area = (b: Box | null) => (b ? Math.max(0, b[2] - b[0]) * Math.max(0, b[3] - b[1]) : 0);
const report: Record<string, { crop: Box; issues: string[] }> = {};
for (const id of Object.keys(closed)) {
  const c = closed[id]!, o = opened[id]!;
  const u: Box | null = c.box && o.box ? [Math.min(c.box[0], o.box[0]), Math.min(c.box[1], o.box[1]), Math.max(c.box[2], o.box[2]), Math.max(c.box[3], o.box[3])] : c.box ?? o.box;
  const pad = 12;
  const crop: Box = u ? [Math.floor(u[0] - pad), Math.floor(u[1] - pad), Math.ceil(u[2] - u[0] + 2 * pad), Math.ceil(u[3] - u[1] + 2 * pad)] : [0, 0, 600, 600];
  const issues = [...new Set([...c.text.filter((t) => !t.endsWith("is covered")).map((t) => t + " (closed)"), ...o.text.map((t) => t + " (open)")])];
  if (c.rect) issues.push(c.rect);
  if (!started[id]) issues.push("opening it moves nothing");
  if (area(c.box) < 0.35 * area(u)) issues.push("the closed state shows little of the object");
  report[id] = { crop, issues };
}

await page.evaluate((bad) => bad.forEach((id) => document.querySelector(`.tile[data-id="${id}"]`)?.classList.add("bad")), Object.keys(report).filter((id) => report[id]!.issues.length));
await page.screenshot({ path: join(out, "sheet-open.png"), fullPage: true });
await page.evaluate(() => document.querySelectorAll(".ksc").forEach((e) => e.classList.remove("open")));
await settle();
await page.screenshot({ path: join(out, "sheet-closed.png"), fullPage: true });
writeFileSync(join(out, "report.json"), JSON.stringify(report, null, 1));
await browser.close();
