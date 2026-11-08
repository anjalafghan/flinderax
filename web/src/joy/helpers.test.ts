import { describe, expect, test } from "bun:test";
import type { FormView, Opt } from "./api";
import { errorField, formToState, tidyTitle, toPayload } from "./form";
import { rupees, standing, toPaise } from "./money";
import { daysLeftInMonth, nudgeFor } from "./nudge";
import { mapGoTo, navFor, pageKey } from "./paths";
import { quickPick, toLocalInput } from "./plan";
import { filterGroups, matches } from "./slips";
import { createUndoTimer, RECHECK_MS, UNDO_MS, type Scheduler } from "./undo";

describe("path mapping", () => {
  test("Go.to stays under /joy, and #middle becomes the Middle page", () => {
    expect(mapGoTo("/joy/slips")).toBe("/joy/slips");
    expect(mapGoTo("/joy/kept#k5")).toBe("/joy/kept#k5");
    expect(mapGoTo("/joy#middle")).toBe("/joy/middle");
    expect(mapGoTo("/joy")).toBe("/joy");
  });
  test("anything else falls back to the jar", () => {
    expect(mapGoTo("https://evil.example")).toBe("/joy");
    expect(mapGoTo("//evil.example")).toBe("/joy");
    expect(mapGoTo("/joyful")).toBe("/joy");
    expect(mapGoTo("")).toBe("/joy");
    expect(mapGoTo(undefined)).toBe("/joy");
  });
  test("pageKey drives the nav underline", () => {
    expect(pageKey("/joy")).toBe("jar");
    expect(pageKey("/joy/")).toBe("jar");
    expect(pageKey("/joy/slips/new")).toBe("slips");
    expect(pageKey("/joy/slips/12")).toBe("slips");
    expect(pageKey("/joy/year/2026")).toBe("year");
    expect(pageKey("/joy/year/2026/3")).toBe("month");
    expect(pageKey("/joy/e/abc")).toBe("envelope");
    expect(navFor("month")).toBe("year");
    expect(navFor("settings")).toBeNull();
  });
});

describe("undo timer", () => {
  function fake() {
    let now = 0;
    let next: { at: number; fn: () => void } | null = null;
    const s: Scheduler = {
      set: (fn, ms) => ((next = { at: now + ms, fn }), next),
      clear: (h) => {
        if (next === h) next = null;
      },
    };
    const advance = (ms: number) => {
      const end = now + ms;
      while (next && next.at <= end) {
        const n = next;
        next = null;
        now = n.at;
        n.fn();
      }
      now = end;
    };
    return { s, advance };
  }

  test("expires after 6.5 s", () => {
    const { s, advance } = fake();
    let expired = 0;
    createUndoTimer({ onExpire: () => expired++, isHeld: () => false, scheduler: s }).start();
    advance(UNDO_MS - 1);
    expect(expired).toBe(0);
    advance(1);
    expect(expired).toBe(1);
  });
  test("waits while hovered or focused, rechecking every 1.5 s", () => {
    const { s, advance } = fake();
    let held = true;
    let expired = 0;
    createUndoTimer({ onExpire: () => expired++, isHeld: () => held, scheduler: s }).start();
    advance(UNDO_MS + RECHECK_MS * 3);
    expect(expired).toBe(0);
    held = false;
    advance(RECHECK_MS);
    expect(expired).toBe(1);
  });
  test("cancel stops it", () => {
    const { s, advance } = fake();
    let expired = 0;
    const t = createUndoTimer({ onExpire: () => expired++, isHeld: () => false, scheduler: s });
    t.start();
    t.cancel();
    advance(UNDO_MS * 2);
    expect(expired).toBe(0);
    expect(t.done).toBe(true);
  });
});

describe("nudge maths", () => {
  test("nothing under 80%, or without a budget", () => {
    expect(nudgeFor(2000000, 1500000, 9)).toBeNull();
    expect(nudgeFor(null, 100, 9)).toBeNull();
    expect(nudgeFor(0, 100, 9)).toBeNull();
  });
  test("85% with 9 days left", () => {
    const n = nudgeFor(2000000, 1700000, 9);
    expect(n?.text).toBe("Dates at 85% with 9 days left: keep the next 2 dates under ₹1,500 each");
  });
  test("one date near the end, and over budget", () => {
    expect(nudgeFor(2000000, 1700000, 4)?.text).toBe("Dates at 85% with 4 days left: keep the next date under ₹3,000");
    expect(nudgeFor(2000000, 2200000, 1)?.text).toContain("1 day left");
    expect(nudgeFor(2000000, 2200000, 5)?.perDatePaise).toBe(0);
  });
  test("days left counts today", () => {
    expect(daysLeftInMonth(new Date(2026, 10, 30))).toBe(1);
    expect(daysLeftInMonth(new Date(2026, 10, 1))).toBe(30);
  });
});

describe("money", () => {
  test("formatting and parsing", () => {
    expect(rupees(150000)).toBe("₹1,500");
    expect(rupees(-1250000)).toBe("₹12,500");
    expect(toPaise("1,500")).toBe(150000);
    expect(toPaise("₹20.5")).toBe(2050);
    expect(toPaise("abc")).toBeNull();
    expect(toPaise("")).toBeNull();
  });
  test("settle-up wording", () => {
    expect(standing(250000).text).toBe("Partner owes you ₹2,500");
    expect(standing(-250000).text).toBe("You owe ₹2,500");
    expect(standing(0).text).toBe("All square");
  });
});

describe("plan quick picks", () => {
  test("tonight is 7 pm before evening, else the next half hour after an hour", () => {
    expect(toLocalInput(quickPick("tonight", new Date(2026, 10, 2, 10, 0)))).toBe("2026-11-02T19:00");
    expect(toLocalInput(quickPick("tonight", new Date(2026, 10, 2, 19, 10)))).toBe("2026-11-02T20:30");
    expect(toLocalInput(quickPick("tonight", new Date(2026, 10, 2, 19, 40)))).toBe("2026-11-02T21:00");
  });
  test("tomorrow evening and Saturday", () => {
    const mon = new Date(2026, 10, 2, 10, 0);
    expect(toLocalInput(quickPick("tomorrow", mon))).toBe("2026-11-03T19:00");
    expect(toLocalInput(quickPick("saturday", mon))).toBe("2026-11-07T11:00");
    expect(toLocalInput(quickPick("saturday", new Date(2026, 10, 7, 9, 0)))).toBe("2026-11-14T11:00");
  });
});

const opt = (value: string, selected = false, label = value): Opt => ({ value, label, selected });
const view = (over: Partial<FormView> = {}): FormView => ({
  id: null, title: "  Sunset at\nMarine Drive ", notes: "", est_cost: "800", venue: "", area: "", location: "", tags: "romantic, foodie",
  currency: "₹", couple: "A & B", error: "",
  settings: [opt("home"), opt("local", true)], durations: [opt("short", true), opt("half_day")], energies: [opt("low"), opt("med", true)],
  people: [opt("A", true), opt("B")], pair_opts: [opt("7", true), opt("9")], me_name: "A",
  closed_opts: [opt("mon", true), opt("tue")], season_from_opts: [opt("11", true)], season_to_opts: [opt("5", true)],
  hours: "", hours_today: "", best_times: [opt("evening")], minutes: "", bookings: [opt("walk_in")], book_ahead_days: "",
  covers: [opt("covered")], cover_unknown: true, bring: "", station: "", station_walk_min: "", details_open: false, ...over,
});

describe("slip form", () => {
  test("view -> state -> payload", () => {
    const p = toPayload(formToState(view()));
    expect(p.title).toBe("Sunset at Marine Drive");
    expect(p.setting).toBe("local");
    expect(p.duration).toBe("short");
    expect(p.energy).toBe("med");
    expect(p.est_cost).toBe("800");
    expect(p.pairs).toEqual([7]);
    expect(p.closed).toEqual(["mon"]);
    expect(p.season_from).toBe("11");
    expect(p.season_to).toBe("5");
    expect(p.cover).toBe("");
    expect(p.added_by).toBe("A");
  });
  test("a blank view still sends setting and duration", () => {
    const p = toPayload(formToState(view({ settings: [opt("home"), opt("local")], durations: [opt("short")], people: [opt("A"), opt("B")] })));
    expect(p.setting).toBe("home");
    expect(p.duration).toBe("short");
    expect(p.added_by).toBe("A");
  });
  test("error -> field", () => {
    expect(errorField("Give it a name.")).toBe("title");
    expect(errorField("The cost should be a whole number.")).toBe("est_cost");
    expect(errorField("Location should look like")).toBe("location");
    expect(errorField("Hours should look like")).toBe("hours");
    expect(errorField("Pick a setting and duration.")).toBeNull();
    expect(tidyTitle(" a \n b ")).toBe("a b");
  });
});

describe("slip search", () => {
  const a = { id: 1, title: "Café crawl", place: "Bandra", tags: ["foodie"], setting: "Local", notes: "" } as never;
  test("every word must match, ignoring accents", () => {
    expect(matches(a, "cafe bandra")).toBe(true);
    expect(matches(a, "cafe juhu")).toBe(false);
    expect(matches(a, "  ")).toBe(true);
  });
  test("groups with no match drop out only while searching", () => {
    const groups = [{ key: "local", label: "Local", items: [a] }, { key: "home", label: "Home", items: [] }];
    expect(filterGroups(groups, "").shown.length).toBe(2);
    expect(filterGroups(groups, "zzz").shown.length).toBe(0);
    expect(filterGroups(groups, "cafe").total).toBe(1);
  });
});
