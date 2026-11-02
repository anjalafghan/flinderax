import { useState } from "react";

import { Header } from "@/components/layout/Header";
import { IngestSetup } from "@/components/feature/IngestSetup";
import { ManualTransaction } from "@/components/feature/ManualTransaction";
import { PasteAlert } from "@/components/feature/PasteAlert";
import { SpendingOverview } from "@/components/feature/SpendingOverview";
import { SpendingRules } from "@/components/feature/SpendingRules";
import { TransactionList } from "@/components/feature/TransactionList";
import { useSpendSummary } from "@/hooks/usePlanner";

export default function SpendingPage() {
  const { data: summary, isLoading, isError } = useSpendSummary(6);
  const [uncategorizedOnly, setUncategorizedOnly] = useState(false);

  const reviewUncategorized = () => {
    setUncategorizedOnly(true);
    document.getElementById("transactions")?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />
      <main className="container mx-auto max-w-3xl space-y-8 px-4 py-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Spending</h1>
          <p className="text-muted-foreground">
            Bank alerts are picked up automatically. Check their categories here; the app learns from your choices.
          </p>
        </div>

        <SpendingOverview
          summary={summary}
          isLoading={isLoading}
          isError={isError}
          onReviewUncategorized={reviewUncategorized}
        />

        <TransactionList uncategorizedOnly={uncategorizedOnly} onUncategorizedOnlyChange={setUncategorizedOnly} />

        <section aria-label="Paste an alert" className="space-y-3 rounded-xl border p-4 md:p-6">
          <div>
            <h2 className="text-lg font-bold">Paste an alert</h2>
            <p className="text-sm text-muted-foreground">Missed one? Copy the bank's message and paste it here.</p>
          </div>
          <PasteAlert />
        </section>

        <section aria-label="Add manually" className="space-y-3">
          <ManualTransaction />
        </section>

        <section aria-label="Import setup">
          <IngestSetup />
        </section>

        <section aria-label="Rules" className="space-y-3">
          <h2 className="text-lg font-bold">Rules</h2>
          <SpendingRules />
        </section>
      </main>
    </div>
  );
}
