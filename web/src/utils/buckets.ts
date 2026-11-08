import type { Bucket } from "@/services/types";

// Mirrors `default_bucket` in src/planner/mod.rs: the bucket a category lands in unless the
// user picks another one.
export function defaultBucket(category: string): Bucket {
  if (["sip", "rd", "savings", "investment"].includes(category)) return "savings";
  if (["subscription", "donation", "entertainment"].includes(category)) return "wants";
  return "needs";
}

export const BUCKET_LABEL: Record<Bucket, string> = {
  needs: "Needs",
  savings: "Savings & investing",
  wants: "Spending",
};

export const BUCKET_HELP: Record<Bucket, string> = {
  needs: "Fixed costs you can't skip: rent, bills, phone, internet, gym, food and travel.",
  savings: "Money you pay yourself first: SIPs, RDs, emergency fund.",
  wants: "Guilt-free spending: eating out, hobbies, subscriptions.",
};
