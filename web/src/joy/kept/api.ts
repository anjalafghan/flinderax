// Kept's API calls. They use the shared request helpers in ../api.ts (bearer token, 401 -> /auth).
import { post as postTo, request, upload, type Go } from "../api";
import { budgetBody, costBody, goalBody, lineBody, moneyBody, rateBody, rolloverBody, whenBody, type RolloverChoice } from "./helpers";
import type { HistoryView, PlannedView } from "./types";

const BASE = "/api/dates";

const post = <T = Go>(path: string, body: unknown = {}) => postTo<T>(BASE + path, body);

export const keptKey = ["joy", "kept"] as const;

export const getKept = () => request<HistoryView>("GET", `${BASE}/kept`);

// ---- one kept date ----
export const rate = (id: number, rating: number | null, note: string) => post(`/history/${id}`, rateBody(rating, note));
export const setCost = (id: number, actual: string, updateEstimate: boolean, cabs: string[]) =>
  post(`/history/${id}/cost`, costBody(actual, updateEstimate, cabs));
export const setLine = (id: number, line: string) => post(`/history/${id}/line`, lineBody(line));
export const reveal = (id: number) => post(`/history/${id}/reveal`);
export const deleteDate = (id: number) => post(`/history/${id}/delete`);
export const deletePhoto = (id: number, photoId: number) => post(`/history/${id}/photos/${photoId}/delete`);
/** Signed `ics_url` / `send_url` in the answer are public links: use them as an href as they are. */
export const setWhen = (ids: string, at: string) => post<PlannedView>("/history/when", whenBody(ids, at));

/** Multipart `photo`, over XHR so the bar can move: adds one more photo to the date. Resolves with the server's Go. */
export function uploadPhoto(id: number, file: Blob, onProgress: (loaded: number, total: number) => void): Promise<Go> {
  const fd = new FormData();
  fd.append("photo", file, "photo.jpg");
  return upload<Go | null>(`${BASE}/history/${id}/photo`, fd, {
    onProgress,
    failMessage: (status, text) => (status === 422 ? text || "That photo couldn’t be used. Try another one." : "That didn’t work. Try again?"),
  }).then((go) => go ?? { ok: true, to: "/joy/kept" });
}

// ---- budget ----
export const setBudget = (amount: string) => post("/budget", budgetBody(amount));
export const rollover = (choice: RolloverChoice) => post("/budget/rollover", rolloverBody(choice));
export const addSpend = (amount: string, note: string, on: string) => post("/budget/spend", moneyBody(amount, note, on));
/** A date we had that never was a slip: straight into Kept (its one-off slip never reaches the jar). */
export const addCustomDate = (d: { title: string; place: string; setting: string; on: string; cost: string }) => post("/history/custom", d);
export const deleteSpend = (id: number) => post(`/budget/spend/${id}/delete`);

// ---- savings jar ----
export const setGoal = (id: string) => post("/savings/goal", goalBody(id));
export const addSaving = (amount: string, note: string) => post("/savings/add", moneyBody(amount, note));
export const deleteSaving = (id: number) => post(`/savings/${id}/delete`);
export const keepGoal = () => post("/savings/keep");

// ---- put-backs ----
/** Clears the "drawn and put back" list that rerolling leaves behind. */
export const clearPutbacks = () => post("/putbacks/clear");
