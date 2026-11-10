// Rupee helpers and the settle-up wording. Pure.

/** 150000 paise -> "₹1,500". */
export function rupees(paise: number, symbol = "₹"): string {
  return `${symbol}${Math.round(Math.abs(paise) / 100).toLocaleString("en-IN")}`;
}

/** "1,500", "₹1500" or "1500.50" -> paise, or null when it isn't a positive-or-zero amount. */
export function toPaise(input: string): number | null {
  const clean = input.replace(/[₹,\s]/g, "");
  if (!/^\d+(\.\d{1,2})?$/.test(clean)) return null;
  return Math.round(parseFloat(clean) * 100);
}

export type Standing = { kind: "owed" | "owe" | "square"; paise: number; text: string };

/** balance_paise > 0: your partner owes you. < 0: you owe them. */
export function standing(balancePaise: number, symbol = "₹"): Standing {
  if (balancePaise > 0) return { kind: "owed", paise: balancePaise, text: `Partner owes you ${rupees(balancePaise, symbol)}` };
  if (balancePaise < 0) return { kind: "owe", paise: -balancePaise, text: `You owe ${rupees(balancePaise, symbol)}` };
  return { kind: "square", paise: 0, text: "All square" };
}
