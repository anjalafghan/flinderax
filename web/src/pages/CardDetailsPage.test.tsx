import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import CardDetailsPage from "./CardDetailsPage";

const card = {
  card_id: "c1",
  card_name: "Everyday Card",
  card_bank: "HDFC",
  card_primary_color: [20, 40, 90],
  card_secondary_color: [60, 90, 160],
  last_4_digits: "4321",
  last_total_due: 12_500,
  last_delta: 500,
  statement_day: 5,
  due_day: 25,
  credit_limit_paise: 20_000_000,
  pay_from_account_id: null,
};

const breakdown = {
  card_id: "c1",
  card_name: "Everyday Card",
  pay_from_account_id: null,
  unbilled_net_paise: null,
  due_now_paise: 1_250_000,
  due_date: "2026-12-25",
  next_bill_est_paise: 0,
  next_statement_date: null,
  next_due_date: null,
  loan_not_due_paise: 0,
  last_updated: null,
  emis: [],
  pending_charges: [],
  stale_pending_paise: 0,
};

const at = { path: "/cards/c1", route: "/cards/:id" };

describe("CardDetailsPage", () => {
  test("shows the card and its history, newest first with the change from the one before", async () => {
    mockApi({
      "/api/card/get_card": card,
      "/api/card/breakdown": breakdown,
      "/api/card/history": {
        histories: [
          {
            transaction_id: "t1",
            total_due_input: 10_000,
            timestamp: "2026-10-05T10:00:00Z",
          },
          {
            transaction_id: "t2",
            total_due_input: 12_500,
            timestamp: "2026-11-05T10:00:00Z",
          },
        ],
      },
    });
    renderPage(<CardDetailsPage />, at);
    expect(await screen.findByText("Transaction history")).toBeTruthy();
    const rows = await screen.findAllByLabelText("Delete transaction");
    expect(rows.length).toBe(2);
    // The card endpoints speak rupees (legacy boundary), so 10,000 -> 12,500 is a rise of ₹2,500.
    expect(await screen.findByText(/\+\s*₹\s?2,500/)).toBeTruthy();
  });

  test("says there are no transactions when the history is empty", async () => {
    mockApi({
      "/api/card/get_card": card,
      "/api/card/breakdown": breakdown,
      "/api/card/history": { histories: [] },
    });
    renderPage(<CardDetailsPage />, at);
    expect(await screen.findByText("No transactions yet.")).toBeTruthy();
  });

  test("shows a loading message while the card loads", () => {
    mockApi({
      "/api/card/get_card": pending,
      "/api/card/breakdown": pending,
      "/api/card/history": pending,
    });
    renderPage(<CardDetailsPage />, at);
    expect(screen.getByText("Loading details...")).toBeTruthy();
  });

  test("says the card was not found when the lookup fails", async () => {
    mockApi({
      "/api/card/get_card": failWith(404, { error: "no such card" }),
      "/api/card/breakdown": failWith(404),
      "/api/card/history": failWith(404),
    });
    renderPage(<CardDetailsPage />, at);
    expect(await screen.findByText("Card not found")).toBeTruthy();
  });
});
