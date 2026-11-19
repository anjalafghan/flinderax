// The keepsake catalogue's shape: one entry per slip, saying what physical object that slip becomes
// once we keep it. The date itself (title, place, day, stars) comes from the API at runtime; a spec
// adds everything that makes the object this one: its form, its drawing, its fixed colours, its
// own printed words and what it does when touched.

/** The physical shapes a keepsake can take. Each one is drawn in styles/exhibits.css. */
export const FORMS = [
  "card", "paw", "stub", "wristband", "tag", "luggage", "label", "sticker", "coaster", "napkin",
  "slip", "sleeve", "token", "plate", "map", "postcard", "envelope", "letter", "badge", "bookmark",
  "index", "certificate", "stamp", "playbill", "matchbook", "jcard", "cartridge", "record", "film", "scorecard",
  "menu", "bag", "cup", "packet", "chart", "pass", "slate", "specimen", "crate", "chit",
] as const;
export type Form = (typeof FORMS)[number];

/** What a keepsake does when touched (and once when it first comes into view). */
export const ANIMS = [
  "jingle", "spray", "steam", "sparkle", "flicker", "sway", "flip", "pour", "bubbles", "flutter",
  "bounce", "ripple", "snow", "fireflies", "thump", "ring", "float", "tick", "walk", "spin",
  "zoom", "shake", "confetti", "twinkle", "drip", "wave", "glow", "beat",
] as const;
export type Anim = (typeof ANIMS)[number];

export interface Spec {
  /** The slip's own title, lower-cased (placeholders like {b} kept): how a kept date finds its keepsake. */
  key: string;
  /** For slips with the same title: which visit this object is for (1, 2, …). */
  visit?: number;
  /** The object's name, as on a museum placard: "Cat paw loyalty card". */
  label: string;
  form: Form;
  /** A Lucide icon name ("Cat") or one of our own drawings ("x:chopsticks"). */
  motif: string;
  anim: Anim;
  /** Fixed colours of the object: the same in light and dark. */
  paper: string;
  ink: string;
  accent: string;
  /** The brand or eyebrow printed on it. */
  head: string;
  /** One or two short lines that only this object would carry. */
  lines: string[];
}
