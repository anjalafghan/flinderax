import { describe, expect, it } from "bun:test";
import { fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { HabitsCard } from "@/components/feature/HabitsCard";
import type { Habit } from "@/services/habits";

// Synthetic data only.
const habit = (overrides: Partial<Habit> & { key: string; title: string }): Habit => ({
  area: "data",
  why: "Covers up to 8 Nov. One upload every 3 months keeps every number real.",
  action: null,
  cadence: "quarterly",
  due_on: "2027-02-06",
  status: "ok",
  impact: "high",
  ...overrides,
});

const overdue = habit({
  key: "bank_statement:acc1",
  title: "Upload your HDFC statement",
  cadence: "quarterly",
  due_on: "2026-02-06",
  status: "overdue",
  action: { label: "Upload", to: "/spending" },
});

const dueSoon = habit({
  key: "card_statement:card1",
  area: "money",
  title: "Pay the card bill in full",
  cadence: "monthly",
  due_on: "2026-11-12",
  status: "due_soon",
});

const others = Array.from({ length: 4 }, (_, i) =>
  habit({ key: `food_budget_${i}`, area: "food", title: `Plan habit ${i + 1}`, cadence: "weekly", due_on: null, status: "ok" }),
);

function withHabits(habits: Habit[]) {
  // staleTime Infinity keeps the seeded data from refetching against the network.
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(["habits"], { habits });
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <HabitsCard />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("HabitsCard", () => {
  it("renders titles, the why line, status pill, due line and the action link", () => {
    withHabits([overdue, dueSoon]);
    const card = screen.getByRole("region", { name: "Your habits" });
    expect(card).toHaveTextContent("What to do and when, from your own numbers");
    expect(card).toHaveTextContent("Upload your HDFC statement");
    expect(card).toHaveTextContent("Covers up to 8 Nov.");
    expect(card).toHaveTextContent("Every 3 months · Overdue since 6 Feb");
    expect(card).toHaveTextContent("Pay the card bill in full");
    expect(card).toHaveTextContent("Monthly · Due 12 Nov");

    expect(screen.getByText("Overdue")).toBeInTheDocument();
    expect(screen.getByText("Due soon")).toBeInTheDocument();

    const link = screen.getByRole("link", { name: "Upload" });
    expect(link).toHaveAttribute("href", "/spending");
  });

  it("shows five habits, and 'Show all' reveals the rest", () => {
    withHabits([overdue, dueSoon, ...others]);
    // Six habits in total: the first five are shown, "Plan habit 4" (the sixth) is hidden.
    expect(screen.getByText("Plan habit 3")).toBeInTheDocument();
    expect(screen.queryByText("Plan habit 4")).toBeNull();

    const toggle = screen.getByRole("button", { name: "Show all 6" });
    fireEvent.click(toggle);
    expect(screen.getByText("Plan habit 3")).toBeInTheDocument();
    expect(screen.getByText("Plan habit 4")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Show fewer" })).toBeInTheDocument();
  });

  it("omits the toggle when there are five or fewer habits", () => {
    withHabits([overdue, dueSoon]);
    expect(screen.queryByRole("button", { name: /Show/ })).toBeNull();
  });

  it("hides the whole card when there are no habits", () => {
    const { container } = withHabits([]);
    expect(container).toBeEmptyDOMElement();
  });
});
