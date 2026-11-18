import { Navigate, useParams } from "react-router-dom";

import { Header } from "@/components/layout/Header";
import { WeddingBudget } from "@/components/feature/WeddingBudget";

/** The budget under a shared goal: items by category, their payments and one verdict. */
export default function WeddingPage() {
  const { goalId } = useParams();
  if (!goalId) return <Navigate to="/" replace />;
  return (
    <div className="flex min-h-screen flex-col">
      <Header />
      <main className="page page-narrow stack">
        <header>
          <h1 className="page-title">Wedding budget</h1>
          <p className="page-sub">What each part costs, what is paid, and whether you can make the next payment.</p>
        </header>
        <WeddingBudget goalId={goalId} />
      </main>
    </div>
  );
}
