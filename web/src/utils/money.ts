// Money on the wire is integer paise (`*_paise`). Convert only at the edges: parse what the
// user typed with `toPaise`, show it with `formatINR`.

const fmt = (digits: number) =>
  new Intl.NumberFormat("en-IN", {
    style: "currency",
    currency: "INR",
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
const whole = fmt(0);
const withPaise = fmt(2);

/** 1234500 -> "₹12,345", 150 -> "₹1.50" (paise shown only when non-zero). */
export function formatINR(paise: number): string {
  return (paise % 100 === 0 ? whole : withPaise).format(paise / 100);
}

/**
 * "1,234.5" / "₹1234" / "-50" -> integer paise, or `null` when the text is not a plain amount
 * (empty, letters, more than two decimals). Works on the digits, never on a float.
 */
export function toPaise(rupees: string): number | null {
  const cleaned = rupees.replace(/[₹,\s]/g, "");
  const m = /^(-?)(\d*)(?:\.(\d{1,2}))?$/.exec(cleaned);
  if (!m || (m[2] === "" && m[3] === undefined)) return null;
  const rupeesPart = Number(m[2] || "0");
  const frac = Number((m[3] ?? "").padEnd(2, "0"));
  const paise = rupeesPart * 100 + frac;
  return m[1] === "-" ? -paise : paise;
}

/** 123450 -> "1234.5", for pre-filling an input. */
export function paiseToInput(paise: number): string {
  const sign = paise < 0 ? "-" : "";
  const abs = Math.abs(paise);
  const rupeesPart = Math.floor(abs / 100);
  const frac = abs % 100;
  return frac === 0 ? `${sign}${rupeesPart}` : `${sign}${rupeesPart}.${String(frac).padStart(2, "0").replace(/0$/, "")}`;
}

/** 4_520_000 -> "₹45.2k", 12_345_600 -> "₹1.2L", 150_000_000 -> "₹15L", for chart axes. */
export function formatCompactINR(paise: number): string {
  const sign = paise < 0 ? "-" : "";
  const r = Math.abs(paise) / 100;
  const trim = (n: number) => String(Math.round(n * 10) / 10);
  if (r >= 10_000_000) return `${sign}₹${trim(r / 10_000_000)}Cr`;
  if (r >= 100_000) return `${sign}₹${trim(r / 100_000)}L`;
  if (r >= 1_000) return `${sign}₹${trim(r / 1_000)}k`;
  return `${sign}₹${Math.round(r)}`;
}

export type AmountOrPercent =
  | { kind: "empty" }
  | { kind: "amount"; paise: number }
  | { kind: "percent"; bps: number }
  | { kind: "invalid" };

/**
 * "18875" / "₹18,875" -> an amount; "25%" / "22.5 %" -> a percentage in basis points (2500 = 25%);
 * "" -> empty. Anything else (or a percentage over 100) is invalid.
 */
export function parseAmountOrPercent(text: string): AmountOrPercent {
  const t = text.trim();
  if (t === "") return { kind: "empty" };
  if (t.endsWith("%")) {
    const num = t.slice(0, -1).trim();
    if (!/^\d{1,3}(\.\d{1,2})?$/.test(num)) return { kind: "invalid" };
    const bps = Math.round(Number(num) * 100);
    return bps >= 0 && bps <= 10_000 ? { kind: "percent", bps } : { kind: "invalid" };
  }
  const paise = toPaise(t);
  return paise === null || paise < 0 ? { kind: "invalid" } : { kind: "amount", paise };
}

/** 2500 -> "25%", 2250 -> "22.5%". */
export function formatPercent(bps: number): string {
  return `${bps / 100}%`;
}
