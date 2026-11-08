import { useState, type SyntheticEvent } from "react";

/** A <details> whose open state lives in React, so an action can close it and a hash can open it. */
export function useFold(initial = false, onOpen?: () => void) {
  const [open, setOpen] = useState(initial);
  const props = {
    open,
    onToggle: (e: SyntheticEvent<HTMLDetailsElement>) => {
      const now = e.currentTarget.open;
      setOpen(now);
      if (now) onOpen?.();
    },
  };
  return { open, setOpen, props };
}
