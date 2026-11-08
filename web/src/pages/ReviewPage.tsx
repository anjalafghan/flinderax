import { Header } from "@/components/layout/Header";
import { MonthClosePanel } from "@/components/feature/MonthClosePanel";
import { TaxPanel } from "@/components/feature/TaxPanel";

export default function ReviewPage() {
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page max-w-3xl space-y-8">
        <div>
          <h1 className="page-title">Review</h1>
          <p className="page-sub">How last month went, and where your tax stands.</p>
        </div>
        <MonthClosePanel />
        <TaxPanel />
      </main>
    </div>
  );
}
