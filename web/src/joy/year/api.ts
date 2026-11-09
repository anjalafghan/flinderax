// The year and month pages' calls. Photo links are signed public URLs: use them in <img src> as-is.
import type { Go } from "../api";
import { get } from "../regulars/http";
import type { ShareData } from "./layout";

const BASE = "/api/dates";

export interface MonthBar {
  letter: string;
  name: string;
  n: number;
  pct: number;
}
export interface BestNight {
  title: string;
  when: string;
  rating: number;
  note: string;
  more: number;
  photo: string;
  /** The kept date, to open all its photos. */
  date_id: number;
}
export interface Kind {
  key: string;
  label: string;
  n: number;
  pct: number;
}
export interface MonthBlot {
  letter: string;
  r: string;
  over: boolean;
  text: string;
}
export interface Money {
  spent: string;
  diff: string;
  over: boolean;
  n: number;
}
export interface JarSlip {
  x: number;
  y: number;
  rot: number;
  kind: string;
}
export interface RatingRow {
  stars: number;
  n: number;
}
export interface YearPhoto {
  id: number;
  /** The kept date it belongs to. */
  date_id: number;
  url: string;
  title: string;
  when: string;
}

export interface YearView {
  me_name: string;
  year: number;
  years: number[];
  dates: number;
  to_come: number;
  sealed_mine: number;
  months: MonthBar[];
  busiest: string;
  vibe_label: string;
  tags: string[];
  best: BestNight | null;
  avg_rating: string;
  rated: number;
  streak_weeks: number;
  money: Money | null;
  cabs: string;
  photos: YearPhoto[];
  busiest_n: number;
  tally: number[];
  tally_more: number;
  kinds: Kind[];
  best_kind: string;
  rating_line: string;
  rating_rows: RatingRow[];
  jar_slips: JarSlip[];
  month_blots: MonthBlot[];
  year_total: string;
  budget_note: string;
  share: ShareData;
}

export interface Entry {
  title: string;
  place: string;
  kind: string;
  kind_label: string;
  day: string;
  stamp: string;
  photo: string;
  rating: number;
  note: string;
  est: string;
  actual: string;
  diff: string;
  over: boolean;
  cabs: string;
  tilt: number;
}
export interface Other {
  amount: string;
  note: string;
  day: string;
}
export interface NightMark {
  day: string;
  photo: string;
}
export interface NightGroup {
  title: string;
  href: string;
  count: number;
  total: string;
  nights: NightMark[];
}
export interface Best {
  title: string;
  rating: number;
  day: string;
}
export interface Nav {
  href: string;
  label: string;
}
export interface MonthView {
  me_name: string;
  year: number;
  month: number;
  month_name: string;
  month_letters: string[];
  year_letters: string[];
  entries: Entry[];
  others: Other[];
  nights: NightGroup[];
  best: Best | null;
  spent: string;
  avg_rating: string;
  prev: Nav | null;
  next: Nav | null;
  to_come: number;
}

export const yearKeys = {
  current: ["joy", "year-current"] as const,
  year: (y: string) => ["joy", "year", y] as const,
  month: (y: string, m: string) => ["joy", "month", y, m] as const,
};

export const getYearGo = () => get<Go>(`${BASE}/year`);
export const getYear = (y: string) => get<YearView>(`${BASE}/year/${encodeURIComponent(y)}`);
export const getMonth = (y: string, m: string) => get<MonthView>(`${BASE}/year/${encodeURIComponent(y)}/${encodeURIComponent(m)}`);
