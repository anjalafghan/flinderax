import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import api, { goalsApi } from "@/services/api";
import { formatDay } from "@/utils/dates";

// Circle: the people API (section B of the Phase 5 contract). Ids are only in paths, names never in URLs.
// Money is integer paise. Dates are "YYYY-MM-DD" and every person's data is private to that user.

export type Circle = "family" | "close" | "friend" | "work" | "other";
export type DateKind = "birthday" | "anniversary" | "remembrance" | "other";
export type TouchKind = "call" | "visit" | "message" | "met";
export type GiftStatus = "idea" | "bought" | "given";
export type CatchUpStatus = "overdue" | "due_soon" | "ok";

export interface PersonRow {
  id: string;
  name: string;
  circle: Circle;
  every_days: number | null;
  last_contact_on: string | null;
  next_due_on: string | null;
  status: CatchUpStatus | null;
  note: string;
  archived: boolean;
}

export interface PersonDate {
  id: string;
  person_id: string;
  kind: DateKind;
  label: string;
  month: number;
  day: number;
  year: number | null;
  remind_days: number;
  gift_budget_paise: number | null;
  next_on: string;
  days_away: number;
  turns: number | null;
}

export interface Touch {
  id: string;
  person_id: string;
  on_day: string;
  kind: TouchKind;
  cost_paise: number | null;
  note: string;
}

export interface Gift {
  id: string;
  person_id: string;
  date_id: string | null;
  for_year: number | null;
  status: GiftStatus;
  title: string;
  amount_paise: number | null;
  on_day: string | null;
  txn_id: string | null;
}

export interface Upcoming {
  date_id: string;
  person_id: string;
  name: string;
  kind: DateKind;
  label: string;
  on: string;
  days_away: number;
  turns: number | null;
  gift_budget_paise: number | null;
  gift_status: "none" | GiftStatus;
}

export interface Season {
  id: string;
  label: string;
  month: number;
  budget_paise: number;
}

export interface PeopleOverview {
  people: PersonRow[];
  archived: PersonRow[];
  upcoming: Upcoming[];
  catch_up: PersonRow[];
}

export interface PersonDetail {
  person: PersonRow;
  dates: PersonDate[];
  gifts: Gift[];
  touches: Touch[];
}

export interface GiftPlanMonth {
  /** "YYYY-MM" */
  month: string;
  budget_paise: number;
  occasions: number;
}

export interface GiftPlan {
  months: GiftPlanMonth[];
  total_paise: number;
  monthly_paise: number;
  given_this_year_paise: number;
  goal_id: string | null;
}

export interface PeopleSettings {
  feed_names: boolean;
}

export interface PersonInput {
  name: string;
  circle: Circle;
  every_days?: number | null;
  note?: string;
}

export interface PersonPatch {
  name?: string;
  circle?: Circle;
  every_days?: number | null;
  note?: string;
  archived?: boolean;
}

export interface TouchInput {
  kind: TouchKind;
  on_day?: string;
  cost_paise?: number;
  note?: string;
}

export interface DateInput {
  kind: DateKind;
  label?: string;
  month: number;
  day: number;
  year?: number;
  remind_days?: number;
  gift_budget_paise?: number | null;
}

export type DatePatch = Partial<DateInput>;

export interface GiftInput {
  person_id: string;
  date_id?: string;
  for_year?: number;
  title: string;
  status?: GiftStatus;
  amount_paise?: number;
  on_day?: string;
  txn_id?: string;
}

export interface GiftPatch {
  title?: string;
  status?: GiftStatus;
  amount_paise?: number | null;
  on_day?: string | null;
}

export interface SeasonInput {
  label: string;
  month: number;
  budget_paise: number;
}

// ---------- transport ----------

/** PATCH, PUT and DELETE through the shared client, so a lapsed session goes back to sign-in. */
function request<T>(method: "PATCH" | "PUT" | "DELETE", url: string, body?: unknown): Promise<T> {
  const call = method === "PUT" ? api.put<T>(url, body) : method === "PATCH" ? api.patch<T>(url, body) : api.del<T>(url);
  return call.then((r) => r.data);
}

// ---------- fetchers ----------

export const peopleKey = ["people"] as const;
export const personKey = (id: string) => ["people", id] as const;
export const giftPlanKey = ["people", "gift-plan"] as const;
export const seasonsKey = ["people", "seasons"] as const;
export const peopleSettingsKey = ["people", "settings"] as const;

export const getPeople = () => api.get<PeopleOverview>("/api/people").then((r) => r.data);
export const getPerson = (id: string) => api.get<PersonDetail>(`/api/people/${id}`).then((r) => r.data);
export const createPerson = (input: PersonInput) => api.post<PersonRow>("/api/people", input).then((r) => r.data);
export const updatePerson = (id: string, patch: PersonPatch) =>
  request<PersonRow>("PATCH", `/api/people/${id}`, patch);
export const deletePerson = (id: string) => request<void>("DELETE", `/api/people/${id}`);

export const addTouch = (personId: string, input: TouchInput) =>
  api.post<PersonRow>(`/api/people/${personId}/touches`, input).then((r) => r.data);
export const deleteTouch = (touchId: string) => request<void>("DELETE", `/api/people/touches/${touchId}`);

export const addDate = (personId: string, input: DateInput) =>
  api.post<PersonDate>(`/api/people/${personId}/dates`, input).then((r) => r.data);
export const updateDate = (dateId: string, patch: DatePatch) =>
  request<PersonDate>("PATCH", `/api/people/dates/${dateId}`, patch);
export const deleteDate = (dateId: string) => request<void>("DELETE", `/api/people/dates/${dateId}`);

export const addGift = (input: GiftInput) => api.post<Gift>("/api/people/gifts", input).then((r) => r.data);
export const updateGift = (giftId: string, patch: GiftPatch) =>
  request<Gift>("PATCH", `/api/people/gifts/${giftId}`, patch);
export const deleteGift = (giftId: string) => request<void>("DELETE", `/api/people/gifts/${giftId}`);

export const getSeasons = () =>
  api.get<{ seasons: Season[] }>("/api/people/seasons").then((r) => r.data.seasons);
export const addSeason = (input: SeasonInput) => api.post<Season>("/api/people/seasons", input).then((r) => r.data);
export const updateSeason = (id: string, patch: Partial<SeasonInput>) =>
  request<Season>("PATCH", `/api/people/seasons/${id}`, patch);
export const deleteSeason = (id: string) => request<void>("DELETE", `/api/people/seasons/${id}`);

export const getGiftPlan = () => api.get<GiftPlan>("/api/people/gift-plan").then((r) => r.data);
export const getPeopleSettings = () => api.get<PeopleSettings>("/api/people/settings").then((r) => r.data);
export const savePeopleSettings = (s: PeopleSettings) => request<PeopleSettings>("PUT", "/api/people/settings", s);

// ---------- hooks ----------

export const usePeople = () => useQuery({ queryKey: peopleKey, queryFn: getPeople, retry: 1 });
export const usePerson = (id: string) =>
  useQuery({ queryKey: personKey(id), queryFn: () => getPerson(id), retry: 1, enabled: id !== "" });
export const useGiftPlan = () => useQuery({ queryKey: giftPlanKey, queryFn: getGiftPlan, retry: 1 });
export const useSeasons = () => useQuery({ queryKey: seasonsKey, queryFn: getSeasons, retry: 1 });
export const usePeopleSettings = () =>
  useQuery({ queryKey: peopleSettingsKey, queryFn: getPeopleSettings, retry: 1 });

/** Any write to people refreshes every people query: the lists, the person, the plan and the dashboard card. */
export function usePeopleMutation<TInput, TOut>(fn: (input: TInput) => Promise<TOut>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    // Not returned: the caller's toast and navigation shouldn't wait for every refetch to finish.
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: peopleKey });
    },
  });
}

/** Creates the "Gifts" goal the gift plan reads from, then refreshes goals and the plan. */
export function useMakeGiftsGoal() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (targetPaise: number) =>
      goalsApi.create({ name: "Gifts", kind: "custom", target_paise: targetPaise }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["goals"] });
      void qc.invalidateQueries({ queryKey: giftPlanKey });
    },
  });
}

// ---------- pure helpers ----------

/** Circles in the order the list shows them. */
export const CIRCLES: { key: Circle; label: string }[] = [
  { key: "family", label: "Family" },
  { key: "close", label: "Close friends" },
  { key: "friend", label: "Friends" },
  { key: "work", label: "Work" },
  { key: "other", label: "Other" },
];

export const circleLabel = (c: Circle): string => CIRCLES.find((x) => x.key === c)?.label ?? "Other";

export const KIND_LABEL: Record<DateKind, string> = {
  birthday: "Birthday",
  anniversary: "Anniversary",
  remembrance: "Remembrance",
  other: "Date",
};

/** The noun used in sentences: "Asha's birthday". */
export const KIND_NOUN: Record<DateKind, string> = {
  birthday: "birthday",
  anniversary: "anniversary",
  remembrance: "remembrance day",
  other: "date",
};

export const TOUCH_LABEL: Record<TouchKind, string> = {
  call: "Call",
  visit: "Visit",
  message: "Message",
  met: "Met up",
};

export const GIFT_LABEL: Record<GiftStatus, string> = {
  idea: "Gift idea",
  bought: "Bought",
  given: "Given",
};

/** "every 4 weeks", "every 1 week", "every 10 days", or "" when no rhythm is set. */
export function everyLabel(days: number | null): string {
  if (days === null) return "";
  if (days % 7 === 0) {
    const weeks = days / 7;
    return weeks === 1 ? "every 1 week" : `every ${weeks} weeks`;
  }
  return `every ${days} days`;
}

/** "Last caught up 3 Nov", or "No catch-up logged yet." */
export function lastCaughtUpLabel(last: string | null): string {
  return last ? `Last caught up ${formatDay(last)}` : "No catch-up logged yet.";
}

const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/** "Sat 14 Dec". The weekday comes from UTC so the viewer's timezone can't move it. */
export function weekdayDay(iso: string): string {
  const [y, m, d] = iso.split("-").map(Number);
  return `${WEEKDAYS[new Date(Date.UTC(y, m - 1, d)).getUTCDay()]} ${formatDay(iso)}`;
}

/** "today", "tomorrow", "in 5 days". Uses the server's count, never the clock. */
export function daysAwayLabel(n: number): string {
  if (n <= 0) return "today";
  if (n === 1) return "tomorrow";
  return `in ${n} days`;
}

/** The viewer's local calendar day as "YYYY-MM-DD", for date inputs and their `max`. */
export function localDay(now: Date = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/** "Asha's birthday", "Asha's Diwali" (any label is used in place of the kind). */
export function dateTitle(name: string, kind: DateKind, label: string): string {
  return `${name}'s ${label.trim() || KIND_NOUN[kind]}`;
}

export function plural(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}
