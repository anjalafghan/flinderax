// Which kept dates' keepsakes this browser has already watched arrive (so each plays by itself once).
// Only history ids, in this browser's own storage; nothing personal.
const KEY = "joy.keepsakes.seen";

function read(): Set<number> {
  try {
    return new Set(
      (JSON.parse(localStorage.getItem(KEY) ?? "[]") as number[]).filter((n) =>
        Number.isInteger(n),
      ),
    );
  } catch {
    return new Set();
  }
}

export const isSeen = (historyId: number) => read().has(historyId);

export function markSeen(historyId: number) {
  try {
    const s = read();
    s.add(historyId);
    localStorage.setItem(KEY, JSON.stringify([...s].slice(-2000)));
  } catch {
    /* private mode: it just plays again next time */
  }
}
