import { useState } from "react";
import { Undo2 } from "lucide-react";

import { movesApi } from "@/services/api";
import { keys, useMoves, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { formatINR } from "@/utils/money";

const LABEL = { moved: "Moved", already_covered: "Already there", skipped: "Skipped" } as const;

export function RecentMoves() {
  const { data } = useMoves();
  const undo = usePlannerMutation(movesApi.undo, [keys.moves, keys.accounts], { success: "Undone" });
  const [all, setAll] = useState(false);
  if (!data || data.length === 0) return null;
  return (
    <section aria-label="Recent moves" className="space-y-4">
      <h2 className="font-display text-[32px] leading-none tracking-tight">What you've recorded</h2>
      <ul className="ledger glass rounded-3xl px-5 py-2 md:px-6">
        {(all ? data : data.slice(0, 8)).map((m) => (
          <li key={m.move_id} className="ledger-row text-sm">
            <div className="min-w-0">
              <p className="font-medium">
                {LABEL[m.mode]} {formatINR(m.amount_paise)}
              </p>
              <p className="text-[13px] text-muted-foreground">
                {m.kind === "allocation" ? "Savings top-up" : "Card bill"} · {m.created_at.slice(0, 10)}
              </p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              disabled={undo.isPending}
              onClick={() => undo.mutate(m.move_id)}
              title="Undo this (balances go back too)"
            >
              <Undo2 className="mr-1 h-4 w-4" /> Undo
            </Button>
          </li>
        ))}
      </ul>
      {data.length > 8 && (
        <Button variant="ghost" size="sm" onClick={() => setAll(!all)}>
          {all ? "Show fewer" : `Show all ${data.length}`}
        </Button>
      )}
    </section>
  );
}
