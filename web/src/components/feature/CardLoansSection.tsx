import { useState } from "react";
import { Trash2, Check, Plus } from "lucide-react";

import { cardPlanApi } from "@/services/api";
import { keys, usePlannerMutation } from "@/hooks/usePlanner";
import type { CardBreakdown } from "@/services/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatINR, toPaise } from "@/utils/money";

/** EMIs and not-yet-posted charges for one card: add, remove, mark as posted. */
export function CardLoansSection({ cardId, breakdown }: { cardId: string; breakdown: CardBreakdown }) {
  const refresh = [keys.breakdown(cardId)];
  const addEmi = usePlannerMutation(cardPlanApi.createEmi, refresh, { success: "EMI added" });
  const removeEmi = usePlannerMutation(cardPlanApi.removeEmi, refresh, { success: "EMI removed" });
  const addPending = usePlannerMutation(cardPlanApi.createPending, refresh, { success: "Charge added" });
  const clearPending = usePlannerMutation(cardPlanApi.clearPending, refresh, { success: "Marked as posted" });

  const [emi, setEmi] = useState({ label: "", monthly: "", remaining: "", months: "" });
  const [charge, setCharge] = useState({ label: "", amount: "" });
  const [error, setError] = useState<string | null>(null);

  const submitEmi = (e: React.FormEvent) => {
    e.preventDefault();
    const monthly = toPaise(emi.monthly);
    const remaining = toPaise(emi.remaining);
    const months = Number(emi.months);
    if (!emi.label.trim() || monthly === null || remaining === null || !Number.isInteger(months) || months < 0) {
      return setError("Fill in the EMI name, monthly amount, remaining principal and months left.");
    }
    setError(null);
    addEmi.mutate(
      { card_id: cardId, label: emi.label.trim(), monthly_paise: monthly, remaining_principal_paise: remaining, months_left: months },
      { onSuccess: () => setEmi({ label: "", monthly: "", remaining: "", months: "" }) },
    );
  };

  const submitCharge = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(charge.amount);
    if (!charge.label.trim() || amount === null || amount <= 0) {
      return setError("Give the pending charge a name and a positive amount.");
    }
    setError(null);
    addPending.mutate(
      { card_id: cardId, label: charge.label.trim(), amount_paise: amount },
      { onSuccess: () => setCharge({ label: "", amount: "" }) },
    );
  };

  return (
    <div className="grid gap-6 md:grid-cols-2">
      <Card>
        <CardHeader className="pb-4">
          <CardTitle className="text-[24px]">EMIs</CardTitle>
          <p className="max-w-prose text-sm text-muted-foreground">Enter once. Months left and balance count down by themselves.</p>
        </CardHeader>
        <CardContent className="space-y-6">
          <ul className="ledger">
            {breakdown.emis.map((e) => (
              <li key={e.emi_id} className="ledger-row text-sm">
                <div className="min-w-0">
                  <p className="truncate font-medium">{e.label}</p>
                  <p className="text-xs text-muted-foreground">
                    {e.months_left === 0
                      ? `${formatINR(e.monthly_paise)}/month, finished (no longer counted). Remove it any time.`
                      : `${formatINR(e.monthly_paise)}/month, ${e.months_left} left, about ${formatINR(e.remaining_principal_paise)} remaining`}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-9 w-9 shrink-0"
                  aria-label={`Remove ${e.label}`}
                  onClick={() => window.confirm(`Remove EMI "${e.label}"?`) && removeEmi.mutate(e.emi_id)}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </li>
            ))}
          </ul>
          {breakdown.emis.length === 0 && <p className="empty-note">No EMIs on this card.</p>}
          <form onSubmit={submitEmi} className="form-grid border-t border-border pt-6" noValidate>
            <div className="field col-span-full">
              <Label htmlFor="emi-label">EMI name</Label>
              <Input id="emi-label" value={emi.label} onChange={(e) => setEmi({ ...emi, label: e.target.value })} placeholder="e.g. AC EMI" />
            </div>
            <div className="field">
              <Label htmlFor="emi-monthly">Monthly</Label>
              <MoneyInput id="emi-monthly" value={emi.monthly} onChange={(e) => setEmi({ ...emi, monthly: e.target.value })} />
            </div>
            <div className="field">
              <Label htmlFor="emi-months">Months left</Label>
              <Input id="emi-months" inputMode="numeric" value={emi.months} onChange={(e) => setEmi({ ...emi, months: e.target.value })} />
            </div>
            <div className="field col-span-full">
              <Label htmlFor="emi-remaining">Remaining principal</Label>
              <MoneyInput id="emi-remaining" value={emi.remaining} onChange={(e) => setEmi({ ...emi, remaining: e.target.value })} />
            </div>
            <div className="form-actions">
              <Button type="submit" disabled={addEmi.isPending}>
                <Plus className="mr-2 h-4 w-4" /> Add EMI
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-4">
          <CardTitle className="text-[24px]">Pending charges</CardTitle>
          <p className="max-w-prose text-sm text-muted-foreground">Spends that haven't posted to the card yet. They drop off by themselves after 30 days.</p>
        </CardHeader>
        <CardContent className="space-y-6">
          <ul className="ledger">
            {breakdown.pending_charges.map((c) => (
              <li key={c.charge_id} className="ledger-row text-sm">
                <div className="min-w-0">
                  <p className="truncate font-medium">{c.label}</p>
                  <p className="text-xs text-muted-foreground">{formatINR(c.amount_paise)}</p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  className="shrink-0"
                  onClick={() => clearPending.mutate(c.charge_id)}
                  title="It has posted, or it was cancelled"
                >
                  <Check className="mr-1 h-4 w-4" /> Posted
                </Button>
              </li>
            ))}
          </ul>
          {breakdown.pending_charges.length === 0 && <p className="empty-note">Nothing pending.</p>}
          <form onSubmit={submitCharge} className="form-grid border-t border-border pt-6" noValidate>
            <div className="field">
              <Label htmlFor="pc-label">Charge</Label>
              <Input id="pc-label" value={charge.label} onChange={(e) => setCharge({ ...charge, label: e.target.value })} placeholder="e.g. Zomato" />
            </div>
            <div className="field">
              <Label htmlFor="pc-amount">Amount</Label>
              <MoneyInput id="pc-amount" value={charge.amount} onChange={(e) => setCharge({ ...charge, amount: e.target.value })} />
            </div>
            <div className="form-actions">
              <Button type="submit" disabled={addPending.isPending}>
                <Plus className="mr-2 h-4 w-4" /> Add pending charge
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
      {error && <p role="alert" className="text-sm text-destructive md:col-span-2">{error}</p>}
    </div>
  );
}
