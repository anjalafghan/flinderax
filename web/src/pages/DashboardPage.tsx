import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { ChevronDown, CircleHelp, Heart, ListFilter, Plus, type LucideIcon } from "lucide-react";
import { Link } from "react-router-dom";

import api from "@/services/api";
import { Header } from "@/components/layout/Header";
import { CardCarousel } from "@/components/feature/CardCarousel";
import { PlannerSummary } from "@/components/feature/PlannerSummary";
import { NetWorthPanel } from "@/components/feature/NetWorthPanel";
import { CardUtilization } from "@/components/feature/CardUtilization";
import { MaturityList } from "@/components/feature/MaturityList";
import { AutoPostings } from "@/components/feature/AutoPostings";
import { SpendingPanel } from "@/components/feature/SpendingPanel";
import { DatesPanel } from "@/components/feature/DatesPanel";
import { DueSoonCard } from "@/components/feature/BillsPanel";
import { InsightsNudge } from "@/components/feature/InsightsPanel";
import { HabitsCard } from "@/components/feature/HabitsCard";
import { CheckInCard } from "@/components/feature/CheckInCard";
import { LifeLineCard } from "@/components/feature/life/LifeLineCard";
import { UpcomingDatesCard } from "@/components/feature/people/UpcomingDatesCard";
import { EmptyScene } from "@/components/ui/EmptyScene";
import { useDashboardHistory } from "@/hooks/usePlanner";
import { StreaksCard } from "@/components/feature/StreaksCard";
import { DatesHome } from "@/components/feature/DatesHome";
import { getCouple, keys as joyKeys } from "@/joy/api";

const QUICK_ACTIONS: { to: string; label: string; hint: string; icon: LucideIcon }[] = [
  { to: "/spending", label: "Log spending", hint: "Paste, upload or add", icon: Plus },
  { to: "/plan#afford", label: "Can I afford it?", hint: "Check before you buy", icon: CircleHelp },
  { to: "/spending", label: "Categorize", hint: "Review new alerts", icon: ListFilter },
  { to: "/joy", label: "Plan a date", hint: "Pick one for the week", icon: Heart },
];

/** A card as /api/card/get_all_cards sends it; colours are packed 0xRRGGBB integers. */
interface CardWire {
  card_id: string;
  card_name: string;
  card_bank: string;
  card_primary_color: number;
  card_secondary_color: number;
  last_4_digits: string | null;
  last_total_due: number | null;
  last_delta: number | null;
}

export interface CardData {
  card_id: string;
  card_name: string;
  card_bank: string;
  card_primary_color: [number, number, number];
  card_secondary_color: [number, number, number];
  last_4_digits: string | null;
  last_total_due: number | null;
  last_delta: number | null;
  /** Only present on the single-card endpoint, not the card list. */
  statement_day?: number | null;
  due_day?: number | null;
  credit_limit_paise?: number | null;
  pay_from_account_id?: string | null;
}

function MoneyDashboard() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [windowWidth, setWindowWidth] = useState(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );
  const [showCarousel, setShowCarousel] = useState(false);

  useEffect(() => {
    const handleResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener("resize", handleResize);

    const t = setTimeout(() => {
      setShowCarousel(true);
    }, 100);

    return () => {
      window.removeEventListener("resize", handleResize);
      clearTimeout(t);
    };
  }, []);

  const isMobile = windowWidth < 640;

  const { data: cards, isLoading } = useQuery({
    queryKey: ["cards"],
    queryFn: async () => {
      const res = await api.get<{ cards: CardWire[] }>("/api/card/get_all_cards");

      return res.data.cards.map((c) => ({
        card_id: c.card_id,
        card_name: c.card_name,
        card_bank: c.card_bank,
        card_primary_color: unpackColor(c.card_primary_color),
        card_secondary_color: unpackColor(c.card_secondary_color),
        last_4_digits: c.last_4_digits,
        last_total_due: c.last_total_due,
        last_delta: c.last_delta,
      })) as CardData[];
    },
  });

  // Tracker data is optional: if this fails, the section is simply hidden.
  const { data: history } = useDashboardHistory(12);



  const unpackColor = (packed: number): [number, number, number] => {
    const r = (packed >> 16) & 0xff;
    const g = (packed >> 8) & 0xff;
    const b = packed & 0xff;
    return [r, g, b];
  };

  const totalDue =
    cards?.reduce(
      (acc: number, card: CardData) => acc + (card.last_total_due || 0),
      0,
    ) || 0;

  // When the cards arrive (or change), reopen the card the user had active. Done during render, not in an effect.
  const [seenCards, setSeenCards] = useState<typeof cards>(undefined);
  if (cards !== seenCards) {
    setSeenCards(cards);
    if (cards && cards.length > 0) {
      const savedCardId = sessionStorage.getItem("dashboard_active_card");
      if (savedCardId) {
        const index = cards.findIndex((c) => c.card_id === savedCardId);
        if (index !== -1) {
          setActiveIndex(index);
        }
      }
    }
  }

  return (
    <div className="min-h-[100dvh] w-full text-foreground transition-colors duration-300 flex flex-col overflow-x-clip">
      <Header />

      <main className="page flex-1">
        <div className="mb-6 w-full">
          <PlannerSummary />
          <StreaksCard />
          <LifeLineCard />
          <InsightsNudge />
          <HabitsCard />
          <CheckInCard />
          <UpcomingDatesCard />
        </div>

        <nav aria-label="Quick actions" className="mb-12 grid w-full grid-cols-[repeat(auto-fit,minmax(210px,1fr))] gap-3">
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

        <div className="mb-12 w-full">
          <DueSoonCard />
        </div>

        <section aria-label="Your cards" className="mb-12 w-full">
          <div className="mb-2 flex items-baseline justify-between gap-3">
            <h2 className="font-display text-[32px] leading-none">Cards</h2>
            <p className="eyebrow">
              {new Intl.NumberFormat("en-IN", { style: "currency", currency: "INR" }).format(totalDue)} due in total
            </p>
          </div>
          <div className="relative flex h-[280px] items-center justify-center overflow-visible perspective-[1200px] md:h-[360px]">
            {!showCarousel || isLoading ? (
              <div className="animate-pulse text-sm text-muted-foreground">Loading cards...</div>
            ) : cards && cards.length > 0 ? (
              <CardCarousel cards={cards} activeIndex={activeIndex} setActiveIndex={setActiveIndex} isMobile={isMobile} />
            ) : (
              <Link to="/cards/new" className="no-underline">
                <EmptyScene scene="coins" title="No cards yet">Add your first card and watch it come alive.</EmptyScene>
              </Link>
            )}
          </div>
          <div className="mt-2 flex justify-center gap-2">
            {cards?.map((_: CardData, idx: number) => (
              <button
                key={idx}
                onClick={() => setActiveIndex(idx)}
                className={`h-2 rounded-full transition-all ${idx === activeIndex ? "w-6 bg-primary" : "w-2 bg-muted-foreground/30"}`}
                aria-label={`Go to card ${idx + 1}`}
              />
            ))}
          </div>
        </section>

        <div className="mb-12 grid w-full items-start gap-6 md:grid-cols-2 [&>section]:mb-0 [&>section]:max-w-none">
          <SpendingPanel />
          <DatesPanel />
        </div>

        {history && (
          <details className="group mb-12 w-full glass rounded-3xl">
            <summary className="flex cursor-pointer list-none items-center justify-between p-4 md:p-5">
              <span>
                <span className="block font-display text-[26px] leading-none">Trends &amp; insights</span>
                <span className="block text-sm text-muted-foreground">Net worth, card utilization, maturities, auto-postings</span>
              </span>
              <ChevronDown className="h-4 w-4 text-muted-foreground transition-transform group-open:rotate-180" aria-hidden />
            </summary>
            <div className="space-y-6 border-t border-border/70 p-4 md:p-5">
              <div className="grid gap-6 md:grid-cols-2">
                <NetWorthPanel months={history.months} />
                <CardUtilization cards={history.cards} />
              </div>
              <MaturityList items={history.maturities} />
              <AutoPostings items={history.auto_postings} enabled={history.auto_post} />
            </div>
          </details>
        )}
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
