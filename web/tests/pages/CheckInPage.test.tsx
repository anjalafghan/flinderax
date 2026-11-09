import { afterEach, describe, expect, it, spyOn, mock } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { toast } from "sonner";

import CheckInPage from "@/pages/CheckInPage";
import { AuthContext } from "@/context/AuthContext";
import api from "@/services/api";
import {
  checkinHistoryKey,
  checkinKey,
  checkinReportKey,
  type CheckinHistory,
  type CheckinStatus,
  type QuarterReport,
} from "@/services/checkin";
import type { Habit } from "@/services/habits";

// Synthetic data only: made-up names and amounts.

const period = { start: "2026-08-09", end: "2026-11-09" };

const status: CheckinStatus = {
  last_done_on: "2026-08-08",
  done_today: false,
  next_due_on: "2026-11-12",
  status: "due_soon",
  period,
  uploads: [
    { kind: "account", id: "acc1", name: "Asha Savings", covered_to: "2026-10-31", needed: true },
    { kind: "card", id: "card1", name: "Acme Card", covered_to: null, needed: true },
  ],
  steps: [
    { key: "statements", title: "Upload your statements", detail: "2 of 2 need a newer statement.", done: false, count: 2 },
    { key: "balances", title: "Confirm your balances", detail: "1 account has an old balance.", done: false, count: 1 },
    { key: "payees", title: "Sort new payees", detail: "₹450 across 2 payees has no category.", done: true, count: 0 },
    { key: "report", title: "Read your quarter", detail: "Finishing the check-in saves this report.", done: false, count: 0 },
  ],
  estimates: [
    {
      account_id: "acc1",
      name: "Asha Savings",
      known_paise: 10_000_000,
      known_on: "2026-11-01",
      estimate_paise: 10_240_000,
      days_since: 8,
      basis: "Usual net ₹3,000 a month in, over 8 days since 1 Nov.",
    },
  ],
};

const habit: Habit = {
  key: "bank_statement:acc1",
  area: "data",
  title: "Upload your HDFC statement",
  why: "One upload every 3 months keeps every number real.",
  action: { label: "Upload", to: "/spending" },
  cadence: "quarterly",
  due_on: "2026-11-12",
  status: "due_soon",
  impact: "high",
};

const report: QuarterReport = {
  period,
  previous: { start: "2026-05-10", end: "2026-08-08" },
  totals: { income_paise: 25_000_000, spend_paise: 18_000_000, invested_paise: 3_000_000, savings_rate_bps: 2800 },
  prev: { income_paise: 24_000_000, spend_paise: 19_000_000, invested_paise: 2_000_000, savings_rate_bps: 2100 },
  categories: [
    { category: "uncategorized", paise: 2_500_000, prev_paise: 900_000, change_bps: 17_778 },
    { category: "food", paise: 1_820_000, prev_paise: 1_320_000, change_bps: 3_812 },
  ],
  wins: ["You kept 28% of your income, up from 21%."],
  watch: ["Food rose 38% to ₹18,200."],
  habits: [habit],
};

const history: CheckinHistory = {
  checkins: [
    {
      done_on: "2026-08-08",
      period: { start: "2026-05-10", end: "2026-08-08" },
      totals: { income_paise: 24_000_000, spend_paise: 19_000_000, invested_paise: 2_000_000, savings_rate_bps: 2100 },
    },
  ],
};

// The fake server: the page's refetches read `current`, so a finish is visible to them.
let current: { status: CheckinStatus; report: QuarterReport | Error; history: CheckinHistory };

const respond = (data: unknown) => Promise.resolve({ data, status: 200, statusText: "OK" }) as never;

function fakeServer() {
  spyOn(api, "get").mockImplementation((url: string) => {
    if (url === "/api/checkin") return respond(current.status);
    if (url === "/api/checkin/report") {
      return current.report instanceof Error ? Promise.reject(current.report) : respond(current.report);
    }
    if (url === "/api/checkin/history") return respond(current.history);
    if (url === "/api/ingest/holder-names") return respond({ names: [] });
    return respond([]);
  });
  // The card list is a protobuf request the statement widget makes; it isn't part of this page's data.
  spyOn(api, "getProtobuf").mockRejectedValue(new Error("offline"));
}

function renderPage(seed: { status: CheckinStatus; report?: QuarterReport | Error; history?: CheckinHistory }) {
  current = { status: seed.status, report: seed.report ?? report, history: seed.history ?? history };
  fakeServer();
  // staleTime Infinity keeps the seeded data from refetching on its own; invalidations still hit the fake server.
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false }, mutations: { retry: false } } });
  qc.setQueryData(checkinKey, seed.status);
  if (!(current.report instanceof Error)) qc.setQueryData(checkinReportKey, current.report);
  qc.setQueryData(checkinHistoryKey, current.history);
  return render(
    <QueryClientProvider client={qc}>
      <AuthContext.Provider value={{ isAuthenticated: true, token: "t", login: () => {}, logout: () => {}, isLoading: false }}>
        <MemoryRouter>
          <CheckInPage />
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  );
}

describe("CheckInPage", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("walks the four steps in order, with the statements, balances and payees each in their own section", () => {
    renderPage({ status });
    expect(screen.getByRole("heading", { level: 1, name: "Quarterly check-in" })).toBeInTheDocument();
    expect(screen.getByText("Covers 9 Aug to 9 Nov.")).toBeInTheDocument();

    const statements = screen.getByRole("region", { name: "Upload your statements" });
    expect(statements).toHaveTextContent("Asha Savings");
    expect(statements).toHaveTextContent("covered to 31 Oct");
    expect(statements).toHaveTextContent("Acme Card");
    expect(statements).toHaveTextContent("no statement yet");
    expect(within(statements).getAllByText("Upload a newer one")).toHaveLength(2);
    expect(statements).toHaveTextContent("2 to do");
    expect(screen.getByLabelText("Statement file")).toBeInTheDocument();

    const balances = screen.getByRole("region", { name: "Confirm your balances" });
    expect(balances).toHaveTextContent("Update balances");
    expect(balances).toHaveTextContent("Usual net ₹3,000 a month in, over 8 days since 1 Nov.");
    expect(within(balances).getByText("estimate")).toBeInTheDocument();
    expect(within(balances).getByText("₹1,02,400")).toBeInTheDocument();

    const payees = screen.getByRole("region", { name: "Sort new payees" });
    expect(payees).toHaveTextContent("Done");
    expect(payees.querySelector("a")).toHaveAttribute("href", "/spending/sort");
  });

  it("shows the quarter report: totals with the previous value, categories with change, wins, watch and habits", () => {
    renderPage({ status });
    const quarter = screen.getByRole("region", { name: "Read your quarter" });
    expect(quarter).toHaveTextContent("₹2,50,000");
    expect(quarter).toHaveTextContent("Before: 21%");
    expect(quarter).toHaveTextContent("Food");
    expect(quarter).toHaveTextContent("+38%");
    // Unsorted spend is left to the payees step, and percentages are whole.
    expect(quarter).not.toHaveTextContent(/uncategori[sz]ed/i);
    expect(quarter).not.toHaveTextContent("38.12%");
    expect(quarter).toHaveTextContent("You kept 28% of your income, up from 21%.");
    expect(quarter).toHaveTextContent("Food rose 38% to ₹18,200.");
    expect(quarter).toHaveTextContent("Upload your HDFC statement");
  });

  it("shows the finished state once the check-in is saved, with no finish button and the quarter still readable", () => {
    const saved: CheckinStatus = {
      ...status,
      status: "ok",
      last_done_on: "2026-11-09",
      done_today: true,
      next_due_on: "2027-02-08",
      period: { start: "2026-08-09", end: "2026-11-09" },
    };
    renderPage({ status: saved });
    expect(screen.getByText("Saved on 9 Nov. Covers 9 Aug to 9 Nov.")).toBeInTheDocument();
    expect(screen.getByText("This check-in is saved. Next one 8 Feb.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Finish check-in" })).toBeNull();
  });

  it("finishes the check-in, then toasts the next date and shows the saved state", async () => {
    const next: CheckinStatus = {
      ...status,
      status: "ok",
      last_done_on: "2026-11-09",
      done_today: true,
      next_due_on: "2027-02-07",
      period: { start: "2026-08-09", end: "2026-11-09" },
    };
    const post = spyOn(api, "post").mockImplementation(() => {
      current.status = next;
      return respond(next);
    });
    const success = spyOn(toast, "success");
    renderPage({ status });

    fireEvent.click(screen.getByRole("button", { name: "Finish check-in" }));
    await waitFor(() => expect(success).toHaveBeenCalledWith("Check-in saved. Next one 7 Feb."));
    expect(post).toHaveBeenCalledWith("/api/checkin/complete", {});
    expect(await screen.findByText("Saved on 9 Nov. Covers 9 Aug to 9 Nov.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Finish check-in" })).toBeNull();
  });

  it("shows a retry when the report can't be loaded", async () => {
    renderPage({ status, report: new Error("offline") });
    const quarter = screen.getByRole("region", { name: "Read your quarter" });
    // The hook retries once, so the error shows after the backoff.
    expect(await within(quarter).findByText("Couldn't load your quarter.", undefined, { timeout: 3000 })).toBeInTheDocument();
    expect(within(quarter).getByRole("button", { name: "Try again" })).toBeInTheDocument();
  });

  it("lists past check-ins, and says so when there are none", () => {
    renderPage({ status });
    expect(screen.getByRole("region", { name: "Past check-ins" })).toHaveTextContent("Saved 8 Aug");
    expect(screen.getByRole("region", { name: "Past check-ins" })).toHaveTextContent("Covers 10 May to 8 Aug");
    cleanup();

    renderPage({ status, history: { checkins: [] } });
    expect(screen.getByText("Your finished check-ins will show here.")).toBeInTheDocument();
  });
});
