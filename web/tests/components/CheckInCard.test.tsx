import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

const toastSuccess = mock(() => {});
const toastError = mock(() => {});
mock.module("sonner", () => ({ toast: { success: toastSuccess, error: toastError, info: () => {} } }));

import { CheckInCard } from "@/components/feature/CheckInCard";
import api from "@/services/api";
import { checkinKey, type CheckinStatus } from "@/services/checkin";

// Synthetic data only.
const base: CheckinStatus = {
  last_done_on: "2026-08-08",
  done_today: false,
  next_due_on: "2026-11-07",
  status: "ok",
  period: { start: "2026-08-09", end: "2026-11-09" },
  uploads: [],
  steps: [],
  estimates: [],
};

const estimate = (name: string, paise: number, id: string) => ({
  account_id: id,
  name,
  known_paise: 1_000_000,
  known_on: "2026-11-01",
  estimate_paise: paise,
  days_since: 8,
  basis: "Usual net ₹12,400 a month in, over 8 days since 1 Nov.",
});

function withStatus(status: CheckinStatus) {
  // staleTime Infinity keeps the seeded data from refetching against the network.
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(checkinKey, status);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>
        <CheckInCard />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

const linkResponse = (url: string) => ({ data: { url }, status: 200, statusText: "OK" }) as never;

describe("CheckInCard", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
    toastSuccess.mockClear();
    toastError.mockClear();
  });

  it("says the next check-in date when the check-in is ok, and links to it", () => {
    withStatus(base);
    const card = screen.getByRole("region", { name: "Quarterly check-in" });
    expect(card).toHaveTextContent("Next check-in 7 Nov.");
    const link = screen.getByRole("link", { name: "Open check-in" });
    expect(link).toHaveAttribute("href", "/checkin");
  });

  it("says the check-in is due when overdue", () => {
    withStatus({ ...base, status: "overdue", next_due_on: "2026-10-01" });
    const card = screen.getByRole("region", { name: "Quarterly check-in" });
    expect(card).toHaveTextContent("Your check-in is due. It takes about 20 minutes.");
    expect(screen.getByRole("link", { name: "Start check-in" })).toHaveAttribute("href", "/checkin");
  });

  it("shows the due date when due soon", () => {
    withStatus({ ...base, status: "due_soon", next_due_on: "2026-11-12" });
    expect(screen.getByRole("region", { name: "Quarterly check-in" })).toHaveTextContent("Check-in due 12 Nov.");
  });

  it("shows at most two estimates, each labelled as an estimate", () => {
    withStatus({
      ...base,
      estimates: [estimate("Asha Savings", 1_240_000, "a1"), estimate("Acme Current", 560_000, "a2"), estimate("Third Account", 100_000, "a3")],
    });
    const card = screen.getByRole("region", { name: "Quarterly check-in" });
    expect(card).toHaveTextContent("Asha Savings: about ₹12,400 (estimate)");
    expect(card).toHaveTextContent("Acme Current: about ₹5,600 (estimate)");
    expect(card).not.toHaveTextContent("Third Account");
  });

  it("fetches the calendar link only on click, then shows it read-only with a copy button", async () => {
    const get = spyOn(api, "get").mockResolvedValue(linkResponse("https://example.test/api/public/reminders/synthetic.ics"));
    withStatus(base);
    expect(get).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Add reminders to your calendar" }));
    const input = await screen.findByLabelText("Your calendar link");
    expect(get).toHaveBeenCalledWith("/api/reminders/link");
    expect(input).toHaveAttribute("readonly");
    expect(input).toHaveValue("https://example.test/api/public/reminders/synthetic.ics");
    expect(screen.getByText(/Paste this into Google Calendar/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Copy link" })).toBeInTheDocument();
  });

  it("copies the calendar link and confirms it", async () => {
    spyOn(api, "get").mockResolvedValue(linkResponse("https://example.test/synthetic.ics"));
    const writeText = mock(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    withStatus(base);

    fireEvent.click(screen.getByRole("button", { name: "Add reminders to your calendar" }));
    fireEvent.click(await screen.findByRole("button", { name: "Copy link" }));
    await waitFor(() => expect(toastSuccess).toHaveBeenCalledWith("Link copied"));
    expect(writeText).toHaveBeenCalledWith("https://example.test/synthetic.ics");
  });

  it("shows a toast when the calendar link cannot be fetched", async () => {
    spyOn(api, "get").mockRejectedValue(new Error("offline"));
    withStatus(base);
    fireEvent.click(screen.getByRole("button", { name: "Add reminders to your calendar" }));
    await waitFor(() => expect(toastError).toHaveBeenCalledWith("Couldn't get the calendar link. Try again."));
    expect(screen.queryByLabelText("Your calendar link")).toBeNull();
  });
});
