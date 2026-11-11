import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import api from "@/services/api";
import { habitsKey } from "@/services/habits";

// Work: the job-search leads (GET/POST /api/work, PUT/DELETE /api/work/{id}). Money is paise.

export type Stage = "idea" | "applied" | "interview" | "offer" | "closed";

export interface Lead {
  id: string;
  company: string;
  role: string;
  stage: Stage;
  /** Monthly take-home, expected or offered. */
  take_home_paise: number | null;
  next_step: string;
  next_on: string | null;
  note: string;
  updated_on: string;
}

/** What the server stores; PUT replaces the whole lead. */
export type LeadInput = Omit<Lead, "id" | "updated_on">;

/** The stages in the order a lead moves through them. */
export const STAGES: { value: Stage; label: string }[] = [
  { value: "idea", label: "Idea" },
  { value: "applied", label: "Applied" },
  { value: "interview", label: "Interviewing" },
  { value: "offer", label: "Offer" },
  { value: "closed", label: "Closed" },
];

export const stageLabel = (s: Stage) => STAGES.find((x) => x.value === s)?.label ?? s;

export const workKey = ["work"] as const;

export const getWork = () => api.get<{ leads: Lead[] }>("/api/work").then((r) => r.data);
export const addLead = (lead: LeadInput) => api.post<Lead>("/api/work", lead).then((r) => r.data);
export const saveLead = (id: string, lead: LeadInput) => api.put<Lead>(`/api/work/${id}`, lead).then((r) => r.data);
/** Starts the job search again with no leads. */
export const clearLeads = () => api.post<{ removed: number }>("/api/work/clear", {}).then((r) => r.data);
export const removeLead = (id: string) => api.del<void>(`/api/work/${id}`).then((r) => r.data);

export const useWork = () => useQuery({ queryKey: workKey, queryFn: getWork, retry: 1 });

/** A write that refreshes the leads and the habits built from them. */
function useWorkWrite<V, T>(fn: (v: V) => Promise<T>) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: workKey });
      void qc.invalidateQueries({ queryKey: habitsKey });
    },
  });
}

export const useAddLead = () => useWorkWrite(addLead);
export const useSaveLead = () => useWorkWrite((v: { id: string; lead: LeadInput }) => saveLead(v.id, v.lead));
export const useRemoveLead = () => useWorkWrite(removeLead);
export const useClearLeads = () => useWorkWrite(() => clearLeads());

/** A next step this many days away, or already late, belongs on this week's list. */
const SOON_DAYS = 7;

export function daysUntil(day: string): number {
  const [y, m, d] = day.split("-").map(Number);
  const today = new Date();
  const from = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
  return Math.round((Date.UTC(y ?? 0, (m ?? 1) - 1, d ?? 1) - from) / 86_400_000);
}

/** Open leads whose next step is late or falls within the week, soonest first. */
export function stepsThisWeek(leads: Lead[]): Lead[] {
  return leads
    .filter((l) => l.stage !== "closed" && l.next_on !== null && daysUntil(l.next_on) <= SOON_DAYS)
    .sort((a, b) => (a.next_on ?? "").localeCompare(b.next_on ?? ""));
}
