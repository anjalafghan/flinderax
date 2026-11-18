import { Header } from "@/components/layout/Header";
import { BodyCard } from "@/components/feature/health/BodyCard";
import { FoodMoneyCard } from "@/components/feature/health/FoodMoneyCard";
import { HabitPicker } from "@/components/feature/health/HabitPicker";
import { WeekCard } from "@/components/feature/health/WeekCard";
import { useDiet } from "@/services/diet";
import { Retry } from "@/components/ui/retry";

/** Food, a few habits and a yearly check-up. Each card loads from the one diet overview. */
export default function HealthPage() {
  const diet = useDiet();
  const o = diet.data;

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page page-narrow stack">
        <div>
          <h1 className="page-title">Health</h1>
          <p className="page-sub">Food, a few habits and a yearly check-up. Nothing you have to log every day.</p>
        </div>

        {diet.isLoading && <div className="h-64 animate-pulse rounded-[28px] bg-muted/40" aria-label="Loading your health" />}

        {diet.isError && !o && <Retry message="Couldn't load your health details." onRetry={() => void diet.refetch()} />}

        {o && (
          <>
            <WeekCard overview={o} />
            <HabitPicker overview={o} />
            <FoodMoneyCard money={o.money} />
            <BodyCard body={o.body} />
          </>
        )}
      </main>
    </div>
  );
}
