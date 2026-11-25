import { beforeEach, describe, expect, test } from "bun:test";
import { fireEvent, screen } from "@testing-library/react";

import { failWith, mockApi, renderPage } from "@/test/page";
import WhatIfPage from "./WhatIfPage";

const point = (month: string, spending: number) => ({
  month,
  spending_paise: spending,
  reserve_paise: 1_000_000,
  fds_paise: 0,
  rds_paise: 0,
  invested_paise: 0,
  total_paise: spending + 1_000_000,
});

const projection = (stress: unknown[] = []) => ({
  baseline: {
    points: [
      point("2026-12", 500_000),
      point("2027-01", 400_000),
      point("2027-02", 300_000),
    ],
    stress,
  },
  scenario: null,
  impacts: [],
});

const answers = (over: Record<string, unknown> = {}) => ({
  "/api/account/list": [],
  "/api/plan/scenarios": { drafts: [] },
  "/api/plan/project": projection(),
  ...over,
});

beforeEach(() => {
  try {
    localStorage.clear();
  } catch {
    /* no storage in this environment */
  }
});

describe("WhatIfPage", () => {
  test("starts empty, and says no month falls short when the projection is clean", async () => {
    mockApi(answers());
    renderPage(<WhatIfPage />);
    expect(screen.getByRole("heading", { name: "What if..." })).toBeTruthy();
    expect(await screen.findByText(/Nothing to try yet/)).toBeTruthy();
    expect(await screen.findByText("No month falls short in this window.")).toBeTruthy();
  });

  test("names the months that fall short", async () => {
    mockApi(
      answers({
        "/api/plan/project": projection([
          { month: "2027-01", kind: "spending_short", amount_paise: 120_000 },
          { month: "2027-02", kind: "spending_short", amount_paise: 250_000 },
        ]),
      }),
    );
    renderPage(<WhatIfPage />);
    expect(await screen.findByText(/in 2 of 2 months/)).toBeTruthy();
    expect(screen.queryByText("No month falls short in this window.")).toBeNull();
  });

  test("adding a scenario item shows its form and hides the empty note", async () => {
    mockApi(answers());
    renderPage(<WhatIfPage />);
    await screen.findByText(/Nothing to try yet/);
    fireEvent.click(screen.getByRole("button", { name: /Recurring deposit/ }));
    expect(screen.queryByText(/Nothing to try yet/)).toBeNull();
  });

  test("a failed projection says so instead of drawing a chart", async () => {
    mockApi(answers({ "/api/plan/project": failWith(500) }));
    renderPage(<WhatIfPage />);
    expect(await screen.findByText("Couldn't compute the projection.")).toBeTruthy();
  });
});
