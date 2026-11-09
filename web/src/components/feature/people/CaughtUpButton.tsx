import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { errorMessage } from "@/components/feature/people/ui";
import { useAddTouch } from "@/components/feature/people/mutations";

/** One tap: logs a call today. The server picks the day, so the viewer's clock never matters. */
export function CaughtUpButton({ personId, name }: { personId: string; name: string }) {
  const add = useAddTouch();
  return (
    <Button
      variant="outline"
      className="min-h-11"
      disabled={add.isPending}
      onClick={() =>
        add.mutate(
          { personId, input: { kind: "call" } },
          {
            onSuccess: () => toast.success(`Caught up with ${name}.`),
            onError: (e) => toast.error(errorMessage(e)),
          },
        )
      }
    >
      Caught up
    </Button>
  );
}
