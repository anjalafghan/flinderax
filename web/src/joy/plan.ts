// "Plan it for a day": the quick picks and the datetime-local value. `now` is a parameter so the
// picks can be tested.

export type WhenKey = "tonight" | "tomorrow" | "saturday";

const pad = (n: number) => String(n).padStart(2, "0");

/** A Date as the value of <input type="datetime-local">: "2026-11-02T19:00". */
export function toLocalInput(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function at(now: Date, days: number, h: number, m = 0): Date {
  const d = new Date(now);
  d.setDate(d.getDate() + days);
  d.setHours(h, m, 0, 0);
  return d;
}

export function quickPick(key: WhenKey, now: Date = new Date()): Date {
  switch (key) {
    case "tonight": {
      // 7 pm, or the next half hour after an hour from now once it's evening already
      if (now.getHours() < 18) return at(now, 0, 19);
      const d = new Date(now.getTime() + 60 * 60 * 1000);
      d.setMinutes(d.getMinutes() <= 30 ? 30 : 60, 0, 0);
      return d;
    }
    case "tomorrow":
      return at(now, 1, 19);
    case "saturday": {
      const n = now.getDay();
      return at(now, n === 6 ? 7 : 6 - n, 11);
    }
  }
}
