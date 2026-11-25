import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import AccountsPage from "./AccountsPage";

const account = (over: Record<string, unknown>) => ({
  account_id: "a1",
  name: "HDFC savings",
  kind: "bank",
  balance_paise: 1_250_000,
  locked: false,
  interest_bps: null,
  maturity_date: null,
  role: "spending",
  institution: null,
  renewal: "payout",
  payout_account_id: null,
  last_4_digits: "1234",
  updated_at: "2026-11-20T00:00:00Z",
  ...over,
});

const history = {
  months: [],
  cards: [],
  maturities: [],
  auto_postings: [],
  auto_post: false,
};

describe("AccountsPage", () => {
  test("splits bank accounts from deposits and totals the spendable ones", async () => {
    mockApi({
      "/api/account/list": [
        account({}),
        account({
          account_id: "a2",
          name: "Sample FD",
          kind: "fd",
          locked: true,
          role: "other",
          balance_paise: 5_000_000,
        }),
      ],
      "/api/dashboard/history": history,
    });
    renderPage(<AccountsPage />);
    expect((await screen.findAllByText("HDFC savings")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("Sample FD").length).toBeGreaterThan(0);
    expect(screen.getByLabelText("Remove HDFC savings")).toBeTruthy();
    expect(screen.queryByText("No accounts yet.")).toBeNull();
  });

  test("says so when there are no accounts or deposits", async () => {
    mockApi({ "/api/account/list": [], "/api/dashboard/history": history });
    renderPage(<AccountsPage />);
    expect(await screen.findByText("No accounts yet.")).toBeTruthy();
    expect(screen.getByText("No deposits yet.")).toBeTruthy();
  });

  test("holds a placeholder while loading", () => {
    mockApi({ "/api/account/list": pending, "/api/dashboard/history": pending });
    renderPage(<AccountsPage />);
    expect(screen.getByRole("heading", { name: "Accounts" })).toBeTruthy();
    expect(document.querySelector(".animate-pulse")).toBeTruthy();
    expect(screen.queryByText("No accounts yet.")).toBeNull();
  });

  test("reports a failed load", async () => {
    mockApi({
      "/api/account/list": failWith(500),
      "/api/dashboard/history": history,
    });
    renderPage(<AccountsPage />);
    expect(await screen.findByText("Couldn't load accounts.")).toBeTruthy();
  });
});
