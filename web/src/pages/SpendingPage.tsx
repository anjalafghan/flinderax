import { useState } from "react";
import { ChevronDown, ClipboardPaste, FileUp, PenLine } from "lucide-react";

import { cn } from "@/utils/cn";

import { Header } from "@/components/layout/Header";
import { Link } from "react-router-dom";
import { ManualTransaction } from "@/components/feature/ManualTransaction";
import { PasteAlert } from "@/components/feature/PasteAlert";
import { UnrecognizedInbox } from "@/components/feature/UnrecognizedInbox";
import { StatementImport } from "@/components/feature/StatementImport";
import { SpendingOverview } from "@/components/feature/SpendingOverview";
import { SpendingRules } from "@/components/feature/SpendingRules";
import { TransactionList } from "@/components/feature/TransactionList";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { useSpendSummary } from "@/hooks/usePlanner";

type AddMode = "paste" | "upload" | "manual";
const ADD_MODES = [
  { id: "upload", label: "Statement", icon: FileUp },
  { id: "paste", label: "Paste", icon: ClipboardPaste },
  { id: "manual", label: "Manual", icon: PenLine },
] as const;

export default function SpendingPage() {
  const { data: summary, isLoading, isError } = useSpendSummary(6);
  const [uncategorizedOnly, setUncategorizedOnly] = useState(false);

  const [addMode, setAddMode] = useState<AddMode>("upload");

  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page stack page-narrow">
        <div>
          <h1 className="page-title">Spending</h1>
          <p className="page-sub">
            Upload your bank statement to bring everything in; bank alerts fill
            the gaps in between. Fix a category and the app learns it.
          </p>
        </div>

        <section aria-label="Add transactions">
          <Card>
            <CardHeader className="pb-4">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <h2 className="font-display brand-text text-[32px] leading-none">
                  Add transactions
                </h2>
                <div
                  role="tablist"
                  aria-label="How to add"
                  className="grid grid-cols-3 gap-1 rounded-xl border border-brand/20 bg-brand-tint p-1 sm:w-auto"
                >
                  {ADD_MODES.map((m) => (
                    <button
                      key={m.id}
                      role="tab"
                      aria-selected={addMode === m.id}
                      onClick={() => setAddMode(m.id)}
                      className={cn(
                        "inline-flex h-11 min-w-0 items-center justify-center gap-1.5 rounded-lg px-1 text-sm font-medium transition-colors sm:px-3",
                        addMode === m.id
                          ? "bg-brand-gradient shadow-[0_8px_18px_-10px_hsl(var(--brand)/0.8)]"
                          : "text-muted-foreground hover:bg-brand-tint hover:text-brand-ink",
                      )}
                    >
                      <m.icon className="hidden h-4 w-4 shrink-0 sm:block" aria-hidden />
                      {m.label}
                    </button>
                  ))}
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <UnrecognizedInbox />
              {addMode === "paste" && (
                <>
                  <p className="max-w-prose text-sm leading-snug text-muted-foreground">
                    Missed one? Copy the bank's message and paste it here.
                  </p>
                  <PasteAlert />
                </>
              )}
              {addMode === "upload" && (
                <>
                  <p className="max-w-prose text-sm leading-snug text-muted-foreground">
                    Upload the statement PDF from your bank (Kotak, for one) and
                    it is read on this device. For HDFC, download it from
                    NetBanking as{" "}
                    <span className="font-medium text-foreground">
                      Delimited (.txt)
                    </span>
                    . Nothing is added twice, so overlapping periods are fine.
                  </p>
                  <StatementImport />
                </>
              )}
              {addMode === "manual" && <ManualTransaction />}
            </CardContent>
          </Card>
        </section>

        <SpendingOverview
          summary={summary}
          isLoading={isLoading}
          isError={isError}
        />

        <TransactionList
          uncategorizedOnly={uncategorizedOnly}
          onUncategorizedOnlyChange={setUncategorizedOnly}
        />

        <details className="group glass rounded-[28px]">
          <summary className="flex cursor-pointer list-none items-center justify-between gap-4 px-6 py-5 md:px-8">
            <span>
              <span className="flex items-center gap-2 font-semibold">
                <span
                  aria-hidden
                  className="inline-block h-2.5 w-2.5 rounded-full bg-grape"
                />
                Rules
              </span>
              <span className="block text-sm leading-snug text-muted-foreground">
                Category rules
              </span>
            </span>
            <ChevronDown
              className="h-4 w-4 shrink-0 text-muted-foreground transition-transform group-open:rotate-180"
              aria-hidden
            />
          </summary>
          <div className="stack border-t border-border p-6 md:p-8">
            <p className="text-sm text-muted-foreground">
              Alert forwarding and your partner's date share are in{" "}
              <Link
                to="/account"
                className="inline-flex min-h-10 items-center font-medium text-primary md:min-h-0"
              >
                Settings
              </Link>
              .
            </p>
            <section aria-label="Rules" className="space-y-3">
              <h2 className="font-display brand-text text-[32px] leading-none">
                Rules
              </h2>
              <SpendingRules />
            </section>
          </div>
        </details>
      </main>
    </div>
  );
}
