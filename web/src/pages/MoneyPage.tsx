import { Link } from "react-router-dom";
import { ArrowRight, CreditCard, Landmark, Plus, Receipt } from "lucide-react";

import { Header } from "@/components/layout/Header";
import { AnalyticsPanel } from "@/components/feature/AnalyticsPanel";
import { InsightsPanel } from "@/components/feature/InsightsPanel";

const LINKS = [
  { to: "/accounts", title: "Accounts", hint: "Bank, FD and RD balances", icon: Landmark },
  { to: "/spending", title: "Spending", hint: "What you've spent, and where", icon: Receipt },
  { to: "/", title: "Cards", hint: "Your credit cards and what's due", icon: CreditCard },
  { to: "/cards/new", title: "Add a card", hint: "Start tracking another card", icon: Plus },
];

export default function MoneyPage() {
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page space-y-10">
        <div>
          <h1 className="page-title">Money</h1>
          <p className="page-sub">Your accounts, cards and spending in one place.</p>
        </div>
        <nav aria-label="Money sections" className="grid gap-3 sm:grid-cols-2">
          {LINKS.map((l) => (
            <Link
              key={l.to}
              to={l.to}
              className="group flex min-h-[76px] items-center gap-4 rounded-[18px] border border-border bg-card px-5 py-4 no-underline transition-colors hover:bg-chip focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-chip">
                <l.icon className="h-[18px] w-[18px]" strokeWidth={1.5} aria-hidden />
              </span>
              <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="text-[15px] font-medium">{l.title}</span>
                <span className="text-[13px] leading-snug text-muted-foreground">{l.hint}</span>
              </span>
              <ArrowRight
                className="h-4 w-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                strokeWidth={1.5}
                aria-hidden
              />
            </Link>
          ))}
        </nav>
        <AnalyticsPanel />
        <InsightsPanel />
      </main>
    </div>
  );
}
