import { Header } from "@/components/layout/Header";
import { MonthClosePanel } from "@/components/feature/MonthClosePanel";
import { TaxPanel } from "@/components/feature/TaxPanel";

export default function ReviewPage() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Header />
      <main className="container mx-auto max-w-3xl space-y-8 px-4 py-6 md:py-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Review</h1>
          <p className="text-muted-foreground">How last month went, and where your tax stands.</p>
        </div>
        <MonthClosePanel />
        <TaxPanel />
      </main>
    </div>
  );
}
