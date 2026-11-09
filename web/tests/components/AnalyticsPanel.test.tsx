import { describe, expect, it } from "bun:test";
import { render, screen, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { AnalyticsPanel } from "@/components/feature/AnalyticsPanel";
import type { AnalyticsOut } from "@/services/analytics";

// Synthetic figures only: made-up payees and amounts.

const MONTH_KEYS = [
  "2025-12",
  "2026-01",
  "2026-02",
  "2026-03",
  "2026-04",
  "2026-05",
  "2026-06",
  "2026-07",
  "2026-08",
  "2026-09",
  "2026-10",
  "2026-11",
];

const months = MONTH_KEYS.map((month) => ({
  month,
  income_paise: 9_000_000,
  spend_paise: 5_000_000,
  invested_paise: 500_000,
  transfers_paise: 0,
  refunds_paise: 0,
  net_paise: 3_500_000,
}));

const fixture: AnalyticsOut = {
  coverage: { first_date: "2026-04-02", last_date: "2026-11-08", imports: 3 },
  months,
  savings_rate_bps: 4100,
  categories: [
    { category: "food", paise: 300_000, share_bps: 3000, count: 12 },
    { category: "groceries", paise: 250_000, share_bps: 2500, count: 8 },
    { category: "shopping", paise: 150_000, share_bps: 1500, count: 4 },
    { category: "transport", paise: 100_000, share_bps: 1000, count: 10 },
    { category: "fuel", paise: 60_000, share_bps: 600, count: 3 },
    { category: "bills", paise: 40_000, share_bps: 400, count: 2 },
    { category: "entertainment", paise: 20_000, share_bps: 200, count: 1 },
  ],
  top_payees: [
    { name: "ACME FOODS PVT LTD", paise: 120_000, count: 2, kind: "merchant" },
    { name: "CORNER TEA", paise: 60_000, count: 3, kind: "shop" },
    { name: "RAVI KUMAR", paise: 30_000, count: 1, kind: "person" },
  ],
  small_ticket: { per_month_count: 3, per_month_paise: 45_000, threshold_paise: 20_000 },
  salary: {
    typical_days: [1, 29],
    usual: "Month-end (29th–1st)",
    avg_paise: 9_000_000,
    last_paise: 9_000_000,
    last_on: "2026-10-29",
    missing_months: ["2026-09"],
    next_expected: "2026-12-01",
  },
  sips: [{ label: "SIP ₹5,000", amount_paise: 500_000, usual_day: 10, months_seen: 3, last_on: "2026-10-11" }],
  autopay: [{ name: "STREAMCO", amount_paise: 59_900, last_on: "2026-10-03", count: 2 }],
  own_transfers: { count: 2, out_paise: 500_000, in_paise: 200_000 },
  frequent_small: [],
  weekday_spend_paise: [100_000, 0, 200_000, 0, 0, 0, 50_000],
  notes: ["Salary arrived between the 1st and the 29th.", "You kept 41% of your income over the last three full months."],
};

const empty: AnalyticsOut = {
  coverage: null,
  months: [],
  savings_rate_bps: null,
  categories: [],
  top_payees: [],
  small_ticket: { per_month_count: 0, per_month_paise: 0, threshold_paise: 20_000 },
  salary: null,
  sips: [],
  autopay: [],
  own_transfers: null,
  frequent_small: [],
  weekday_spend_paise: [0, 0, 0, 0, 0, 0, 0],
  notes: [],
};

function withData(data: AnalyticsOut) {
  const qc = new QueryClient();
  qc.setQueryData(["analytics"], data);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <AnalyticsPanel />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("AnalyticsPanel", () => {
  it("names the statement span in the header and shows the savings rate", () => {
    withData(fixture);
    const panel = screen.getByRole("region", { name: "Your money, from your statements" });
    expect(panel).toHaveTextContent("Based on statements from 2 Apr to 8 Nov");
    expect(panel).toHaveTextContent("41%");
    expect(panel).toHaveTextContent("of your income kept over the last three full months.");
  });

  it("draws the twelve-month chart with a text summary for screen readers", () => {
    withData(fixture);
    const chart = screen.getByRole("img", { name: "Money in and out, last 12 months" });
    expect(chart.children).toHaveLength(12);
    expect(chart.children[0]).toHaveAttribute("title", "Dec 2025: in ₹90,000, out ₹55,000");
  });

  it("shows the salary days, average, next expected date and the missing month", () => {
    withData(fixture);
    const salary = screen.getByRole("region", { name: "Salary" });
    expect(salary).toHaveTextContent("Month-end (29th–1st)");
    expect(salary).toHaveTextContent("₹90,000");
    expect(salary).toHaveTextContent("29 Oct");
    expect(salary).toHaveTextContent("1 Dec");
    expect(salary).toHaveTextContent("No salary in Sep 2026.");
  });

  it("lists SIPs with their usual day and the monthly total", () => {
    withData(fixture);
    const sips = screen.getByRole("region", { name: "SIPs" });
    expect(sips).toHaveTextContent("₹5,000 a month");
    expect(sips).toHaveTextContent("SIP ₹5,000");
    expect(sips).toHaveTextContent("debits around the 10th · 3 months");
  });

  it("labels merchants and regular shops, but not guessed people", () => {
    withData(fixture);
    const payees = screen.getByRole("region", { name: "Top payees" });
    expect(within(payees).getByText("Acme Foods Pvt Ltd")).toBeInTheDocument();
    expect(within(payees).getByText("Merchant")).toBeInTheDocument();
    expect(within(payees).getByText("Regular shop")).toBeInTheDocument();
    expect(within(payees).queryByText("Person")).toBeNull();
    expect(payees).toHaveTextContent("₹1,200");
  });

  it("uses the server's pay window for the salary card", () => {
    withData(fixture);
    expect(screen.getByRole("region", { name: "Salary" })).toHaveTextContent("Month-end (29th–1st)");
  });

  it("shows uncategorised spending as a prompt, not a bar", () => {
    withData({
      ...fixture,
      categories: [{ category: "uncategorized", paise: 500_000, share_bps: 5_000, count: 40 }, ...fixture.categories],
    });
    const categories = screen.getByRole("region", { name: "Where the money goes" });
    expect(categories).toHaveTextContent("₹5,000 across 40 payments (50%) isn't categorised yet.");
    expect(within(categories).getByRole("link", { name: "Categorise a few payees" })).toHaveAttribute("href", "/spending");
  });

  it("shows only the top six categories", () => {
    withData(fixture);
    const categories = screen.getByRole("region", { name: "Where the money goes" });
    expect(categories).toHaveTextContent("food");
    expect(categories).toHaveTextContent("₹3,000 · 30%");
    expect(categories).toHaveTextContent("bills");
    expect(categories).not.toHaveTextContent("entertainment");
  });

  it("shows AutoPay, the own-transfer line and the small-payment line in whole rupees", () => {
    withData(fixture);
    const autopay = screen.getByRole("region", { name: "AutoPay" });
    expect(autopay).toHaveTextContent("Streamco");
    expect(autopay).toHaveTextContent("₹599");
    expect(autopay).toHaveTextContent("2 payments · last 3 Oct");

    const small = screen.getByRole("region", { name: "Small payments and transfers" });
    expect(small).toHaveTextContent("₹450 a month goes on 3 payments under ₹200.");
    expect(small).toHaveTextContent("2 transfers to and from your own accounts: ₹5,000 out, ₹2,000 in.");
  });

  it("shows the weekday bars and the plain-sentence notes", () => {
    withData(fixture);
    expect(screen.getByRole("img", { name: "Spending by weekday, last 90 days" })).toBeInTheDocument();
    const notes = screen.getByRole("region", { name: "Notes" });
    expect(notes).toHaveTextContent("Salary arrived between the 1st and the 29th.");
  });

  it("hides the sections that have nothing to show", () => {
    withData({
      ...fixture,
      autopay: [],
      top_payees: [],
      own_transfers: null,
      small_ticket: { ...fixture.small_ticket, per_month_paise: 0 },
    });
    expect(screen.queryByRole("region", { name: "AutoPay" })).toBeNull();
    expect(screen.queryByRole("region", { name: "Top payees" })).toBeNull();
    expect(screen.queryByRole("region", { name: "Small payments and transfers" })).toBeNull();
    expect(screen.getByRole("region", { name: "SIPs" })).toBeInTheDocument();
  });

  it("shows the upload prompt with a link to Spending when there is no statement coverage", () => {
    withData(empty);
    expect(screen.getByText("Upload a statement to see this.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Go to Spending" })).toHaveAttribute("href", "/spending");
    expect(screen.queryByRole("img", { name: "Money in and out, last 12 months" })).toBeNull();
  });

  it("says the savings rate is unavailable when there is no full month of income", () => {
    withData({ ...fixture, savings_rate_bps: null });
    expect(screen.getByRole("region", { name: "Your money, from your statements" })).toHaveTextContent(
      "Savings rate needs a full month of income.",
    );
  });
});
