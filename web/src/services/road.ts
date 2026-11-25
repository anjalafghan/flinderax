import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

// The JSON of GET /api/plan/road (src/handlers/road.rs, src/planner/road.rs). Money is paise.

export type RoadArea = "money" | "family" | "health";
export type RoadSource = "life" | "goal" | "payment" | "cover";

export interface Milestone {
  /** "YYYY-MM-DD". */
  on: string;
  title: string;
  area: RoadArea;
  source: RoadSource;
  cost_paise: number;
  saved_paise: number;
  per_month_paise: number;
  months_away: number;
  link: string;
}

export interface RoadPrompt {
  title: string;
  body: string;
  link: string;
}

export interface Road {
  milestones: Milestone[];
  prompts: RoadPrompt[];
  pace_all_paise: number;
  biggest: Milestone | null;
}

export const roadKey = ["plan", "road"] as const;

export const getRoad = () =>
  api.get<Road>("/api/plan/road").then((r) => r.data);

export const useRoad = () =>
  useQuery({ queryKey: roadKey, queryFn: getRoad, retry: 1 });
