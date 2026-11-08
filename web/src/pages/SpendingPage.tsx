import { useState } from "react";
import { ChevronDown, ClipboardPaste, FileUp, PenLine } from "lucide-react";

import { cn } from "@/utils/cn";

import { Header } from "@/components/layout/Header";
import { IngestSetup } from "@/components/feature/IngestSetup";
import { JoyConnect } from "@/components/feature/JoyConnect";
import { ManualTransaction } from "@/components/feature/ManualTransaction";
import { PasteAlert } from "@/components/feature/PasteAlert";
import { StatementImport } from "@/components/feature/StatementImport";
import { SpendingOverview } from "@/components/feature/SpendingOverview";
import { SpendingRules } from "@/components/feature/SpendingRules";
import { TransactionList } from "@/components/feature/TransactionList";
import { useSpendSummary } from "@/hooks/usePlanner";

type AddMode = "paste" | "upload" | "manual";
const ADD_MODES = [
  { id: "paste", label: "Paste", icon: ClipboardPaste },
  { id: "upload", label: "Upload", icon: FileUp },
  { id: "manual", label: "Manual", icon: PenLine },
] as const;

export default function SpendingPage() {
  const { data: summary, isLoading, isError } = useSpendSummary(6);
  const [uncategorizedOnly, setUncategorizedOnly] = useState(false);

  const reviewUncategorized = () => {
    setUncategorizedOnly(true);
    document.getElementById("transactions")?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  const [addMode, setAddMode] = useState<AddMode>("paste");

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl space-y-8">
        <div>
          <h1 className="page-title">Spending</h1>
          <p className="page-sub">Bank alerts are picked up automatically. Fix a category and the app learns it.</p>
        </div>

        <SpendingOverview
          summary={summary}
          isLoading={isLoading}
          isError={isError}
          onReviewUncategorized={reviewUncategorized}
        />

        <TransactionList uncategorizedOnly={uncategorizedOnly} onUncategorizedOnlyChange={setUncategorizedOnly} />

        <section aria-label="Add transactions" className="space-y-4 glass rounded-3xl p-4 md:p-6">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <h2 className="font-display text-[32px] leading-none">Add transactions</h2>
            <div role="tablist" aria-label="How to add" className="grid grid-cols-3 gap-1 rounded-xl bg-muted p-1">
              {ADD_MODES.map((m) => (
                <button
                  key={m.id}
                  role="tab"
                  aria-selected={addMode === m.id}
                  onClick={() => setAddMode(m.id)}
                  className={cn(
                    "inline-flex h-9 items-center justify-center gap-1.5 rounded-lg px-3 text-sm font-medium transition-colors",
                    addMode === m.id ? "bg-card text-foreground" : "text-muted-foreground hover:text-foreground",
                  )}
                >
                  <m.icon className="h-4 w-4" aria-hidden />
                  {m.label}
                </button>
              ))}
            </div>
          </div>
          {addMode === "paste" && (
            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">Missed one? Copy the bank's message and paste it here.</p>
              <PasteAlert />
            </div>
          )}
          {addMode === "upload" && (
            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">
                Drop the CSV from your bank's website. Nothing is added twice, so overlapping files are fine.
              </p>
              <StatementImport />
            </div>
          )}
          {addMode === "manual" && <ManualTransaction />}
        </section>

        <details className="group glass rounded-3xl">
          <summary className="flex cursor-pointer list-none items-center justify-between p-4 md:p-5">
            <span>
              <span className="block font-semibold">Automation &amp; rules</span>
              <span className="block text-sm text-muted-foreground">Alert forwarding, Joy budget link, category rules</span>
            </span>
            <ChevronDown className="h-4 w-4 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden />
          </summary>
          <div className="space-y-8 border-t border-border/70 p-4 md:p-5">
            <section aria-label="Import setup">
              <IngestSetup />
            </section>
            <section aria-label="Joy date budget">
              <JoyConnect />
            </section>
            <section aria-label="Rules" className="space-y-3">
              <h2 className="font-display text-[32px] leading-none">Rules</h2>
              <SpendingRules />
            </section>
          </div>
        </details>
      </main>
    </div>
  );
}
