import { lazy, Suspense } from "react";
import { Navigate, useParams } from "react-router-dom";

import GradientText from "@/components/bits/GradientText";
import { Header } from "@/components/layout/Header";
import { WeddingBudget } from "@/components/feature/WeddingBudget";

// WebGL confetti, loaded after the page so it stays out of its first chunk.
const HeroParticles = lazy(
  () => import("@/components/feature/life/HeroParticles"),
);

// Blush, rose, berry and a little marigold: the wedding's own colours, whatever the tab.
const WEDDING = ["#c9bfae", "#8a8073", "#8a8073", "#c9bfae"];

/** The budget under a shared goal: items by category, their payments and one verdict. */
export default function WeddingPage() {
  const { goalId } = useParams();
  if (!goalId) return <Navigate to="/" replace />;
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page page-narrow stack">
        <header className="relative isolate overflow-hidden rounded-[28px] px-1 py-4">
          <Suspense fallback={null}>
            <HeroParticles
              colors={WEDDING}
              count={60}
              shape="sparkle"
              drift="up"
            />
          </Suspense>
          <h1 className="m-0 font-display text-4xl font-normal leading-none tracking-tight md:text-5xl">
            <GradientText
              colors={WEDDING.slice(0, 3)}
              animationSpeed={10}
              className="pb-[0.08em]"
            >
              Wedding budget
            </GradientText>
          </h1>
          <p className="page-sub">
            What each part costs, what is paid, and whether you can make the
            next payment.
          </p>
        </header>
        <WeddingBudget goalId={goalId} />
      </main>
    </div>
  );
}
