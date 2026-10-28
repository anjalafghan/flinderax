import { useSummary } from "@/hooks/usePlanner";
import { SafeToSpendHero } from "@/components/feature/SafeToSpendHero";
import { ActionList } from "@/components/feature/ActionList";
import { Button } from "@/components/ui/button";

/** The home-screen block, kept clean: the number and what to do. The working lives on /working. */
export function PlannerSummary() {
  const { data, isLoading, isError, refetch } = useSummary();

  if (isLoading) {
    return <div className="h-48 animate-pulse rounded-3xl bg-muted/40" aria-label="Loading safe to spend" />;
  }
  if (isError || !data) {
    return (
      <div className="rounded-3xl border border-dashed p-6 text-center text-sm text-muted-foreground">
        Couldn't load your plan.{" "}
        <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SafeToSpendHero summary={data} />
      <ActionList actions={data.actions} />
    </div>
  );
}
