import { ArrowRight, Heart, Wallet } from "lucide-react";
import { Link } from "react-router-dom";

import GradientText from "@/components/bits/GradientText";
import StarBorder from "@/components/bits/StarBorder";
import { Header } from "@/components/layout/Header";
import { DatesPanel } from "@/components/feature/DatesPanel";
import { useSection } from "@/theme/useSection";

/**
 * Today for a partner who only came for Dates: no accounts, cards or income. Safe to Spend and the
 * card carousel would be empty for them, so the home is the date jar, with money as an opt-in.
 */
export function DatesHome({ partnerName }: { partnerName: string | null }) {
  const { colors, dark } = useSection();
  return (
    <div className="flex min-h-[100dvh] w-full flex-col overflow-x-clip text-foreground">
      <Header />
      <main className="page flex-1">
        <section
          aria-label="Dates"
          className="glass relative mb-6 w-full overflow-hidden rounded-[28px] p-6 md:p-8"
        >
          {/* a warm rose bloom behind the greeting */}
          <div
            aria-hidden
            className="pointer-events-none absolute -right-16 -top-20 h-64 w-64 rounded-full bg-brand/25 blur-3xl"
          />
          <div
            aria-hidden
            className="pointer-events-none absolute -bottom-24 -left-10 h-56 w-56 rounded-full bg-sun/20 blur-3xl"
          />
          <p className="eyebrow relative text-brand-ink">Dates</p>
          <h1 className="relative font-display text-[40px] leading-none tracking-tight">
            <GradientText
              colors={[colors[0], colors[1], colors[2], colors[0]]}
              animationSpeed={10}
              glow={0.35}
            >
              {partnerName ? `You and ${partnerName}` : "Your dates"}
            </GradientText>
          </h1>
          <p className="relative mt-3 max-w-prose text-muted-foreground">
            Pick something to do together, plan it, and keep the memory. Your
            date budget shows below once you spend on one.
          </p>
          <StarBorder
            as={Link}
            to="/joy"
            className="mt-5 min-h-[48px] font-medium shadow-lg shadow-brand/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            backgroundColor="linear-gradient(120deg, hsl(var(--brand)), hsl(var(--brand-2)))"
            textColor="hsl(var(--brand-fg))"
            color={dark ? "#ffffff" : colors[1]}
            trailColor={colors[0]}
            theme={dark ? "dark" : "light"}
            radius={24}
            sparkle
            glow={0.8}
          >
            <Heart className="h-5 w-5 fill-current" aria-hidden /> Open the jar{" "}
            <ArrowRight className="h-4 w-4" aria-hidden />
          </StarBorder>
        </section>

        <div className="mb-6 w-full">
          <DatesPanel />
        </div>

        <section
          aria-label="Money"
          className="w-full rounded-[28px] border border-dashed border-sky/50 bg-sky/5 p-6"
        >
          <div className="flex items-start gap-3">
            <Wallet
              className="mt-1 h-5 w-5 shrink-0 text-sky"
              aria-hidden
            />
            <div>
              <h2 className="font-semibold">Not tracking money here</h2>
              <p className="text-sm text-muted-foreground">
                That's fine: Dates works on its own. If you want Safe to Spend
                and your cards too, add an account or a card.
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
