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
  /** Matches bank alerts to this account. */
  last_4_digits: string | null;
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
  last_4_digits?: string | null;
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
  /** When on, the server adds salary and subtracts fixed payments on their dates (default true). */
  auto_post: boolean;
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
  kind: "commitment" | "card_due" | "card_bill" | "planned_date";
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

// ---------- dashboard history ----------

export interface HistoryMonth {
  /** "YYYY-MM". */
  month: string;
  liquid_paise: number;
  locked_paise: number;
  card_debt_paise: number;
  net_worth_paise: number;
}

export interface HistoryCard {
  card_id: string;
  name: string;
  outstanding_paise: number;
  /** null when no credit limit is set. */
  limit_paise: number | null;
  /** 1234 = 12.34%. null when no credit limit is set. */
  utilization_bps: number | null;
}

export interface HistoryMaturity {
  account_id: string;
  name: string;
  kind: AccountKind;
  balance_paise: number;
  /** YYYY-MM-DD. */
  maturity_date: string;
  days_left: number;
}

export interface HistoryAutoPosting {
  /** YYYY-MM-DD. */
  occurs_on: string;
  label: string;
  account_name: string;
  /** Signed: + credit, - debit. */
  amount_paise: number;
}

export interface DashboardHistory {
  /** Oldest to newest; the last entry is the current month. May be empty. */
  months: HistoryMonth[];
  cards: HistoryCard[];
  /** Upcoming only, soonest first. */
  maturities: HistoryMaturity[];
  /** The 10 most recent, newest first. */
  auto_postings: HistoryAutoPosting[];
  auto_post: boolean;
}

// ---------- spending (automatic transaction tracking) ----------

export type TxnDirection = "debit" | "credit";
export type TxnSource = "sms" | "email" | "paste" | "manual";

export interface Txn {
  txn_id: string;
  /** YYYY-MM-DD. */
  occurred_on: string;
  direction: TxnDirection;
  /** Always positive; `direction` gives the sign. */
  amount_paise: number;
  merchant: string | null;
  category: string;
  card_id: string | null;
  account_id: string | null;
  /** Card or account name. */
  source_name: string | null;
  last4: string | null;
  source: TxnSource;
  created_at: string;
}

export interface NewTxn {
  occurred_on: string;
  direction: TxnDirection;
  amount_paise: number;
  merchant: string | null;
  category: string;
}

export interface UpdateTxnCategory {
  txn_id: string;
  category: string;
  merchant?: string | null;
  /** Make "this merchant -> this category" a rule. */
  remember: boolean;
}

export interface ParseResult {
  status: "created" | "duplicate" | "unrecognized";
  transaction: Txn | null;
  balance_updated: { account_name: string; balance_paise: number } | null;
}

export interface StatementResult {
  total: number;
  added: number;
  duplicates: number;
  /** Bank-alert rows this statement confirmed. The same number as `upgraded`. */
  matched_alerts: number;
  skipped: number;
  uncategorized: number;
  first_date: string | null;
  last_date: string | null;
  balance_updated: { account_name: string; balance_paise: number } | null;
  /** Rows from earlier imports whose category changed when this import re-checked them. */
  older_recategorized?: number;
  /** A balance entered after the statement ends, kept instead of the statement's closing balance. */
  balance_kept?: {
    account_name: string;
    balance_paise: number;
    set_on: string;
    statement_paise: number;
    statement_on: string;
  } | null;
  /** Bank-alert rows this statement confirmed (kept, with the statement's reference and narration). */
  upgraded: number;
  /** Rows added or confirmed that got a category from structure, rules or keywords. */
  categorized: number;
  /** Each balance follows from the one before it. Only meaningful when opening_paise is set. */
  chain_ok: boolean;
  /** Null when the file has no balance column. */
  opening_paise: number | null;
  closing_paise: number | null;
  /** Bank-alert transactions on this account or card, inside the statement's dates, that it never confirmed. */
  not_on_statement: number;
  /** A payee the user seems to pay and receive from, offered as their own name when none are saved. */
  holder_name_suggestion: string | null;
}

/** How a file's amounts say which side a row is on. */
export type SignRule =
  | { kind: "separate_columns" }
  | { kind: "negative_is_debit" }
  | { kind: "dr_cr_column"; column: number }
  | { kind: "cr_suffix" };

/** Which column holds what. Column numbers count from zero; header_row counts every line of the file, blank ones too. */
export interface StatementMapping {
  header_row: number;
  date: number;
  narration: number;
  debit: number | null;
  credit: number | null;
  amount: number | null;
  sign: SignRule;
  balance: number | null;
  reference: number | null;
  date_order: "dmy" | "mdy" | "ymd";
}

/** The file's header and the rows under it, for mapping its columns. */
export interface StatementPreview {
  header_row: number;
  rows: string[][];
  /** The mapping automatic detection would use, or null when it finds no header. */
  guessed: StatementMapping | null;
}

export interface StatementPreviewResponse {
  preview: StatementPreview;
  /** Identifies the layout: a saved mapping is found again by it. */
  signature: string;
  known_profile: boolean;
}

/** The body of the 422 an upload answers with when the file's layout can't be read automatically. */
export interface StatementNeedsMapping {
  error: "needs_mapping";
  preview: StatementPreview;
}

/** Where a statement goes, and optionally the mapping to read it with. */
export interface StatementTarget {
  account_id?: string;
  card_id?: string;
  mapping?: StatementMapping;
  save_profile?: boolean;
  label?: string;
}

export interface SpendMonth {
  /** "YYYY-MM". */
  month: string;
  spent_paise: number;
  by_category: { category: string; amount_paise: number }[];
}

export interface SpendSummary {
  /** Oldest to newest; the last entry is the current month. */
  months: SpendMonth[];
  top_merchants: { merchant: string; amount_paise: number; count: number }[];
  uncategorized_count: number;
}

export interface IngestKeyStatus {
  has_key: boolean;
  created_at: string | null;
  last_used_at: string | null;
}

export interface IngestRule {
  pattern: string;
  category: string;
}

// ---------- Joy (shared date-jar app) ----------

export interface JoyMonth {
  /** "YYYY-MM". */
  month: string;
  budget_paise: number | null;
  carry_paise: number;
  spent_paise: number;
  your_share_paise: number;
  your_budget_paise: number | null;
}

export interface JoyRecent {
  /** "YYYY-MM-DD". */
  day: string;
  title: string;
  kind: "date" | "other" | "regular";
  amount_paise: number;
  your_share_paise: number;
}

export interface JoySummary {
  connected: boolean;
  partner_paise: number;
  month: JoyMonth | null;
  recent: JoyRecent[];
  labelled: number;
}

// ---------- bills (what is due, and what looks like a subscription) ----------

export type BillKind = "monthly" | "yearly" | "card_bill" | "date";

export interface Bill {
  /** "YYYY-MM-DD" */
  date: string;
  label: string;
  amount_paise: number;
  kind: BillKind;
  account_id: string | null;
  /** For card bills: how much of this is already moved aside. */
  covered_paise: number;
}

export interface SinkingFund {
  label: string;
  amount_paise: number;
  /** "YYYY-MM-DD" */
  due: string;
  saved_paise: number;
}

export interface BillsResponse {
  bills: Bill[];
  total_paise: number;
  sinking_funds: SinkingFund[];
}

export interface BillSuggestion {
  merchant: string;
  label: string;
  every: "monthly" | "yearly";
  amount_paise: number;
  day: number;
  month: number | null;
  last_on: string;
  times: number;
  /** The previous amount, when this one went up. */
  was_paise: number | null;
}

// ---------- goals ----------

export type GoalStatus = "done" | "on_track" | "behind" | "open";
export type GoalKind = "emergency" | "custom";

export interface Goal {
  id: string;
  name: string;
  target_paise: number;
  saved_paise: number;
  this_month_paise: number;
  needs_monthly_paise: number;
  months_left: number | null;
  status: GoalStatus;
  reached_by: string | null;
  fixes: string[];
  /** Not in every response yet; the edit form starts blank when missing. */
  kind?: GoalKind;
  target_date?: string | null;
  priority?: number;
  account_id?: string | null;
}

export interface GoalsResponse {
  goals: Goal[];
  available_paise: number;
  unallocated_paise: number;
  /** Months of needs covered by savings, in tenths (42 = 4.2 months). */
  months_covered_tenths: number | null;
  emergency_suggested_paise: number | null;
}

export interface GoalInput {
  goal_id?: string;
  name: string;
  kind?: GoalKind;
  target_paise: number;
  target_date?: string | null;
  priority?: number;
  account_id?: string | null;
}

// ---------- can I afford it ----------

export type AffordPay =
  | { mode: "now" }
  | { mode: "emi"; months: number; rate_bps: number; fee_paise: number };

export interface AffordRequest {
  amount_paise: number;
  pay: AffordPay;
}

export type AffordAnswer = "yes" | "yes_from_savings" | "wait" | "no";

export interface AffordResult {
  answer: AffordAnswer;
  headline: string;
  reasons: string[];
  safe_to_spend_after_paise: number;
  monthly_paise: number;
  total_cost_paise: number;
  extra_cost_paise: number;
  /** "YYYY-MM-DD" when the answer is "wait". */
  wait_until: string | null;
}

// ---------- review: month close and tax ----------

/** GET /api/review/month. Money is paise; `sentences` are the plain-language lines shown first. */
export interface MonthReview {
  month: string;
  income_paise: number;
  spent_paise: number;
  allowance_paise: number;
  over_by_paise: number;
  top_categories: { category: string; amount_paise: number }[];
  dates: { spent_paise: number; budget_paise: number | null; your_share_paise: number } | null;
  savings_rate_bps: number | null;
  net_worth_change_paise: number | null;
  sentences: string[];
}

/** Section ids the server accepts for tax deductions. */
export type TaxSectionId = "80C" | "80CCD1B" | "80D" | "HRA" | "24B" | "other";

export interface TaxSectionRow {
  section: TaxSectionId;
  claimed_paise: number;
  /** Still to come this year, shown as a lighter segment on top of `claimed_paise`. */
  projected_paise: number;
  limit_paise: number | null;
  left_paise: number | null;
  note: string;
}

export interface TaxEntry {
  entry_id: string;
  section: TaxSectionId;
  label: string;
  amount_paise: number;
}

export interface TaxCommitment {
  commitment_id: string;
  label: string;
  tax_section: TaxSectionId | null;
  paid_paise: number;
  projected_paise: number;
}

export interface TaxRegime {
  old_tax_paise: number;
  new_tax_paise: number;
  better: "old" | "new";
  note: string;
  /** Old-regime deductions (excl. standard deduction) above which old wins; null = new wins always. */
  break_even_deductions_paise: number | null;
  /** Break-even minus claimed deductions, never below zero; null when there is no break-even. */
  deductions_needed_paise: number | null;
}

export interface TaxLtcg {
  exemption_paise: number;
  used_paise: number;
  left_paise: number;
  days_left_in_fy: number;
  note: string;
}

export interface TaxReminder {
  date: string;
  text: string;
  days_left: number;
}

/** GET /api/review/tax. `fy` is the financial year label such as "2026-27". */
export interface TaxReview {
  fy: string;
  from: string;
  to: string;
  sections: TaxSectionRow[];
  entries: TaxEntry[];
  commitments: TaxCommitment[];
  interest_income_paise: number;
  gross_salary_paise: number;
  regime: TaxRegime;
  reminders: TaxReminder[];
  ltcg: TaxLtcg;
}

export interface NewTaxEntry {
  section: TaxSectionId;
  label: string;
  amount_paise: number;
  fy?: string;
}
