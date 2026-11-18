import { useQuery } from "@tanstack/react-query";
import { ArrowRight, CircleHelp, CreditCard, Heart, ListFilter, Plus, type LucideIcon } from "lucide-react";
import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { CoupleGoalsCard } from "@/components/feature/CoupleGoalsCard";
import { ComingUpCard } from "@/components/feature/ComingUpCard";
import { PlannerSummary } from "@/components/feature/PlannerSummary";
import { SpendingPanel } from "@/components/feature/SpendingPanel";
import { DatesPanel } from "@/components/feature/DatesPanel";
import { InsightsNudge } from "@/components/feature/InsightsPanel";
import { HabitsCard } from "@/components/feature/HabitsCard";
import { CheckInCard } from "@/components/feature/CheckInCard";
import { LifeLineCard } from "@/components/feature/life/LifeLineCard";
import { StreaksCard } from "@/components/feature/StreaksCard";
import { DatesHome } from "@/components/feature/DatesHome";
import { formatRupees, useCards } from "@/hooks/useCards";
import { getCouple, keys as joyKeys } from "@/joy/api";

const QUICK_ACTIONS: { to: string; label: string; hint: string; icon: LucideIcon }[] = [
  { to: "/spending", label: "Log spending", hint: "Paste, upload or add", icon: Plus },
  { to: "/plan#afford", label: "Can I afford it?", hint: "Check before you buy", icon: CircleHelp },
  { to: "/spending/sort", label: "Categorize", hint: "Sort new payees", icon: ListFilter },
  { to: "/joy", label: "Plan a date", hint: "Pick one for the week", icon: Heart },
];


/** One line on Today instead of the whole carousel: what is due across the cards, and a way in. */
function CardsLine() {
  const { data: cards } = useCards();
  if (!cards) return null;
  const totalDue = cards.reduce((acc, c) => acc + (c.last_total_due || 0), 0);
  return (
    <Link
      to="/cards"
      viewTransition
      className="flex min-h-[64px] items-center gap-4 rounded-[18px] border border-border px-5 no-underline transition-colors hover:bg-[hsl(var(--chip))]"
    >
      <CreditCard className="h-[20px] w-[20px] shrink-0" strokeWidth={1.5} aria-hidden />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-[15px] font-medium">Cards</span>
        <span className="text-[13px] text-muted-foreground">
          {cards.length === 0 ? "Add your first card" : `${formatRupees(totalDue)} due across ${cards.length} card${cards.length === 1 ? "" : "s"}`}
        </span>
      </span>
      <ArrowRight className="h-4 w-4 shrink-0 text-muted-foreground" strokeWidth={1.5} aria-hidden />
    </Link>
  );
}

function MoneyDashboard() {
  return (
    <div className="flex min-h-[100dvh] w-full flex-col overflow-x-clip text-foreground transition-colors duration-300">
      <Header />

      <main className="page stack flex-1 [&>*]:!my-0">
        <h1 className="sr-only">Today</h1>
        <PlannerSummary />
        <ComingUpCard />
        <CoupleGoalsCard />

        <nav aria-label="Quick actions" className="grid w-full grid-cols-[repeat(auto-fit,minmax(210px,1fr))] gap-3">
          {QUICK_ACTIONS.map((a) => (
            <Link
              key={a.label}
              to={a.to}
              viewTransition
              className="flex min-h-[72px] items-center gap-3.5 rounded-[18px] border border-border px-5 no-underline transition-colors hover:bg-[hsl(var(--chip))] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <a.icon className="h-[22px] w-[22px] shrink-0" strokeWidth={1.5} aria-hidden />
              <span className="flex flex-col gap-0.5">
                <span className="text-[15px] font-medium">{a.label}</span>
                <span className="text-[13px] text-muted-foreground">{a.hint}</span>
              </span>
            </Link>
          ))}
        </nav>

        <CardsLine />
        <InsightsNudge />

        <div className="grid w-full items-start gap-6 md:grid-cols-2 [&>section]:mb-0 [&>section]:max-w-none">
          <SpendingPanel />
          <DatesPanel />
        </div>

        <div className="stack [&>*]:!my-0">
          <StreaksCard />
          <HabitsCard />
          <LifeLineCard />
          <CheckInCard />
        </div>
      </main>
    </div>
  );
}

/** Today. A partner who is in a couple but has entered no money gets the Dates home instead. */
export default function DashboardPage() {
  const { data: couple, isLoading } = useQuery({ queryKey: joyKeys.couple, queryFn: getCouple, retry: 1 });
  // Wait for the answer so a Dates-only partner never sees the empty money screen flash by.
  if (isLoading) return <div className="min-h-[100dvh]" aria-busy="true" />;
  if (couple?.in_couple && !couple.uses_money) return <DatesHome partnerName={couple.partner_name} />;
  return <MoneyDashboard />;
}
