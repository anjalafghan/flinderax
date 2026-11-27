// The connector's tools. Every one is a GET against an existing joy endpoint; the server already
// enforces the privacy rules (sealed surprises answer 404, quiet marks and one partner's answers
// are never returned), so nothing here tries to filter. Money is integer paise in the API.
import { z } from "zod";

export interface ToolDef {
  name: string;
  description: string;
  path: (args: any) => string;
  query?: (args: any) => Record<string, string | number | undefined>;
  input?: z.ZodRawShape;
}

const year = z.number().int().min(2000).max(2100);
const month = z.number().int().min(1).max(12);

export const tools: ToolDef[] = [
  {
    name: "get_brief",
    description: "One compact daily summary: bills and commitments due in seven days, safe-to-spend this month, reward fund score, wedding and emergency fund pace, unread bank-text count, and the next kept date (sealed titles stay hidden). Money is integer paise.",
    path: () => "/api/brief",
  },
  {
    name: "preview_draw",
    description: "Preview one date-jar slip that fits a specified evening without recording a draw, peek, or kept date. Give the evening date and optional budget in integer paise and maximum duration.",
    input: {
      when: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
      budget_paise: z.number().int().min(0).optional(),
      max_duration: z.enum(["short", "half_day", "full_day", "weekend"]).optional(),
    },
    path: () => "/api/dates/preview-draw",
    query: (a) => ({ when: a.when, budget_paise: a.budget_paise, max_duration: a.max_duration }),
  },
  // Money (Flinderax). Amounts are integer paise (100 paise = 1 rupee).
  {
    name: "money_summary",
    description: "Dashboard summary: where the month stands. Amounts in paise.",
    path: () => "/api/dashboard/summary",
  },
  {
    name: "money_insights",
    description: "Spending insights and observations from the dashboard.",
    path: () => "/api/dashboard/insights",
  },
  {
    name: "money_analytics",
    description: "Spending analytics by category and period.",
    path: () => "/api/dashboard/analytics",
  },
  {
    name: "money_forecast",
    description: "Forward-looking cash forecast from the plan.",
    path: () => "/api/plan/forecast",
  },
  {
    name: "money_commitments",
    description: "Recurring commitments (rent, subscriptions, loans).",
    path: () => "/api/plan/commitment/list",
  },
  {
    name: "money_income",
    description: "Income sources in the plan.",
    path: () => "/api/plan/income/list",
  },
  {
    name: "money_bills_upcoming",
    description: "Upcoming bills.",
    path: () => "/api/bills",
  },
  {
    name: "money_year_outflows",
    description: "The year view of large planned outflows.",
    path: () => "/api/year",
  },
  {
    name: "money_goals",
    description: "Savings goals and progress.",
    path: () => "/api/goals",
  },
  {
    name: "money_cards",
    description: "All cards with balances and limits.",
    path: () => "/api/card/get_all_cards",
  },
  {
    name: "money_transactions",
    description: "Recent transactions.",
    path: () => "/api/ingest/transactions",
  },
  {
    name: "money_uncategorized_payees",
    description:
      "Payees with untagged spending, biggest first: how many rows, how much, the last date, a suggested category when joy has one, and a sample narration (long reference numbers masked). Locked rows are left out. Defaults: last 365 days, 40 payees.",
    input: {
      days: z.number().int().min(1).max(3650).optional(),
      limit: z.number().int().min(1).max(200).optional(),
    },
    path: () => "/api/ingest/payees",
    query: (a) => ({ status: "uncategorized", days: a.days, limit: a.limit }),
  },
  {
    name: "money_month_review",
    description: "Month review.",
    path: () => "/api/review/month",
  },
  {
    name: "money_checkin",
    description: "Status of the money check-in.",
    path: () => "/api/checkin",
  },
  {
    name: "money_reconcile",
    description:
      "What did joy miss? For each bank account, compares the change between two observed balances (bank alerts with Avl Bal, statement balances, typed balances) with the transactions, auto-posts and moves joy recorded. status is gap, reconciled or never (fewer than two observed balances). Each gap gives its window, the signed gap (negative = money went out that joy did not see), the expected range, unattributed rows with no account as a hint, and the newest alert date (a long silence usually means the phone forwarder stopped). Amounts in paise.",
    path: () => "/api/ingest/reconcile",
  },
  {
    name: "rewards_fund",
    description:
      "The reward fund: this month's score across money (40%), health, relationships and future (20% each) with each area's points against its learned target and a tip for the weakest one; what past months paid (up to ₹2,000 a month, never more than was saved under the spending allowance); and the wallet balance. Use it to coach which small step would raise this month's payout. Amounts in paise.",
    path: () => "/api/rewards/fund",
  },
  {
    name: "rewards_list",
    description:
      "Treats on the wishlist and what unlocks each (a goal share, a streak, or simply enough in the wallet), with their state and claim history. Amounts in paise.",
    path: () => "/api/rewards",
  },

  {
    name: "money_export_all",
    description:
      "Everything you own as one JSON file: every table of money and people data, plus your couple's Dates data (sealed surprises and your partner's marks excluded). Large; use it for deep, multi-month analysis. Amounts are paise.",
    path: () => "/api/export",
  },
  {
    name: "analyses_list",
    description:
      "Analyses already saved in the app, newest first. Read these before writing a new one so you build on them instead of repeating them.",
    path: () => "/api/analyses",
  },

  // Dates (Joy). Surprises sealed for the signed-in person are never returned.
  {
    name: "dates_coming_up",
    description: "What is coming up: planned dates, bills and occasions.",
    path: () => "/api/coming-up",
  },
  {
    name: "dates_jar",
    description: "The date jar: the current draw state.",
    path: () => "/api/dates/jar",
  },
  {
    name: "dates_slips",
    description: "The slips in the jar (ideas for dates).",
    path: () => "/api/dates/slips",
  },
  {
    name: "dates_packs",
    description: "Available slip packs.",
    path: () => "/api/dates/packs",
  },
  {
    name: "dates_kept",
    description: "Kept and planned dates, and past ones with outcomes.",
    path: () => "/api/dates/kept",
  },
  {
    name: "dates_regulars",
    description: "Regular outings and how often they happen.",
    path: () => "/api/dates/regulars",
  },
  {
    name: "dates_year",
    description: "Dates for a year (defaults to the current year).",
    input: { year: year.optional() },
    path: (a) => (a.year ? `/api/dates/year/${a.year}` : "/api/dates/year"),
  },
  {
    name: "dates_month",
    description: "Dates for one month.",
    input: { year, month },
    path: (a) => `/api/dates/year/${a.year}/${a.month}`,
  },
  {
    name: "dates_summary",
    description: "Couple summary: partner link and shared state.",
    path: () => "/api/joy/summary",
  },
  {
    name: "dates_settle",
    description: "Who paid for what between the two of you, and the balance.",
    path: () => "/api/joy/settle",
  },
];
