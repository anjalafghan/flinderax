import { authHeadersFor } from "@/session";

// Writes that must not be lost when there is no signal: a transaction typed on a phone, a new slip,
// a check-in. Each is sent with an Idempotency-Key; if the network fails it waits on the phone with
// that same key and is sent when the signal is back. The server does it once however often it is sent.

const STORE = "joy-outbox-v1";

/** What to tell someone whose write is waiting for a signal. */
export const QUEUED_NOTE =
  "No signal: saved on this phone. It will be sent when you are back online.";

export interface Queued {
  key: string;
  url: string;
  body: unknown;
  label: string;
  /** Session that created this write; legacy unscoped entries are never transmitted. */
  sessionId?: string;
}

export function loadQueue(): Queued[] {
  try {
    return JSON.parse(localStorage.getItem(STORE) ?? "[]") as Queued[];
  } catch {
    return [];
  }
}

function save(q: Queued[]) {
  try {
    localStorage.setItem(STORE, JSON.stringify(q));
  } catch {
    /* storage blocked: the item is lost with the tab, which is no worse than before */
  }
}

export function enqueue(item: Queued) {
  const sessionId = localStorage.getItem("token");
  if (!sessionId) return;
  save([...loadQueue(), { ...item, sessionId }]);
}

const post = (url: string, body: unknown, key: string) =>
  fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": key,
      ...authHeadersFor(localStorage.getItem("token")),
    },
    body: JSON.stringify(body),
  });

/** Sends a POST. If the network is down it is kept for later. A refusal from the server comes back as the response. */
export async function sendOrQueue(
  url: string,
  body: unknown,
  label: string,
): Promise<{ queued: boolean; response?: Response }> {
  const key = crypto.randomUUID();
  try {
    return { queued: false, response: await post(url, body, key) };
  } catch {
    if (!localStorage.getItem("token")) return { queued: false };
    enqueue({ key, url, body, label });
    return { queued: true };
  }
}

/** Sends what is waiting, oldest first. Stops at the first network or server failure; drops (and names) what the server refuses. */
export async function flush(): Promise<{
  sent: number;
  dropped: string[];
  left: number;
}> {
  const sessionId = localStorage.getItem("token");
  const all = loadQueue();
  let queue = sessionId ? all.filter((item) => item.sessionId === sessionId) : [];
  const otherSessions = all.filter((item) => item.sessionId !== sessionId);
  let sent = 0;
  const dropped: string[] = [];
  while (queue.length > 0) {
    const item = queue[0]!;
    let res: Response;
    try {
      res = await post(item.url, item.body, item.key);
    } catch {
      break;
    }
    // An expired login is recoverable: retain the write until this session is authenticated again.
    if (res.status >= 500 || res.status === 401) break;
    if (res.ok) sent += 1;
    else dropped.push(item.label);
    queue = queue.slice(1);
    save([...otherSessions, ...queue]);
  }
  return { sent, dropped, left: queue.length };
}

/** The line to show after a flush, or nothing when there is nothing to say. */
export function describeFlush(r: {
  sent: number;
  dropped: string[];
  left: number;
}): string | null {
  if (r.sent === 0 && r.dropped.length === 0) return null;
  const sent =
    r.sent > 0
      ? `Sent ${r.sent} saved ${r.sent === 1 ? "item" : "items"}.`
      : "";
  const lost =
    r.dropped.length > 0 ? ` Couldn't save: ${r.dropped.join(", ")}.` : "";
  return (sent + lost).trim();
}
