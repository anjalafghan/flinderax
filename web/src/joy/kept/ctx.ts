import { createContext, useContext } from "react";
import type { Go } from "../api";

export interface FinishOptions {
  /** Where focus lands afterwards; defaults to the Go target's #hash. */
  focus?: string;
  after?: () => void;
}

export interface KeptCtx {
  currency: string;
  /** Follow a Go: refetch Kept (or navigate when it leaves the page), then move focus. */
  finish: (go: Go, opts?: FinishOptions) => Promise<void>;
  /** Refetch and move focus, without a Go. */
  refresh: (focus?: string) => Promise<void>;
  /** The date that was just opened (its flap lifts, "surprise!"). */
  revealed: number | null;
  setRevealed: (id: number | null) => void;
}

export const KeptContext = createContext<KeptCtx | null>(null);

export function useKept(): KeptCtx {
  const c = useContext(KeptContext);
  if (!c) throw new Error("useKept outside KeptPage");
  return c;
}
