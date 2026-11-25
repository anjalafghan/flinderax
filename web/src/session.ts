// What a login leaves on this device, and how it is removed when the login ends.
import { storageGet } from "@/utils/storage";
// Personal data kept in the browser: the token and its expiry, the what-if drafts, and the offline
// copies of API answers that public/sw.js keeps in Cache Storage (one cache per token, "joy-data-<hash>").
// The theme key is a display preference, not personal data, so it stays.

const PERSONAL_KEYS = [
  "token",
  "expires_at",
  "joy-whatif-v1",
  "flinderax-whatif-v1",
];
const DATA_PREFIX = "joy-data-";
// Copies saved before the rename; logout and the start-up sweep remove them too.
const OLD_DATA_PREFIX = "fx-data-";
const isDataCache = (k: string) =>
  k.startsWith(DATA_PREFIX) || k.startsWith(OLD_DATA_PREFIX);

// A web login lives in an HttpOnly cookie that scripts cannot read. What the page keeps under
// "token" is only an id for this login ("s_…"): it names the saved offline copies and tells the
// page it is signed in. Older logins still hold a bearer token there and keep working until they end.
const SESSION_PREFIX = "s_";

export function newSessionId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(18));
  return (
    SESSION_PREFIX +
    [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("")
  );
}

export const isCookieSession = (token: string | null): token is string =>
  !!token && token.startsWith(SESSION_PREFIX);

/** What to remember after a sign-in answer: a cookie login keeps only an id (the token is in the
 *  HttpOnly cookie and is not kept anywhere scripts can read); an older server answer keeps the token. */
export function loginToKeep(res: {
  access_token: string;
  expires_at: number;
  cookie?: boolean;
}): { token: string; expiresAt: number } {
  return {
    token: res.cookie ? newSessionId() : res.access_token,
    expiresAt: res.expires_at,
  };
}

/** Headers for an API request: the session id and the marker the server requires on changes (cookie
 *  login), or the bearer header (older login), or nothing when signed out. */
export function authHeadersFor(token: string | null): Record<string, string> {
  if (!token) return {};
  return isCookieSession(token)
    ? { "X-Session-Id": token, "X-Joy": "1" }
    : { Authorization: `Bearer ${token}` };
}

/** Cache name public/sw.js uses for a login: a hash of the Authorization header, or "sid:" + the session id. */
export async function offlineCacheName(token: string): Promise<string> {
  const key = isCookieSession(token) ? `sid:${token}` : `Bearer ${token}`;
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(key),
  );
  const hex = [...new Uint8Array(digest)]
    .slice(0, 8)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return DATA_PREFIX + hex;
}

/** Deletes saved API copies, except the one cache named in `keep`. Never throws. */
async function dropOfflineCopies(keep?: string): Promise<void> {
  try {
    if (typeof caches === "undefined") return;
    const keys = await caches.keys();
    await Promise.all(
      keys
        .filter((k) => isDataCache(k) && k !== keep)
        .map((k) => caches.delete(k)),
    );
  } catch {
    /* storage blocked: the service worker's own purge still runs */
  }
}

/**
 * Ends the session on this device, whatever the reason (logout, expiry, a 401): removes the personal
 * localStorage keys and every saved API copy. Safe to call right before a redirect.
 */
export function clearSession(): void {
  for (const key of PERSONAL_KEYS) {
    try {
      localStorage.removeItem(key);
    } catch {
      /* ignore */
    }
  }
  try {
    navigator.serviceWorker?.controller?.postMessage({ type: "logout" });
  } catch {
    /* ignore */
  }
  void dropOfflineCopies();
}

/** At startup: keeps only the current token's saved copies (none when signed out), drops older ones. */
export async function sweepOfflineCopies(): Promise<void> {
  let token: string | null = null;
  try {
    token = storageGet("token");
  } catch {
    /* treat as signed out */
  }
  await dropOfflineCopies(token ? await offlineCacheName(token) : undefined);
}
