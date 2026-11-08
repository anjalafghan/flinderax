import { useCallback, useState } from "react";
import { errText, useBusy } from "../components/useBusy";

/** useBusy's double-tap lock plus an error line the form can show. */
export function useAction() {
  const { busy, run } = useBusy();
  const [err, setErr] = useState("");
  const act = useCallback(
    (fn: () => Promise<unknown>) =>
      run(async () => {
        setErr("");
        try {
          await fn();
        } catch (e) {
          setErr(errText(e));
        }
      }),
    [run],
  );
  return { busy, err, setErr, act };
}
