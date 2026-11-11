import { ArrowRight, Heart, Wallet } from "lucide-react";
import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { DatesPanel } from "@/components/feature/DatesPanel";

/**
 * Today for a partner who only came for Dates: no accounts, cards or income. Safe to Spend and the
 * card carousel would be empty for them, so the home is the date jar, with money as an opt-in.
 */
export function DatesHome({ partnerName }: { partnerName: string | null }) {
  return (
    <div className="flex min-h-[100dvh] w-full flex-col overflow-x-clip text-foreground">
      <Header />
      <main className="page flex-1">
        <section aria-label="Dates" className="glass mb-6 w-full rounded-3xl p-6 md:p-8">
          <p className="eyebrow">Dates</p>
          <h1 className="font-display text-[40px] leading-none tracking-tight">
            {partnerName ? `You and ${partnerName}` : "Your dates"}
          </h1>
          <p className="mt-3 max-w-prose text-muted-foreground">
            Pick something to do together, plan it, and keep the memory. Your date budget shows below once you spend on one.
          </p>
          <Link
            to="/joy"
            className="mt-5 inline-flex min-h-[48px] items-center gap-2 rounded-full bg-primary px-6 font-medium text-primary-foreground no-underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Heart className="h-5 w-5" aria-hidden /> Open the jar <ArrowRight className="h-4 w-4" aria-hidden />
          </Link>
        </section>

        <div className="mb-6 w-full">
          <DatesPanel />
        </div>

        <section aria-label="Money" className="w-full rounded-3xl border border-dashed border-border p-6">
          <div className="flex items-start gap-3">
            <Wallet className="mt-1 h-5 w-5 shrink-0 text-muted-foreground" aria-hidden />
            <div>
              <h2 className="font-semibold">Not tracking money here</h2>
              <p className="text-sm text-muted-foreground">
                That's fine: Dates works on its own. If you want Safe to Spend and your cards too, add an account or a card.
              </p>
              <div className="mt-3 flex flex-wrap gap-4 text-sm font-medium">
                <Link to="/plan" className="text-primary">
                  Set up money
                </Link>
                <Link to="/cards/new" className="text-primary">
                  Add a card
                </Link>
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
