import { useCallback, useRef, useState } from "react";
import { ApiError } from "../api";

/** Double-tap safety: a ref blocks a second call in the same tick, state disables the button. */
export function useBusy() {
  const lock = useRef(false);
  const [busy, setBusy] = useState(false);
  const run = useCallback(async <T>(fn: () => Promise<T>): Promise<T | undefined> => {
    if (lock.current) return undefined;
    lock.current = true;
    setBusy(true);
    try {
      return await fn();
    } finally {
      lock.current = false;
      setBusy(false);
    }
  }, []);
  return { busy, run };
}

export function errText(e: unknown, fallback = "That didn’t work. Try again."): string {
  if (e instanceof ApiError) return e.message || fallback;
  if (e instanceof TypeError) return "No connection. Try again in a moment.";
  return fallback;
}
