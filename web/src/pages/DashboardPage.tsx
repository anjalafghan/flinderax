import { useQuery } from "@tanstack/react-query";
import {
  ArrowRight,
  CircleHelp,
  CreditCard,
  Heart,
  LayoutGrid,
  ListFilter,
  Plus,
  type LucideIcon,
} from "lucide-react";
import { Link } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import GlareHover from "@/components/bits/GlareHover";
import Magnet from "@/components/bits/Magnet";
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
import { RewardFundCard } from "@/components/feature/RewardFundCard";
import { DatesHome } from "@/components/feature/DatesHome";
import { formatRupees, useCards } from "@/hooks/useCards";
import { getCouple, keys as joyKeys } from "@/joy/api";

/** A party colour per tile, picked for what it is about. `c` names the --c-* token. */
type Party = "coral" | "sun" | "sky" | "rose" | "grape" | "mint";

const QUICK_ACTIONS: {
  to: string;
  label: string;
  hint: string;
  icon: LucideIcon;
  c: Party;
}[] = [
  {
    to: "/spending",
    label: "Log spending",
    hint: "Paste, upload or add",
    icon: Plus,
    c: "coral",
  },
  {
    to: "/plan#afford",
    label: "Can I afford it?",
    hint: "Check before you buy",
    icon: CircleHelp,
    c: "sun",
  },
  {
    to: "/spending/sort",
    label: "Categorize",
    hint: "Sort new payees",
    icon: ListFilter,
    c: "sky",
  },
  {
    to: "/joy",
    label: "Plan a date",
    hint: "Pick one for the week",
    icon: Heart,
    c: "rose",
  },
];

/** A round icon bubble filled with one party colour, lit from the top. */
function Bubble({ icon: Icon, c }: { icon: LucideIcon; c: Party }) {
  return (
    <span
      aria-hidden
      className="inline-flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl text-background transition-transform duration-300 group-hover:-rotate-6 group-hover:scale-110"
      style={{
        background: `linear-gradient(145deg, hsl(var(--c-${c}) / 0.85), hsl(var(--c-${c})))`,
        boxShadow: `0 8px 18px -8px hsl(var(--c-${c}) / 0.8), inset 0 1px 0 hsl(0 0% 100% / 0.35)`,
      }}
    >
      <Icon className="h-[22px] w-[22px]" strokeWidth={1.8} />
    </span>
  );
}

/** The tinted edge and wash a line or tile wears in its party colour. */
const partyStyle = (c: Party) => ({
  borderColor: `hsl(var(--c-${c}) / 0.35)`,
  background: `linear-gradient(120deg, hsl(var(--c-${c}) / 0.12), hsl(var(--card) / 0.7) 60%)`,
});

/** One line on Today instead of the whole carousel: what is due across the cards, and a way in. */
function CardsLine() {
  const { data: cards } = useCards();
  if (!cards) return null;
  const totalDue = cards.reduce((acc, c) => acc + (c.last_total_due || 0), 0);
  return (
    <Link
      to="/cards"
      viewTransition
      className="group flex min-h-[64px] items-center gap-4 rounded-[18px] border px-4 no-underline transition-[transform,box-shadow] hover:-translate-y-px hover:shadow-[0_14px_30px_-18px_hsl(var(--c-sky))]"
      style={partyStyle("sky")}
    >
      <Bubble icon={CreditCard} c="sky" />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-[15px] font-medium">Cards</span>
        <span className="text-[13px] text-muted-foreground">
          {cards.length === 0
            ? "Add your first card"
            : `${formatRupees(totalDue)} due across ${cards.length} card${cards.length === 1 ? "" : "s"}`}
        </span>
      </span>
      <ArrowRight
        className="h-4 w-4 shrink-0 text-brand-ink transition-transform group-hover:translate-x-1"
        strokeWidth={1.5}
        aria-hidden
      />
    </Link>
  );
}

/** The whole picture on one page: money so far, what's ahead, the jar, and the road. */
function GlanceLine() {
  return (
    <Link
      to="/glance"
      viewTransition
      className="group flex min-h-[64px] items-center gap-4 rounded-[18px] border px-4 no-underline transition-[transform,box-shadow] hover:-translate-y-px hover:shadow-[0_14px_30px_-18px_hsl(var(--c-grape))]"
      style={partyStyle("grape")}
    >
      <Bubble icon={LayoutGrid} c="grape" />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-[15px] font-medium">Life at a glance</span>
        <span className="text-[13px] text-muted-foreground">
          Money, the jar and the road ahead on one page
        </span>
      </span>
      <ArrowRight
        className="h-4 w-4 shrink-0 text-brand-ink transition-transform group-hover:translate-x-1"
        strokeWidth={1.5}
        aria-hidden
      />
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

        <nav
          aria-label="Quick actions"
          className="grid w-full grid-cols-[repeat(auto-fit,minmax(210px,1fr))] gap-3"
        >
          {QUICK_ACTIONS.map((a) => (
            // A gentle pull towards the pointer and a glare sweep on hover; both rest under reduced motion.
            <Magnet
              key={a.label}
              padding={24}
              magnetStrength={10}
              maxOffset={6}
              scale={1.02}
              style={{ display: "block" }}
            >
              <GlareHover
                width="100%"
                borderRadius="18px"
                background="transparent"
                borderColor="transparent"
                glareColor="#ffffff"
                glareOpacity={0.4}
                blendMode="normal"
                className="!grid !place-items-stretch !border-0"
              >
                <Link
                  to={a.to}
                  viewTransition
                  className="group flex min-h-[72px] items-center gap-3.5 rounded-[18px] border px-4 no-underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
                  style={partyStyle(a.c)}
                >
                  <Bubble icon={a.icon} c={a.c} />
                  <span className="flex flex-col gap-0.5">
                    <span className="text-[15px] font-semibold">{a.label}</span>
                    <span className="text-[13px] text-muted-foreground">
                      {a.hint}
                    </span>
                  </span>
                </Link>
              </GlareHover>
            </Magnet>
          ))}
        </nav>

        <CardsLine />
        <GlanceLine />
        <InsightsNudge />

        <div className="grid w-full items-start gap-6 md:grid-cols-2 [&>section]:mb-0 [&>section]:max-w-none">
          <SpendingPanel />
          <DatesPanel />
        </div>

        <div className="stack [&>*]:!my-0">
          <RewardFundCard />
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
  const { data: couple, isLoading } = useQuery({
    queryKey: joyKeys.couple,
    queryFn: getCouple,
    retry: 1,
  });
  // Wait for the answer so a Dates-only partner never sees the empty money screen flash by.
  if (isLoading) return <div className="min-h-[100dvh]" aria-busy="true" />;
  if (couple?.in_couple && !couple.uses_money)
    return <DatesHome partnerName={couple.partner_name} />;
  return <MoneyDashboard />;
}
