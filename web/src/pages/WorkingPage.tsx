import { ArrowLeft } from "lucide-react";
import { Link } from "react-router-dom";

import { useSummary } from "@/hooks/usePlanner";
import { Header } from "@/components/layout/Header";
import { PotsPanel } from "@/components/feature/PotsPanel";
import { HowItsCalculated } from "@/components/feature/HowItsCalculated";
import { BudgetSplit } from "@/components/feature/BudgetSplit";
import { RecoveryCard } from "@/components/feature/RecoveryCard";
import { UpcomingList } from "@/components/feature/UpcomingList";
import { RecentMoves } from "@/components/feature/RecentMoves";
import { formatINR } from "@/utils/money";

/** Everything behind the dashboard number: pots, the calculation, the split, what's coming. */
export default function WorkingPage() {
  const { data, isLoading, isError } = useSummary();
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-4xl space-y-8">
        <div>
          <Link to="/" className="mb-3 inline-flex items-center text-sm text-muted-foreground hover:text-foreground">
            <ArrowLeft className="mr-1 h-4 w-4" /> Back to dashboard
          </Link>
          <h1 className="page-title">The working</h1>
          <p className="page-sub">How the dashboard number is made, step by step.</p>
        </div>

        {isLoading && <div className="h-40 animate-pulse rounded-xl bg-muted/40" />}
        {isError && <p className="text-destructive">Couldn't load the working.</p>}

        {data && (
          <>
            <PotsPanel pots={data.pots} />
            <div className="grid gap-8 md:grid-cols-2">
              <HowItsCalculated lines={data.breakdown} />
              <div className="space-y-8">
                <BudgetSplit budget={data.budget} spending={data.spending} />
                {data.next_period && (
                  <section aria-label="Next period" className="glass rounded-3xl p-4 text-sm">
                    <p className="font-semibold">Heads-up: {data.next_period.label}</p>
                    <p className="mt-1 text-muted-foreground">
                      {formatINR(data.next_period.income_paise)} arrives, and {formatINR(data.next_period.committed_paise)} of
                      fixed payments are already due after it, leaving {formatINR(data.next_period.left_paise)}.
                    </p>
                  </section>
                )}
              </div>
            </div>
            <RecoveryCard summary={data} />
            <div className="grid gap-8 md:grid-cols-2">
              <UpcomingList items={data.upcoming} limit={8} />
              <RecentMoves />
            </div>
          </>
        )}
      </main>
    </div>
  );
}
