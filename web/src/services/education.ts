import api from "@/services/api";
import type { Consequences } from "@/services/lifeline";
import { paiseToInput, toPaise } from "@/utils/money";

// Child and education plan: GET/PUT/DELETE /api/plan/education. Money is integer paise and rates are
// basis points. Fee inflation and returns are guesses; the server sends the low/base/high range.

export interface EduStage {
  name: string;
  start_age: number;
  today_cost_paise: number;
}

export interface EduPlan {
  child_birth_year: number;
  stages: EduStage[];
  inflation_bps: number;
  return_bps: number;
  corpus_paise: number;
}

export interface EduStageCost {
  name: string;
  start_age: number;
  year: number;
  months_away: number;
  today_cost_paise: number;
  future_cost_paise: number;
  started: boolean;
}

export interface EduScenario {
  label: string;
  inflation_bps: number;
  return_bps: number;
  future_costs_paise: number[];
  total_future_cost_paise: number;
  monthly_top_up_paise: number;
  months_to_save: number;
}

export interface EduSummary {
  child_birth_year: number;
  child_age: number;
  corpus_paise: number;
  stages: EduStageCost[];
  low: EduScenario;
  base: EduScenario;
  high: EduScenario;
  nothing_left: boolean;
  notes: string[];
}

export interface EduEffect {
  consequences: Consequences;
  without_at_end_today_paise: number;
  with_at_end_today_paise: number;
  note: string;
}

export interface EducationOut {
  saved: boolean;
  plan: EduPlan;
  summary: EduSummary;
  lifeline: EduEffect;
}

export const educationKey = ["education"] as const;

export const MAX_STAGES = 6;

export const getEducation = () => api.get<EducationOut>("/api/plan/education").then((r) => r.data);
export const saveEducation = (plan: EduPlan) => api.put<EducationOut>("/api/plan/education", plan).then((r) => r.data);
export const forgetEducation = () => api.del<void>("/api/plan/education").then((r) => r.data);

/** The form's text fields: rupees as typed, percentages as typed. */
export interface EduDraft {
  child_birth_year: string;
  inflation: string;
  return_pct: string;
  corpus: string;
  stages: { name: string; start_age: string; cost: string }[];
}

/** Basis points as the percentage text a person types: 1150 -> "11.5". */
export const bpsToText = (bps: number) => String(bps / 100);

export function toEduDraft(p: EduPlan): EduDraft {
  return {
    child_birth_year: String(p.child_birth_year),
    inflation: bpsToText(p.inflation_bps),
    return_pct: bpsToText(p.return_bps),
    corpus: p.corpus_paise === 0 ? "" : paiseToInput(p.corpus_paise),
    stages: p.stages.map((s) => ({ name: s.name, start_age: String(s.start_age), cost: paiseToInput(s.today_cost_paise) })),
  };
}

/** A percentage as whole basis points, from digits only (no float arithmetic on the way). */
export function percentToBps(label: string, text: string, max: number): number {
  const m = /^(\d{1,3})(?:\.(\d{1,2}))?$/.exec(text.trim());
  if (!m) throw new Error(`${label} must be a percentage like 10 or 11.5.`);
  const bps = Number(m[1]) * 100 + Number((m[2] ?? "").padEnd(2, "0"));
  if (bps > max * 100) throw new Error(`${label} must be between 0 and ${max}.`);
  return bps;
}

function wholeNumber(label: string, text: string, min: number, max: number): number {
  const t = text.trim();
  const n = Number(t);
  if (!/^\d+$/.test(t) || n < min || n > max) throw new Error(`${label} must be a whole number from ${min} to ${max}.`);
  return n;
}

/** Turns the form into a plan to save, or throws a sentence to show. Money goes through `toPaise`. */
export function buildEduPlan(d: EduDraft): EduPlan {
  if (d.stages.length === 0) throw new Error("Add at least one stage, like School or College.");
  const stages = d.stages.map((s, i) => {
    const label = s.name.trim() || `Stage ${i + 1}`;
    if (s.name.trim() === "") throw new Error(`Stage ${i + 1} needs a name.`);
    const cost = toPaise(s.cost);
    if (cost === null || cost < 0) throw new Error(`${label}: enter a plain rupee amount for today's cost, like 150000.`);
    return { name: s.name.trim(), start_age: wholeNumber(`${label}: starting age`, s.start_age, 0, 40), today_cost_paise: cost };
  });
  const corpus = d.corpus.trim() === "" ? 0 : toPaise(d.corpus);
  if (corpus === null || corpus < 0) throw new Error("Already set aside: enter a plain rupee amount, like 200000.");
  return {
    child_birth_year: wholeNumber("Birth year", d.child_birth_year, 1900, 2200),
    stages,
    inflation_bps: percentToBps("Fee inflation", d.inflation, 20),
    return_bps: percentToBps("Expected return", d.return_pct, 30),
    corpus_paise: corpus,
  };
}

/** "Starts in 2044" or "Has begun", for a stage row. */
export function stageWhen(s: Pick<EduStageCost, "year" | "started">): string {
  return s.started ? "Has begun" : `Due ${s.year}`;
}
