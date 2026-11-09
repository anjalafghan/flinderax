import type { CardBreakdown } from "@/services/types";
import { daysAgo, formatDay } from "@/utils/dates";
import { formatINR } from "@/utils/money";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

function Row({ title, note, amount, hint }: { title: string; note?: string; amount: number; hint: string }) {
  return (
    <div className="ledger-row">
      <div className="min-w-0">
        <p className="font-medium">{title}</p>
        {note && <p className="text-xs text-muted-foreground">{note}</p>}
        <p className="mt-1 text-xs text-muted-foreground">{hint}</p>
      </div>
      <p className="shrink-0 font-display text-lg tabular-nums">{formatINR(amount)}</p>
    </div>
  );
}

/** Plain-English version of the bank's four numbers. */
export function CardBreakdownPanel({ breakdown: b }: { breakdown: CardBreakdown }) {
  const noDates = !b.due_date && !b.next_due_date;
  return (
    <Card>
      <CardHeader className="pb-4">
        <CardTitle className="text-[32px]">What you owe</CardTitle>
        <p className="text-sm text-muted-foreground">
          {b.last_updated ? `Updated ${daysAgo(b.last_updated) === 0 ? "today" : `${daysAgo(b.last_updated)} days ago`}` : "No bank snapshot yet"}
        </p>
      </CardHeader>
      <CardContent>
        <div className="ledger">
          <Row
            title="Due now"
            note={b.due_date ? `Pay by ${formatDay(b.due_date)}` : undefined}
            amount={b.due_now_paise}
            hint="Your current statement. Pay this to stay clear."
          />
          <Row
            title="Next bill"
            note={b.next_due_date ? `Pay by ${formatDay(b.next_due_date)} (estimate)` : "Estimate"}
            amount={b.next_bill_est_paise}
            hint={
              b.next_statement_date
                ? `Statement on ${formatDay(b.next_statement_date)}. Includes this month's EMI and pending charges.`
                : "Everything spent since the statement plus this month's EMI and pending charges."
            }
          />
          <Row
            title="Loan, not due"
            amount={b.loan_not_due_paise}
            hint="EMI balance that is part of 'outstanding' but isn't billed yet."
          />
        </div>
        {noDates && (
          <p className="mt-4 max-w-prose text-sm text-muted-foreground">
            Add a statement day and due day with “Edit Card” to see payment dates.
          </p>
        )}
      </CardContent>
    </Card>
  );
}
