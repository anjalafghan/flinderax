import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { toast } from "sonner";

import HealthPage from "@/pages/HealthPage";
import api from "@/services/api";
import { jsonResponse, marksFor, overview, renderWithProviders, respond, stubFetch } from "../components/health/fixture";

// Synthetic data only. The page reads one diet overview and the three mark kinds.

const serve = (data = overview) =>
  spyOn(api, "get").mockImplementation((url: string) => {
    if (url === "/api/diet") return respond(data);
    if (url === "/api/diet/weeks/2026-10-26") return respond({ week_start: "2026-10-26", feel: 3, note: "", entries: [{ habit_id: "h2", days_hit: 3 }] });
    if (url.startsWith("/api/diet/weeks/")) return respond({ week_start: url.slice(-10), feel: null, note: "", entries: [] });
    if (url.startsWith("/api/diet/marks")) {
      const kind = new URL(url, "http://x").searchParams.get("kind") ?? "";
      return respond({ marks: marksFor(kind) });
    }
    return respond({});
  });

describe("HealthPage", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("shows the title and the four cards once the overview loads", async () => {
    serve();
    renderWithProviders(<HealthPage />);
    expect(screen.getByRole("heading", { level: 1, name: "Health" })).toBeInTheDocument();
    expect(screen.getByText("Food, a few habits and a yearly check-up. Nothing you have to log every day.")).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "This week" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Your habits" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Food money" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Body" })).toBeInTheDocument();
  });

  it("logs this week: the stepper and feel go out in a PUT, and the page says it saved", async () => {
    serve();
    const put = stubFetch(() => jsonResponse({ ...overview.week }));
    const success = spyOn(toast, "success");
    renderWithProviders(<HealthPage />);

    expect(await screen.findByText("2 of 5 days")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "More days for Home-cooked dinner" }));
    expect(screen.getByText("3 of 5 days")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("radio", { name: "Great" }));
    fireEvent.click(screen.getByRole("button", { name: "Save week" }));

    await waitFor(() => expect(success).toHaveBeenCalledWith("Week saved."));
    const [url, init] = put.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/diet/weeks/2026-11-02");
    expect(init.method).toBe("PUT");
    expect(JSON.parse(String(init.body))).toEqual({
      feel: 5,
      note: "",
      prep_hours_x10: null,
      entries: [
        { habit_id: "h1", days_hit: 3 },
        { habit_id: "h2", days_hit: 0 },
      ],
    });
  });

  it("keeps the stepper within 0 to 7 and the target label honest", async () => {
    serve({ ...overview, week: { ...overview.week, entries: [{ habit_id: "h1", days_hit: 7 }] } });
    renderWithProviders(<HealthPage />);
    expect(await screen.findByText("7 of 5 days")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "More days for Home-cooked dinner" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Fewer days for Walk 30 minutes" })).toBeDisabled();
  });

  it("back-fills a past week, starting from what was logged that week", async () => {
    serve();
    renderWithProviders(<HealthPage />);
    await screen.findByText("2 of 5 days");

    fireEvent.change(screen.getByLabelText("Log a past week"), { target: { value: "2026-10-26" } });
    expect(await screen.findByText("You're logging the week of 26 Oct.")).toBeInTheDocument();
    // That week had 3 walks and no home dinners logged.
    expect(await screen.findByText("3 of 4 days")).toBeInTheDocument();
    expect(screen.getByText("0 of 5 days")).toBeInTheDocument();
    expect(screen.queryByText("2 of 5 days")).toBeNull();
  });

  it("lists the last twelve weeks with logged state and the hit share", async () => {
    serve();
    renderWithProviders(<HealthPage />);
    await screen.findByText("Last 12 weeks");
    const recent = screen.getByText("Last 12 weeks").parentElement as HTMLElement;
    expect(within(recent).getAllByText("Logged")).toHaveLength(6);
    expect(within(recent).getAllByText("Not logged")).toHaveLength(6);
    expect(within(recent).getByText("Week of 2 Nov")).toBeInTheDocument();
  });

  it("asks for habits when none are tracked", async () => {
    serve({ ...overview, habits: [] });
    renderWithProviders(<HealthPage />);
    expect(await screen.findByText("Pick up to five habits to track each week.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add habit" })).toBeInTheDocument();
  });

  it("shows a retry when the overview can't be loaded, and loads again on retry", async () => {
    let fail = true;
    spyOn(api, "get").mockImplementation((url: string) => {
      if (url === "/api/diet" && fail) return Promise.reject(new Error("offline"));
      if (url === "/api/diet") return respond(overview);
      return respond({ marks: [] });
    });
    renderWithProviders(<HealthPage />);
    // The hook retries once, so the error shows after the backoff.
    expect(await screen.findByText("Couldn't load your health details.", undefined, { timeout: 3000 })).toBeInTheDocument();
    fail = false;
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByRole("heading", { name: "This week" })).toBeInTheDocument();
  });
});
