import type { Renewal, ScenarioItem, ScenarioKind } from "@/services/types";
import { toPaise } from "@/utils/money";

/** One scenario item as typed into the form: everything is a string until it is validated. */
export interface ScenarioDraft {
  id: string;
  name: string;
  kind: ScenarioKind;
  amount: string;
  start: string;
  tenure: string;
  rate: string;
  fundedFrom: string;
  payoutTo: string;
  renewal: Renewal;
  target: string;
  targetDate: string;
}

const ISO = /^\d{4}-\d{2}-\d{2}$/;

/** Turns a draft into a request item, or `null` while it isn't complete/valid yet. */
export function toScenarioItem(d: ScenarioDraft): ScenarioItem | null {
  const amount = toPaise(d.amount);
  if (amount === null || amount <= 0 || !ISO.test(d.start)) return null;
  const isDeposit = d.kind !== "one_off";
  const tenure = Number(d.tenure);
  if (isDeposit && !(Number.isInteger(tenure) && tenure >= 1 && tenure <= 120)) return null;
  const rate = d.rate.trim() ? Number(d.rate) : 0;
  if (isDeposit && !(rate >= 0 && rate <= 30)) return null;
  const target = d.target.trim() ? toPaise(d.target) : null;
  if (d.target.trim() && (target === null || target <= 0)) return null;
  return {
    name: d.name.trim() || (d.kind === "rd" ? "RD" : d.kind === "fd" ? "FD" : "One-off"),
    kind: d.kind,
    amount_paise: amount,
    start: d.start,
    tenure_months: isDeposit ? tenure : null,
    rate_bps: isDeposit ? Math.round(rate * 100) : null,
    funded_from: d.fundedFrom || null,
    payout_to: isDeposit && d.payoutTo ? d.payoutTo : null,
    renewal: d.kind === "fd" ? d.renewal : null,
    target_paise: target,
    target_date: target !== null && ISO.test(d.targetDate) ? d.targetDate : null,
  };
}

/** The 10th of next month, as `YYYY-MM-DD`: a sensible default start for a new item. */
export function defaultStart(now: Date = new Date()): string {
  const y = now.getFullYear() + (now.getMonth() === 11 ? 1 : 0);
  const m = (now.getMonth() + 1) % 12;
  return `${y}-${String(m + 1).padStart(2, "0")}-10`;
}

export function newDraft(kind: ScenarioKind = "rd", now: Date = new Date()): ScenarioDraft {
  return {
    id: Math.random().toString(36).slice(2),
    name: kind === "rd" ? "Wedding RD" : kind === "fd" ? "New FD" : "One-off spend",
    kind,
    amount: kind === "rd" ? "10000" : "",
    start: defaultStart(now),
    tenure: "12",
    rate: kind === "one_off" ? "" : "7",
    fundedFrom: "",
    payoutTo: "",
    renewal: "payout",
    target: "",
    targetDate: "",
  };
}
