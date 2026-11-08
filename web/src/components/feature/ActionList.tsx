import { CheckCircle2, ArrowRightLeft, Info } from "lucide-react";

import type { DashboardSummary, LedgerAction, MoveMode } from "@/services/types";
import { movesApi } from "@/services/api";
import { keys, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const STYLE = {
  urgent: "border-rose-500/40 bg-rose-500/10",
  warn: "border-amber-500/40 bg-amber-500/10",
  info: "border-border bg-muted/30",
} as const;

/** The click that tells the app what you did in your bank. */
function LedgerButtons({ ledger }: { ledger: LedgerAction }) {
  const record = usePlannerMutation(movesApi.record, [keys.moves, keys.accounts], {
    success: "Recorded",
  });
  const send = (mode: MoveMode, amount: number) =>
    record.mutate({
      kind: ledger.kind,
      key: ledger.key,
      amount_paise: amount,
      mode,
      from_account_id: ledger.from_account_id,
      to_account_id: ledger.to_account_id,
    });
  const busy = record.isPending;
  const isCard = ledger.kind === "card_bill";
  return (
    <div className="mt-2 flex flex-wrap gap-2">
      {ledger.amount_paise > 0 && (
        <Button size="sm" disabled={busy} onClick={() => send("moved", ledger.amount_paise)}>
          I moved {formatINR(ledger.amount_paise)}
        </Button>
      )}
      {isCard && (
        <Button size="sm" variant="outline" disabled={busy} onClick={() => send("already_covered", ledger.amount_paise)}>
          Already in the reserve
        </Button>
      )}
      {!isCard && ledger.skippable_paise > 0 && (
        <Button size="sm" variant="outline" disabled={busy} onClick={() => send("skipped", ledger.skippable_paise)}>
          Skip {formatINR(ledger.skippable_paise)} this month
        </Button>
      )}
      {!isCard && (
        <Button
          size="sm"
          variant="ghost"
          disabled={busy}
          onClick={() => send("already_covered", ledger.amount_paise + ledger.skippable_paise)}
        >
          Already done
        </Button>
      )}
    </div>
  );
}

export function ActionList({ actions }: { actions: DashboardSummary["actions"] }) {
  return (
    <section aria-label="Do this" className="space-y-3">
      <h2 className="text-lg font-bold tracking-tight">Do this</h2>
      {actions.length === 0 ? (
        <p className="flex items-center gap-2 rounded-xl border border-dashed p-4 text-sm text-muted-foreground">
          <CheckCircle2 className="h-4 w-4 text-emerald-500" /> Nothing to move. You're sorted.
        </p>
      ) : (
        <ul className="space-y-2">
          {actions.map((a) => (
            <li
              key={a.text}
              data-severity={a.severity}
              className={cn("rounded-xl border p-3 text-sm", STYLE[a.severity])}
            >
              <div className="flex items-start gap-3">
                {a.severity === "info" ? (
                  <Info className="mt-0.5 h-4 w-4 shrink-0" />
                ) : (
                  <ArrowRightLeft className="mt-0.5 h-4 w-4 shrink-0" />
                )}
                <span>{a.text}</span>
              </div>
              {a.ledger && <LedgerButtons ledger={a.ledger} />}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
