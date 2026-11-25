import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { RewardFundCard } from "@/components/feature/RewardFundCard";
import type { AreaScore, FundOut } from "@/services/rewards";

// Synthetic data only.
const area = (
  o: Partial<AreaScore> & Pick<AreaScore, "area" | "label">,
): AreaScore => ({
  weight_bps: 2000,
  raw: 10,
  target: 20,
  score_bps: 5000,
  tip: "Do a small thing.",
  ...o,
});

const fund: FundOut = {
  wallet_paise: 150000,
  paid_in_paise: 200000,
  spent_paise: 50000,
  max_payout_paise: 200000,
  fund_since: "2026-10",
  this_month: {
    month: "2026-11",
    days_left: 10,
    score: {
      areas: [
        area({
          area: "money",
          label: "Money",
          weight_bps: 4000,
          raw: 70,
          target: 66,
          score_bps: 10000,
        }),
        area({
          area: "health",
          label: "Health",
          raw: 0,
          score_bps: 0,
          tip: "Log this week's diet habits.",
        }),
        area({ area: "relationships", label: "Relationships" }),
        area({ area: "future", label: "Future" }),
      ],
      score_bps: 6000,
      earned_paise: 120000,
      saved_paise: 0,
      payout_paise: 0,
      held_back_paise: 120000,
      focus: "health",
    },
  },
  months: [
    {
      month: "2026-10",
      score_bps: 8000,
      earned_paise: 160000,
      saved_paise: 100000,
      payout_paise: 100000,
      detail: null,
    },
  ],
};

function view() {
  const qc = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false } },
  });
  qc.setQueryData(["rewards", "fund"], fund);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <RewardFundCard />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("reward fund card", () => {
  it("shows the wallet, this month's areas and the tip for the weakest one", () => {
    view();
    expect(screen.getByText("₹1,500")).toBeTruthy();
    expect(screen.getAllByRole("progressbar")).toHaveLength(4);
    expect(screen.getByText("Log this week's diet habits.")).toBeTruthy();
    expect(screen.getByText(/nothing is saved yet/)).toBeTruthy();
    expect(screen.getByText("10 days left")).toBeTruthy();
    expect(screen.getByText("Past months (1)")).toBeTruthy();
    expect(
      screen
        .getByRole("link", { name: "Spend it on a treat" })
        .getAttribute("href"),
    ).toBe("/cards#rewards");
  });
});
