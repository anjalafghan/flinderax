import { describe, expect, it } from "bun:test";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";

import { SwipeSorter } from "@/components/feature/SwipeSorter";
import type { Txn } from "@/services/types";

const txn = (id: string, merchant: string | null, category: string, direction: Txn["direction"] = "debit"): Txn => ({
  txn_id: id,
  occurred_on: "2026-11-05",
  direction,
  amount_paise: 25_000,
  merchant,
  category,
  card_id: null,
  account_id: null,
  source_name: null,
  last4: null,
  source: "sms",
  created_at: "2026-11-05 10:00:00",
  category_locked: false,
});

const sorted = [txn("s1", "Swiggy", "food"), txn("s2", "Swiggy", "food")];
const queue = [txn("a", "Swiggy", "uncategorized"), txn("b", "Mystery", "uncategorized"), txn("c", null, "uncategorized")];

function setup() {
  const calls: { id: string; category: string; remember: boolean }[] = [];
  let closed = false;
  render(
    <SwipeSorter
      queue={queue}
      all={[...sorted, ...queue]}
      onCategorise={(t, category, remember) => calls.push({ id: t.txn_id, category, remember })}
      onClose={() => (closed = true)}
    />,
  );
  return { calls, wasClosed: () => closed };
}

describe("SwipeSorter", () => {
  it("offers the category you used for this merchant and saves it with 'remember'", async () => {
    const { calls } = setup();
    expect(screen.getByText("0 sorted · 3 to go")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Use Food & dining" }));
    await waitFor(() => expect(calls).toEqual([{ id: "a", category: "food", remember: true }]));
    await waitFor(() => expect(screen.getByText("1 sorted · 2 to go")).toBeTruthy());
  });

  it("says so, instead of doing nothing, when there is nothing to suggest", async () => {
    setup();
    fireEvent.click(screen.getByRole("button", { name: "Later" })); // a -> back of the line
    await waitFor(() => expect(screen.getByText(/Mystery/)).toBeTruthy());
    expect(screen.getByRole("button", { name: "No suggestion" })).toBeDisabled();
    fireEvent.keyDown(screen.getByRole("group", { name: /Mystery/ }), { key: "ArrowRight" });
    expect(await screen.findByText(/Nothing to suggest/)).toBeTruthy();
  });

  it("a tapped chip saves that category instead of the suggestion", async () => {
    const { calls } = setup();
    fireEvent.click(screen.getByRole("button", { name: "Groceries" }));
    await waitFor(() => expect(calls).toEqual([{ id: "a", category: "groceries", remember: true }]));
  });

  it("finishes with a thank-you and a Done button", async () => {
    const { calls, wasClosed } = setup();
    const pickFirstChip = async (n: number) => {
      fireEvent.click(screen.getAllByRole("button", { name: "Food & dining" })[0]);
      await waitFor(() => expect(calls.length).toBe(n));
    };
    await pickFirstChip(1);
    await pickFirstChip(2);
    await pickFirstChip(3);
    expect(calls[2]).toEqual({ id: "c", category: "food", remember: false });
    expect(await screen.findByText("All sorted!")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    expect(wasClosed()).toBe(true);
  });
});

describe("SwipeSorter chips", () => {
  it("keep their order while you sort, so a tap never lands on a moved chip", async () => {
    render(
      <SwipeSorter
        queue={[txn("p", "Metro", "uncategorized"), txn("q", "Cafe", "uncategorized")]}
        all={[txn("p", "Metro", "uncategorized"), txn("q", "Cafe", "uncategorized")]}
        onCategorise={() => {}}
        onClose={() => {}}
      />,
    );
    const names = () => screen.getAllByRole("button").map((b) => b.textContent).slice(1, 7);
    const before = names();
    fireEvent.click(screen.getByRole("button", { name: "Transport" }));
    await waitFor(() => expect(screen.getByText("1 sorted · 1 to go")).toBeTruthy());
    expect(names()).toEqual(before);
  });
});
