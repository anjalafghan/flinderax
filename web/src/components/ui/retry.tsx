import { CloudOff } from "lucide-react";

import { Button } from "@/components/ui/button";

/** A failed load: say so in plain words and offer to try again. */
export function Retry({
  message,
  onRetry,
}: {
  message: string;
  onRetry: () => void;
}) {
  return (
    <div className="flex flex-col items-center gap-3 rounded-[28px] border border-dashed border-sun/50 bg-gradient-to-br from-sun/10 via-coral/5 to-transparent px-6 py-8 text-center text-sm text-muted-foreground">
      <span
        aria-hidden
        className="flex h-11 w-11 items-center justify-center rounded-full bg-sun/20 text-[hsl(var(--foreground))] dark:text-sun"
      >
        <CloudOff className="h-5 w-5" strokeWidth={1.8} />
      </span>
      <p className="m-0">
        {message}{" "}
        <Button
          variant="link"
          className="h-auto p-0 font-semibold text-brand-ink"
          onClick={onRetry}
        >
          Try again
        </Button>
      </p>
    </div>
  );
}
