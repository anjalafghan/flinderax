// Wire types for the planner endpoints. All money is integer paise; dates are "YYYY-MM-DD".

export type AccountKind = "bank" | "fd" | "rd" | "cash";

export type AccountRole = "spending" | "reserve" | "other";
export type Renewal = "payout" | "renew_principal" | "renew_all";

export interface Account {
  account_id: string;
  name: string;
  kind: AccountKind;
  balance_paise: number;
  locked: boolean;
  interest_bps: number | null;
  maturity_date: string | null;
  /** spending: salary lands + SIP debits; reserve: savings, card bills, needs. */
  role: AccountRole;
  /** Where an FD / RD is held. */
  institution: string | null;
  renewal: Renewal;
  payout_account_id: string | null;
  updated_at: string;
}

export interface NewAccount {
  name: string;
  kind: AccountKind;
  balance_paise: number;
  locked?: boolean;
  interest_bps?: number | null;
  maturity_date?: string | null;
  role?: AccountRole;
  institution?: string | null;
  renewal?: Renewal;
  payout_account_id?: string | null;
}

export type AccountUpdate = Omit<NewAccount, "balance_paise"> & { account_id: string; locked: boolean };

export type Bucket = "needs" | "savings" | "wants";

export interface Commitment {
  commitment_id: string;
  label: string;
  amount_paise: number;
  day_of_month: number;
  account_id: string | null;
  category: string;
  bucket: Bucket;
  frequency: "monthly" | "yearly";
  /** 1-12, for yearly bills (null = month not known yet). */
  due_month: number | null;
  /** Where the money lands when it is a transfer to savings (an RD, an FD top-up...). */
  to_account_id: string | null;
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
  /** Monthly spending allowance as an amount. Set this or `spending_bps`, not both. */
  spending_paise: number | null;
  /** Monthly spending allowance as a share of income, in basis points (2500 = 25%). */
  spending_bps: number | null;
  /** Day of the month the savings and needs money should have reached the reserve account. */
  allocation_day: number;
  /** Savings in the reserve account that card bills must not eat; null = no floor. */
  savings_floor_paise: number | null;
  /** Pay periods over which an overspend is paid back. */
  recovery_months: number;
  week_start: number;
}

export interface SnapshotInput {
  card_id: string;
  total_due_paise: number;
  outstanding_paise: number;
  unbilled_spent_paise?: number | null;
  unbilled_credit_paise?: number | null;
  /** First snapshot only: true = money owed before tracking began, false = new spending. */
  starting_balance?: boolean;
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
  pay_from_account_id: string | null;
  unbilled_net_paise: number | null;
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

export interface BreakdownLine {
  label: string;
  /** Signed: money in is positive, money set aside or going out is negative. */
  amount_paise: number;
  note: string | null;
  total: boolean;
}

export interface LedgerAction {
  kind: "card_bill" | "allocation";
  key: string;
  /** How much to move now. */
  amount_paise: number;
  from_account_id: string | null;
  to_account_id: string | null;
  /** The part that can't be moved right now (offered as "skip this month"). */
  skippable_paise: number;
}

export interface Budget {
  income_paise: number;
  needs_paise: number;
  spending_paise: number;
  savings_paise: number;
  sip_paise: number;
  rest_savings_paise: number;
  allocation_paise: number;
}

export interface Pots {
  spending: {
    account_name: string | null;
    balance_paise: number;
    committed_paise: number;
    to_move_paise: number;
    free_paise: number;
  };
  reserve: {
    account_name: string | null;
    balance_paise: number;
    required_paise: number;
    free_savings_paise: number;
    floor_paise: number;
    below_floor: boolean;
  };
  locked: { total_paise: number; count: number };
}

export interface NextPeriod {
  label: string;
  arrives: string;
  income_paise: number;
  committed_paise: number;
  left_paise: number;
}

export interface DashboardSummary {
  safe_to_spend_paise: number;
  per_week_paise: number;
  status: Status;
  /** The next payday: the safe number covers today up to this date. */
  until: string;
  days_left: number;
  period_start: string;
  period_label: string | null;
  next_income: { label: string; arrives: string } | null;
  short_by_paise: number;
  breakdown: BreakdownLine[];
  budget: Budget;
  pots: Pots;
  spending: {
    tracked: boolean;
    spent_paise: number;
    /** Allowance assumed used before tracking started, at an even pace. */
    before_tracking_paise: number;
    allowance_paise: number;
    left_paise: number;
    overspend_paise: number;
  };
  recovery: { owed_paise: number; months: number; per_month_paise: number } | null;
  next_period: NextPeriod | null;
  upcoming: UpcomingItem[];
  actions: { text: string; severity: "urgent" | "warn" | "info"; ledger: LedgerAction | null }[];
  stale: { what: string; days_old: number }[];
}

export type MoveMode = "moved" | "already_covered" | "skipped";

export interface RecordMove {
  kind: "card_bill" | "allocation";
  key: string;
  amount_paise: number;
  mode: MoveMode;
  from_account_id?: string | null;
  to_account_id?: string | null;
}

export interface MoveRow {
  move_id: string;
  kind: string;
  key: string;
  amount_paise: number;
  mode: MoveMode;
  from_account_id: string | null;
  to_account_id: string | null;
  created_at: string;
}

// ---------- projection / what-if ----------

export type ScenarioKind = "rd" | "fd" | "one_off";

export interface ScenarioItem {
  name: string;
  kind: ScenarioKind;
  /** rd: per month. fd / one_off: the lump sum. */
  amount_paise: number;
  /** YYYY-MM-DD of the first instalment / deposit / spend. */
  start: string;
  tenure_months: number | null;
  rate_bps: number | null;
  funded_from: string | null;
  payout_to: string | null;
  renewal: Renewal | null;
  target_paise: number | null;
  target_date: string | null;
}

export interface ProjectionPoint {
  month: string;
  spending_paise: number;
  reserve_paise: number;
  fds_paise: number;
  rds_paise: number;
  invested_paise: number;
  total_paise: number;
}

export interface Stress {
  month: string;
  kind: "spending_short" | "reserve_below_floor" | "fd_needed";
  amount_paise: number;
}

export interface Series {
  points: ProjectionPoint[];
  stress: Stress[];
}

export interface Absorption {
  cost_paise: number;
  from_rest_savings_paise: number;
  from_sip_paise: number;
  from_spending_paise: number;
  from_fd_paise: number;
  rest_savings_after_paise: number;
  sip_after_paise: number;
  spending_after_paise: number;
}

export interface ItemImpact {
  name: string;
  monthly_paise: number;
  savings_first: Absorption | null;
  spending_first: Absorption | null;
  savings_first_thresholds: { label: string; up_to_paise: number }[];
  maturity_value_paise: number;
  goal: {
    target_paise: number;
    target_date: string;
    projected_paise: number;
    gap_paise: number;
    required_monthly_paise: number | null;
  } | null;
}

export interface Projection {
  baseline: Series;
  scenario: Series | null;
  impacts: ItemImpact[];
}
