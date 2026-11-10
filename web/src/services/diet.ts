import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import api from "@/services/api";

// Mirrors the diet and health contract, section A (GET /api/diet and the endpoints under it).
// Money is integer paise, dates are "YYYY-MM-DD", weight and waist are tenths (kg x10, cm x10).

export interface DietPreset {
  key: string;
  label: string;
  target_days: number;
}

export interface DietHabit {
  id: string;
  label: string;
  preset_key: string | null;
  target_days: number;
  active: boolean;
}

export interface WeekEntry {
  habit_id: string;
  days_hit: number;
}

export interface Week {
  week_start: string;
  feel: number | null;
  note: string;
  /** Hours of job preparation that week, in tenths. Older responses lack it. */
  prep_hours_x10?: number | null;
  entries: WeekEntry[];
}

export interface RecentWeek {
  week_start: string;
  logged: boolean;
  hit_bps: number | null;
}

export interface FoodMonth {
  /** "YYYY-MM" */
  month: string;
  eating_out_paise: number;
  groceries_paise: number;
  delivery_orders: number;
}

export interface FoodMoney {
  months: FoodMonth[];
  eating_out_share_bps: number | null;
  /** Hundredths: 350 is 3.5 orders a week. */
  delivery_per_week_x100: number;
  cap_paise: number | null;
}

export interface BodySummary {
  weight_x10: number | null;
  weight_on: string | null;
  prev_weight_x10: number | null;
  waist_x10: number | null;
  last_checkup_on: string | null;
  checkup_due_on: string | null;
}

export interface DietOverview {
  presets: DietPreset[];
  habits: DietHabit[];
  week: Week;
  recent: RecentWeek[];
  money: FoodMoney;
  body: BodySummary;
}

export type MarkKind = "weight" | "waist" | "checkup";

export interface Mark {
  id: string;
  kind: MarkKind;
  measured_on: string;
  value_x10: number | null;
  note: string;
}

export interface NewHabit {
  label: string;
  preset_key?: string;
  target_days: number;
}

export interface HabitPatch {
  label?: string;
  target_days?: number;
  active?: boolean;
  sort?: number;
}

export interface WeekInput {
  feel?: number | null;
  note?: string;
  prep_hours_x10?: number | null;
  entries: WeekEntry[];
}

export interface NewMark {
  kind: MarkKind;
  measured_on: string;
  value_x10?: number;
  note?: string;
}

export const dietKey = ["diet"] as const;
export const dietMarksKey = (kind: MarkKind) => ["diet", "marks", kind] as const;

export const getDiet = () => api.get<DietOverview>("/api/diet").then((r) => r.data);
export const getDietMarks = (kind: MarkKind, limit = 12) =>
  api.get<{ marks: Mark[] }>(`/api/diet/marks?kind=${kind}&limit=${limit}`).then((r) => r.data.marks);
export const addDietHabit = (habit: NewHabit) => api.post<DietHabit>("/api/diet/habits", habit).then((r) => r.data);
export const addDietMark = (mark: NewMark) => api.post<Mark>("/api/diet/marks", mark).then((r) => r.data);
export const saveDietCap = (eating_out_cap_paise: number | null) =>
  sendJson<{ eating_out_cap_paise: number | null }>("PUT", "/api/diet/settings", { eating_out_cap_paise });
export const updateDietHabit = (id: string, patch: HabitPatch) =>
  sendJson<DietHabit>("PATCH", `/api/diet/habits/${id}`, patch);
export const removeDietHabit = (id: string) => sendJson<void>("DELETE", `/api/diet/habits/${id}`);
export const saveDietWeek = (weekStart: string, week: WeekInput) =>
  sendJson<Week>("PUT", `/api/diet/weeks/${weekStart}`, week);
export const removeDietMark = (id: string) => sendJson<void>("DELETE", `/api/diet/marks/${id}`);

/** PUT, PATCH and DELETE through the shared client, so a lapsed session goes back to sign-in. */
function sendJson<T>(method: "PUT" | "PATCH" | "DELETE", url: string, body?: unknown): Promise<T> {
  const call = method === "PUT" ? api.put<T>(url, body) : method === "PATCH" ? api.patch<T>(url, body) : api.del<T>(url);
  return call.then((r) => r.data);
}

export const useDiet = () => useQuery({ queryKey: dietKey, queryFn: getDiet, retry: 1 });
/** One week as logged (entries empty when nothing was). Under ["diet"], so saves refresh it too. */
export const getDietWeek = (weekStart: string) => api.get<Week>(`/api/diet/weeks/${weekStart}`).then((r) => r.data);
export const useDietWeek = (weekStart: string, enabled: boolean) =>
  useQuery({ queryKey: [...dietKey, "week", weekStart], queryFn: () => getDietWeek(weekStart), enabled, retry: 1 });

export const useDietMarks = (kind: MarkKind) =>
  useQuery({ queryKey: dietMarksKey(kind), queryFn: () => getDietMarks(kind), retry: 1 });

/** Every diet write refreshes the whole diet query, marks included, so the body and money cards stay current. */
function useDietWrite<V, R>(fn: (v: V) => Promise<R>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => qc.invalidateQueries({ queryKey: dietKey }),
  });
}

export const useAddDietHabit = () => useDietWrite(addDietHabit);
export const useUpdateDietHabit = () => useDietWrite((v: { id: string; patch: HabitPatch }) => updateDietHabit(v.id, v.patch));
export const useRemoveDietHabit = () => useDietWrite(removeDietHabit);
export const useSaveDietWeek = () => useDietWrite((v: { weekStart: string; week: WeekInput }) => saveDietWeek(v.weekStart, v.week));
export const useAddDietMark = () => useDietWrite(addDietMark);
export const useRemoveDietMark = () => useDietWrite(removeDietMark);
export const useSaveDietCap = () => useDietWrite(saveDietCap);

/** Today as "YYYY-MM-DD" in the viewer's own calendar. */
export const todayIso = (now: Date = new Date()): string => {
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
};

/** 705 -> "70.5 kg". */
export const kgLabel = (x10: number): string => `${(x10 / 10).toFixed(1)} kg`;

/** 845 -> "84.5 cm". */
export const cmLabel = (x10: number): string => `${(x10 / 10).toFixed(1)} cm`;

/** A signed weight change with a true minus sign: -12 -> "−1.2 kg", 5 -> "+0.5 kg". */
export const signedKg = (x10: number): string => {
  const abs = (Math.abs(x10) / 10).toFixed(1);
  if (x10 === 0) return "0.0 kg";
  return `${x10 > 0 ? "+" : "−"}${abs} kg`;
};

/**
 * "70.5" -> 705. Accepts up to one decimal and returns tenths, or null for anything else. `min` and `max`
 * are tenths too (200 and 3000 for weight), inclusive.
 */
export const parseTenths = (text: string, min: number, max: number): number | null => {
  const t = text.trim();
  if (!/^\d{1,4}(\.\d)?$/.test(t)) return null;
  const x10 = Math.round(Number(t) * 10);
  return x10 >= min && x10 <= max ? x10 : null;
};

/** Whole-number orders a week for the food money card: 350 -> "3.5". */
export const ordersPerWeek = (x100: number): string => (x100 / 100).toFixed(1).replace(/\.0$/, "");
