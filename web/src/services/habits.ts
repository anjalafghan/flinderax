import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

// Mirrors the Habit contract (habits-contract.md). Served sorted: overdue, due_soon, ok; max 12.

export interface Habit {
  key: string;
  area: "data" | "money" | "food" | "goals" | "protection";
  title: string;
  why: string;
  action: { label: string; to: string } | null;
  cadence: "once" | "weekly" | "monthly" | "quarterly" | "yearly";
  due_on: string | null;
  status: "overdue" | "due_soon" | "ok";
  impact: "high" | "medium" | "low";
}

export interface HabitsOut {
  habits: Habit[];
}

export const getHabits = () => api.get<HabitsOut>("/api/dashboard/habits").then((r) => r.data);

/** Habits move with the data uploads and the planner, so they share the dashboard refresh cycle. */
export const useHabits = () => useQuery({ queryKey: ["habits"], queryFn: getHabits, retry: 1 });
