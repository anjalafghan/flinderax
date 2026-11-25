// joy offline support. The app shell (index.html, scripts, styles, fonts) is kept so the app
// opens with no signal; a few read-only screens are kept per user for 7 days, so the last numbers
// still show offline. Nothing is ever written or queued offline: writes just fail and say so.
// Copies are named by a hash of the login token, so one person's copy is never shown to another,
// and logout deletes them all.

const SHELL = "joy-shell-v1";
const DATA_PREFIX = "joy-data-";
const MAX_AGE_MS = 7 * 24 * 60 * 60 * 1000;
const NETWORK_TIMEOUT_MS = 2000;
// Read-only API answers worth keeping for offline viewing.
const KEEP = [
  /^\/api\/dashboard\/(summary|history)$/,
  /^\/api\/bills$/,
  /^\/api\/goals$/,
  /^\/api\/ingest\/(summary|transactions)$/,
  /^\/api\/dates\/(jar|slips|kept|regulars|year(\/\d+){0,2})$/,
  /^\/api\/joy\/summary$/,
];

self.addEventListener("install", (e) => {
  e.waitUntil(
    caches
      .open(SHELL)
      .then((c) => c.addAll(["/", "/index.html"]))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (e) => {
  e.waitUntil(
    caches
      .keys()
      // Older shells, and everything else: the old fx-* names and the old Joy app's joy-static-* caches.
      .then((keys) =>
        Promise.all(
          keys
            .filter((k) => k !== SHELL && !k.startsWith(DATA_PREFIX))
            .map((k) => caches.delete(k)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

// The page posts {type: "logout"} so every saved copy goes with the login.
self.addEventListener("message", (e) => {
  if (e.data && e.data.type === "logout") {
    e.waitUntil(
      caches
        .keys()
        .then((keys) =>
          Promise.all(
            keys
              .filter((k) => k.startsWith(DATA_PREFIX))
              .map((k) => caches.delete(k)),
          ),
        ),
    );
  }
});

async function cacheName(req) {
  // An older login sends a bearer token; a cookie login sends only a harmless session id (the cookie
  // itself is HttpOnly). Either one names this login's saved copies.
  const sid = req.headers.get("x-session-id");
  const auth = req.headers.get("authorization") || (sid ? "sid:" + sid : null);
  if (!auth) return null;
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(auth),
  );
  const hex = [...new Uint8Array(digest)]
    .slice(0, 8)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return DATA_PREFIX + hex;
}

async function userCache(req) {
  const name = await cacheName(req);
  return name && caches.open(name);
}

function withTimeout(p, ms) {
  return Promise.race([
    p,
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error("timeout")), ms),
    ),
  ]);
}

async function keptApi(req) {
  const cache = await userCache(req);
  try {
    const res = await withTimeout(fetch(req), NETWORK_TIMEOUT_MS);
    // The login is no longer valid: its saved copies go too.
    if (res.status === 401 && cache) await caches.delete(await cacheName(req));
    if (cache && res.ok) {
      const body = await res.clone().blob();
      const headers = new Headers(res.headers);
      headers.set("x-joy-saved-at", String(Date.now()));
      await cache.put(req.url, new Response(body, { status: 200, headers }));
    }
    return res;
  } catch (err) {
    const saved = cache && (await cache.match(req.url));
    const at = saved && Number(saved.headers.get("x-joy-saved-at"));
    if (saved && Date.now() - at < MAX_AGE_MS) {
      const headers = new Headers(saved.headers);
      headers.set("x-joy-offline", "1");
      return new Response(await saved.blob(), { status: 200, headers });
    }
    throw err;
  }
}

async function shell(req) {
  const cache = await caches.open(SHELL);
  try {
    const res = await fetch(req);
    if (res.ok && req.method === "GET") cache.put(req, res.clone());
    return res;
  } catch {
    return (await cache.match(req)) || (await cache.match("/index.html"));
  }
}

self.addEventListener("fetch", (e) => {
  const req = e.request;
  const url = new URL(req.url);
  if (req.method !== "GET" || url.origin !== self.location.origin) return;
  if (url.pathname.startsWith("/api/")) {
    if (KEEP.some((r) => r.test(url.pathname))) e.respondWith(keptApi(req));
    return; // everything else on the API goes straight to the network
  }
  if (req.mode === "navigate") {
    e.respondWith(shell(new Request("/index.html")));
    return;
  }
  e.respondWith(shell(req));
});

// Notifications: the server sends a count by kind ("Today: 2 bills."), never names or amounts, so a
// lock screen shows nothing private. Tapping one opens the app on the page it names.
self.addEventListener("push", (e) => {
  let m = {};
  try {
    m = e.data ? e.data.json() : {};
  } catch {
    m = {};
  }
  e.waitUntil(
    self.registration.showNotification(m.title || "joy", {
      body: m.body || "",
      icon: "/icon-192.png",
      badge: "/icon-192.png",
      data: { url: m.url || "/" },
    }),
  );
});

self.addEventListener("notificationclick", (e) => {
  e.notification.close();
  const url = (e.notification.data && e.notification.data.url) || "/";
  e.waitUntil(
    self.clients
      .matchAll({ type: "window", includeUncontrolled: true })
      .then((open) => {
        for (const c of open) {
          if ("focus" in c) {
            if ("navigate" in c) c.navigate(url);
            return c.focus();
          }
        }
        return self.clients.openWindow(url);
      }),
  );
});
