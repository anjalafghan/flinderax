// Regulars, nights and photos. Photo links in these views are signed public URLs: use them in <img src> as-is.
import type { Go } from "../api";
import { get, post, upload } from "./http";
import type { NewRegularBody, WentBody } from "./days";

const BASE = "/api/dates";

export interface ListItem {
  id: number;
  title: string;
  place: string;
  days: string;
  usual: string;
  count: number;
}

export interface DayBox {
  value: number;
  label: string;
  checked: boolean;
}

export interface StoppedItem {
  id: number;
  title: string;
}

export interface RegularsView {
  me_name: string;
  items: ListItem[];
  /** Regulars you stopped tracking, so any can come back. */
  stopped: StoppedItem[];
  days: DayBox[];
}

export interface Stamp {
  id: number;
  day: string;
  year: string;
  elsewhere: string;
  amount: string;
  photo: string;
  tilt: number;
}

export interface SpreadView {
  me_name: string;
  regular_id: number;
  title: string;
  place: string;
  maps_url: string;
  days: string;
  count: number;
  elsewhere: number;
  total: string;
  average: string;
  /** Weekday numbers (0 = Sunday), for the edit form. */
  day_values: number[];
  /** The usual amount as digits. */
  usual_raw: number;
  stamps: Stamp[];
}

/** The jar page's "Tonight?" card, for any page that wants to show it. */
export interface OpenNight {
  regular_id: number;
  title: string;
  maps_url: string;
  usual: string;
  usual_raw: number;
  day: string;
  label: string;
  today: boolean;
}

export interface WentTonight {
  visit_id: number;
  title: string;
  place: string;
  amount: string;
  photo: string;
}

export interface RegularCardData {
  open: OpenNight[];
  went: WentTonight[];
  ceiling: string;
  show: boolean;
}

export interface PhotoSaved {
  ok: boolean;
  photo: string;
}

export const regKeys = {
  list: ["joy", "regulars"] as const,
  spread: (id: string) => ["joy", "regular", id] as const,
};

export const getRegulars = () => get<RegularsView>(`${BASE}/regulars`);
export const createRegular = (body: NewRegularBody) => post<Go>(`${BASE}/regulars`, body);
export const getSpread = (id: string) => get<SpreadView>(`${BASE}/regulars/${encodeURIComponent(id)}`);
export const updateRegular = (id: number | string, body: NewRegularBody) => post<Go>(`${BASE}/regulars/${encodeURIComponent(String(id))}`, body);
/** Stop tracking: the server archives it, so the nights already stamped stay in the year. */
export const stopTracking = (id: number | string) => post<Go>(`${BASE}/regulars/${encodeURIComponent(String(id))}/delete`, {});
export const restoreRegular = (id: number | string) => post<Go>(`${BASE}/regulars/${encodeURIComponent(String(id))}/restore`, {});
export const clearSkips = (id: number | string) => post<Go>(`${BASE}/regulars/${encodeURIComponent(String(id))}/skips/clear`, {});
export const wentTo = (id: number, body: WentBody) => post<Go>(`${BASE}/regulars/${id}/went`, body);
export const skipTo = (id: number, body: { day: string; back: string }) => post<Go>(`${BASE}/regulars/${id}/skip`, body);
export const deleteNight = (visitId: number, back = "") => post<Go>(`${BASE}/nights/${visitId}/delete`, { back });
export const deletePhoto = (visitId: number, back = "") => post<Go>(`${BASE}/nights/${visitId}/photo/delete`, { back });
export const uploadPhoto = (visitId: number, jpeg: Blob, onProgress?: (f: number) => void) => {
  const form = new FormData();
  form.append("photo", jpeg, "night.jpg");
  return upload<PhotoSaved>(`${BASE}/nights/${visitId}/photo`, form, onProgress);
};
