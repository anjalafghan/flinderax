import { describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { DueSoonCard } from "@/components/feature/BillsPanel";
import { keys } from "@/hooks/usePlanner";
import type { BillsResponse } from "@/services/types";
import { localIsoDate } from "@/utils/planner";

const iso = (offset: number) => {
  const d = new Date();
  d.setDate(d.getDate() + offset);
  return localIsoDate(d);
};

function withData(data: BillsResponse) {
  const qc = new QueryClient();
  qc.setQueryData([...keys.bills, 30], data);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <DueSoonCard />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

const bill = (offset: number, label: string) => ({
  date: iso(offset),
  label,
  amount_paise: 49_900,
  kind: "monthly" as const,
  account_id: null,
  covered_paise: 0,
});

describe("DueSoonCard", () => {
  it("lists up to three bills due within a week, with a link to all bills", () => {
    withData({
      bills: [bill(1, "Gym"), bill(2, "Phone"), bill(3, "Internet"), bill(4, "Rent"), bill(20, "Far away")],
      total_paise: 0,
      sinking_funds: [],
    });
    const card = screen.getByLabelText("Due soon");
    expect(card).toHaveTextContent("Gym");
    expect(card).toHaveTextContent("Phone");
    expect(card).toHaveTextContent("Internet");
    expect(card).not.toHaveTextContent("Rent");
    expect(card).not.toHaveTextContent("Far away");
    expect(screen.getByRole("link", { name: "All bills" })).toHaveAttribute("href", "/plan#bills");
  });

  it("hides itself when nothing is due within a week", () => {
    withData({ bills: [bill(15, "Later")], total_paise: 0, sinking_funds: [] });
    expect(screen.queryByLabelText("Due soon")).toBeNull();
  });
});
