import { RotateCcw } from "lucide-react";

import { Button } from "@/components/ui/button";

interface Props {
  onReset: () => void;
  /** The button text. "Reset" puts a form back; "Clear" empties one. */
  label?: string;
  disabled?: boolean;
  /** When set, the person confirms first (for resets that cannot be taken back). */
  confirm?: string;
  className?: string;
}

/** One look for every Reset / Clear / Revert button, so they are easy to find and read the same everywhere. */
export function ResetButton({ onReset, label = "Reset", disabled, confirm, className }: Props) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      className={className}
      disabled={disabled}
      onClick={() => {
        if (confirm && !window.confirm(confirm)) return;
        onReset();
      }}
    >
      <RotateCcw className="mr-1.5 h-3.5 w-3.5" aria-hidden />
      {label}
    </Button>
  );
}
