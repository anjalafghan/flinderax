import { beforeEach, describe, expect, it } from "bun:test";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { StreaksCard } from "@/components/feature/StreaksCard";
import type { StreaksOut } from "@/services/streaks";

function withStreaks(data: StreaksOut) {
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(["streaks"], data);
  return render(
    <QueryClientProvider client={qc}>
      <StreaksCard />
    </QueryClientProvider>,
  );
}

describe("StreaksCard", () => {
  beforeEach(() => localStorage.clear());

  it("shows each run with an encouraging line", () => {
    withStreaks({ logging: { current: 5, best: 9 }, saved: { current: 2, best: 2 }, dates: { current: 0, best: 3 } });
    const card = screen.getByRole("region", { name: "Your streaks" });
    expect(card).toHaveTextContent("5 days in a row");
    expect(card).toHaveTextContent("2 months in a row");
    expect(card).toHaveTextContent("Best: 3 months");
    expect(card).toHaveTextContent("Start a new run this month.");
  });

  it("leaves out the dates run outside a couple, and hides itself with nothing to cheer", () => {
    const { unmount } = withStreaks({ logging: { current: 1, best: 1 }, saved: { current: 0, best: 0 }, dates: null });
    expect(screen.queryByText("Dates")).toBeNull();
    unmount();
    withStreaks({ logging: { current: 0, best: 0 }, saved: { current: 0, best: 0 }, dates: null });
    expect(screen.queryByRole("region", { name: "Your streaks" })).toBeNull();
  });

  it("remembers that a milestone was celebrated, so it does not repeat", () => {
    withStreaks({ logging: { current: 7, best: 7 }, saved: { current: 0, best: 0 }, dates: null });
    expect(localStorage.getItem("streak-seen:logging:7")).toBe("1");
  });
});
