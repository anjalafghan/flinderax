// What a login leaves on this device, and how it is removed when the login ends.
// Personal data kept in the browser: the token and its expiry, the what-if drafts, and the offline
// copies of API answers that public/sw.js keeps in Cache Storage (one cache per token, "fx-data-<hash>").
// The theme key is a display preference, not personal data, so it stays.

const PERSONAL_KEYS = ["token", "expires_at", "flinderax-whatif-v1"];
const DATA_PREFIX = "fx-data-";

/** Cache name public/sw.js uses for a token: a hash of the Authorization header value. */
export async function offlineCacheName(token: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(`Bearer ${token}`));
  const hex = [...new Uint8Array(digest)].slice(0, 8).map((b) => b.toString(16).padStart(2, "0")).join("");
  return DATA_PREFIX + hex;
}

/** Deletes saved API copies, except the one cache named in `keep`. Never throws. */
async function dropOfflineCopies(keep?: string): Promise<void> {
  try {
    if (typeof caches === "undefined") return;
    const keys = await caches.keys();
    await Promise.all(keys.filter((k) => k.startsWith(DATA_PREFIX) && k !== keep).map((k) => caches.delete(k)));
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
    token = localStorage.getItem("token");
  } catch {
    /* treat as signed out */
  }
  await dropOfflineCopies(token ? await offlineCacheName(token) : undefined);
}
