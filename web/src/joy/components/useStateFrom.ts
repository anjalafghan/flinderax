import { useState, type Dispatch, type SetStateAction } from "react";

/**
 * State that restarts from `value` whenever a dep changes, the way a useEffect that calls setState would.
 * It does the reset during render, so there is no extra pass. Deps compare with Object.is, as effect deps do.
 */
export function useStateFrom<T>(value: T, deps: readonly unknown[]): [T, Dispatch<SetStateAction<T>>] {
  const [state, setState] = useState<T>(value);
  const [seen, setSeen] = useState<readonly unknown[]>(deps);
  if (deps.length !== seen.length || deps.some((d, i) => !Object.is(d, seen[i]))) {
    setSeen(deps);
    setState(value);
  }
  return [state, setState];
}
