import api from "@/services/api";
import { formatINR } from "@/utils/money";

// Life line: the contract lives in the API (GET /api/plan/lifeline, POST /api/plan/lifeline/try,
// and /api/plan/lifeline/choices). Money is integer paise, rates are basis points.

export type Choice =
  | { kind: "job_switch"; month: string; take_home_paise: number; gap_months: number; raise_bps: number | null }
  | { kind: "house"; year: number; price_paise: number; down_bps: number; loan_bps: number; tenure_years: number; rent_saved_paise: number }
  | { kind: "child"; birth_year: number; another: boolean }
  | { kind: "retire_at"; age: number }
  | { kind: "big_spend"; name: string; year: number; amount_paise: number; every_years: number | null }
  | { kind: "break"; start: string; months: number }
  | { kind: "save_more"; monthly_paise: number }
  | { kind: "stay_active"; medical_cut_bps: number | null };

export type ChoiceKind = Choice["kind"];

export interface LineRow {
  year: number;
  age: number;
  income_paise: number;
  spend_paise: number;
  short_paise: number;
  net_worth_paise: number;
  net_worth_today_paise: number;
  events: string[];
}

export interface Marks {
  free_at_age: number | null;
  runs_out_at_age: number | null;
  retire_age: number;
  end_age: number;
  at_60_today_paise: number | null;
  at_end_today_paise: number;
  unfunded: string[];
}

export interface Line {
  rows: LineRow[];
  marks: Marks;
  notes: string[];
}

export interface SavedChoice {
  id: string;
  label: string;
  in_plan: boolean;
  choice: Choice;
}

export interface Working {
  accounts_paise: number;
  plan_saved_paise: number;
  loans_paise: number;
  wealth_paise: number;
  expenses_paise: number;
  income_paise: number;
  note: string;
}

export interface LifelineOut {
  saved_profile: boolean;
  line: Line;
  choices: SavedChoice[];
  working: Working;
}

export interface Consequences {
  lines: string[];
  at_60_change_paise: number | null;
  at_end_change_paise: number;
  better: boolean;
}

export interface TryOut {
  base: Line;
  with: Line;
  consequences: Consequences;
}

/** Query key for the life line; the page, the Today card and their tests share it. */
export const lifelineKey = ["lifeline"] as const;

export const getLifeline = () => api.get<LifelineOut>("/api/plan/lifeline").then((r) => r.data);

export const tryChoices = (base: Choice[], withChoices: Choice[]) =>
  api.post<TryOut>("/api/plan/lifeline/try", { base, with: withChoices }).then((r) => r.data);

export const keepChoice = (input: { label: string; choice: Choice; in_plan: boolean }) =>
  api.post<SavedChoice>("/api/plan/lifeline/choices", input).then((r) => r.data);

export const updateChoice = (id: string, patch: { label?: string; choice?: Choice; in_plan?: boolean }) =>
  api.patch<SavedChoice>(`/api/plan/lifeline/choices/${id}`, patch).then((r) => r.data);

/** Takes every choice off the line. */
export const clearChoices = () => api.post<{ removed: number }>("/api/plan/lifeline/choices/clear", {}).then((r) => r.data);

export const dropChoice = (id: string) => api.del<void>(`/api/plan/lifeline/choices/${id}`).then((r) => r.data);

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** "2027-04" -> "Apr 2027". Anything else comes back as it is. */
export function monthLabel(ym: string): string {
  const m = /^(\d{4})-(\d{2})/.exec(ym);
  const name = m ? MONTHS[Number(m[2]) - 1] : undefined;
  return m && name ? `${name} ${m[1]}` : ym;
}

const pct = (bps: number) => `${Math.round(bps / 10) / 10}%`;

/** A short name for a choice, used as its default label. */
export function describeChoice(c: Choice): string {
  switch (c.kind) {
    case "job_switch":
      return c.take_home_paise > 0
        ? `New job at ${formatINR(c.take_home_paise)} a month from ${monthLabel(c.month)}`
        : `Leave my job in ${monthLabel(c.month)}`;
    case "house":
      return `Buy a ${formatINR(c.price_paise)} home in ${c.year}`;
    case "child":
      return c.another ? `Another child in ${c.birth_year}` : `A child in ${c.birth_year}`;
    case "retire_at":
      return `Retire at ${c.age}`;
    case "big_spend": {
      const what = c.name.trim() || "Big spend";
      const again = c.every_years ? `, then every ${c.every_years} years` : "";
      return `${what}: ${formatINR(c.amount_paise)} in ${c.year}${again}`;
    }
    case "break":
      return `${c.months} months off work from ${monthLabel(c.start)}`;
    case "save_more":
      return `Spend ${formatINR(c.monthly_paise)} less a month`;
    case "stay_active":
      return `Stay active (medical costs ${pct(c.medical_cut_bps ?? 3000)} lower after 60)`;
  }
}

/** "at 57", or "not before 90" when savings never cover the bills on their own. */
export function freeLabel(m: Marks): string {
  return m.free_at_age === null ? `Not before ${m.end_age}` : `At ${m.free_at_age}`;
}

export function lastsLabel(m: Marks): string {
  return m.runs_out_at_age === null ? `Past ${m.end_age}` : `Runs short at ${m.runs_out_at_age}`;
}
