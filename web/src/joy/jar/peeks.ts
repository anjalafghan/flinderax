// Peek counting, straight from the server's card. The server works out the cap, whose turn it is
// and the bonus peek; the page only draws what it says.

export interface PeekInput {
  rerolls_left: number;
  max_rerolls: number;
  bonus_peek: boolean;
  free_putback: boolean;
}

export interface PeekView {
  dots: { full: boolean; bonus: boolean }[];
  label: string;
  /** " · one's a bonus" while every peek, bonus included, is still there. */
  bonusNote: boolean;
  canPutBack: boolean;
}

export function peekView(c: PeekInput): PeekView {
  const dots = Array.from({ length: Math.max(0, c.max_rerolls) }, (_, i) => ({
    full: i < c.rerolls_left,
    bonus: c.bonus_peek && i === c.max_rerolls - 1,
  }));
  const left = c.rerolls_left;
  const label = c.free_putback
    ? "This one goes back for free."
    : left > 0
      ? `${left} ${left === 1 ? "peek" : "peeks"} left`
      : "No more peeking. This one's yours.";
  return {
    dots,
    label,
    bonusNote: !c.free_putback && left > 0 && c.bonus_peek && left === c.max_rerolls,
    canPutBack: left > 0 || c.free_putback,
  };
}

/** The dot that was spent by the put-back that produced this card (shown full, then empties). */
export function spentDot(newLeft: number, wasPutBack: boolean, free: boolean): number {
  return wasPutBack && !free ? newLeft : -1;
}
