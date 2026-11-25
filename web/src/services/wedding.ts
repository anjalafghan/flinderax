import api from "@/services/api";
import { errorMessageOf } from "@/services/errors";
import { sendOrQueue } from "@/offline";
import { clearSession } from "@/session";

// The budget under a couple goal (the wedding). Money on the wire is integer paise; the `*_input`
// strings are what a form field holds ("1200.50"), made by the server.

export type VerdictStatus = "on_track" | "tight" | "short";

export interface Milestone {
  milestone_id: string;
  item_id: string;
  due_on: string | null;
  amount_paise: number;
  amount_input: string;
  paid: boolean;
  paid_on: string | null;
  overdue: boolean;
}

export interface BudgetItem {
  item_id: string;
  name: string;
  category: string;
  estimate_paise: number;
  estimate_input: string;
  quote_paise: number | null;
  quote_input: string;
  vendor: string;
  notes: string;
  planned_paise: number;
  paid_paise: number;
  remaining_paise: number;
  milestones: Milestone[];
}

export interface CategoryTotal {
  category: string;
  items: number;
  planned_paise: number;
  committed_paise: number;
  estimated_paise: number;
  paid_paise: number;
  remaining_paise: number;
}

export interface Budget {
  goal_id: string;
  name: string;
  target_paise: number;
  target_date: string | null;
  saved_paise: number;
  on_hand_paise: number;
  expected_monthly_paise: number;
  categories: string[];
  items: BudgetItem[];
  summary: {
    categories: CategoryTotal[];
    planned_paise: number;
    committed_paise: number;
    estimated_paise: number;
    paid_paise: number;
    remaining_paise: number;
    scheduled_paise: number;
    unscheduled_paise: number;
    overdue_paise: number;
  };
  verdict: {
    status: VerdictStatus;
    gap_paise: number;
    by: string | null;
    line: string;
  };
  next_due: {
    milestone_id: string;
    item_name: string;
    due_on: string;
    amount_paise: number;
    days_away: number;
  } | null;
}

export interface ItemInput {
  name: string;
  category: string;
  estimate_paise: number;
  quote_paise: number | null;
  vendor: string;
  notes: string;
}

export const budgetKey = (goalId: string) =>
  ["wedding-budget", goalId] as const;

const base = (goalId: string) => `/api/couple-goals/${goalId}`;
type Ok = { status: boolean };

export const weddingApi = {
  get: (goalId: string) =>
    api.get<Budget>(`${base(goalId)}/budget`).then((r) => r.data),
  addItem: (goalId: string, i: ItemInput) =>
    api.post<Ok>(`${base(goalId)}/items`, i).then((r) => r.data),
  updateItem: (goalId: string, itemId: string, i: ItemInput) =>
    api.post<Ok>(`${base(goalId)}/items/${itemId}`, i).then((r) => r.data),
  removeItem: (goalId: string, itemId: string) =>
    api
      .post<Ok>(`${base(goalId)}/items/${itemId}/delete`, {})
      .then((r) => r.data),
  addMilestone: (
    goalId: string,
    itemId: string,
    m: { due_on: string | null; amount_paise: number },
  ) =>
    api
      .post<Ok>(`${base(goalId)}/items/${itemId}/milestones`, m)
      .then((r) => r.data),
  removeMilestone: (goalId: string, milestoneId: string) =>
    api
      .post<Ok>(`${base(goalId)}/milestones/${milestoneId}/delete`, {})
      .then((r) => r.data),
  unpay: (goalId: string, milestoneId: string) =>
    api
      .post<Ok>(`${base(goalId)}/milestones/${milestoneId}/unpay`, {})
      .then((r) => r.data),
  /** Marks a payment made. Works with no signal: kept on the phone and sent later (`queued`), done once however often it is sent. */
  pay: async (
    goalId: string,
    milestoneId: string,
    label: string,
  ): Promise<{ queued: boolean }> => {
    const r = await sendOrQueue(
      `${base(goalId)}/milestones/${milestoneId}/pay`,
      { idempotency_key: crypto.randomUUID() },
      label,
    );
    if (r.queued || !r.response) return { queued: true };
    if (r.response.status === 401) {
      clearSession();
      window.location.href = "/auth";
      throw new Error("Please sign in again.");
    }
    if (!r.response.ok)
      throw new Error(
        errorMessageOf(
          await r.response.text().catch(() => ""),
          r.response.status,
        ),
      );
    return { queued: false };
  },
};
