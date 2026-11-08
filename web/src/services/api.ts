import type {
  Account,
  AccountUpdate,
  MoveRow,
  Projection,
  RecordMove,
  ScenarioItem,
  CardBreakdown,
  Commitment,
  DashboardSummary,
  Emi,
  Income,
  NewAccount,
  NewCommitment,
  NewEmi,
  NewIncome,
  DashboardHistory,
  PlanSettings,
  SnapshotInput,
  IngestKeyStatus,
  StatementResult,
  IngestRule,
  NewTxn,
  ParseResult,
  SpendSummary,
  Txn,
  UpdateTxnCategory,
  JoyConnect,
  JoySummary,
  BillsResponse,
  BillSuggestion,
  GoalInput,
  GoalsResponse,
  AffordRequest,
  AffordResult,
  MonthReview,
  TaxReview,
  TaxSectionId,
  NewTaxEntry,
} from "./types";

// The Rust server serves both the API and this app, so every request is same-origin.

interface ApiResponse<T> {
  data: T;
  status: number;
  statusText: string;
}

const handleUnauthorized = () => {
  localStorage.removeItem("token");
  window.location.href = "/auth";
};

// Login and register answer 401 for a wrong password; that is a result for the form to show, not a session expiry.
const isAuthEndpoint = (url: string) => url === "/api/common/login" || url === "/api/common/register";

/**
 * Sends a request. Always resolves with a Response or rejects with an Error, so callers never hang.
 * A 401 on an authenticated request ends the session and redirects to /auth; the rejection still
 * settles the promise. A 401 on the login/register endpoints just rejects with the server's message.
 */
async function send(method: "GET" | "POST", url: string, body?: unknown): Promise<Response> {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  const token = localStorage.getItem("token");
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const response = await fetch(url, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (!response.ok) {
    const errorBody = await response.text().catch(() => "");
    if (response.status === 401 && token && !isAuthEndpoint(url)) {
      handleUnauthorized();
    }
    throw new Error(errorBody || `Request failed with status ${response.status}`);
  }
  return response;
}

async function json<T>(method: "GET" | "POST", url: string, body?: unknown): Promise<ApiResponse<T>> {
  const response = await send(method, url, body);
  return {
    data: await response.json(),
    status: response.status,
    statusText: response.statusText,
  };
}

async function binary(method: "GET" | "POST", url: string, body?: unknown): Promise<ArrayBuffer> {
  const response = await send(method, url, body);
  return response.arrayBuffer();
}

const api = {
  get: <T>(url: string) => json<T>("GET", url),
  post: <T>(url: string, body: unknown) => json<T>("POST", url, body),
  postProtobuf: (url: string, body: unknown) => binary("POST", url, body),
  getProtobuf: (url: string) => binary("GET", url),
};

export default api;

// ---------- typed planner endpoints ----------

const data = <T>(p: Promise<ApiResponse<T>>) => p.then((r) => r.data);
type Id = { id: string; status: boolean };
type Ok = { status: boolean };

export const getDashboardHistory = (months = 12) =>
  data(api.get<DashboardHistory>(`/api/dashboard/history?months=${months}`));

export const dashboardApi = {
  summary: () => data(api.get<DashboardSummary>("/api/dashboard/summary")),
  history: getDashboardHistory,
};

export const accountApi = {
  list: () => data(api.get<Account[]>("/api/account/list")),
  create: (a: NewAccount) =>
    data(api.post<{ account_id: string; status: boolean }>("/api/account/create", a)),
  update: (a: AccountUpdate) => data(api.post<Ok>("/api/account/update", a)),
  updateBalance: (account_id: string, balance_paise: number) =>
    data(api.post<Ok>("/api/account/update_balance", { account_id, balance_paise })),
  remove: (account_id: string) => data(api.post<Ok>("/api/account/delete", { account_id })),
};

export const planApi = {
  commitments: () => data(api.get<Commitment[]>("/api/plan/commitment/list")),
  createCommitment: (c: NewCommitment) => data(api.post<Id>("/api/plan/commitment/create", c)),
  updateCommitment: (c: Commitment) => data(api.post<Ok>("/api/plan/commitment/update", c)),
  removeCommitment: (id: string) => data(api.post<Ok>("/api/plan/commitment/delete", { id })),

  incomes: () => data(api.get<Income[]>("/api/plan/income/list")),
  createIncome: (i: NewIncome) => data(api.post<Id>("/api/plan/income/create", i)),
  updateIncome: (i: Income) => data(api.post<Ok>("/api/plan/income/update", i)),
  removeIncome: (id: string) => data(api.post<Ok>("/api/plan/income/delete", { id })),

  settings: () => data(api.get<PlanSettings>("/api/plan/settings/get")),
  project: (body: { months?: number; spend_pct?: number; items: ScenarioItem[] }) =>
    data(api.post<Projection>("/api/plan/project", body)),
  updateSettings: (s: PlanSettings) => data(api.post<Ok>("/api/plan/settings/update", s)),
};

export const cardPlanApi = {
  breakdown: (card_id: string) => data(api.post<CardBreakdown>("/api/card/breakdown", { card_id })),
  snapshot: (s: SnapshotInput) =>
    data(api.post<{ snapshot_id: string; status: boolean }>("/api/card/snapshot", s)),
  createEmi: (e: NewEmi) => data(api.post<Id>("/api/card/emi/create", e)),
  updateEmi: (e: Emi) => data(api.post<Ok>("/api/card/emi/update", e)),
  removeEmi: (emi_id: string) => data(api.post<Ok>("/api/card/emi/delete", { emi_id })),
  createPending: (p: { card_id: string; label: string; amount_paise: number }) =>
    data(api.post<Id>("/api/card/pending/create", p)),
  clearPending: (charge_id: string) => data(api.post<Ok>("/api/card/pending/clear", { charge_id })),
};

export const movesApi = {
  record: (m: RecordMove) => data(api.post<{ move_id: string; status: boolean }>("/api/moves/record", m)),
  undo: (move_id: string) => data(api.post<Ok>("/api/moves/undo", { move_id })),
  list: () => data(api.get<MoveRow[]>("/api/moves/list")),
};

// ---------- spending (automatic transaction tracking) ----------

export const ingestApi = {
  transactions: (month?: string) =>
    data(api.get<Txn[]>(`/api/ingest/transactions${month ? `?month=${encodeURIComponent(month)}` : ""}`)),
  updateCategory: (u: UpdateTxnCategory) =>
    data(api.post<{ status: boolean; recategorized: number }>("/api/ingest/transactions/update", u)),
  removeTxn: (txn_id: string) => data(api.post<Ok>("/api/ingest/transactions/delete", { txn_id })),
  createTxn: (t: NewTxn) => data(api.post<{ txn_id: string; status: boolean }>("/api/ingest/transactions/create", t)),
  parse: (text: string) => data(api.post<ParseResult>("/api/ingest/parse", { text })),
  importStatement: (b: { csv: string; account_id?: string }) =>
    data(api.post<StatementResult>("/api/ingest/statement", b)),
  summary: (months = 6) => data(api.get<SpendSummary>(`/api/ingest/summary?months=${months}`)),
  keyStatus: () => data(api.get<IngestKeyStatus>("/api/ingest/key/status")),
  rotateKey: () => data(api.post<{ key: string }>("/api/ingest/key/rotate", {})),
  rules: () => data(api.get<IngestRule[]>("/api/ingest/rules")),
  deleteRule: (pattern: string) => data(api.post<Ok>("/api/ingest/rules/delete", { pattern })),
};

// ---------- Joy (shared date jar) ----------

export const joyApi = {
  summary: () => data(api.get<JoySummary>("/api/joy/summary")),
  connect: (c: JoyConnect) => data(api.post<Ok>("/api/joy/connect", c)),
  setPartner: (partner_paise: number) => data(api.post<Ok>("/api/joy/partner", { partner_paise })),
  disconnect: () => data(api.post<Ok>("/api/joy/disconnect", {})),
};

// ---------- bills, goals, can I afford it ----------

export const billsApi = {
  list: (days = 30) => data(api.get<BillsResponse>(`/api/bills?days=${days}`)),
  suggestions: () => data(api.get<BillSuggestion[]>("/api/bills/suggestions")),
  dismiss: (merchant: string) => data(api.post<Ok>("/api/bills/suggestions/dismiss", { merchant })),
};

export const goalsApi = {
  list: () => data(api.get<GoalsResponse>("/api/goals")),
  create: (g: GoalInput) => data(api.post<{ goal_id: string; status: boolean }>("/api/goals/create", g)),
  update: (g: GoalInput & { goal_id: string }) => data(api.post<Ok>("/api/goals/update", g)),
  archive: (goal_id: string) => data(api.post<Ok>("/api/goals/archive", { goal_id })),
  contribute: (goal_id: string, amount_paise: number, note?: string) =>
    data(api.post<Ok>("/api/goals/contribute", { goal_id, amount_paise, note })),
};

export const affordApi = {
  check: (r: AffordRequest) => data(api.post<AffordResult>("/api/plan/afford", r)),
};

// ---------- review: month close and tax ----------

const query = (params: Record<string, string | undefined>) => {
  const parts = Object.entries(params)
    .filter(([, v]) => v !== undefined)
    .map(([k, v]) => `${k}=${encodeURIComponent(v as string)}`);
  return parts.length ? `?${parts.join("&")}` : "";
};

export const reviewApi = {
  month: (month?: string) => data(api.get<MonthReview>(`/api/review/month${query({ month })}`)),
  tax: (fy?: string) => data(api.get<TaxReview>(`/api/review/tax${query({ fy })}`)),
  addTaxEntry: (e: NewTaxEntry) => data(api.post<{ entry_id: string }>("/api/review/tax/entry", e)),
  deleteTaxEntry: (entry_id: string) => data(api.post<Ok>("/api/review/tax/entry/delete", { entry_id })),
  setCommitmentTaxSection: (commitment_id: string, tax_section: TaxSectionId | null) =>
    data(api.post<Ok>("/api/review/tax/commitment", { commitment_id, tax_section })),
};
