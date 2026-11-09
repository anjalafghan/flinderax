import { describe, expect, it } from "bun:test";
import { render, screen, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { ForecastPanel } from "@/components/feature/ForecastPanel";
import { forecastKey, type Forecast, type GoalForecast } from "@/services/forecast";

// Synthetic figures only: made-up goals and amounts.

const goal = (over: Partial<GoalForecast>): GoalForecast => ({
  key: "wedding",
  label: "Wedding",
  when: "2028-02-10",
  target_paise: 2_700_000_00,
  saved_paise: 50_000_00,
  needed_monthly_paise: 9_000_000,
  funded_monthly_paise: 9_000_000,
  likelihood_bps: 8249,
  p10_paise: 2_400_000_00,
  p50_paise: 2_750_000_00,
  p90_paise: 3_100_000_00,
  status: "likely",
  levers: [],
  ...over,
});

const path = [
  { year: 2026, p10_paise: 100_000_00, p50_paise: 120_000_00, p90_paise: 140_000_00 },
  { year: 2027, p10_paise: 300_000_00, p50_paise: 340_000_00, p90_paise: 380_000_00 },
  { year: 2028, p10_paise: 600_000_00, p50_paise: 700_000_00, p90_paise: 800_000_00 },
];

const fixture: Forecast = {
  basis: {
    source: "statements",
    months_used: 6,
    income_paise: 14_000_000,
    spend_paise: 9_000_000,
    invested_paise: 2_000_000,
    surplus_paise: 5_000_000,
    surplus_sd_paise: 400_000,
    entered_income_paise: 15_000_000,
    note: null,
  },
  goals: [
    goal({}),
    goal({
      key: "child_education",
      label: "Child's education",
      when: "2047-06-01",
      likelihood_bps: 3120,
      p10_paise: 1_000_000_00,
      p50_paise: 2_000_000_00,
      p90_paise: 3_500_000_00,
      target_paise: 4_000_000_00,
      needed_monthly_paise: 4_000_000,
      funded_monthly_paise: 1_500_000,
      status: "unlikely",
      levers: [
        { kind: "save_more", text: "Put ₹6,200 more a month towards it to make it likely.", monthly_paise: 620_000, months: null, target_paise: null },
        { kind: "trim_target", text: "A ₹3,00,000 budget instead of ₹4,00,000 would be likely.", monthly_paise: null, months: null, target_paise: 3_000_000_00 },
      ],
    }),
  ],
  path,
  summary: "1 of 2 goals looks likely on what you save today. The Child's education is the hardest.",
  saved_profile: true,
};

function withData(data: Forecast) {
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(forecastKey, data);
  return render(
    <QueryClientProvider client={qc}>
      <ForecastPanel />
    </QueryClientProvider>,
  );
}

describe("ForecastPanel", () => {
  it("shows the title, the summary and the statement basis in whole rupees", () => {
    withData(fixture);
    const panel = screen.getByRole("region", { name: "Will you get there?" });
    expect(panel).toHaveTextContent(fixture.summary);
    expect(panel).toHaveTextContent(
      "Based on 6 months of statements: about ₹1,40,000 in, ₹90,000 out, ₹50,000 left each month.",
    );
  });

  it("uses the plan's numbers as the basis when there are not enough statements", () => {
    withData({
      ...fixture,
      basis: { ...fixture.basis, source: "profile", months_used: 0, note: null },
    });
    const panel = screen.getByRole("region", { name: "Will you get there?" });
    expect(panel).toHaveTextContent(
      "Based on your plan's numbers. Upload statements for a forecast from what really happens.",
    );
    expect(panel).not.toHaveTextContent("months of statements");
  });

  it("shows the basis note when the statements and the plan differ", () => {
    withData({
      ...fixture,
      basis: {
        ...fixture.basis,
        note: "Your statements show ₹1,40,000 a month coming in; your plan says ₹1,55,000.",
      },
    });
    expect(screen.getByText("Your statements show ₹1,40,000 a month coming in; your plan says ₹1,55,000.")).toBeInTheDocument();
  });

  it("says when the last check-in tempered the forecast", () => {
    const line = "Since your check-in on 10 Aug 2026, about 50% of the money you had left each month reached your goals. This forecast assumes the same.";
    withData({ ...fixture, basis: { ...fixture.basis, follow_through_bps: 5_000, calibration_note: line } });
    expect(screen.getByText(line)).toBeInTheDocument();
  });

  it("gives each goal a meter with its whole-number likelihood", () => {
    withData(fixture);
    const wedding = screen.getByRole("meter", { name: "Chance of reaching Wedding on time" });
    expect(wedding).toHaveAttribute("aria-valuenow", "82");
    expect(wedding).toHaveAttribute("aria-valuemin", "0");
    expect(wedding).toHaveAttribute("aria-valuemax", "100");
    expect(wedding).toHaveAttribute("aria-valuetext", "82% likely");
    expect(screen.getByText("82% likely")).toBeInTheDocument();

    const education = screen.getByRole("meter", { name: "Chance of reaching Child's education on time" });
    expect(education).toHaveAttribute("aria-valuenow", "31");
    expect(screen.getByText("31% likely")).toHaveClass("tone-over");
    expect(screen.getByText("82% likely")).toHaveClass("tone-good");
  });

  it("shows the date, the likely range, the target and the monthly funding for each goal", () => {
    withData(fixture);
    const wedding = screen.getByText("Wedding").closest("li");
    expect(wedding).not.toBeNull();
    const row = within(wedding as HTMLElement);
    expect(row.getByText("10 Feb 2028")).toBeInTheDocument();
    expect(row.getByText("Likely between ₹24,00,000 and ₹31,00,000 (target ₹27,00,000).")).toBeInTheDocument();
    expect(row.getByText("Getting ₹90,000 of the ₹90,000 a month it needs.")).toBeInTheDocument();
  });

  it("says when a goal gets more than it needs", () => {
    withData({ ...fixture, goals: [{ ...fixture.goals[0], funded_monthly_paise: 12_000_000, needed_monthly_paise: 9_000_000 }] });
    expect(screen.getByText("Gets ₹1,20,000 a month, more than the ₹90,000 it needs.")).toBeInTheDocument();
  });

  it("lists the levers only for goals that are not likely, and rounds paise away", () => {
    withData({
      ...fixture,
      goals: [
        goal({ likelihood_bps: 8000, needed_monthly_paise: 9_000_050, funded_monthly_paise: 9_000_050 }),
        fixture.goals[1],
      ],
    });
    const list = screen.getByText("Put ₹6,200 more a month towards it to make it likely.").closest("ul");
    expect(list).not.toBeNull();
    expect(within(list as HTMLElement).getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("A ₹3,00,000 budget instead of ₹4,00,000 would be likely.")).toBeInTheDocument();
    expect(screen.getByText("Getting ₹90,001 of the ₹90,001 a month it needs.")).toBeInTheDocument();
  });

  it("shows no levers under a goal that is already likely", () => {
    const { container } = withData({ ...fixture, goals: [goal({})] });
    expect(container.querySelector("li ul")).toBeNull();
    expect(screen.queryByText(/would be likely/)).toBeNull();
  });

  it("draws the fan chart with a text summary of the last year", () => {
    withData(fixture);
    const chart = screen.getByRole("img", {
      name: /Forecast of your goal pots from 2026 to 2028\. In 2028 the middle case is ₹7,00,000/,
    });
    expect(chart).toBeInTheDocument();
    expect(chart.getAttribute("aria-label")).toContain("₹6,00,000 and ₹8,00,000");
    expect(chart.querySelectorAll("polygon")).toHaveLength(1);
    expect(chart.querySelector("polyline")).not.toBeNull();
    expect(within(chart).getByText("2026")).toBeInTheDocument();
    expect(within(chart).getByText("2028")).toBeInTheDocument();
  });

  it("leaves out the chart when there is no path", () => {
    withData({ ...fixture, path: [] });
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("nudges towards the life plan when the details are not saved, and still shows the emergency fund", () => {
    withData({
      ...fixture,
      saved_profile: false,
      goals: [goal({ key: "emergency", label: "Emergency fund", when: null, likelihood_bps: 10000, p10_paise: 6_000_000, p50_paise: 6_000_000, p90_paise: 6_000_000, target_paise: 6_000_000 })],
    });
    expect(screen.getByText("Add your goals below for a forecast of them.")).toBeInTheDocument();
    expect(screen.getByText("Emergency fund")).toBeInTheDocument();
    expect(screen.getByText("Now")).toBeInTheDocument();
    expect(screen.getByText("100% likely")).toBeInTheDocument();
  });

  it("gives a fully saved goal a meter at 100", () => {
    withData({ ...fixture, goals: [goal({ likelihood_bps: 10000 })] });
    expect(screen.getByRole("meter")).toHaveAttribute("aria-valuenow", "100");
  });
});
