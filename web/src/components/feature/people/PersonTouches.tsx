import { toast } from "sonner";

import { errorMessage, NOTE } from "@/components/feature/people/ui";
import { Button } from "@/components/ui/button";
import { ResetButton } from "@/components/ui/reset-button";
import { rupees } from "@/services/checkin";
import { clearTouches, deleteTouch, TOUCH_LABEL, usePeopleMutation, type Touch } from "@/services/people";
import { formatDay } from "@/utils/dates";

/** The last twenty catch-ups, newest first. A mistake can be deleted. */
export function PersonTouches({ touches, personId }: { touches: Touch[]; personId: string }) {
  const remove = usePeopleMutation(deleteTouch);
  const clearAll = usePeopleMutation(clearTouches);
  return (
    <section aria-labelledby="person-touches-title" className="flex flex-col gap-5">
      <h2 id="person-touches-title" className="m-0 font-display text-[32px] leading-none">
        Catch-ups
      </h2>
      {touches.length > 0 ? (
        <ul className="ledger">
          {touches.map((t) => (
            <li key={t.id} className="ledger-row">
              <span className="flex min-w-0 flex-col gap-1">
                <span className="text-[15px] font-medium">{`${TOUCH_LABEL[t.kind]} · ${formatDay(t.on_day)}`}</span>
                <span className="text-[13px] text-muted-foreground">
                  {[t.cost_paise !== null ? `Cost ${rupees(t.cost_paise)}` : "", t.note].filter(Boolean).join(" · ") || "No note"}
                </span>
              </span>
              <Button
                variant="ghost"
                className="min-h-11 shrink-0"
                disabled={remove.isPending}
                onClick={() =>
                  remove.mutate(t.id, {
                    onSuccess: () => toast.success("Catch-up removed."),
                    onError: (err) => toast.error(errorMessage(err)),
                  })
                }
              >
                {`Delete ${TOUCH_LABEL[t.kind].toLowerCase()} on ${formatDay(t.on_day)}`}
              </Button>
            </li>
          ))}
        </ul>
      ) : (
        <p className={NOTE}>No catch-ups logged yet.</p>
      )}
      {touches.length > 0 && (
        <div>
          <ResetButton
            label="Reset: clear all catch-ups"
            disabled={clearAll.isPending}
            confirm="Remove every catch-up with this person, including older ones that are not listed? This cannot be undone."
            onReset={() =>
              clearAll.mutate(personId, {
                onSuccess: () => toast.success("Catch-ups cleared."),
                onError: (err) => toast.error(errorMessage(err)),
              })
            }
          />
        </div>
      )}
    </section>
  );
}
