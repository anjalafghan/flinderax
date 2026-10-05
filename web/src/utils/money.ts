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
