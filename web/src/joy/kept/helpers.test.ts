import { describe, expect, test } from "bun:test";
import {
  budgetBody, costBody, dealPose, dealSeed, fitEdge, flatIds, isBudgetHash, lineBody, moneyBody, moveFor, needsShrink,
  photoOf, rateBody, rolloverBody, rotOf, sinceLabel, splitTo, stepIndex, tapeOf, uploadLabel, whenBody,
} from "./helpers";
import type { KeptView, Month } from "./types";

const slip = (id: number) => ({ id }) as KeptView;
const months: Month[] = [
  { label: "October", href: "/joy/year/2026/10", items: [slip(9), slip(8)] },
  { label: "September", href: "/joy/year/2026/9", items: [slip(3)] },
];

describe("photo shrink maths", () => {
  test("small jpeg/png/webp go up untouched", () => {
    expect(needsShrink(1_000_000, "image/jpeg", true)).toBe(false);
    expect(needsShrink(2_499_999, "image/webp", true)).toBe(false);
  });
  test("big or unusual files are redrawn", () => {
    expect(needsShrink(2_500_000, "image/jpeg", true)).toBe(true);
    expect(needsShrink(100, "image/heic", true)).toBe(true);
    expect(needsShrink(100, "", true)).toBe(true);
  });
  test("no decoder: upload as it is", () => expect(needsShrink(9e6, "image/heic", false)).toBe(false));
  test("longest edge caps at 2048, keeping the ratio", () => {
    expect(fitEdge(4000, 3000)).toEqual({ width: 2048, height: 1536 });
    expect(fitEdge(3000, 4000)).toEqual({ width: 1536, height: 2048 });
  });
  test("never scales up", () => expect(fitEdge(800, 600)).toEqual({ width: 800, height: 600 }));
  test("never collapses to zero", () => {
    expect(fitEdge(20000, 1).height).toBe(1);
    expect(fitEdge(0, 0)).toEqual({ width: 1, height: 1 });
  });
  test("upload label", () => {
    expect(uploadLabel(50, 200)).toBe("Uploading… 25%");
    expect(uploadLabel(200, 200)).toBe("Developing…");
    expect(uploadLabel(0, 0)).toBe("Uploading…");
  });
  test("several photos say which one is going up", () => {
    expect(photoOf(2, 3, "Uploading… 40%")).toBe("Photo 2 of 3: Uploading… 40%");
    expect(photoOf(1, 1, "Developing…")).toBe("Developing…");
  });
});

describe("the book", () => {
  test("rotation and tape follow the id", () => {
    expect(rotOf(15)).toBe(1);
    expect(tapeOf(10)).toBe(1);
    expect(rotOf(-1)).toBe(6);
  });
  test("grouping: first month is last in the list; ids flatten in order", () => {
    expect(sinceLabel(months)).toBe("September");
    expect(sinceLabel([])).toBe("");
    expect(flatIds(months)).toEqual([9, 8, 3]);
  });
  test("keyboard moves", () => {
    expect(moveFor("j", false)).toBe("next");
    expect(moveFor("k", false)).toBe("prev");
    expect(moveFor("ArrowDown", false)).toBeNull();
    expect(moveFor("ArrowDown", true)).toBe("next");
    expect(moveFor("End", true)).toBe("last");
    expect(moveFor("x", true)).toBeNull();
  });
  test("step stops at the ends", () => {
    expect(stepIndex(3, -1, "next")).toBe(0);
    expect(stepIndex(3, 2, "next")).toBe(2);
    expect(stepIndex(3, 0, "prev")).toBe(0);
    expect(stepIndex(3, -1, "prev")).toBe(0);
    expect(stepIndex(3, 1, "last")).toBe(2);
    expect(stepIndex(0, -1, "next")).toBe(-1);
  });
  test("deal angles are stable and small", () => {
    expect(dealSeed(4)).toBe(dealSeed(4));
    for (let i = 0; i < 20; i++) expect(Math.abs(dealPose(i).deg)).toBeLessThanOrEqual(2.5);
  });
  test("Go targets split into path and hash", () => {
    expect(splitTo("/joy/kept#k5")).toEqual({ path: "/joy/kept", hash: "#k5" });
    expect(splitTo("/joy/kept")).toEqual({ path: "/joy/kept", hash: "" });
    expect(isBudgetHash("#budget")).toBe(true);
    expect(isBudgetHash("#k-budget")).toBe(true);
    expect(isBudgetHash("#k-savings")).toBe(false);
  });
});

describe("request bodies", () => {
  test("rating clears with an empty string", () => {
    expect(rateBody(4, "lovely")).toEqual({ rating: "4", note: "lovely" });
    expect(rateBody(null, "")).toEqual({ rating: "", note: "" });
  });
  test("line is trimmed", () => expect(lineBody("  hi ")).toEqual({ line: "hi" }));
  test("cost carries one cab field per person", () => {
    expect(costBody(" 5000 ", true, ["200", ""])).toEqual({ actual: "5000", update_estimate: "1", cab_0: "200", cab_1: "" });
    expect(costBody("", false, []).update_estimate).toBe("");
  });
  test("when, budget, money, rollover", () => {
    expect(whenBody("4,5", "2026-10-18T19:00")).toEqual({ ids: "4,5", at: "2026-10-18T19:00" });
    expect(budgetBody(" 6000")).toEqual({ amount: "6000", back: "/joy/kept" });
    expect(moneyBody("1800 ", " Dosa ", "")).toEqual({ amount: "1800", note: "Dosa", on: "" });
    expect(rolloverBody("save")).toEqual({ choice: "save" });
  });
});
