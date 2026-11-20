import { Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Retry } from "@/components/ui/retry";
import { useAnalyses, useRemoveAnalysis } from "@/services/analyses";
import { formatDay } from "@/utils/dates";

/** Reports written back to your account (by Claude, through the connector). Shown as plain text. */
export function AnalysesPanel() {
  const q = useAnalyses();
  const remove = useRemoveAnalysis();
  const items = q.data?.analyses ?? [];

  return (
    <section aria-labelledby="analyses-title" className="space-y-3">
      <h2 id="analyses-title" className="m-0 font-display text-[32px] leading-none">
        Analyses
      </h2>
      {q.isLoading && <div className="h-24 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading analyses" />}
      {q.isError && !q.data && <Retry message="Couldn't load your analyses." onRetry={() => void q.refetch()} />}
      {q.data && items.length === 0 && (
        <p className="empty-note m-0">Nothing here yet. Ask Claude, through the connector, to analyse your money and save what it finds.</p>
      )}
      {items.map((a) => (
        <Card key={a.id}>
          <CardContent className="space-y-3 p-5 md:p-6">
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <h3 className="m-0 text-base font-semibold">{a.title}</h3>
                <p className="m-0 text-xs text-muted-foreground">
                  {formatDay(a.created_on)} · {a.source}
                </p>
              </div>
              <Button
                variant="ghost"
                size="icon"
                aria-label={`Delete analysis: ${a.title}`}
                disabled={remove.isPending}
                onClick={() => remove.mutate(a.id)}
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
            <p className="m-0 whitespace-pre-wrap break-words text-sm leading-relaxed">{a.body}</p>
          </CardContent>
        </Card>
      ))}
    </section>
  );
}
