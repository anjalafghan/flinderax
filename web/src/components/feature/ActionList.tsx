import { CheckCircle2 } from "lucide-react";

import type {
  DashboardSummary,
  LedgerAction,
  MoveMode,
} from "@/services/types";
import { movesApi } from "@/services/api";
import { keys, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { formatINR } from "@/utils/money";

/** The click that tells the app what you did in your bank. */
function LedgerButtons({ ledger }: { ledger: LedgerAction }) {
  const record = usePlannerMutation(
    movesApi.record,
    [keys.moves, keys.accounts],
    {
      success: "Recorded",
    },
  );
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
    <div className="mt-3 flex flex-wrap gap-2 pl-[26px]">
      {ledger.amount_paise > 0 && (
        <Button
          size="sm"
          disabled={busy}
          onClick={() => send("moved", ledger.amount_paise)}
        >
          I moved {formatINR(ledger.amount_paise)}
        </Button>
      )}
      {isCard && (
        <Button
          size="sm"
          variant="outline"
          disabled={busy}
          onClick={() => send("already_covered", ledger.amount_paise)}
        >
          Already in the reserve
        </Button>
      )}
      {!isCard && ledger.skippable_paise > 0 && (
        <Button
          size="sm"
          variant="outline"
          disabled={busy}
          onClick={() => send("skipped", ledger.skippable_paise)}
        >
          Skip {formatINR(ledger.skippable_paise)} this month
        </Button>
      )}
      {!isCard && (
        <Button
          size="sm"
          variant="ghost"
          disabled={busy}
          onClick={() =>
            send(
              "already_covered",
              ledger.amount_paise + ledger.skippable_paise,
            )
          }
        >
          Already done
        </Button>
      )}
    </div>
  );
}

export function ActionList({
  actions,
}: {
  actions: DashboardSummary["actions"];
}) {
  return (
    <section aria-label="Do this" className="space-y-4">
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-2">
        <h2 className="brand-text font-display text-[32px] leading-none">
          Do this
        </h2>
        <span className="pill">
          {actions.length > 1
            ? `${actions.length} things, now`
            : "one thing, now"}
        </span>
      </div>
      {actions.length === 0 ? (
        <p className="empty-note flex items-center gap-2 !border-border !bg-card !text-muted-foreground">
          <CheckCircle2 className="h-4 w-4 shrink-0" aria-hidden />{" "}
          Nothing to move. You're sorted.
        </p>
      ) : (
        <ul className="space-y-3">
          {actions.map((a) => (
            <li
              key={a.text}
              data-severity={a.severity}
              className="relative overflow-hidden rounded-[1.25rem] border border-border bg-card px-5 py-4 text-[15px] md:px-7"
            >
              <div className="flex items-start gap-4">
                <span
                  aria-hidden
                  className="mt-2 h-2 w-2 shrink-0 rounded-full bg-muted-foreground/70"
                />
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
