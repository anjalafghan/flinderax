import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";

import { AutoPostings } from "@/components/feature/AutoPostings";
import { CardUtilization } from "@/components/feature/CardUtilization";
import { MaturityList } from "@/components/feature/MaturityList";
import { NetWorthPanel } from "@/components/feature/NetWorthPanel";
import type { HistoryCard, HistoryMonth } from "@/services/types";

const months: HistoryMonth[] = [
  { month: "2026-08", liquid_paise: 100_000, locked_paise: 500_000, card_debt_paise: 0, net_worth_paise: 600_000 },
  { month: "2026-09", liquid_paise: 150_000, locked_paise: 500_000, card_debt_paise: 20_000, net_worth_paise: 630_000 },
];

describe("NetWorthPanel", () => {
  it("shows the empty state when there is no history", () => {
    render(<NetWorthPanel months={[]} />);
    expect(screen.getByText("Net worth history starts building as you update balances.")).toBeInTheDocument();
  });

  it("shows current net worth and the change from last month", () => {
    render(<NetWorthPanel months={months} />);
    expect(screen.getByText("₹6,300")).toBeInTheDocument();
    expect(screen.getByText(/\+₹300 vs last month/)).toBeInTheDocument();
    expect(screen.getByText("Card debt")).toBeInTheDocument();
  });
});

describe("CardUtilization", () => {
  it("renders nothing without cards", () => {
    const { container } = render(<CardUtilization cards={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("asks for a credit limit when none is set", () => {
    const cards: HistoryCard[] = [
      { card_id: "a", name: "HDFC", outstanding_paise: 150_000, limit_paise: null, utilization_bps: null },
    ];
    render(<CardUtilization cards={cards} />);
    expect(screen.getByText("add a credit limit to see usage")).toBeInTheDocument();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });

  it("shows usage against the limit", () => {
    const cards: HistoryCard[] = [
      { card_id: "a", name: "HDFC", outstanding_paise: 123_400, limit_paise: 1_000_000, utilization_bps: 1234 },
    ];
    render(<CardUtilization cards={cards} />);
    expect(screen.getByText(/₹1,234 of ₹10,000/)).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "12");
  });
});

describe("MaturityList", () => {
  it("hides when nothing matures", () => {
    const { container } = render(<MaturityList items={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("lists upcoming maturities with days left", () => {
    render(
      <MaturityList
        items={[{ account_id: "x", name: "SBI FD", kind: "fd", balance_paise: 5_000_000, maturity_date: "2027-03-31", days_left: 149 }]}
      />,
    );
    expect(screen.getByText("SBI FD")).toBeInTheDocument();
    expect(screen.getByText(/in 149 days/)).toBeInTheDocument();
  });
});

describe("AutoPostings", () => {
  const items = [{ occurs_on: "2026-11-01", label: "Salary", account_name: "HDFC", amount_paise: 9_000_000 }];

  it("is hidden when auto-update is off", () => {
    const { container } = render(<AutoPostings items={items} enabled={false} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("shows signed amounts when on", () => {
    render(<AutoPostings items={items} enabled />);
    expect(screen.getByText("Salary")).toBeInTheDocument();
    expect(screen.getByText("+₹90,000")).toBeInTheDocument();
  });
});
