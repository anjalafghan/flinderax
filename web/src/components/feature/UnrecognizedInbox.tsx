import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { ingestApi } from "@/services/api";
import { Button } from "@/components/ui/button";

const KEY = ["ingest-unrecognized"] as const;

/**
 * Bank texts the parser couldn't read, kept for 30 days. Each one is a transaction that is
 * missing from Safe to Spend until it is added by hand (Add → Manual) and dismissed here.
 */
export function UnrecognizedInbox() {
  const qc = useQueryClient();
  const inbox = useQuery({ queryKey: KEY, queryFn: ingestApi.unrecognized });
  const dismiss = useMutation({
    mutationFn: ingestApi.dismissUnrecognized,
    onSuccess: () => qc.invalidateQueries({ queryKey: KEY }),
  });
  const items = inbox.data?.items ?? [];
  if (items.length === 0) return null;
  return (
    <section
      aria-label="Messages we couldn't read"
      className="space-y-2 rounded-lg border border-dashed border-sun/60 bg-sun/10 p-3 text-sm"
    >
      <p className="font-medium">
        {items.length} {items.length === 1 ? "message" : "messages"} we couldn't
        read
      </p>
      <p className="text-muted-foreground">
        These are missing from your spending. Add each by hand, then dismiss it.
      </p>
      <ul className="space-y-2">
        {items.map((i) => (
          <li
            key={i.alert_id}
            className="flex items-start justify-between gap-3"
          >
            <span className="break-words text-muted-foreground">{i.text}</span>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={dismiss.isPending}
              onClick={() => dismiss.mutate(i.alert_id)}
            >
              Dismiss
            </Button>
          </li>
        ))}
      </ul>
    </section>
  );
}
