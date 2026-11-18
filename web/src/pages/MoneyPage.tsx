import { Header } from "@/components/layout/Header";
import { AnalyticsPanel } from "@/components/feature/AnalyticsPanel";
import { InsightsPanel } from "@/components/feature/InsightsPanel";
import { NetWorthPanel } from "@/components/feature/NetWorthPanel";
import { useDashboardHistory } from "@/hooks/usePlanner";

/** Money › Overview: net worth, where the money goes, and what needs a look. The sub-tabs lead to the rest. */
export default function MoneyPage() {
  const { data: history } = useDashboardHistory(12);
  return (
    <div className="min-h-screen text-foreground">
      <Header />
      <main className="page stack">
        <div>
          <h1 className="page-title">Money</h1>
          <p className="page-sub">How the household is doing, from your statements and balances.</p>
        </div>
        {history && <NetWorthPanel months={history.months} />}
        <AnalyticsPanel />
        <InsightsPanel />
      </main>
    </div>
  );
}
