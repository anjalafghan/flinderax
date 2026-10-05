import { useSummary } from "@/hooks/usePlanner";
import { SafeToSpendHero } from "@/components/feature/SafeToSpendHero";
import { ActionList } from "@/components/feature/ActionList";
import { UpcomingList } from "@/components/feature/UpcomingList";
import { Button } from "@/components/ui/button";
import { Link } from "react-router-dom";

/** The home-screen block: hero number, "do this today" list and the next payments. */
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
      <div className="grid gap-6 md:grid-cols-2">
        <ActionList actions={data.actions} />
        <UpcomingList items={data.upcoming} />
      </div>
      <p className="text-center text-xs text-muted-foreground">
        Numbers come from your <Link className="underline" to="/accounts">accounts</Link> and{" "}
        <Link className="underline" to="/plan">plan</Link>.
      </p>
    </div>
  );
}
