// Regulars: weekday labels, form payloads and error text. Pure.
import { ApiError, type Go } from "../api";

export const WEEKDAY_PLURALS = ["Sundays", "Mondays", "Tuesdays", "Wednesdays", "Thursdays", "Fridays", "Saturdays"] as const;

/** "2,4" -> "Tuesdays and Thursdays" (same as the server's weekdays_label). */
export function weekdaysLabel(weekdays: string): string {
  const names = weekdays
    .split(",")
    .map((d) => d.trim())
    .filter((d) => /^\d+$/.test(d))
    .map((d) => WEEKDAY_PLURALS[Number(d)])
    .filter((n): n is (typeof WEEKDAY_PLURALS)[number] => n !== undefined);
  if (names.length === 0) return "";
  if (names.length === 1) return names[0] as string;
  return `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
}

/** The weekdays as the server keeps them: only 0-6, each once, in order. "4,2,4,9,x" -> [2,4]. */
export function cleanDays(days: ReadonlyArray<string | number>): number[] {
  const out = new Set<number>();
  for (const d of days) {
    const s = String(d).trim();
    if (!/^\d+$/.test(s)) continue;
    const n = Number(s);
    if (n >= 0 && n <= 6) out.add(n);
  }
  return [...out].sort((a, b) => a - b);
}

/** Whole rupees from what was typed: commas and spaces dropped, digits only. Blank -> "". null if unusable. */
export function parseRupees(input: string): string | null {
  const t = input.replace(/[,\s]/g, "");
  if (t === "") return "";
  return /^\d{1,12}$/.test(t) ? String(Number(t)) : null;
}

export interface NewRegularForm {
  title: string;
  place: string;
  maps_url: string;
  days: number[];
  cost: string;
  since: string;
}

export interface NewRegularBody {
  title: string;
  place: string;
  maps_url: string;
  days: number[];
  cost: string;
  since: string;
}

/** Checks the add form the way the server will; the second member is a message to show, or "". */
export function newRegularBody(f: NewRegularForm): { body: NewRegularBody | null; error: string } {
  const title = f.title.trim().slice(0, 60);
  if (!title) return { body: null, error: "Give it a name." };
  const days = cleanDays(f.days);
  if (days.length === 0) return { body: null, error: "Pick at least one day." };
  const cost = parseRupees(f.cost);
  if (cost === null) return { body: null, error: "The usual spend should be a number, like 300." };
  const maps = f.maps_url.trim();
  return {
    body: { title, place: f.place.trim().slice(0, 120), maps_url: /^https?:\/\//i.test(maps) ? maps : "", days, cost, since: f.since },
    error: "",
  };
}

export interface WentBody {
  day: string;
  amount: string;
  place: string;
  back: string;
}

/** One-tap "Went": blank amount and place mean the regular's usual. */
export const wentUsual = (day: string, back = ""): WentBody => ({ day, amount: "", place: "", back });

/** "Somewhere else, or a different amount". null amount means what was typed isn't a number. */
export function wentElse(day: string, place: string, amount: string, back = ""): { body: WentBody | null; error: string } {
  const a = parseRupees(amount);
  if (a === null) return { body: null, error: "Spent should be a number, like 300." };
  return { body: { day, amount: a, place: place.trim().slice(0, 80), back }, error: "" };
}

export const skipBody = (day: string, back = "") => ({ day, back });

/** The server's 422 is `{error}`; anything else falls back to the text it sent, or a plain line. */
export function apiMessage(e: unknown, fallback = "That didn’t work. Try again."): string {
  if (e instanceof ApiError) {
    const d = e.data as { error?: unknown } | null;
    if (d && typeof d.error === "string" && d.error) return d.error;
    if (e.status >= 500) return fallback;
    return e.message && !e.message.startsWith("{") ? e.message : fallback;
  }
  if (e instanceof TypeError) return "No connection. Try again in a moment.";
  return fallback;
}

export const isGo = (g: unknown): g is Go => !!g && typeof (g as Go).to === "string";

/** A path under /joy for `back` (the server only accepts paths that start with "/"). */
export const regularPath = (id: number | string) => `/joy/regulars/${id}`;

/** "1 night" / "3 nights" */
export const nightsWord = (n: number) => (n === 1 ? "night" : "nights");
