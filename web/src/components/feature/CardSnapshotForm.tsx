import { useState } from "react";

import { cardPlanApi } from "@/services/api";
import { keys, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { InfoTip } from "@/components/ui/info-tip";
import { Label } from "@/components/ui/label";
import { MoneyInput } from "@/components/ui/money-input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { toPaise } from "@/utils/money";

const HELP = {
  total_due:
    "The amount on your latest statement that you must pay by the due date. Paying this in full avoids interest and late fees.",
  outstanding:
    "Everything you owe the bank right now: the statement, everything you've spent since, and the remaining EMI balance.",
  unbilled_spent:
    "Spends since the last statement that haven't been billed yet. Kept for reference only, it is never used in the maths.",
  unbilled_credit:
    "Refunds and payments since the last statement. Kept for reference only, it is never used in the maths.",
} as const;

function Field({
  id,
  label,
  help,
  value,
  onChange,
  optional,
}: {
  id: string;
  label: string;
  help: string;
  value: string;
  onChange: (v: string) => void;
  optional?: boolean;
}) {
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-1.5">
        <Label htmlFor={id}>{label}</Label>
        {optional && <span className="text-xs text-muted-foreground">(optional)</span>}
        <InfoTip text={help} />
      </div>
      <MoneyInput id={id} value={value} onChange={(e) => onChange(e.target.value)} />
    </div>
  );
}

/** "Update from bank app": copy the four numbers the bank shows. */
export function CardSnapshotForm({ cardId }: { cardId: string }) {
  const [totalDue, setTotalDue] = useState("");
  const [outstanding, setOutstanding] = useState("");
  const [unbilledSpent, setUnbilledSpent] = useState("");
  const [unbilledCredit, setUnbilledCredit] = useState("");
  const [error, setError] = useState<string | null>(null);

  const save = usePlannerMutation(
    cardPlanApi.snapshot,
    [keys.breakdown(cardId), ["card", cardId], ["history", cardId], ["cards"]],
    { success: "Card updated from bank app" },
  );

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const due = toPaise(totalDue);
    const out = toPaise(outstanding);
    const spent = unbilledSpent.trim() ? toPaise(unbilledSpent) : null;
    const credit = unbilledCredit.trim() ? toPaise(unbilledCredit) : null;
    if (due === null || out === null) return setError("Enter total due and outstanding as amounts, e.g. 61488.50");
    if (due < 0) return setError("Total due can't be negative");
    if ((unbilledSpent.trim() && spent === null) || (unbilledCredit.trim() && credit === null)) {
      return setError("Unbilled amounts must be plain numbers");
    }
    setError(null);
    save.mutate(
      {
        card_id: cardId,
        total_due_paise: due,
        outstanding_paise: out,
        unbilled_spent_paise: spent,
        unbilled_credit_paise: credit,
      },
      {
        onSuccess: () => {
          setTotalDue("");
          setOutstanding("");
          setUnbilledSpent("");
          setUnbilledCredit("");
        },
      },
    );
  };

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg">Update from bank app</CardTitle>
        <p className="text-sm text-muted-foreground">Copy the numbers exactly as your bank app shows them.</p>
      </CardHeader>
      <CardContent>
        <form onSubmit={submit} className="space-y-4" noValidate>
          <div className="grid gap-4 sm:grid-cols-2">
            <Field id="total-due" label="Total due" help={HELP.total_due} value={totalDue} onChange={setTotalDue} />
            <Field id="outstanding" label="Outstanding" help={HELP.outstanding} value={outstanding} onChange={setOutstanding} />
            <Field id="unbilled-spent" label="Unbilled spent" help={HELP.unbilled_spent} value={unbilledSpent} onChange={setUnbilledSpent} optional />
            <Field id="unbilled-credit" label="Unbilled credit" help={HELP.unbilled_credit} value={unbilledCredit} onChange={setUnbilledCredit} optional />
          </div>
          {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
          <Button type="submit" className="w-full" disabled={save.isPending}>
            {save.isPending ? "Saving..." : "Save snapshot"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
