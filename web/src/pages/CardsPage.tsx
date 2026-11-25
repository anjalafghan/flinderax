import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Plus } from "lucide-react";

import { formatRupees, useCards } from "@/hooks/useCards";
import { Header } from "@/components/layout/Header";
import { CardCarousel } from "@/components/feature/CardCarousel";
import { CardUtilization } from "@/components/feature/CardUtilization";
import { RewardsPanel } from "@/components/feature/RewardsPanel";
import { EmptyScene } from "@/components/ui/EmptyScene";
import GradientText from "@/components/bits/GradientText";
import { SECTION_COLORS } from "@/theme/sections";
import { useDashboardHistory } from "@/hooks/usePlanner";

/** Money › Cards: every card, what is due across them, how much of each limit is used, and Add card. */
export default function CardsPage() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [windowWidth, setWindowWidth] = useState(
    typeof window !== "undefined" ? window.innerWidth : 1200,
  );
  const [showCarousel, setShowCarousel] = useState(false);

  useEffect(() => {
    const onResize = () => setWindowWidth(window.innerWidth);
    window.addEventListener("resize", onResize);
    // The 3D carousel measures itself; give layout a beat first.
    const t = setTimeout(() => setShowCarousel(true), 100);
    return () => {
      window.removeEventListener("resize", onResize);
      clearTimeout(t);
    };
  }, []);

  const { data: cards, isLoading } = useCards();
  const { data: history } = useDashboardHistory(12);
  const totalDue =
    cards?.reduce((acc, c) => acc + (c.last_total_due || 0), 0) || 0;

  // When the cards arrive (or change), reopen the card the user had active. Done during render, not in an effect.
  const [seenCards, setSeenCards] = useState<typeof cards>(undefined);
  if (cards !== seenCards) {
    setSeenCards(cards);
    const saved = sessionStorage.getItem("dashboard_active_card");
    const index =
      saved && cards ? cards.findIndex((c) => c.card_id === saved) : -1;
    if (index !== -1) setActiveIndex(index);
  }

  const hasCards = !!cards && cards.length > 0;

  return (
    <div className="min-h-[100dvh] w-full overflow-x-clip text-foreground">
      <Header />
      <main className="page stack">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div className="min-w-0">
            <h1 className="page-title">Cards</h1>
            <p className="page-sub">
              {hasCards ? (
                <>
                  <GradientText
                    colors={[...SECTION_COLORS.money.light, "#8a8073"]}
                    animationSpeed={6}
                    className="font-semibold"
                  >
                    {formatRupees(totalDue)}
                  </GradientText>{" "}
                  due across your cards
                </>
              ) : (
                "Your credit cards and what's due on them."
              )}
            </p>
          </div>
          <Link
            to="/cards/new"
            className="inline-flex h-11 shrink-0 items-center gap-2 rounded-full bg-brand-gradient px-5 text-sm font-medium no-underline shadow-[0_10px_24px_-12px_hsl(var(--brand)/0.7)] transition-opacity hover:opacity-90"
          >
            <Plus className="h-4 w-4" strokeWidth={1.8} aria-hidden />
            Add card
          </Link>
        </div>

        <section aria-label="Your cards">
          <div className="relative flex h-[280px] before:pointer-events-none before:absolute before:inset-x-[15%] before:top-1/2 before:h-1/2 before:-translate-y-1/2 before:rounded-full before:bg-brand-gradient before:opacity-25 before:blur-3xl items-center justify-center overflow-visible perspective-[1200px] md:h-[360px]">
            {!showCarousel || isLoading ? (
              <div className="animate-pulse text-sm text-muted-foreground">
                Loading cards…
              </div>
            ) : hasCards ? (
              <CardCarousel
                cards={cards}
                activeIndex={activeIndex}
                setActiveIndex={setActiveIndex}
                isMobile={windowWidth < 640}
              />
            ) : (
              <Link to="/cards/new" className="no-underline">
                <EmptyScene scene="coins" title="No cards yet">
                  Add your first card and watch it come alive.
                </EmptyScene>
              </Link>
            )}
          </div>
          {hasCards && cards.length > 1 && (
            <div className="mt-2 flex justify-center gap-2">
              {cards.map((c, idx) => (
                <button
                  key={c.card_id}
                  onClick={() => setActiveIndex(idx)}
                  className={`h-2 rounded-full transition-all ${idx === activeIndex ? "w-6 bg-brand-gradient" : "w-2 bg-brand/25"}`}
                  aria-label={`Go to ${c.card_name}`}
                />
              ))}
            </div>
          )}
        </section>

        {history && <CardUtilization cards={history.cards} />}
        <RewardsPanel />
      </main>
    </div>
  );
}
