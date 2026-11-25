import { useSummary } from "@/hooks/usePlanner";
import { SafeToSpendHero } from "@/components/feature/SafeToSpendHero";
import { ActionList } from "@/components/feature/ActionList";
import { Button } from "@/components/ui/button";
import { SetupCard } from "@/components/feature/SetupCard";
import { isBlankSummary } from "@/utils/setup";
import { Skeleton } from "@/components/ui/skeleton";

/** The home-screen block, kept clean: the number and what to do. The working lives on /working. */
export function PlannerSummary() {
  const { data, isLoading, isError, refetch } = useSummary();

  if (isLoading) {
    return (
      <Skeleton className="h-48 rounded-[28px]" label="Loading safe to spend" />
    );
  }
  if (isError || !data) {
    return (
      <div className="rounded-[28px] border border-dashed border-brand/40 bg-brand-tint p-6 text-center text-sm text-brand-ink">
        Couldn't load your plan.{" "}
        <Button variant="link" className="h-auto p-0" onClick={() => refetch()}>
          Try again
        </Button>
      </div>
    );
  }

  if (isBlankSummary(data)) return <SetupCard summary={data} />;

  return (
    <div className="space-y-6">
      <SafeToSpendHero summary={data} />
      <ActionList actions={data.actions} />
    </div>
  );
}
