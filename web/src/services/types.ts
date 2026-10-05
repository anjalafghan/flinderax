// Wire types for the planner endpoints. All money is integer paise; dates are "YYYY-MM-DD".

export type AccountKind = "bank" | "fd" | "rd" | "cash";

export interface Account {
  account_id: string;
  name: string;
  kind: AccountKind;
  balance_paise: number;
  locked: boolean;
  interest_bps: number | null;
  maturity_date: string | null;
  updated_at: string;
}

export interface NewAccount {
  name: string;
  kind: AccountKind;
  balance_paise: number;
  locked?: boolean;
  interest_bps?: number | null;
  maturity_date?: string | null;
}

export interface Commitment {
  commitment_id: string;
  label: string;
  amount_paise: number;
  day_of_month: number;
  account_id: string | null;
  category: string;
  active: boolean;
  start_month: string | null;
  end_month: string | null;
}

export type NewCommitment = Omit<Commitment, "commitment_id">;

export interface Income {
  income_id: string;
  label: string;
  amount_paise: number;
  day_of_month: number;
  account_id: string | null;
}

export type NewIncome = Omit<Income, "income_id">;

export interface PlanSettings {
  cushion_paise: number;
  next_period_living_paise: number;
  week_start: number;
}

export interface SnapshotInput {
  card_id: string;
  total_due_paise: number;
  outstanding_paise: number;
  unbilled_spent_paise?: number | null;
  unbilled_credit_paise?: number | null;
}

export interface Emi {
  emi_id: string;
  label: string;
  monthly_paise: number;
  remaining_principal_paise: number;
  months_left: number;
}

export type NewEmi = Omit<Emi, "emi_id"> & { card_id: string };

export interface PendingCharge {
  charge_id: string;
  label: string;
  amount_paise: number;
}

export interface CardBreakdown {
  card_id: string;
  card_name: string;
  due_now_paise: number;
  due_date: string | null;
  next_bill_est_paise: number;
  next_statement_date: string | null;
  next_due_date: string | null;
  loan_not_due_paise: number;
  last_updated: string | null;
  emis: Emi[];
  pending_charges: PendingCharge[];
}

export type Status = "green" | "yellow" | "red";

export interface UpcomingItem {
  date: string;
  label: string;
  amount_paise: number;
  kind: "commitment" | "card_due" | "card_bill";
}

export interface DashboardSummary {
  safe_to_spend_paise: number;
  per_week_paise: number;
  status: Status;
  until: string;
  period_label: string | null;
  next_income: { label: string; arrives: string } | null;
  lowest_point: { amount_paise: number; date: string };
  upcoming: UpcomingItem[];
  actions: { text: string; severity: "urgent" | "warn" | "info" }[];
  stale: { what: string; days_old: number }[];
}
