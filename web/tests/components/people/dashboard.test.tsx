import { afterEach, describe, expect, it, mock } from "bun:test";
import { cleanup, screen, waitFor } from "@testing-library/react";

import { UpcomingDatesCard } from "@/components/feature/people/UpcomingDatesCard";
import type { PeopleOverview, Upcoming } from "@/services/people";
import { NOT_FOUND, renderWithRouter, serve } from "./fake";

// Synthetic data only.

const upcoming = (over: Partial<Upcoming>): Upcoming => ({
  date_id: "d1",
  person_id: "p1",
  name: "Person A",
  kind: "birthday",
  label: "",
  on: "2026-11-20",
  days_away: 5,
  turns: null,
  gift_budget_paise: null,
  gift_status: "none",
  ...over,
});

const overview = (up: Upcoming[]): PeopleOverview => ({ people: [], archived: [], upcoming: up, catch_up: [] });

const settle = () => new Promise((r) => setTimeout(r, 20));

describe("UpcomingDatesCard", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("shows the next three dates within 14 days, soonest first, with the gift state", async () => {
    serve(() => ({
      json: overview([
        upcoming({ date_id: "a", name: "Person A", on: "2026-11-14", days_away: 1, turns: 60, gift_status: "bought" }),
        upcoming({ date_id: "b", person_id: "p2", name: "Aunt B", kind: "anniversary", on: "2026-11-18", days_away: 5 }),
        upcoming({ date_id: "c", person_id: "p3", name: "Person C", on: "2026-11-22", days_away: 13 }),
        upcoming({ date_id: "d", person_id: "p4", name: "Person D", on: "2026-12-01", days_away: 22 }),
      ]),
    }));
    renderWithRouter(<UpcomingDatesCard />, "/");

    expect(await screen.findByRole("heading", { name: "Coming up" })).toBeInTheDocument();
    expect(screen.getByText("Person A's birthday")).toBeInTheDocument();
    expect(screen.getByText("Sat 14 Nov · tomorrow · turns 60")).toBeInTheDocument();
    expect(screen.getByText("Bought")).toBeInTheDocument();
    expect(screen.getByText("Aunt B's anniversary")).toBeInTheDocument();
    expect(screen.getByText("Person C's birthday")).toBeInTheDocument();
    expect(screen.queryByText("Person D's birthday")).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Person A's birthday/ })).toHaveAttribute("href", "/people/p1");
    expect(screen.getByRole("link", { name: "All people" })).toHaveAttribute("href", "/people");
  });

  it("renders nothing when no date falls within 14 days", async () => {
    const calls = serve(() => ({ json: overview([upcoming({ days_away: 30 })]) }));
    const { container } = renderWithRouter(<UpcomingDatesCard />, "/");
    await waitFor(() => expect(calls.some((c) => c.url === "/api/people")).toBe(true));
    await settle();
    expect(container).toBeEmptyDOMElement();
  });

  it("renders nothing when the people request fails", async () => {
    const calls = serve(() => NOT_FOUND);
    const { container } = renderWithRouter(<UpcomingDatesCard />, "/");
    await waitFor(() => expect(calls.some((c) => c.url === "/api/people")).toBe(true));
    await settle();
    expect(container).toBeEmptyDOMElement();
  });

  it("tolerates an answer without an upcoming list", async () => {
    const calls = serve(() => ({ json: [] }));
    const { container } = renderWithRouter(<UpcomingDatesCard />, "/");
    await waitFor(() => expect(calls.some((c) => c.url === "/api/people")).toBe(true));
    await settle();
    expect(container).toBeEmptyDOMElement();
  });
});
