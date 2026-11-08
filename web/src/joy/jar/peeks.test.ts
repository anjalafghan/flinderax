import { describe, expect, test } from "bun:test";
import { peekView, spentDot } from "./peeks";

const card = (o: Partial<Parameters<typeof peekView>[0]> = {}) => ({ rerolls_left: 3, max_rerolls: 3, bonus_peek: false, free_putback: false, ...o });

describe("peek counting from the server's card", () => {
  test("three peeks, all there", () => {
    const v = peekView(card());
    expect(v.dots.map((d) => d.full)).toEqual([true, true, true]);
    expect(v.label).toBe("3 peeks left");
    expect(v.canPutBack).toBe(true);
    expect(v.bonusNote).toBe(false);
  });
  test("it's your turn: a fourth, bonus peek", () => {
    const v = peekView(card({ rerolls_left: 4, max_rerolls: 4, bonus_peek: true }));
    expect(v.dots).toHaveLength(4);
    expect(v.dots.map((d) => d.bonus)).toEqual([false, false, false, true]);
    expect(v.label).toBe("4 peeks left");
    expect(v.bonusNote).toBe(true);
  });
  test("the bonus note goes once one is spent", () => {
    const v = peekView(card({ rerolls_left: 3, max_rerolls: 4, bonus_peek: true }));
    expect(v.bonusNote).toBe(false);
    expect(v.dots.map((d) => d.full)).toEqual([true, true, true, false]);
  });
  test("one left reads singular", () => {
    expect(peekView(card({ rerolls_left: 1 })).label).toBe("1 peek left");
  });
  test("after the last: no more peeking, and no put-back button", () => {
    const v = peekView(card({ rerolls_left: 0 }));
    expect(v.label).toBe("No more peeking. This one's yours.");
    expect(v.canPutBack).toBe(false);
    expect(v.dots.every((d) => !d.full)).toBe(true);
  });
  test("a generated slip goes back free, even with no peeks left", () => {
    const v = peekView(card({ rerolls_left: 0, free_putback: true }));
    expect(v.label).toBe("This one goes back for free.");
    expect(v.canPutBack).toBe(true);
  });
  test("a put-back that spent a peek shows that dot full until the card lands", () => {
    expect(spentDot(2, true, false)).toBe(2);
    expect(spentDot(2, false, false)).toBe(-1);
    expect(spentDot(2, true, true)).toBe(-1); // free: nothing was spent
  });
});
