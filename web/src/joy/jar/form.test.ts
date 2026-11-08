import { describe, expect, test } from "bun:test";
import { applyRelax, compact, EMPTY_FILTERS, filtersToForm, putBackForm, stampDate, toggleSetting, toggleStart, vetoForm } from "./form";

describe("form serialisation", () => {
  test("an empty sentence only says when", () => {
    expect(filtersToForm({ ...EMPTY_FILTERS, when: "2026-11-05" })).toEqual({ when: "2026-11-05" });
    expect(filtersToForm(EMPTY_FILTERS)).toEqual({});
  });
  test("filled blanks go over as strings, places as a list", () => {
    const f = { ...EMPTY_FILTERS, when: "any", start: "office" as const, budget: " 2000 ", max_duration: "half_day", settings: ["home", "local"], tag: "cozy", energy: "low", max_km: "30", from: "1" };
    expect(filtersToForm(f)).toEqual({ when: "any", start: "office", budget: "2000", max_duration: "half_day", settings: ["home", "local"], tag: "cozy", energy: "low", max_km: "30", from: "1" });
  });
  test("drawing from the middle sends only that, and when/start", () => {
    const f = { ...EMPTY_FILTERS, when: "2026-11-05", budget: "500", tag: "cozy", start: "home" as const };
    expect(filtersToForm(f, true)).toEqual({ middle: "1", when: "2026-11-05", start: "home" });
  });
  test("compact drops empties and blank list items", () => {
    expect(compact({ budget: "", tag: "x", settings: ["", ""] })).toEqual({ tag: "x" });
  });
  test("tapping a place circles it; tapping again un-circles it", () => {
    expect(toggleSetting(["home"], "local")).toEqual(["home", "local"]);
    expect(toggleSetting(["home", "local"], "home")).toEqual(["local"]);
    expect(toggleStart("", "office")).toBe("office");
    expect(toggleStart("office", "office")).toBe("");
    expect(toggleStart("office", "home")).toBe("home");
  });
});

describe("put back, veto and relax", () => {
  // The server's `state` is the kept filters plus where the draw has got to, WITHOUT reroll/veto.
  const state = { when: "2026-11-05", budget: "1500", settings: ["home"], excluded: "4,9", used: "1", current: "9" };
  test("put back is the server's state plus reroll=1", () => {
    expect(putBackForm(state)).toEqual({ ...state, reroll: "1" });
  });
  test("veto is the state plus veto=1 and no reroll", () => {
    expect(vetoForm({ ...state, reroll: "1" })).toEqual({ ...state, veto: "1" });
    expect(vetoForm(state).reroll).toBeUndefined();
  });
  test("neither mutates the card's state", () => {
    const copy = structuredClone(state);
    putBackForm(state);
    vetoForm(state);
    expect(state).toEqual(copy);
  });
  test("relaxing writes the loosened filters back into the sentence", () => {
    const f = { ...EMPTY_FILTERS, when: "2026-11-05", budget: "500", settings: ["home"], tag: "cozy" };
    const next = applyRelax(f, { when: "any", budget: "800", settings: ["home", "local"] });
    expect(next.budget).toBe("800");
    expect(next.when).toBe("any");
    expect(next.settings).toEqual(["home", "local"]);
    expect(next.tag).toBe(""); // the relax form didn't keep it
    expect(f.budget).toBe("500");
  });
});

describe("stamp date", () => {
  test("the server's date wins; otherwise today, as Joy stamps it", () => {
    expect(stampDate("14 · 10 · 2026")).toBe("14 · 10 · 2026");
    expect(stampDate("", new Date(2026, 10, 8))).toBe("08 · 11 · 2026");
  });
});
