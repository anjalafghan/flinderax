// The JSON of GET /api/dates/kept (src/joy/api/history.rs) and POST /api/dates/history/when.
import type { Opt } from "../api";

export interface Cab {
  name: string;
  fare: string;
}

export interface Lines {
  can_write: boolean;
  mine: string;
  theirs: string;
  theirs_waiting: boolean;
  partner: string;
}

export interface KeptView {
  id: number;
  sealed: boolean;
  title: string;
  setting: string;
  place: string;
  day: string;
  day_label: string;
  rating: number | null;
  note: string;
  from_name: string;
  surprise_for: string;
  can_reveal: boolean;
  hints: string[];
  planned_label: string;
  planned_input: string;
  ics_url: string;
  /** The cover (first) photo's signed link. */
  photo_url: string | null;
  /** Every photo, oldest first; the first is the cover. */
  photos: KeptPhoto[];
  /** Room for more photos on this date. */
  can_add_photo: boolean;
  cost_est: string;
  cost_actual: string;
  cost_actual_raw: string;
  cost_diff: string;
  cost_over: boolean;
  cabs: Cab[];
  pair_ids: string;
  lines: Lines;
}

export interface Month {
  label: string;
  href: string;
  items: KeptView[];
}

export interface SkippedView {
  title: string;
  day_label: string;
}

export interface Rollover {
  month: string;
  spent: string;
  budget: string;
  left: string;
}

export interface BudgetState {
  spent: string;
  budget: string;
  budget_raw: string;
  pct: number;
  over: boolean;
  carry_note: string;
  rollover: Rollover | null;
}

export interface Goal {
  title: string;
  cost: string;
  pct: number;
  to_go: string;
  ready: boolean;
}

export interface SavingLine {
  id: number;
  amount: string;
  note: string;
  on: string;
  removable: boolean;
}

export interface SavingsState {
  saved: string;
  saved_raw: number;
  goal: Goal | null;
  choices: Opt[];
  recent: SavingLine[];
}

export interface OtherSpend {
  id: number;
  amount: string;
  note: string;
  on: string;
}

export interface PastMonth {
  label: string;
  spent: string;
  budget: string;
}

export interface HistoryView {
  months: Month[];
  kept: number;
  skipped: SkippedView[];
  budget: BudgetState;
  me_name: string;
  currency: string;
  cost_insight: string;
  past: PastMonth[];
  other_spend: OtherSpend[];
  savings: SavingsState;
}

export interface PlannedView {
  ids: string;
  label: string;
  input: string;
  ics_url: string;
  send_url: string;
  partner: string;
}

/** One photo of a kept date: its id and a signed link (good for an hour). */
export interface KeptPhoto {
  id: number;
  url: string;
}
