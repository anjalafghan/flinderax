// The slip form: server FormView -> editable state -> POST body. Pure.
import type { FormView, Opt, SlipBody } from "./api";

export interface FormState {
  title: string;
  setting: string;
  est_cost: string;
  duration: string;
  energy: string;
  venue: string;
  area: string;
  location: string;
  tags: string;
  notes: string;
  closed: string[];
  season_from: string;
  season_to: string;
  hours: string;
  best_time: string;
  minutes: string;
  booking: string;
  book_ahead_days: string;
  cover: string;
  bring: string;
  station: string;
  station_walk_min: string;
  pairs: string[];
  added_by: string;
}

const selected = (opts: Opt[], fallback = ""): string => opts.find((o) => o.selected)?.value ?? fallback;
const ticked = (opts: Opt[]): string[] => opts.filter((o) => o.selected).map((o) => o.value);

export function formToState(v: FormView): FormState {
  return {
    title: v.title,
    // Like the old radio group: someone always wrote it, and a slip always has a setting.
    setting: selected(v.settings, v.settings[0]?.value ?? "local"),
    est_cost: v.est_cost,
    duration: selected(v.durations, v.durations[0]?.value ?? "short"),
    energy: selected(v.energies, "med"),
    venue: v.venue,
    area: v.area,
    location: v.location,
    tags: v.tags,
    notes: v.notes,
    closed: ticked(v.closed_opts),
    season_from: selected(v.season_from_opts),
    season_to: selected(v.season_to_opts),
    hours: v.hours,
    best_time: selected(v.best_times),
    minutes: v.minutes,
    booking: selected(v.bookings),
    book_ahead_days: v.book_ahead_days,
    cover: v.cover_unknown ? "" : selected(v.covers),
    bring: v.bring,
    station: v.station,
    station_walk_min: v.station_walk_min,
    pairs: ticked(v.pair_opts),
    added_by: selected(v.people, v.people[0]?.value ?? ""),
  };
}

/** The title is one line of thought: runs of whitespace (and pasted newlines) become one space. */
export const tidyTitle = (t: string) => t.replace(/\s+/g, " ").trim();

export function toPayload(s: FormState): SlipBody {
  return {
    title: tidyTitle(s.title),
    notes: s.notes,
    est_cost: s.est_cost.trim(),
    setting: s.setting,
    duration: s.duration,
    venue: s.venue.trim(),
    area: s.area.trim(),
    location: s.location.trim(),
    tags: s.tags.trim(),
    energy: s.energy || "med",
    added_by: s.added_by,
    pairs: s.pairs.map(Number).filter((n) => Number.isInteger(n)),
    closed: s.closed,
    season_from: s.season_from,
    season_to: s.season_to,
    hours: s.hours.trim(),
    best_time: s.best_time,
    minutes: s.minutes.trim(),
    booking: s.booking,
    book_ahead_days: s.book_ahead_days.trim(),
    cover: s.cover,
    bring: s.bring.trim(),
    station: s.station.trim(),
    station_walk_min: s.station_walk_min.trim(),
  };
}

/** Which field an error message is about, so it can be marked invalid and focused. */
export function errorField(error: string): "title" | "est_cost" | "location" | "hours" | null {
  if (!error) return null;
  if (error.includes("name")) return "title";
  if (error.includes("cost")) return "est_cost";
  if (error.includes("Location")) return "location";
  if (error.includes("Hours")) return "hours";
  return null;
}
