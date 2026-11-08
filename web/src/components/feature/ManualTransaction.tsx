import { useState } from "react";

import { ingestApi } from "@/services/api";
import type { TxnDirection } from "@/services/types";
import { INGEST_INVALIDATE, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Select } from "@/components/ui/select";
import { toPaise } from "@/utils/money";
import { CATEGORY_IDS, categoryLabel, dateKey } from "@/utils/spending";

const blank = () => ({ date: dateKey(new Date()), direction: "debit" as TxnDirection, amount: "", merchant: "", category: "uncategorized" });

/** For spending that never sent an alert, such as cash paid out of your wallet. */
export function ManualTransaction() {
  const [f, setF] = useState(blank);
  const [error, setError] = useState<string | null>(null);
  const create = usePlannerMutation(ingestApi.createTxn, INGEST_INVALIDATE, { success: "Transaction added" });

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const amount = toPaise(f.amount);
    if (amount === null || amount <= 0) return setError("Enter an amount greater than zero.");
    if (!f.date) return setError("Pick a date.");
    setError(null);
    create.mutate(
      {
        occurred_on: f.date,
        direction: f.direction,
        amount_paise: amount,
        merchant: f.merchant.trim() || null,
        category: f.category,
      },
      { onSuccess: () => setF(blank()) },
    );
  };

  return (
    <details className="group border-t border-border pt-4">
      <summary className="cursor-pointer text-sm font-medium">Add a transaction by hand</summary>
      <form onSubmit={submit} className="form-grid mt-6" noValidate>
        <div className="field">
          <Label htmlFor="tx-date">Date</Label>
          <Input id="tx-date" type="date" value={f.date} onChange={(e) => setF({ ...f, date: e.target.value })} />
        </div>
        <div className="field">
          <Label htmlFor="tx-direction">Money</Label>
          <Select id="tx-direction" value={f.direction} onChange={(e) => setF({ ...f, direction: e.target.value as TxnDirection })}>
            <option value="debit">Went out (spent)</option>
            <option value="credit">Came in</option>
          </Select>
        </div>
        <div className="field">
          <Label htmlFor="tx-amount">Amount</Label>
          <MoneyInput id="tx-amount" value={f.amount} onChange={(e) => setF({ ...f, amount: e.target.value })} />
        </div>
        <div className="field">
          <Label htmlFor="tx-merchant">Merchant (optional)</Label>
          <Input id="tx-merchant" value={f.merchant} onChange={(e) => setF({ ...f, merchant: e.target.value })} placeholder="e.g. Corner chai shop" />
        </div>
        <div className="field col-span-full">
          <Label htmlFor="tx-category">Category</Label>
          <Select id="tx-category" value={f.category} onChange={(e) => setF({ ...f, category: e.target.value })}>
            {CATEGORY_IDS.map((id) => (
              <option key={id} value={id}>
                {categoryLabel(id)}
              </option>
            ))}
          </Select>
        </div>
        {error && <p role="alert" className="col-span-full text-sm text-destructive">{error}</p>}
        <div className="form-actions">
          <Button type="submit" disabled={create.isPending}>
            {create.isPending ? "Adding..." : "Add transaction"}
          </Button>
        </div>
      </form>
    </details>
  );
}
