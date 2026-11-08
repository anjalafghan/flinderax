// Small entrances done with the Web Animations API, skipped for prefers-reduced-motion.
import { dealParams } from "./layout";

export const reduced = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

/** Polaroids dealt onto the wall at seeded angles, one after another. */
export function dealIn(root: ParentNode, seed: number): void {
  if (reduced()) return;
  root.querySelectorAll<HTMLElement>(".y-photos > .y-polaroid").forEach((el, i) => {
    if (typeof el.animate !== "function") return;
    const { rot, dx, delay } = dealParams(seed, i);
    el.animate(
      [
        { transform: `translate(${dx}px, -90px) rotate(${rot}deg) scale(1.08)`, opacity: 0 },
        { transform: "none", opacity: 1 },
      ],
      { duration: 560, delay, easing: "cubic-bezier(.2,.8,.25,1.15)", fill: "backwards" },
    );
  });
}

/** A slip per date drops into the jar's mouth. */
export function dropSlips(root: ParentNode): void {
  if (reduced()) return;
  root.querySelectorAll<SVGGElement>(".y-jar-slip").forEach((g, i) => {
    if (typeof g.animate !== "function") return;
    g.animate([{ translate: "0 -230px", opacity: 0 }, { translate: "0 0", opacity: 1 }], {
      duration: 520,
      delay: 450 + i * 110,
      easing: "cubic-bezier(.3,.7,.3,1.15)",
      fill: "backwards",
    });
  });
}

/** Month stickers pressed on one by one. */
export function pressStickers(root: ParentNode): void {
  if (reduced()) return;
  root.querySelectorAll<HTMLElement>(".m-stickers span").forEach((s, i) => {
    if (typeof s.animate !== "function") return;
    s.animate(
      [
        { scale: "0", opacity: 0 },
        { scale: "1", opacity: 1 },
      ],
      { duration: 340, delay: 180 + i * 55, easing: "cubic-bezier(.3,.6,.3,1.4)", fill: "backwards" },
    );
  });
}

export type Turn = "next" | "prev";

/** The month sheet turns like a page: it swings away from the spine, the next one swings in. */
export function turnOut(sheet: HTMLElement, dir: Turn): Promise<void> {
  if (reduced() || typeof sheet.animate !== "function") return Promise.resolve();
  const a = sheet.animate(
    [
      { transform: "rotateY(0)", opacity: 1 },
      { transform: `rotateY(${dir === "next" ? -62 : 62}deg)`, opacity: 0.1 },
    ],
    { duration: 260, easing: "cubic-bezier(.5,0,.8,.4)", fill: "forwards" },
  );
  sheet.style.transformOrigin = dir === "next" ? "left center" : "right center";
  return a.finished.then(
    () => undefined,
    () => undefined,
  );
}

export function turnIn(sheet: HTMLElement, dir: Turn): void {
  if (reduced() || typeof sheet.animate !== "function") return;
  sheet.style.transformOrigin = dir === "next" ? "right center" : "left center";
  sheet.animate(
    [
      { transform: `rotateY(${dir === "next" ? 40 : -40}deg)`, opacity: 0 },
      { transform: "rotateY(0)", opacity: 1 },
    ],
    { duration: 340, easing: "cubic-bezier(.2,.7,.2,1)" },
  );
}
