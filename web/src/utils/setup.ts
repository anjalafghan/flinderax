import type { DashboardSummary } from "@/services/types";

/**
 * A brand-new account has no income, no account and no spending, so "A bit over plan" and a red ₹0 would
 * be a verdict on nothing. Until there is something to judge, Today shows this instead.
 */
export function isBlankSummary(s: DashboardSummary): boolean {
  return !s.next_income && !s.spending.tracked && s.pots.spending.account_name === null && s.pots.locked.count === 0;
}
