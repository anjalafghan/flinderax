import { Button } from "@/components/ui/button";

/** A failed load: say so in plain words and offer to try again. */
export function Retry({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="rounded-[28px] border border-dashed border-border px-6 py-8 text-center text-sm text-muted-foreground">
      {message}{" "}
      <Button variant="link" className="h-auto p-0" onClick={onRetry}>
        Try again
      </Button>
    </div>
  );
}
