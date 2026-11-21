// The unrecognised-alert inbox: bank texts joy's regexes could not read sit in `unrecognized_alerts`
// for 30 days. These helpers let Claude act as the fallback parser: read them (masked), post what it
// works out through the normal create route, then dismiss. Pure functions; no IO.

/** Categories the server accepts (`src/alerts.rs` CATEGORIES). */
export const CATEGORIES = [
  "food", "groceries", "shopping", "transport", "fuel", "bills", "entertainment", "health", "travel", "rent",
  "emi", "cash", "dates", "people", "transfer", "income", "investments", "fees", "refund", "other", "uncategorized",
] as const;

const MAX_AMOUNT_PAISE = 100_000_000_000_000n;

/** Card, account, phone and reference numbers: 8+ digits in a row, or a 4-4-4-4 card. Keeps the last 4. */
export function maskDigits(text: string): string {
  return text
    .replace(/\b\d{4}[ -]\d{4}[ -]\d{4}[ -]\d{4}\b/g, (m) => `XX${m.slice(-4)}`)
    .replace(/\d{8,}/g, (m) => `XX${m.slice(-4)}`);
}

/** Rupees as typed ("1,250.50", "₹450") to integer paise, the same rules as `validate::paise_from_text`. No floats. */
export function rupeesToPaise(text: string): number {
  const t = text.trim().replace(/^₹/, "").replace(/[, ]/g, "");
  if (t.startsWith("-")) throw new Error("Amount can't be negative.");
  const [whole = "", frac = "", extra] = t.split(".");
  if (extra !== undefined || (whole === "" && frac === "") || !/^\d*$/.test(whole) || !/^\d*$/.test(frac) || frac.length > 2) {
    throw new Error("That amount didn't look right.");
  }
  const paise = BigInt(whole || "0") * 100n + BigInt(frac.padEnd(2, "0") || "0");
  if (paise <= 0n) throw new Error("Amount must be more than zero.");
  if (paise > MAX_AMOUNT_PAISE) throw new Error("That amount is too large.");
  return Number(paise);
}

export interface NewTxn {
  occurred_on: string;
  direction: "debit" | "credit";
  amount_rupees: string;
  merchant?: string;
  category?: string;
  alert_id?: string;
  occurrence?: number;
}

/**
 * The Idempotency-Key for a transaction. From an alert it is the alert id, so a second run of the inbox
 * never posts twice. Typed by hand it is the fields themselves; two real, identical spends on one day
 * pass occurrence 2, 3...
 */
export async function txnKey(t: NewTxn, paise: number): Promise<string> {
  if (t.alert_id) return `mcp-txn:a:${t.alert_id}`.slice(0, 100);
  const raw = [t.occurred_on, t.direction, paise, (t.merchant ?? "").trim().toLowerCase(), t.occurrence ?? 1].join("|");
  return `mcp-txn:h:${await shortHash(raw)}`;
}

/** First 16 bytes of SHA-256, as hex. */
export async function shortHash(text: string): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)));
  return [...digest.slice(0, 16)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export const SORT_INBOX_PROMPT = [
  "Sort my joy inbox: bank texts the app could not read.",
  "1. Call money_unrecognized. Numbers of 8 digits or more are masked to their last 4; that is enough.",
  "2. For each text decide: a transaction, not a transaction (an OTP, a promo, a balance-only message, a failed or reversed payment), or unsure.",
  "3. For each transaction work out the date (YYYY-MM-DD; the message date, else the day it arrived), debit or credit, the amount in rupees exactly as written, the merchant or payee, and a category.",
  "4. Show me one table of what you propose (the transactions, plus the ones you'd dismiss and why) and wait for my yes. Do not post anything before I answer.",
  "5. After I say yes: call money_add_transaction for each one with its alert_id (it clears the alert from the inbox once the transaction is saved), and money_dismiss_unrecognized for the non-transactions. Leave the unsure ones alone.",
  "6. Finally say how many were added, dismissed and left, and name any bank template that kept failing so the parser can learn it.",
  "Never change or delete existing transactions.",
].join("\n");
