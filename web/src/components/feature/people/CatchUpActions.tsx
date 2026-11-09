import { useId, useState } from "react";

import { Button } from "@/components/ui/button";
import { CaughtUpButton } from "@/components/feature/people/CaughtUpButton";
import { LogTouchForm } from "@/components/feature/people/LogTouchForm";

/** "Caught up" in one tap, or log it another way in a panel that opens below. */
export function CatchUpActions({ personId, name }: { personId: string; name: string }) {
  const panelId = useId();
  const [logging, setLogging] = useState(false);
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2">
        <CaughtUpButton personId={personId} name={name} />
        <Button
          variant="ghost"
          className="min-h-11"
          aria-expanded={logging}
          aria-controls={panelId}
          onClick={() => setLogging((v) => !v)}
        >
          {logging ? "Close" : "Log another way"}
        </Button>
      </div>
      {logging && (
        <div id={panelId}>
          <LogTouchForm personId={personId} onDone={() => setLogging(false)} />
        </div>
      )}
    </div>
  );
}
