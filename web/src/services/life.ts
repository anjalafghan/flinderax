import api from "@/services/api";

// Life plan: the contract lives in the API (GET /api/plan/life, POST /api/plan/life/profile).
// Money is integer paise, rates are basis points, dates are "YYYY-MM-DD".

export interface LifeProfile {
  my_age: number;
  partner_age: number | null;
  my_take_home_paise: number;
  partner_take_home_paise: number;
  monthly_expenses_paise: number | null;
  wedding_date: string | null;
  wedding_budget_paise: number | null;
  wedding_saved_paise: number;
  child_birth_year: number | null;
  child_education_today_paise: number;
  child_saved_paise: number;
  retirement_age: number;
  retirement_saved_paise: number;
  emergency_saved_paise: number;
  term_cover_paise: number;
  health_cover_paise: number;
  loans_paise: number;
  /** Yearly raise on each take-home (bps) and the age the life line runs to. Older saves lack them. */
  raise_bps?: number;
  partner_raise_bps?: number;
  live_to_age?: number;
}

export interface LifeOut {
  saved: boolean;
  profile: LifeProfile;
  plan: LifePlan;
}

export interface LifePlan {
  assumptions: {
    general_inflation_bps: number;
    education_inflation_bps: number;
    medical_inflation_bps: number;
    wedding_inflation_bps: number;
    equity_return_bps: number;
    hybrid_return_bps: number;
    debt_return_bps: number;
    step_up_bps: number;
    swr_bps: number;
  };
  goals: LifeGoal[];
  insurance: {
    term_needed_paise: number;
    term_have_paise: number;
    term_gap_paise: number;
    health_needed_paise: number;
    health_have_paise: number;
    health_gap_paise: number;
    notes: string[];
  };
  monthly: {
    total_sip_paise: number;
    total_step_up_sip_paise: number;
    surplus_paise: number;
    fits: boolean;
    shortfall_paise: number;
  };
  timeline: { year: number; age: number; events: string[] }[];
}

export interface LifeGoal {
  key: "emergency" | "wedding" | "child_birth" | "child_education" | "retirement";
  label: string;
  when: string | null;
  years_away: number;
  today_cost_paise: number;
  future_cost_paise: number;
  saved_paise: number;
  gap_paise: number;
  monthly_sip_paise: number;
  step_up_sip_paise: number;
  mix: "debt" | "hybrid" | "equity";
  mix_note: string;
  my_share_paise: number;
  partner_share_paise: number;
  status: "done" | "on_track" | "needs_more";
  note: string;
}

/** Query key for the life plan; the page and its tests share it. */
export const lifeKey = ["life-plan"] as const;

export const getLife = async (): Promise<LifeOut> => (await api.get<LifeOut>("/api/plan/life")).data;

/** Forgets the saved details: the plan goes back to the defaults. */
export const resetLifeProfile = async (): Promise<LifeOut> => (await api.post<LifeOut>("/api/plan/life/reset", {})).data;

export const saveLifeProfile = async (p: LifeProfile): Promise<LifeOut> =>
  (await api.post<LifeOut>("/api/plan/life/profile", p)).data;
