// Swiping the drawn slip, from Joy's jar.tsx and drag.ts. Pure decisions only.
import { clamp } from "./motion";

/** px of sideways swipe that commits. */
export const YES = 110;
/** The slip moves a fifth as far as the finger, so it never leaves a phone's screen. */
export const FOLLOW = 0.2;
/** The most the slip is shown moved. */
export const MOST = 26;

export const shown = (dx: number) => clamp(dx * FOLLOW, -MOST, MOST);

export type Outcome = "yes" | "back" | "none";

/**
 * What letting go does. 110 px commits; a quick flick (>50 px at >0.5 px/ms) does too.
 * Right = "It's a date", left = "Put it back". Either needs its button to exist.
 */
export function swipeOutcome(o: { dx: number; vx: number; canYes: boolean; canBack: boolean; cancelled?: boolean }): Outcome {
  if (o.cancelled) return "none";
  if (o.canYes && (o.dx > YES || (o.dx > 50 && o.vx > 0.5))) return "yes";
  if (o.canBack && (o.dx < -YES || (o.dx < -50 && o.vx < -0.5))) return "back";
  return "none";
}

/** The pose while the finger is down. A direction with nothing behind it barely moves. */
export function dragPose(dx: number, can: boolean) {
  const x = shown(can ? dx : dx * 0.2);
  return { x, y: Math.abs(x) * 0.1, r: x / 50, s: 1 - (Math.abs(x) / MOST) * 0.06 };
}

/** A sideways drag starts once it is clearly sideways; a vertical one is left to scroll the page. */
export function axisDecision(dx: number, dy: number): "start" | "abort" | "wait" {
  const ax = Math.abs(dx);
  const ay = Math.abs(dy);
  if (ax > 8 && ax > ay * 1.25) return "start";
  if (ay > 10) return "abort";
  return "wait";
}
