// Pure helpers for the Kept page: photo sizing, grouping and keyboard order, request bodies.
import type { Month } from "./types";

// ---- photos (kept.js `shrink`) ----
export const MAX_EDGE = 2048;
/** Files under this that the server already takes as-is go up untouched. */
export const SKIP_BELOW_BYTES = 2.5e6;
export const JPEG_QUALITY = 0.88;

/** Big or unusual files get redrawn as a JPEG, if the browser can decode them at all. */
export function needsShrink(size: number, type: string, canDecode: boolean): boolean {
  if (!canDecode) return false;
  return !(size < SKIP_BELOW_BYTES && /^image\/(jpeg|png|webp)$/.test(type));
}

/** The canvas size: longest edge capped at `max`, never scaled up, never below 1 px. */
export function fitEdge(width: number, height: number, max = MAX_EDGE): { width: number; height: number } {
  const longest = Math.max(width, height);
  const k = longest > 0 ? Math.min(1, max / longest) : 1;
  return { width: Math.max(1, Math.round(width * k)), height: Math.max(1, Math.round(height * k)) };
}

/** The progress line while a photo goes up; once every byte is sent the server is developing it. */
export function uploadLabel(loaded: number, total: number): string {
  if (total <= 0) return "Uploading…";
  return loaded < total ? `Uploading… ${Math.round((loaded / total) * 100)}%` : "Developing…";
}

/** The same line when several photos go up one after another: "Photo 2 of 3: Uploading… 40%". */
export const photoOf = (n: number, of: number, label: string) => (of > 1 ? `Photo ${n} of ${of}: ${label}` : label);

// ---- the book ----
/** The slip's angle and tape come from its id, so they hold still between visits. */
export const rotOf = (id: number) => ((id % 7) + 7) % 7;
export const tapeOf = (id: number) => ((id % 3) + 3) % 3;

export const dateWord = (n: number) => (n === 1 ? "date" : "dates");

/** "Together, since X": Joy's first month is the last one listed (newest first). */
export function sinceLabel(months: Month[]): string {
  return months.length ? months[months.length - 1].label : "";
}

/** Every slip id in reading order (newest month first), for keyboard moves. */
export function flatIds(months: Month[]): number[] {
  return months.flatMap((m) => m.items.map((h) => h.id));
}

export type Move = "next" | "prev" | "first" | "last";

/** j/k from anywhere in the book; arrows and Home/End once a slip itself has focus. */
export function moveFor(key: string, onSlip: boolean): Move | null {
  if (key === "j") return "next";
  if (key === "k") return "prev";
  if (!onSlip) return null;
  if (key === "ArrowDown" || key === "ArrowRight") return "next";
  if (key === "ArrowUp" || key === "ArrowLeft") return "prev";
  if (key === "Home") return "first";
  if (key === "End") return "last";
  return null;
}

/** Index to land on; -1 current means "none yet". Stops at the ends. */
export function stepIndex(length: number, current: number, move: Move): number {
  if (length <= 0) return -1;
  switch (move) {
    case "first":
      return 0;
    case "last":
      return length - 1;
    case "next":
      return Math.min(length - 1, current + 1);
    case "prev":
      return current < 0 ? 0 : Math.max(0, current - 1);
  }
}

/** The same angle every visit: a stable number from 0 to 1 per index (reveal.ts). */
export function dealSeed(i: number): number {
  const v = Math.sin(i * 12.9898 + 4.1) * 43758.5453;
  return v - Math.floor(v);
}

/** Where a slip is dealt in from. */
export function dealPose(i: number): { y: number; deg: number } {
  return { y: 22, deg: (dealSeed(i) - 0.5) * 5 };
}

/** "/joy/kept#k5" -> path and hash (hash keeps its #, or ""). */
export function splitTo(to: string): { path: string; hash: string } {
  const at = to.indexOf("#");
  return at < 0 ? { path: to, hash: "" } : { path: to.slice(0, at), hash: to.slice(at) };
}

export const isBudgetHash = (hash: string) => /^#(k-)?budget$/.test(hash);

// ---- request bodies ----
export function rateBody(rating: number | null, note: string) {
  return { rating: rating ? String(rating) : "", note };
}

export function lineBody(line: string) {
  return { line: line.trim() };
}

/** Blank clears an actual or a cab fare; `update_estimate` is "1" or "". */
export function costBody(actual: string, updateEstimate: boolean, cabs: string[]) {
  const body: Record<string, string> = { actual: actual.trim(), update_estimate: updateEstimate ? "1" : "" };
  cabs.forEach((fare, i) => {
    body[`cab_${i}`] = fare.trim();
  });
  return body;
}

export function whenBody(ids: string, at: string) {
  return { ids, at };
}

export function budgetBody(amount: string) {
  return { amount: amount.trim(), back: "/joy/kept" };
}

/** Money spent (or put in the jar): blank date means today, which the server reads. */
export function moneyBody(amount: string, note: string, on = "") {
  return { amount: amount.trim(), note: note.trim(), on };
}

export function goalBody(id: string) {
  return { id };
}

export type RolloverChoice = "roll" | "save" | "fresh";
export const rolloverBody = (choice: RolloverChoice) => ({ choice });
