import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

mock.module("sonner", () => ({ toast: { success: () => {}, error: () => {}, info: () => {} } }));

import { CardSnapshotForm } from "@/components/feature/CardSnapshotForm";
import api from "@/services/api";

const renderForm = (isFirst: boolean) =>
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { mutations: { retry: false } } })}>
      <CardSnapshotForm cardId="card-1" isFirst={isFirst} />
    </QueryClientProvider>,
  );

const fill = () => {
  fireEvent.change(screen.getByLabelText("Total due"), { target: { value: "10,000" } });
  fireEvent.change(screen.getByLabelText("Outstanding"), { target: { value: "10000" } });
  fireEvent.click(screen.getByText("Save snapshot"));
};

describe("CardSnapshotForm", () => {
  afterEach(() => mock.restore());

  it("asks what the first snapshot is, and sends 'new spending' when chosen", async () => {
    const post = spyOn(api, "post").mockResolvedValue({ data: { status: true }, status: 200, statusText: "OK" } as never);
    renderForm(true);
    expect(screen.getByText("My starting balance.")).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText(/New spending/));
    fill();
    await waitFor(() => expect(post).toHaveBeenCalled());
    expect(post.mock.calls[0][0]).toBe("/api/card/snapshot");
    expect(post.mock.calls[0][1]).toMatchObject({
      card_id: "card-1",
      total_due_paise: 1_000_000,
      outstanding_paise: 1_000_000,
      starting_balance: false,
    });
  });

  it("defaults the first snapshot to a starting balance", async () => {
    const post = spyOn(api, "post").mockResolvedValue({ data: { status: true }, status: 200, statusText: "OK" } as never);
    renderForm(true);
    fill();
    await waitFor(() => expect(post).toHaveBeenCalled());
    expect(post.mock.calls[0][1]).toMatchObject({ starting_balance: true });
  });

  it("doesn't ask once the card already has snapshots", async () => {
    const post = spyOn(api, "post").mockResolvedValue({ data: { status: true }, status: 200, statusText: "OK" } as never);
    renderForm(false);
    expect(screen.queryByText("My starting balance.")).toBeNull();
    fill();
    await waitFor(() => expect(post).toHaveBeenCalled());
    expect("starting_balance" in (post.mock.calls[0][1] as object)).toBe(false);
  });

  it("rejects an invalid unbilled number", () => {
    renderForm(false);
    fireEvent.change(screen.getByLabelText("Total due"), { target: { value: "0" } });
    fireEvent.change(screen.getByLabelText("Outstanding"), { target: { value: "100" } });
    fireEvent.change(screen.getByLabelText(/Unbilled spent/), { target: { value: "abc" } });
    fireEvent.click(screen.getByText("Save snapshot"));
    expect(screen.getByRole("alert").textContent).toMatch(/plain numbers/);
  });
});
