import { useEffect } from "react";
import { toast } from "sonner";

import { describeFlush, flush, loadQueue } from "@/offline";

/** Sends what was saved while offline: at start, and again whenever the signal comes back. */
export function useOutbox(signedIn: boolean) {
  useEffect(() => {
    if (!signedIn) return;
    let running = false;
    const run = async () => {
      if (running || loadQueue().length === 0) return;
      running = true;
      try {
        const said = describeFlush(await flush());
        if (said) toast.info(said);
      } finally {
        running = false;
      }
    };
    void run();
    window.addEventListener("online", run);
    return () => window.removeEventListener("online", run);
  }, [signedIn]);
}
