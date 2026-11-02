import { Trash2 } from "lucide-react";

import { ingestApi } from "@/services/api";
import { keys, useIngestRules, usePlannerMutation } from "@/hooks/usePlanner";
import { Button } from "@/components/ui/button";
import { categoryLabel } from "@/utils/spending";

/** Merchants you've told the app about ("Remember for ..."). New alerts from them are categorised automatically. */
export function SpendingRules() {
  const { data: rules, isLoading, isError } = useIngestRules();
  const remove = usePlannerMutation(ingestApi.deleteRule, [keys.ingestRules], { success: "Rule removed" });

  return (
    <div className="space-y-3">
      <p className="text-sm text-muted-foreground">
        When you tick "Remember", the merchant's future transactions get that category automatically.
      </p>
      {isLoading && <div className="h-16 animate-pulse rounded-xl bg-muted/40" />}
      {isError && <p className="text-sm text-destructive">Couldn't load rules.</p>}
      {rules &&
        (rules.length === 0 ? (
          <p className="rounded-xl border border-dashed p-4 text-sm text-muted-foreground">No rules yet.</p>
        ) : (
          <ul className="divide-y rounded-xl border">
            {rules.map((r) => (
              <li key={r.pattern} className="flex items-center justify-between gap-3 p-3 text-sm">
                <p className="min-w-0 truncate">
                  <span className="font-medium">{r.pattern}</span>
                  <span className="text-muted-foreground"> → {categoryLabel(r.category)}</span>
                </p>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 shrink-0"
                  aria-label={`Remove rule for ${r.pattern}`}
                  disabled={remove.isPending}
                  onClick={() => window.confirm(`Stop auto-categorising ${r.pattern}?`) && remove.mutate(r.pattern)}
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </li>
            ))}
          </ul>
        ))}
    </div>
  );
}
