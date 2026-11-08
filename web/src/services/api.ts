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
  IngestRule,
  NewTxn,
  ParseResult,
  SpendSummary,
  Txn,
  UpdateTxnCategory,
  JoyConnect,
  JoySummary,
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

async function send(method: "GET" | "POST", url: string, body?: unknown): Promise<Response | null> {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  const token = localStorage.getItem("token");
  if (token) headers["Authorization"] = `Bearer ${token}`;

  const response = await fetch(url, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  if (response.status === 401) {
    handleUnauthorized();
    return null;
  }
  if (!response.ok) {
    const errorBody = await response.text().catch(() => "");
    throw new Error(errorBody || `Request failed with status ${response.status}`);
  }
  return response;
}

async function json<T>(method: "GET" | "POST", url: string, body?: unknown): Promise<ApiResponse<T>> {
  const response = await send(method, url, body);
  // Never resolve after a 401 so callers don't run on a page that is already redirecting.
  if (!response) return new Promise(() => {});
  return {
    data: await response.json(),
    status: response.status,
    statusText: response.statusText,
  };
}

async function binary(method: "GET" | "POST", url: string, body?: unknown): Promise<ArrayBuffer> {
  const response = await send(method, url, body);
  if (!response) return new Promise(() => {});
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
