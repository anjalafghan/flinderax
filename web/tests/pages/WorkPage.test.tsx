import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";

import WorkPage from "@/pages/WorkPage";
import { stepsThisWeek } from "@/pages/WeekPage";
import api from "@/services/api";
import type { Lead } from "@/services/work";
import { renderWithProviders, respond } from "../components/health/fixture";

// Synthetic data only: Company A, Company B.

const lead = (over: Partial<Lead>): Lead => ({
  id: "l1",
  company: "Company A",
  role: "Engineer",
  stage: "applied",
  take_home_paise: null,
  next_step: "",
  next_on: null,
  note: "",
  updated_on: "2026-11-01",
  ...over,
});

const isoIn = (days: number) => {
  const d = new Date();
  d.setDate(d.getDate() + days);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
};

describe("WorkPage", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("groups leads by stage and offers the life line only where a salary is known", async () => {
    const leads = [
      lead({ id: "l2", company: "Company B", stage: "offer", take_home_paise: 22_000_000, next_step: "Reply to the offer", next_on: "2026-11-20" }),
      lead({ id: "l1" }),
    ];
    spyOn(api, "get").mockImplementation(() => respond({ leads }));
    renderWithProviders(<WorkPage />);

    const offer = (await screen.findByRole("heading", { name: "Offer" })).closest("section") as HTMLElement;
    expect(within(offer).getByText("Company B")).toBeInTheDocument();
    expect(within(offer).getByText("Engineer · ₹2,20,000 a month · Next: Reply to the offer by 20 Nov")).toBeInTheDocument();
    expect(within(offer).getByRole("button", { name: "See it on my life line" })).toBeInTheDocument();

    const applied = screen.getByRole("heading", { name: "Applied" }).closest("section") as HTMLElement;
    expect(within(applied).getByText("Company A")).toBeInTheDocument();
    expect(within(applied).queryByRole("button", { name: "See it on my life line" })).toBeNull();
  });

  it("adds a lead with what was typed, and refuses one with no company", async () => {
    spyOn(api, "get").mockImplementation(() => respond({ leads: [] }));
    const post = spyOn(api, "post").mockImplementation(() => respond(lead({})));
    renderWithProviders(<WorkPage />);
    expect(await screen.findByText("No leads yet. Add the first company you would like to work for.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Add lead" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Company: fill this in.");
    expect(post).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText("Company"), { target: { value: " Company A " } });
    fireEvent.change(screen.getByLabelText("Take-home a month (₹)"), { target: { value: "2,20,000" } });
    fireEvent.click(screen.getByRole("button", { name: "Add lead" }));
    await waitFor(() =>
      expect(post).toHaveBeenCalledWith("/api/work", {
        company: "Company A",
        role: "",
        stage: "idea",
        take_home_paise: 22_000_000,
        next_step: "",
        next_on: null,
        note: "",
      }),
    );
  });
});

describe("stepsThisWeek", () => {
  it("keeps open leads whose step is late or within a week, soonest first", () => {
    const leads = [
      lead({ id: "far", next_on: isoIn(30) }),
      lead({ id: "soon", next_on: isoIn(3) }),
      lead({ id: "late", next_on: isoIn(-4) }),
      lead({ id: "closed", stage: "closed", next_on: isoIn(1) }),
      lead({ id: "none" }),
    ];
    expect(stepsThisWeek(leads).map((l) => l.id)).toEqual(["late", "soon"]);
  });
});
