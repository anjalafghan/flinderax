// The jar's API calls. They use the shared request helpers in ../api.ts (bearer token, 401 -> /auth),
// so nothing here knows about tokens.
import { get, post } from "../api";
import type { RegularCardData } from "../regulars/api";
import type { PickForm } from "./form";

export interface Opt {
  value: string;
  label: string;
  selected: boolean;
}

export interface Slip {
  id: number;
  title: string;
  setting: string;
  outdoor: boolean;
}

export interface BudgetView {
  spent: string;
  budget: string;
  budget_raw: string;
  pct: number;
  over: boolean;
  carry_note: string;
}

export interface StepView {
  at: string;
  minutes: number;
  what: string;
  mine: boolean;
}

export interface TaskView {
  id: number;
  task: string;
  due: string;
  whose: string;
  mine: boolean;
  done: boolean;
}

export interface DayPlan {
  history_id: number;
  title: string;
  when: string;
  steps: StepView[];
  tight: boolean;
  hours_note: string;
  plan_b: { title: string; place: string } | null;
  rain: number | null;
  after: string;
  tasks: TaskView[];
  sealed: boolean;
  ics_url: string;
  start: number;
  leave_before: number;
  my_open_tasks: number;
}

export interface MiddleView {
  enabled: boolean;
  partner_in: boolean;
  partner: string;
  wa_url: string;
  both_in: boolean;
}

/** The jar's "Tonight?" card: the same data the Regulars card takes. */
export type RegularsCard = RegularCardData;

export interface DecideView {
  couple: string;
  office_name: string;
  currency: string;
  count: number;
  slips: Slip[];
  durations: Opt[];
  settings: Opt[];
  tags: Opt[];
  energies: Opt[];
  kms: Opt[];
  froms: Opt[];
  me_name: string;
  turn_name: string;
  my_turn: boolean;
  rain: number | null;
  memory: { title: string; when: string } | null;
  budget: BudgetView;
  nudge: string;
  nudge_soon: boolean;
  middle: MiddleView;
  whens: Opt[];
  next_plan: DayPlan | null;
  regulars: RegularsCard;
}

export interface TravelLine {
  name: string;
  cab: string;
  cab_time: string;
  train: string;
  train_time: string;
}

export interface ActivityView {
  id: number;
  title: string;
  notes: string;
  place: string;
  cost: string;
  duration: string;
  setting: string;
  setting_key: string;
  tags: string[];
  distances: string[];
  maps_url: string | null;
  times_done: number;
  generated: boolean;
  travel: TravelLine[];
  travel_note: string;
  cost_note: string;
  closed_label: string;
  uber_url: string | null;
  door: string;
}

export interface PairOffer {
  id: number;
  title: string;
  cost: string;
  duration: string;
  together: string;
  over: string;
}

export interface PickCard {
  activity: ActivityView | null;
  message: string;
  relax_label: string;
  relax: PickForm | null;
  done: boolean;
  stamp_date: string;
  rerolls_left: number;
  max_rerolls: number;
  free_putback: boolean;
  pairs: PairOffer[];
  paired: ActivityView | null;
  state: PickForm | null;
  bonus_peek: boolean;
  can_surprise: boolean;
  can_veto: boolean;
  partner: string;
  surprise: boolean;
  history_ids: string;
  share_text: string;
  wa_url: string;
  middle: boolean;
  when: string;
  start: string;
  when_label: string;
  plan_line: string;
  route: StepView[];
  route_note: string;
  plan: DayPlan | null;
}

export interface MatchCount {
  count: number;
  total: number;
  ids: number[];
}

export interface AcceptOpts {
  with?: string;
  surprise?: boolean;
  when?: string;
  start?: string;
}

const BASE = "/api/dates";

export const jarApi = {
  jar: () => get<DecideView>(`${BASE}/jar`),
  pick: (form: PickForm) => post<PickCard>(`${BASE}/pick`, form),
  match: (form: PickForm) => post<MatchCount>(`${BASE}/match`, form),
  accept: (id: number, o: AcceptOpts) => {
    const body: Record<string, string> = {};
    if (o.with) body.with = o.with;
    if (o.surprise) body.surprise = "1";
    if (o.when) body.when = o.when;
    if (o.start) body.start = o.start;
    return post<PickCard>(`${BASE}/accept/${id}`, body);
  },
  /** Swap to (or back from) Plan B. Answers with the refreshed plan. */
  planB: (historyId: number) => post<DayPlan | null>(`${BASE}/history/${historyId}/plan-b`, {}),
  /** Tick (or untick) one of my own jobs. 403 if it was dealt to my partner. Answers with the refreshed plan. */
  taskDone: (taskId: number, done: boolean) => post<DayPlan | null>(`${BASE}/tasks/${taskId}/done`, { done: done ? "1" : "0" }),
  /** Put a day on a date that is already kept (`ids` is the comma-separated history ids). */
  when: (ids: string, at: string) => post<{ label: string; ics_url: string; partner: string }>(`${BASE}/history/when`, { ids, at }),
};
