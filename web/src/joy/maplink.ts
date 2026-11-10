// Pull coordinates out of a pasted Google Maps link. Pure; the browser can't expand short links.

export type MapLink =
  | { kind: "coords"; location: string }
  | { kind: "short" } // maps.app.goo.gl / goo.gl/maps: needs the full address from the address bar
  | { kind: "none" };

const NUM = "-?\\d{1,3}(?:\\.\\d+)?";

function valid(lat: number, lng: number): boolean {
  return Number.isFinite(lat) && Number.isFinite(lng) && Math.abs(lat) <= 90 && Math.abs(lng) <= 180;
}

function pair(lat: string, lng: string): MapLink {
  const a = Number(lat);
  const b = Number(lng);
  return valid(a, b) ? { kind: "coords", location: `${lat}, ${lng}` } : { kind: "none" };
}

/** `/@19.13,72.81,17z`, `?q=19.13,72.81` (also `ll=`, `query=`, `destination=`) or `!3d19.13!4d72.81`. */
export function parseMapLink(input: string): MapLink {
  const raw = input.trim();
  if (!raw) return { kind: "none" };
  let text = raw;
  try {
    text = decodeURIComponent(raw);
  } catch {
    // keep the raw text
  }
  // A pin's own coordinates (!3d!4d) are more exact than the map centre (@).
  const pin = new RegExp(`!3d(${NUM})!4d(${NUM})`).exec(text);
  if (pin) return pair(pin[1] as string, pin[2] as string);
  const at = new RegExp(`@(${NUM}),\\s*(${NUM})`).exec(text);
  if (at) return pair(at[1] as string, at[2] as string);
  const q = new RegExp(`[?&](?:q|ll|query|destination|center)=\\s*(${NUM})\\s*,\\s*(${NUM})(?:[&#]|$)`).exec(text);
  if (q) return pair(q[1] as string, q[2] as string);
  if (/(?:^|\/\/)(?:maps\.app\.goo\.gl|goo\.gl\/maps)/i.test(text)) return { kind: "short" };
  return { kind: "none" };
}
