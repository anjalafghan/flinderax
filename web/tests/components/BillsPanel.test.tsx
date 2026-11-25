import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

type ToastOptions = { action?: { label: string; onClick: () => void } };
const toasts: { text: string; options?: ToastOptions }[] = [];
mock.module("sonner", () => ({
  toast: {
    success: (text: string, options?: ToastOptions) =>
      toasts.push({ text, options }),
    error: () => {},
    info: () => {},
  },
}));

import { BillsPanel } from "@/components/feature/BillsPanel";
import { keys } from "@/hooks/usePlanner";
import { planApi } from "@/services/api";
import type { Bill, BillsResponse } from "@/services/types";
import { localIsoDate } from "@/utils/planner";

const inDays = (n: number) => {
  const d = new Date();
  d.setDate(d.getDate() + n);
  return localIsoDate(d);
};

const bill = (
  label: string,
  kind: Bill["kind"],
  commitment_id: string | null,
  date = inDays(10),
): Bill => ({
  date,
  label,
  amount_paise: 400_000,
  kind,
  account_id: null,
  covered_paise: 0,
  commitment_id,
});

function withData(data: BillsResponse) {
  const qc = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false } },
  });
  qc.setQueryData([...keys.bills, 30], data);
  qc.setQueryData(keys.billSuggestions, []);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <BillsPanel />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("BillsPanel", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("offers Not this month only on a payment every few months, and puts it off", async () => {
    const putOff = spyOn(planApi, "putOff").mockResolvedValue({
      first_month: "2027-01",
      waiting_from: "2026-12",
    });
    const due = inDays(10);
    const month = new Date(`${due}T00:00:00Z`).toLocaleString("en-IN", {
      month: "long",
      timeZone: "UTC",
    });
    withData({
      bills: [
        bill("SIP", "monthly", "c-sip", due),
        bill("perfume", "periodic", "c-perfume", due),
      ],
      total_paise: 800_000,
      sinking_funds: [],
    });

    const buttons = screen.getAllByRole("button", { name: /^Not in / });
    expect(buttons).toHaveLength(1);
    expect(buttons[0]).toHaveTextContent(`Not in ${month}`);
    expect(buttons[0]).toHaveAccessibleName(
      `Not in ${month}: put perfume off a month`,
    );
    fireEvent.click(buttons[0]);
    await waitFor(() => expect(putOff).toHaveBeenCalledWith("c-perfume"));

    // The toast says where it went and offers Undo, which brings it back.
    const undo = spyOn(planApi, "undoPutOff").mockResolvedValue({
      first_month: "2026-12",
      waiting_from: null,
    });
    await waitFor(() =>
      expect(toasts.at(-1)?.text).toBe(
        "perfume moved to January. Nothing more is set aside until then.",
      ),
    );
    expect(toasts.at(-1)?.options?.action?.label).toBe("Undo");
    toasts.at(-1)?.options?.action?.onClick();
    expect(undo).toHaveBeenCalledWith("c-perfume");
  });
});
