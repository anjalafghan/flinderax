import { describe, expect, test } from "bun:test";
import { addHearts, HEART_TTL, makeHearts, MAX_HEARTS, pruneHearts } from "./hearts";

describe("hearts", () => {
  test("a burst is 2 or 3 hearts", () => {
    expect(makeHearts(10, 10, 0, 1, () => 0)).toHaveLength(2);
    expect(makeHearts(10, 10, 0, 1, () => 0.9)).toHaveLength(3);
    expect(new Set(makeHearts(10, 10, 0, 7, () => 0.9).map((h) => h.id)).size).toBe(3);
  });
  test("never more than 5 alive: the oldest go first", () => {
    let list = makeHearts(0, 0, 0, 1, () => 0.9);
    list = addHearts(list, makeHearts(0, 0, 10, 4, () => 0.9));
    expect(list).toHaveLength(MAX_HEARTS);
    expect(list.map((h) => h.id)).toEqual([2, 3, 4, 5, 6]);
  });
  test("cleared after 2.2 s", () => {
    const list = [...makeHearts(0, 0, 0, 1, () => 0), ...makeHearts(0, 0, 1500, 3, () => 0)];
    expect(pruneHearts(list, HEART_TTL - 1)).toHaveLength(4);
    expect(pruneHearts(list, HEART_TTL + 1)).toHaveLength(2);
    expect(pruneHearts(list, 1500 + HEART_TTL)).toHaveLength(0);
  });
});
