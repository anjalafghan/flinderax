import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { InsightsNudge, InsightsPanel } from "@/components/feature/InsightsPanel";
import type { InsightsOut } from "@/services/insights";

const empty: InsightsOut = {
  refunds: [],
  charges: { last_12m_paise: 0, items: [] },
  duplicates: [],
  unusual: [],
  health: {
    score: 40,
    savings_rate_bps: null,
    emergency_months_tenths: null,
    income_cv_bps: null,
    actions: [],
  },
  spend_to_payday: null,
};

const fixture: InsightsOut = {
  refunds: [
    {
      txn_id: "t1",
      occurred_on: "2026-11-01",
      amount_paise: 85_000,
      merchant: "Swiggy",
      rail: "p2m",
      due_by: "2026-11-06",
      days_overdue: 5,
      compensation_paise: 50_000,
      status: "overdue",
    },
  ],
  charges: {
    last_12m_paise: 149_900,
    items: [
      { label: "Annual fee", amount_paise: 99_900, count: 1 },
      { label: "Late payment fee", amount_paise: 50_000, count: 2 },
    ],
  },
  duplicates: [],
  unusual: [],
  health: {
    score: 62,
    savings_rate_bps: 1850,
    emergency_months_tenths: 34,
    income_cv_bps: 900,
    actions: [{ text: "Move ₹5,000 a month into your emergency fund", impact: "high" }],
  },
  spend_to_payday: { days: 12, p10_paise: 1_000_000, p50_paise: 1_500_000, p90_paise: 2_200_000 },
};

function withData(data: InsightsOut, ui: "panel" | "nudge" = "panel") {
  const qc = new QueryClient();
  qc.setQueryData(["insights"], data);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>{ui === "panel" ? <InsightsPanel /> : <InsightsNudge />}</MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("InsightsPanel", () => {
  it("shows the health score, the overdue refund with its compensation, and the charges total", () => {
    withData(fixture);
    const panel = screen.getByRole("region", { name: "Insights" });
    expect(panel).toHaveTextContent("62/100");
    expect(panel).toHaveTextContent("Watch");
    expect(panel).toHaveTextContent("Move ₹5,000 a month into your emergency fund");

    const refunds = screen.getByRole("region", { name: "Failed UPI refunds owed" });
    expect(refunds).toHaveTextContent("Swiggy");
    expect(refunds).toHaveTextContent("5 days late · bank owes ₹500");
    expect(refunds).toHaveTextContent("to a merchant");

    const charges = screen.getByRole("region", { name: "Charges & fees" });
    expect(charges).toHaveTextContent("₹1,499");
    expect(charges).toHaveTextContent("Late payment fee");
  });

  it("gives the spend-to-payday range", () => {
    withData(fixture);
    expect(screen.getByRole("region", { name: "Spending to payday" })).toHaveTextContent(
      "Between ₹10,000 and ₹22,000 likely to go before payday in 12 days (most likely ₹15,000).",
    );
  });

  it("hides the sections that have nothing to show", () => {
    withData(fixture);
    expect(screen.queryByRole("region", { name: "Possible duplicate charges" })).toBeNull();
    expect(screen.queryByRole("region", { name: "Unusual months" })).toBeNull();
  });

  it("shows the empty state when there are no transactions to read", () => {
    withData(empty);
    expect(screen.getByText(/Not enough transactions yet/)).toBeInTheDocument();
    expect(screen.queryByRole("region", { name: "Failed UPI refunds owed" })).toBeNull();
    expect(screen.queryByRole("region", { name: "Charges & fees" })).toBeNull();
  });
});

describe("InsightsNudge", () => {
  it("shows the overdue refund alert when a refund is overdue", () => {
    withData(fixture, "nudge");
    expect(screen.getByText("1 failed UPI refund overdue · bank owes ₹500")).toBeInTheDocument();
  });

  it("falls back to the payday range when nothing is overdue", () => {
    withData({ ...fixture, refunds: [] }, "nudge");
    expect(screen.getByText(/likely to go before payday in 12 days/)).toBeInTheDocument();
  });

  it("renders nothing when there is nothing to say", () => {
    const { container } = withData({ ...empty, spend_to_payday: null }, "nudge");
    expect(container).toBeEmptyDOMElement();
  });
});
