import { useQuery } from "@tanstack/react-query";

import api from "@/services/api";

// Mirrors the forecast contract (section "Web", API "Forecast"). Money is integer paise, likelihoods
// and rates are basis points, dates are "YYYY-MM-DD".

export interface Basis {
  source: "statements" | "profile";
  /** Full months the averages come from (max 12). */
  months_used: number;
  income_paise: number;
  spend_paise: number;
  invested_paise: number;
  surplus_paise: number;
  surplus_sd_paise: number;
  entered_income_paise: number;
  /** e.g. when statements and the plan differ by more than 10%. */
  note: string | null;
}

export interface Lever {
  kind: "save_more" | "push_date" | "trim_target" | "cut_spend";
  text: string;
  monthly_paise: number | null;
  months: number | null;
  target_paise: number | null;
}

export interface GoalForecast {
  key: string;
  label: string;
  when: string | null;
  target_paise: number;
  saved_paise: number;
  needed_monthly_paise: number;
  funded_monthly_paise: number;
  likelihood_bps: number;
  p10_paise: number;
  p50_paise: number;
  p90_paise: number;
  status: "likely" | "possible" | "unlikely";
  /** Only when status is not "likely"; up to three. */
  levers: Lever[];
}

export interface YearBand {
  year: number;
  p10_paise: number;
  p50_paise: number;
  p90_paise: number;
}

export interface Forecast {
  basis: Basis;
  goals: GoalForecast[];
  /** Total of all goal pots, per year, until the last goal. */
  path: YearBand[];
  /** One sentence, e.g. "3 of 5 goals look likely on what you save today." */
  summary: string;
  saved_profile: boolean;
}

/** Query key for the forecast; saving the life profile invalidates it. */
export const forecastKey = ["forecast"] as const;

export const getForecast = () => api.get<Forecast>("/api/plan/forecast").then((r) => r.data);

export const useForecast = () => useQuery({ queryKey: forecastKey, queryFn: getForecast, retry: 1 });
