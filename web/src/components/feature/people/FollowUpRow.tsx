import { Link } from "react-router-dom";
import { toast } from "sonner";

import { errorMessage } from "@/components/feature/people/ui";
import { Button } from "@/components/ui/button";
import { deleteFollowUp, followUpWhen, patchFollowUp, usePeopleMutation, type FollowUp } from "@/services/people";
import { cn } from "@/utils/cn";

/** One follow-up: tick it off (or back on), or delete it after a confirm. `showName` adds who it is for. */
export function FollowUpRow({ f, showName }: { f: FollowUp; showName?: boolean }) {
  const tick = usePeopleMutation(patchFollowUp);
  const remove = usePeopleMutation(deleteFollowUp);
  const done = f.state === "done";
  return (
    <li className="ledger-row">
      <label className="flex min-w-0 flex-1 items-start gap-3">
        <input
          type="checkbox"
          className="mt-1 h-5 w-5 shrink-0"
          checked={done}
          disabled={tick.isPending}
          onChange={() =>
            tick.mutate(
              { id: f.id, done: !done },
              { onError: (err) => toast.error(errorMessage(err)) },
            )
          }
          aria-label={`${done ? "Not done" : "Done"}: ${f.text}`}
        />
        <span className="flex min-w-0 flex-col gap-1">
          <span className={cn("text-[15px] font-medium", done && "text-muted-foreground line-through")}>{f.text}</span>
          <span className={cn("text-[13px] text-muted-foreground", f.state === "overdue" && "font-semibold text-destructive")}>
            {showName && (
              <>
                <Link to={`/people/${f.person_id}`} className="font-medium text-primary">
                  {f.person_name}
                </Link>
                {" · "}
              </>
            )}
            {followUpWhen(f)}
          </span>
        </span>
      </label>
      <Button
        variant="ghost"
        className="min-h-11 shrink-0"
        disabled={remove.isPending}
        aria-label={`Delete follow-up: ${f.text}`}
        onClick={() => {
          if (!window.confirm("Delete this follow-up?")) return;
          remove.mutate(f.id, { onError: (err) => toast.error(errorMessage(err)) });
        }}
      >
        Delete
      </Button>
    </li>
  );
}
