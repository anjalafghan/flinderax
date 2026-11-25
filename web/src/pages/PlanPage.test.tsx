import { describe, expect, test } from "bun:test";
import { fireEvent, screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import PlanPage from "./PlanPage";

const income = {
  income_id: "i1",
  label: "Salary",
  amount_paise: 8_500_000,
  day_of_month: 30,
  account_id: null,
};

const commitment = {
  commitment_id: "c1",
  label: "Index fund SIP",
  amount_paise: 2_500_000,
  day_of_month: 5,
  account_id: null,
  category: "investments",
  bucket: "savings",
  frequency: "monthly",
  due_month: null,
  every_months: 1,
  first_month: null,
  to_account_id: null,
  active: true,
  start_month: null,
  end_month: null,
};

const answers = (over: Record<string, unknown> = {}) => ({
  "/api/account/list": [],
  "/api/plan/income/list": [income],
  "/api/plan/commitment/list": [commitment],
  ...over,
});

describe("PlanPage", () => {
  test("opens on this month and counts income and fixed payments in the section bar", async () => {
    mockApi(answers());
    renderPage(<PlanPage />, { path: "/plan" });
    expect(screen.getByRole("heading", { name: "Budget" })).toBeTruthy();
    const nav = screen.getByRole("navigation", { name: "Budget sections" });
    expect(nav.querySelector('[aria-current="true"]')?.textContent).toBe(
      "This month",
    );
    // The counts arrive with the queries.
    expect(await screen.findByRole("button", { name: /^Income\s*1$/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /^Fixed payments\s*1$/ })).toBeTruthy();
  });

  test("the hash picks the section, and the bar switches it", async () => {
    mockApi(answers());
    renderPage(<PlanPage />, { path: "/plan#income" });
    expect(await screen.findByText("Salary")).toBeTruthy();
    expect(screen.getByLabelText("Remove Salary")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^Fixed payments/ }));
    expect(await screen.findByText("Index fund SIP")).toBeTruthy();
  });

  test("says so when there is no income, but not while it is still loading", async () => {
    mockApi(answers({ "/api/plan/income/list": [] }));
    const first = renderPage(<PlanPage />, { path: "/plan#income" });
    expect(await screen.findByText(/No income yet/)).toBeTruthy();
    first.unmount();

    mockApi(answers({ "/api/plan/income/list": pending }));
    renderPage(<PlanPage />, { path: "/plan#income" });
    expect(screen.queryByText(/No income yet/)).toBeNull();
  });

  test("a failed income load never shows the empty-state copy", async () => {
    mockApi(answers({ "/api/plan/income/list": failWith(500) }));
    renderPage(<PlanPage />, { path: "/plan#income" });
    await screen.findByRole("heading", { name: "Budget" });
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByText(/No income yet/)).toBeNull();
  });
});
