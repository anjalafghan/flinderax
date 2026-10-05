import type {
  Account,
  AccountKind,
  CardBreakdown,
  Commitment,
  DashboardSummary,
  Emi,
  Income,
  NewAccount,
  NewCommitment,
  NewEmi,
  NewIncome,
  PlanSettings,
  SnapshotInput,
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

export const dashboardApi = {
  summary: () => data(api.get<DashboardSummary>("/api/dashboard/summary")),
};

export const accountApi = {
  list: () => data(api.get<Account[]>("/api/account/list")),
  create: (a: NewAccount) =>
    data(api.post<{ account_id: string; status: boolean }>("/api/account/create", a)),
  update: (a: {
    account_id: string;
    name: string;
    kind: AccountKind;
    locked: boolean;
    interest_bps?: number | null;
    maturity_date?: string | null;
  }) => data(api.post<Ok>("/api/account/update", a)),
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
