// Every request the Dates section makes. Same bearer-token pattern as services/api.ts, but it keeps
// the HTTP status and parsed body, because a refused slip save is a 422 that carries a FormView.
import type { JoySummary } from "@/services/types";
import { clearSession } from "@/session";

export interface Opt {
  value: string;
  label: string;
  selected: boolean;
}

export interface ActivityView {
  id: number;
  title: string;
  notes: string;
  place: string;
  cost: string;
  duration: string;
  setting: string;
  setting_key: string;
  tags: string[];
  distances: string[];
  maps_url: string | null;
  times_done: number;
  generated: boolean;
  pairs: string[];
  closed_label: string;
  vetoed_by: string;
  door: string;
}

export interface Group {
  key: string;
  label: string;
  items: ActivityView[];
}

export interface WishlistView {
  me_name: string;
  groups: Group[];
  archived: ActivityView[];
  total: number;
}

export interface FormView {
  id: number | null;
  title: string;
  notes: string;
  est_cost: string;
  venue: string;
  area: string;
  location: string;
  tags: string;
  currency: string;
  couple: string;
  error: string;
  settings: Opt[];
  durations: Opt[];
  energies: Opt[];
  people: Opt[];
  pair_opts: Opt[];
  me_name: string;
  closed_opts: Opt[];
  season_from_opts: Opt[];
  season_to_opts: Opt[];
  hours: string;
  hours_today: string;
  best_times: Opt[];
  minutes: string;
  bookings: Opt[];
  book_ahead_days: string;
  covers: Opt[];
  cover_unknown: boolean;
  bring: string;
  station: string;
  station_walk_min: string;
  details_open: boolean;
}

/** The JSON body of POST /slips and POST /slips/{id}. */
export interface SlipBody {
  title: string;
  notes: string;
  est_cost: string;
  setting: string;
  duration: string;
  venue: string;
  area: string;
  location: string;
  tags: string;
  energy: string;
  added_by: string;
  pairs: number[];
  closed: string[];
  season_from: string;
  season_to: string;
  hours: string;
  best_time: string;
  minutes: string;
  booking: string;
  book_ahead_days: string;
  cover: string;
  bring: string;
  station: string;
  station_walk_min: string;
}

export interface Go {
  ok: boolean;
  to: string;
}

export interface SlipPlanned {
  title: string;
  label: string;
  ics_url: string;
  partner: string;
  wa_url: string;
  share_text: string;
  history_id: number;
  to: string;
}

export interface EnvelopeStep {
  at: string;
  minutes: number;
  what: string;
  mine: boolean;
}

export interface EnvelopeView {
  from: string;
  to: string;
  kicker: string;
  title: string;
  when: string;
  place: string;
  maps_url: string | null;
  meta: string;
  notes: string;
  steps: EnvelopeStep[];
  sealed: boolean;
  hours_note: string;
}

export interface CoupleStatus {
  in_couple: boolean;
  me: number | null;
  partner_name: string | null;
  waiting: boolean;
  /** The couple's money symbol. */
  currency: string;
  /** Any money data entered. False for a partner who only uses Dates. */
  uses_money: boolean;
}

export interface Invite {
  code: string;
  expires_in_days: number;
}

export interface Checkin {
  who: number;
  energy: string;
  budget: number | null;
  max_duration: string;
  reach: string;
  vibe: string;
}

export interface MiddleView {
  enabled: boolean;
  mine: Checkin | null;
  partner_in: boolean;
  partner: string;
  energies: Opt[];
  budgets: Opt[];
  times: Opt[];
  reaches: Opt[];
  vibes: Opt[];
  wa_url: string;
  both_in: boolean;
  no_vibe: boolean;
}

export interface MiddleBody {
  energy?: string;
  budget?: string;
  max_duration?: string;
  reach?: string;
  vibe?: string;
  clear?: string;
}

export interface Settlement {
  id: number;
  payer_is_you: boolean;
  amount_paise: number;
  on_day: string;
  note: string;
}

export interface SettleOut {
  you_paid_paise: number;
  partner_paid_paise: number;
  your_share_bps: number;
  balance_paise: number;
  settlements: Settlement[];
}

export class ApiError<T = unknown> extends Error {
  status: number;
  data: T | null;
  constructor(status: number, message: string, data: T | null) {
    super(message);
    this.status = status;
    this.data = data;
  }
}

interface Options {
  /** false for the public envelope: no Authorization header and no 401 redirect. */
  auth?: boolean;
}

/** The bearer header from localStorage. Empty when storage is blocked: the server then answers 401. */
export function authHeaders(): Record<string, string> {
  try {
    const token = localStorage.getItem("token");
    return token ? { Authorization: `Bearer ${token}` } : {};
  } catch {
    return {};
  }
}

/** Sends the visitor to sign in. The promise never settles, because the page is leaving. */
function toLogin<T>(): Promise<T> {
  clearSession();
  window.location.href = "/auth";
  return new Promise<T>(() => {});
}

function parse(text: string): unknown {
  try {
    return text ? JSON.parse(text) : null;
  } catch {
    return null;
  }
}

export async function request<T>(method: "GET" | "POST", url: string, body?: unknown, { auth = true }: Options = {}): Promise<T> {
  const res = await fetch(url, {
    method,
    headers: { "Content-Type": "application/json", ...(auth ? authHeaders() : {}) },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  if (res.status === 401 && auth) return toLogin<T>();
  const text = await res.text().catch(() => "");
  const data = parse(text);
  if (!res.ok) throw new ApiError(res.status, text.trim() || `Request failed with status ${res.status}`, data);
  return data as T;
}

export const get = <T>(url: string, o?: Options) => request<T>("GET", url, undefined, o);
export const post = <T>(url: string, body: unknown = {}) => request<T>("POST", url, body);

export interface UploadOptions {
  /** Bytes sent so far and the total, as XHR reports them. */
  onProgress?: (loaded: number, total: number) => void;
  /** The error text for a refused upload. Defaults to the body, or the status. */
  failMessage?: (status: number, text: string) => string;
}

/** Multipart POST over XHR, because fetch cannot report upload progress. Resolves with the parsed body (null when there is none). */
export function upload<T>(url: string, form: FormData, { onProgress, failMessage }: UploadOptions = {}): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("POST", url);
    for (const [k, v] of Object.entries(authHeaders())) xhr.setRequestHeader(k, v);
    xhr.upload.onprogress = (e) => {
      if (e.lengthComputable) onProgress?.(e.loaded, e.total);
    };
    xhr.onerror = () => reject(new TypeError("network"));
    xhr.onload = () => {
      if (xhr.status === 401) {
        void toLogin<T>();
        return;
      }
      const text = xhr.responseText.trim();
      const data = parse(text);
      if (xhr.status >= 200 && xhr.status < 300) resolve(data as T);
      else reject(new ApiError(xhr.status, failMessage ? failMessage(xhr.status, text) : text || `Request failed with status ${xhr.status}`, data));
    };
    xhr.send(form);
  });
}

/** Query keys, so a page can refresh exactly what it changed. */
export const keys = {
  couple: ["joy", "couple"] as const,
  slips: ["joy", "slips"] as const,
  slipForm: (id: string) => ["joy", "slip-form", id] as const,
  jar: ["joy", "jar"] as const,
  summary: ["joy", "summary"] as const,
  settle: ["joy", "settle"] as const,
  envelope: (token: string) => ["joy", "envelope", token] as const,
};

// ---- couple ----
export const getCouple = () => get<CoupleStatus>("/api/couple");
export const createCouple = () => post<{ status: boolean }>("/api/couple/create");
export const inviteCode = () => post<Invite>("/api/couple/invite");
export const joinCouple = (code: string) => post<{ status: boolean }>("/api/couple/join", { code });
export const leaveCouple = () => post<{ status: boolean }>("/api/couple/leave");

// ---- slips ----
export const getSlips = () => get<WishlistView>("/api/dates/slips");
export const getSlipForm = (id: string | null) => get<FormView>(id ? `/api/dates/slips/${id}` : "/api/dates/slips/new");

/** A save is either accepted (Go) or refused with the form echoed back and `error` set. */
export type SaveResult = { ok: true; go: Go } | { ok: false; form: FormView };

export async function saveSlip(id: string | null, body: SlipBody): Promise<SaveResult> {
  try {
    return { ok: true, go: await post<Go>(id ? `/api/dates/slips/${id}` : "/api/dates/slips", body) };
  } catch (e) {
    if (e instanceof ApiError && e.status === 422 && e.data && typeof (e.data as FormView).error === "string") {
      return { ok: false, form: e.data as FormView };
    }
    throw e;
  }
}

export const archiveSlip = (id: number) => post<Go>(`/api/dates/slips/${id}/archive`);
export const restoreSlip = (id: number) => post<Go>(`/api/dates/slips/${id}/restore`);
export const deleteSlip = (id: number) => post<Go>(`/api/dates/slips/${id}/delete`);
export const unvetoSlip = (id: number) => post<Go>(`/api/dates/slips/${id}/unveto`);
export const planSlip = (id: number, at: string) => post<SlipPlanned>(`/api/dates/slips/${id}/plan`, { at });

// ---- jar (the page here only needs tonight's "meet in the middle" part) ----
export const getJarMiddle = () => get<{ middle: MiddleView }>("/api/dates/jar");
export const checkIn = (body: MiddleBody) => post<Go>("/api/dates/middle", body);

// ---- envelope (public) ----
export const getEnvelope = (token: string) =>
  get<EnvelopeView>(`/api/public/dates/e/${encodeURIComponent(token)}`, { auth: false });

// ---- money ----
export const getSummary = () => get<JoySummary>("/api/joy/summary");
export const setPartnerShare = (partner_paise: number) => post<{ status: boolean }>("/api/joy/partner", { partner_paise });
export const resetPartnerShare = () => post<{ status: boolean }>("/api/joy/partner/reset");
export const getSettle = () => get<SettleOut>("/api/joy/settle");
export const deleteSettlement = (id: number) => post<{ status: boolean }>("/api/joy/settle/delete", { id });
export const clearSettlements = () => post<{ status: boolean; removed: number }>("/api/joy/settle/clear");
export const recordSettlement = (you_paid: boolean, amount_paise: number, note?: string) =>
  post<{ status: boolean }>("/api/joy/settle", { you_paid, amount_paise, note });
