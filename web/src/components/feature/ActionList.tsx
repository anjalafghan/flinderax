import { CheckCircle2 } from "lucide-react";

import type { DashboardSummary, LedgerAction, MoveMode } from "@/services/types";
import { movesApi } from "@/services/api";
import { keys, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { formatINR } from "@/utils/money";
import { cn } from "@/utils/cn";

const STYLE = {
  urgent: "[--dot:var(--destructive)]",
  warn: "[--dot:var(--glow-a)]",
  info: "[--dot:var(--muted-foreground)]",
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
      <div className="flex items-baseline justify-between gap-4">
        <h2 className="font-display text-4xl leading-none md:text-[44px]">Do this</h2>
        <span className="eyebrow">{actions.length > 1 ? `${actions.length} things, now` : "one thing, now"}</span>
      </div>
      {actions.length === 0 ? (
        <p className="flex items-center gap-2 rounded-3xl border border-dashed border-primary/25 bg-card/30 p-4 text-sm text-muted-foreground">
          <CheckCircle2 className="h-4 w-4 text-primary" /> Nothing to move. You're sorted.
        </p>
      ) : (
        <ul className="space-y-2">
          {actions.map((a) => (
            <li
              key={a.text}
              data-severity={a.severity}
              className={cn("glass rounded-[1.25rem] px-5 py-4 text-[15px] md:px-7", STYLE[a.severity])}
            >
              <div className="flex items-start gap-4">
                <span aria-hidden className="mt-2 h-2.5 w-2.5 shrink-0 rounded-full" style={{ background: "hsl(var(--dot))", boxShadow: "0 0 16px hsl(var(--dot))" }} />
                <span className="text-base">{a.text}</span>
              </div>
              {a.ledger && <LedgerButtons ledger={a.ledger} />}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
