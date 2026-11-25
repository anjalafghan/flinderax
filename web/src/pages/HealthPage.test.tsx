import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import HealthPage from "./HealthPage";

const overview = {
  presets: [{ key: "walk", label: "Walk", target_days: 5 }],
  habits: [
    {
      id: "h1",
      label: "Walk after dinner",
      preset_key: "walk",
      target_days: 5,
      active: true,
    },
  ],
  week: {
    week_start: "2026-11-23",
    feel: null,
    note: "",
    entries: [{ habit_id: "h1", days_hit: 3 }],
  },
  recent: [],
  money: {
    months: [
      {
        month: "2026-11",
        eating_out_paise: 450_000,
        groceries_paise: 800_000,
        delivery_orders: 6,
      },
    ],
    eating_out_share_bps: 3600,
    delivery_per_week_x100: 150,
    cap_paise: null,
  },
  body: {
    weight_x10: 700,
    weight_on: "2026-11-01",
    prev_weight_x10: null,
    waist_x10: null,
    last_checkup_on: null,
    checkup_due_on: null,
  },
};

describe("HealthPage", () => {
  test("shows the habits, the food money and the body card", async () => {
    mockApi({ "/api/diet": overview, "/api/diet/marks": { marks: [] } });
    renderPage(<HealthPage />);
    expect(await screen.findByRole("heading", { name: "Health" })).toBeTruthy();
    expect((await screen.findAllByText(/Walk after dinner/)).length).toBeGreaterThan(0);
    expect((await screen.findAllByText(/₹4,500/)).length).toBeGreaterThan(0);
  });

  test("shows a skeleton while loading", () => {
    mockApi({ "/api/diet": pending });
    renderPage(<HealthPage />);
    expect(screen.getByLabelText("Loading your health")).toBeTruthy();
  });

  test("offers a retry when the overview fails", async () => {
    mockApi({ "/api/diet": failWith(500) });
    renderPage(<HealthPage />);
    expect(await screen.findByText(/Couldn't load your health details/)).toBeTruthy();
  });
});
