import { adoptRenewedToken } from "@/services/renew";
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
  StatementPreviewResponse,
  StatementTarget,
  IngestRule,
  NewTxn,
  ParseResult,
  SpendSummary,
  Txn,
  UpdateTxnCategory,
  JoySummary,
  ArchivedGoal,
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

import { authHeadersFor, clearSession } from "../session";
import { errorMessageOf } from "./errors";
import { sendOrQueue } from "@/offline";

// The Rust server serves both the API and this app, so every request is same-origin.

interface ApiResponse<T> {
  data: T;
  status: number;
  statusText: string;
}

const handleUnauthorized = () => {
  clearSession();
  window.location.href = "/auth";
};

// Login and register answer 401 for a wrong password; that is a result for the form to show, not a session expiry.
const isAuthEndpoint = (url: string) => url === "/api/common/login" || url === "/api/common/register";

/**
 * Sends a request. Always resolves with a Response or rejects with an Error, so callers never hang.
 * A 401 on an authenticated request ends the session and redirects to /auth; the rejection still
 * settles the promise. A 401 on the login/register endpoints just rejects with the server's message.
 */
type Method = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

async function send(method: Method, url: string, body?: unknown): Promise<Response> {
  const headers: HeadersInit = { "Content-Type": "application/json" };
  const token = localStorage.getItem("token");
  Object.assign(headers, authHeadersFor(token));

  const response = await fetch(url, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  adoptRenewedToken(response);
  if (!response.ok) {
    const errorBody = await response.text().catch(() => "");
    if (response.status === 401 && token && !isAuthEndpoint(url)) {
      handleUnauthorized();
    }
    throw new Error(errorMessageOf(errorBody, response.status));
  }
  return response;
}

async function json<T>(method: Method, url: string, body?: unknown): Promise<ApiResponse<T>> {
  const response = await send(method, url, body);
  // A 204 (a delete) has no body.
  const text = response.status === 204 ? "" : await response.text();
  return {
    data: (text ? JSON.parse(text) : undefined) as T,
    status: response.status,
    statusText: response.statusText,
  };
}

const api = {
  get: <T>(url: string) => json<T>("GET", url),
  post: <T>(url: string, body: unknown) => json<T>("POST", url, body),
  put: <T>(url: string, body: unknown) => json<T>("PUT", url, body),
  patch: <T>(url: string, body: unknown) => json<T>("PATCH", url, body),
  del: <T = void>(url: string) => json<T>("DELETE", url),
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
  scenarios: () => data(api.get<{ drafts: unknown[] }>("/api/plan/scenarios")),
  saveScenarios: (drafts: unknown[]) => data(api.post<Ok>("/api/plan/scenarios", { drafts })),
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
  resetSettings: () => data(api.post<Ok>("/api/plan/settings/reset", {})),
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
  record: (m: RecordMove) =>
    data(api.post<{ move_id: string; status: boolean }>("/api/moves/record", { idempotency_key: crypto.randomUUID(), ...m })),
  undo: (move_id: string) => data(api.post<Ok>("/api/moves/undo", { move_id })),
  list: () => data(api.get<MoveRow[]>("/api/moves/list")),
};

// ---------- spending (automatic transaction tracking) ----------

/** A statement multipart post. No Content-Type header: the browser must set the multipart boundary itself. */
async function postStatement<T>(url: string, form: FormData): Promise<T> {
  const token = localStorage.getItem("token");
  const response = await fetch(url, {
    method: "POST",
    headers: authHeadersFor(token),
    body: form,
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new Error(text ? errorMessageOf(text, response.status) : `Upload failed (${response.status})`);
  }
  return (await response.json()) as T;
}

export const ingestApi = {
  unrecognized: () => data(api.get<{ count: number; items: { alert_id: string; text: string; source: string; received_at: string }[] }>("/api/ingest/unrecognized")),
  dismissUnrecognized: (alert_id: string) => data(api.post<Ok>("/api/ingest/unrecognized/dismiss", { alert_id })),
  transactions: (month?: string) =>
    data(api.get<Txn[]>(`/api/ingest/transactions${month ? `?month=${encodeURIComponent(month)}` : ""}`)),
  updateCategory: (u: UpdateTxnCategory) =>
    data(api.post<{ status: boolean; recategorized: number }>("/api/ingest/transactions/update", u)),
  removeTxn: (txn_id: string) => data(api.post<Ok>("/api/ingest/transactions/delete", { txn_id })),
  /** Works with no signal: if the phone is offline it is kept and sent later (`queued`). */
  createTxn: async (t: NewTxn): Promise<{ txn_id: string; status: boolean; queued?: boolean }> => {
    const r = await sendOrQueue("/api/ingest/transactions/create", t, `Transaction: ${t.merchant ?? "payment"}`);
    if (r.queued || !r.response) return { txn_id: "", status: true, queued: true };
    if (r.response.status === 401) {
      handleUnauthorized();
      throw new Error("Please sign in again.");
    }
    const text = await r.response.text().catch(() => "");
    if (!r.response.ok) throw new Error(errorMessageOf(text, r.response.status));
    return JSON.parse(text) as { txn_id: string; status: boolean };
  },
  parse: (text: string) => data(api.post<ParseResult>("/api/ingest/parse", { text })),
  importStatement: (b: { csv: string; account_id?: string }) =>
    data(api.post<StatementResult>("/api/ingest/statement", b)),
  /** Multipart upload: the server reads the file in memory and never stores it. */
  uploadStatement: async (file: File, target: StatementTarget = {}): Promise<StatementResult> => {
    const form = new FormData();
    form.append("file", file, file.name);
    if (target.account_id) form.append("account_id", target.account_id);
    if (target.card_id) form.append("card_id", target.card_id);
    if (target.mapping) {
      form.append("mapping", JSON.stringify(target.mapping));
      form.append("save_profile", target.save_profile ? "true" : "false");
      form.append("label", target.label ?? "");
    }
    return postStatement<StatementResult>("/api/ingest/statement/upload", form);
  },
  /** Reads the file's header and first rows so its columns can be mapped. Nothing is stored. */
  previewStatement: (file: File, target: { card_id?: string } = {}): Promise<StatementPreviewResponse> => {
    const form = new FormData();
    form.append("file", file, file.name);
    if (target.card_id) form.append("card_id", target.card_id);
    return postStatement<StatementPreviewResponse>("/api/ingest/statement/preview", form);
  },
  holderNames: () => data(api.get<{ names: string[] }>("/api/ingest/holder-names")),
  saveHolderNames: (names: string[]) => data(api.post<{ names: string[] }>("/api/ingest/holder-names", { names })),
  recategorize: () => data(api.post<{ status: boolean; recategorized: number }>("/api/ingest/recategorize", {})),
  summary: (months = 6) => data(api.get<SpendSummary>(`/api/ingest/summary?months=${months}`)),
  keyStatus: () => data(api.get<IngestKeyStatus>("/api/ingest/key/status")),
  rotateKey: () => data(api.post<{ key: string }>("/api/ingest/key/rotate", {})),
  rules: () => data(api.get<IngestRule[]>("/api/ingest/rules")),
  deleteRule: (pattern: string) => data(api.post<Ok>("/api/ingest/rules/delete", { pattern })),
  /** Forgets every taught category. Rows already sorted keep the category they have. */
  clearRules: () => data(api.post<{ status: boolean; removed: number }>("/api/ingest/rules/clear", {})),
  /** Forgets the column layouts saved for statements. */
  clearLayouts: () => data(api.post<{ status: boolean; removed: number }>("/api/ingest/statement/layouts/clear", {})),
  /** Hands a hand-set category back to the automatic one. */
  unlock: (txn_id: string) => data(api.post<{ status: boolean; category: string }>("/api/ingest/transactions/unlock", { txn_id })),
};

// ---------- Joy (shared date jar) ----------

export const joyApi = {
  summary: () => data(api.get<JoySummary>("/api/joy/summary")),
  setPartner: (partner_paise: number) => data(api.post<Ok>("/api/joy/partner", { partner_paise })),
};

// ---------- bills, goals, can I afford it ----------

export const billsApi = {
  list: (days = 30) => data(api.get<BillsResponse>(`/api/bills?days=${days}`)),
  suggestions: () => data(api.get<BillSuggestion[]>("/api/bills/suggestions")),
  dismiss: (merchant: string) => data(api.post<Ok>("/api/bills/suggestions/dismiss", { merchant })),
  restore: () => data(api.post<Ok & { restored: number }>("/api/bills/suggestions/restore", {})),
};

export const goalsApi = {
  list: () => data(api.get<GoalsResponse>("/api/goals")),
  create: (g: GoalInput) => data(api.post<{ goal_id: string; status: boolean }>("/api/goals/create", g)),
  update: (g: GoalInput & { goal_id: string }) => data(api.post<Ok>("/api/goals/update", g)),
  archive: (goal_id: string) => data(api.post<Ok>("/api/goals/archive", { goal_id })),
  unarchive: (goal_id: string) => data(api.post<Ok>("/api/goals/unarchive", { goal_id })),
  archived: () => data(api.get<ArchivedGoal[]>("/api/goals/archived")),
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
  /** Removes every manual deduction for a financial year (the current one by default). */
  clearTaxEntries: (fy?: string) => data(api.post<{ status: boolean; removed: number }>("/api/review/tax/entries/clear", { fy })),
  setCommitmentTaxSection: (commitment_id: string, tax_section: TaxSectionId | null) =>
    data(api.post<Ok>("/api/review/tax/commitment", { commitment_id, tax_section })),
};
