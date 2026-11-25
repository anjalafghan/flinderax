import api from "@/services/api";
import { percentToBps } from "@/services/education";
import { toPaise } from "@/utils/money";

// POST /api/plan/guidance: a month plan (what to do, what it changes, why). Money is integer paise.

export interface GuidanceStep {
  what: string;
  impact: string;
  why: string;
}

export interface Guidance {
  spend_cap_paise: number;
  shared_mine_paise: number;
  free_to_spend_paise: number;
  fixed_paise: number;
  surplus_paise: number;
  monthly_cost_paise: number;
  emergency_target_paise: number;
  months_to_emergency: number | null;
  steps: GuidanceStep[];
  warnings: string[];
}

export interface GuidanceInput {
  salary_paise: number;
  fixed: { label: string; amount_paise: number }[];
  sip_paise: number;
  spend_bps: number;
  shared: { label: string; total_paise: number; partner_paise: number }[];
  emergency_saved_paise: number;
}

export interface GuidanceDraft {
  salary: string;
  sip: string;
  spend_pct: string;
  saved: string;
  fixed: { label: string; amount: string }[];
  shared: { label: string; total: string; partner: string }[];
}

export const getGuidance = (i: GuidanceInput) =>
  api.post<Guidance>("/api/plan/guidance", i).then((r) => r.data);

function amount(label: string, text: string, blankIsZero = true): number {
  if (text.trim() === "" && blankIsZero) return 0;
  const p = toPaise(text);
  if (p === null || p < 0)
    throw new Error(`${label}: enter a plain rupee amount, like 25000.`);
  return p;
}

/** The form's text as a request, or throws a sentence to show. Money goes through `toPaise`. */
export function buildGuidanceInput(d: GuidanceDraft): GuidanceInput {
  const salary = amount("Salary", d.salary, false);
  if (salary <= 0) throw new Error("Salary must be more than zero.");
  return {
    salary_paise: salary,
    sip_paise: amount("SIP", d.sip),
    spend_bps: percentToBps("Spending share", d.spend_pct, 100),
    emergency_saved_paise: amount("Already saved", d.saved),
    fixed: d.fixed
      .filter((f) => f.label.trim() !== "" || f.amount.trim() !== "")
      .map((f) => ({
        label: f.label.trim() || "Fixed cost",
        amount_paise: amount(f.label || "Fixed cost", f.amount),
      })),
    shared: d.shared
      .filter((s) => s.label.trim() !== "" || s.total.trim() !== "")
      .map((s) => ({
        label: s.label.trim() || "Shared",
        total_paise: amount(s.label || "Shared", s.total),
        partner_paise: amount(
          `${s.label || "Shared"}: partner's part`,
          s.partner,
        ),
      })),
  };
}
