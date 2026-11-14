import { afterEach, describe, expect, it, mock } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

const calls: string[] = [];
const real = await import("../../src/joy/regulars/api");
mock.module("../../src/joy/regulars/api", () => ({
  ...real,
  setNightAmount: async (id: number, amount: string) => { calls.push(`amount:${id}:${amount}`); return { ok: true, to: "/" }; },
  deleteNight: async (id: number) => { calls.push(`delete:${id}`); return { ok: true, to: "/" }; },
  wentTo: async (id: number, body: { day: string; amount: string }) => { calls.push(`went:${id}:${body.day}:${body.amount}`); return { ok: true, to: "/" }; },
}));

const { EditAmount } = await import("../../src/joy/regulars/EditAmount");
const { PastNight } = await import("../../src/joy/regulars/PastNight");
const { RegularCard } = await import("../../src/joy/regulars/RegularCard");

const wrap = (ui: React.ReactElement) =>
  render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      <MemoryRouter>{ui}</MemoryRouter>
    </QueryClientProvider>,
  );

afterEach(() => {
  cleanup();
  calls.length = 0;
});

describe("fixing what a night cost", () => {
  it("refuses a blank or non-number, then saves a typed amount with its commas dropped", async () => {
    let saved = 0;
    wrap(<EditAmount visitId={7} initial={300} onSaved={() => saved++} onCancel={() => {}} />);
    const box = screen.getByLabelText("Spent") as HTMLInputElement;
    fireEvent.change(box, { target: { value: "" } });
    fireEvent.submit(box.closest("form")!);
    expect(await screen.findByRole("alert")).toHaveTextContent("number");
    fireEvent.change(box, { target: { value: "1,250" } });
    fireEvent.submit(box.closest("form")!);
    await waitFor(() => expect(saved).toBe(1));
    expect(calls).toEqual(["amount:7:1250"]);
  });
});

describe("logging a missed night", () => {
  it("needs a day, then adds the night on that day", async () => {
    wrap(<PastNight regularId={3} usualRaw={300} />);
    fireEvent.click(screen.getByRole("button", { name: "Add to Kept" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Pick the day");
    expect(calls).toEqual([]);
    fireEvent.change(screen.getByLabelText("The day"), { target: { value: "2026-07-04" } });
    fireEvent.click(screen.getByRole("button", { name: "Add to Kept" }));
    await waitFor(() => expect(calls).toEqual(["went:3:2026-07-04:300"]));
  });

  it("Reset puts the form back", () => {
    wrap(<PastNight regularId={3} usualRaw={300} />);
    const day = screen.getByLabelText("The day") as HTMLInputElement;
    fireEvent.change(day, { target: { value: "2026-07-04" } });
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(day.value).toBe("");
  });
});

describe("undoing a night", () => {
  const card = { show: true, open: [], ceiling: "", went: [{ visit_id: 9, title: "Blue Tokai", place: "", amount: "₹300", amount_raw: 300, history_id: 4, photo: "" }] };

  it("asks first, and only removes it when you say yes", async () => {
    const ask = mock(() => false);
    window.confirm = ask;
    wrap(<RegularCard card={card} onChange={() => {}} />);
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    expect(ask).toHaveBeenCalledTimes(1);
    expect(calls).toEqual([]);
    window.confirm = () => true;
    fireEvent.click(screen.getByRole("button", { name: "Undo" }));
    await waitFor(() => expect(calls).toEqual(["delete:9"]));
  });
});
