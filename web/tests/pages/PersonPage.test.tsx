import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { Route, Routes } from "react-router-dom";
import { toast } from "sonner";

import PersonPage from "@/pages/PersonPage";
import type { PersonDetail } from "@/services/people";
import { NOT_FOUND, renderWithRouter, serve, type Call, type Reply } from "../components/people/fake";

// Synthetic data only: Person A, fixed 2026 dates.

const detail: PersonDetail = {
  person: {
    id: "p1",
    name: "Person A",
    circle: "close",
    every_days: 28,
    last_contact_on: "2026-10-20",
    next_due_on: "2026-11-17",
    status: "ok",
    note: "Likes long walks",
    archived: false,
  },
  dates: [
    {
      id: "d1",
      person_id: "p1",
      kind: "birthday",
      label: "",
      month: 11,
      day: 14,
      year: 1966,
      remind_days: 7,
      gift_budget_paise: 200_000,
      next_on: "2026-11-14",
      days_away: 5,
      turns: 60,
    },
  ],
  gifts: [
    { id: "g1", person_id: "p1", date_id: "d1", for_year: 2026, status: "idea", title: "A book", amount_paise: null, on_day: null, txn_id: null },
  ],
  touches: [{ id: "t1", person_id: "p1", on_day: "2026-10-20", kind: "call", cost_paise: null, note: "" }],
};

function routes(extra: (c: Call) => Reply | undefined = () => undefined) {
  return (c: Call): Reply => {
    const custom = extra(c);
    if (custom) return custom;
    if (c.method === "GET" && c.url === "/api/people/p1") return { json: detail };
    if (c.method === "POST" && c.url === "/api/people/p1/touches") return { json: detail.person };
    if (c.method === "POST" && c.url === "/api/people/gifts") return { json: detail.gifts[0] };
    if (c.method === "POST" && c.url === "/api/people/p1/dates") return { json: detail.dates[0] };
    if (c.method === "PATCH") return { json: detail.person };
    if (c.method === "DELETE") return { status: 204 };
    return NOT_FOUND;
  };
}

// The page sits under a routes table so a delete can navigate somewhere real.
const ui = (
  <Routes>
    <Route path="/people/:id" element={<PersonPage />} />
    <Route path="/people" element={<p>Circle list</p>} />
  </Routes>
);

describe("PersonPage", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("shows the person, their dates, gifts and catch-ups", async () => {
    serve(routes());
    renderWithRouter(ui, "/people/p1");

    expect(await screen.findByRole("heading", { level: 1, name: "Person A" })).toBeInTheDocument();
    expect(screen.getByText("Close friends · every 4 weeks · Last caught up 20 Oct")).toBeInTheDocument();
    const dates = screen.getByRole("region", { name: "Dates" });
    expect(within(dates).getByText("Person A's birthday")).toBeInTheDocument();
    expect(within(dates).getByText("Sat 14 Nov · in 5 days · turns 60")).toBeInTheDocument();
    expect(within(dates).getByText("Gift budget ₹2,000")).toBeInTheDocument();

    const gifts = screen.getByRole("region", { name: "Gifts" });
    expect(within(gifts).getByText("A book")).toBeInTheDocument();
    expect(within(gifts).getByRole("combobox", { name: "Status for A book" })).toHaveValue("idea");

    const touches = screen.getByRole("region", { name: "Catch-ups" });
    expect(within(touches).getByText("Call · 20 Oct")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Circle" })).toHaveAttribute("href", "/people");
  });

  it("logs a one-tap catch-up with no day", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    fireEvent.click(await screen.findByRole("button", { name: "Caught up" }));

    await waitFor(() => expect(success).toHaveBeenCalledWith("Caught up with Person A."));
    expect(calls.find((c) => c.url === "/api/people/p1/touches")?.body).toEqual({ kind: "call" });
  });

  it("adds a gift for a date, linked to that date and its year", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    fireEvent.click(await screen.findByRole("button", { name: "Add a gift" }));

    const form = screen.getByRole("form", { name: "Add a gift" });
    fireEvent.change(within(form).getByLabelText("What is the gift?"), { target: { value: "Tea set" } });
    fireEvent.change(within(form).getByLabelText("For which date? (optional)"), { target: { value: "d1" } });
    fireEvent.change(within(form).getByLabelText("Amount (₹, optional)"), { target: { value: "1500" } });
    fireEvent.submit(form);

    await waitFor(() => expect(success).toHaveBeenCalledWith("Gift added."));
    expect(calls.find((c) => c.method === "POST" && c.url === "/api/people/gifts")?.body).toEqual({
      person_id: "p1",
      title: "Tea set",
      status: "idea",
      date_id: "d1",
      for_year: 2026,
      amount_paise: 150_000,
    });
  });

  it("moves a gift from idea to bought", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    const status = await screen.findByRole("combobox", { name: "Status for A book" });
    fireEvent.change(status, { target: { value: "bought" } });

    await waitFor(() => expect(success).toHaveBeenCalledWith("A book is now bought."));
    expect(calls.find((c) => c.method === "PATCH" && c.url === "/api/people/gifts/g1")?.body).toEqual({ status: "bought" });
  });

  it("deletes a catch-up by its own id", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    fireEvent.click(await screen.findByRole("button", { name: "Delete call on 20 Oct" }));

    await waitFor(() => expect(success).toHaveBeenCalledWith("Catch-up removed."));
    expect(calls.some((c) => c.method === "DELETE" && c.url === "/api/people/touches/t1")).toBe(true);
  });

  it("adds a yearly date with its day, month and gift budget", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    fireEvent.click(await screen.findByRole("button", { name: "Add a date" }));

    const form = screen.getByRole("form", { name: "Add a date" });
    fireEvent.change(within(form).getByLabelText("Month"), { target: { value: "3" } });
    fireEvent.change(within(form).getByLabelText("Day"), { target: { value: "22" } });
    fireEvent.change(within(form).getByLabelText("Remind me how many days before"), { target: { value: "10" } });
    fireEvent.change(within(form).getByLabelText("Gift budget (₹, optional)"), { target: { value: "2500" } });
    fireEvent.submit(form);

    await waitFor(() => expect(success).toHaveBeenCalledWith("Date added."));
    expect(calls.find((c) => c.method === "POST" && c.url === "/api/people/p1/dates")?.body).toEqual({
      kind: "birthday",
      label: "",
      month: 3,
      day: 22,
      remind_days: 10,
      gift_budget_paise: 250_000,
    });
  });

  it("archives the person with a PATCH that only sends the flag", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    fireEvent.click(await screen.findByRole("button", { name: "Archive" }));

    await waitFor(() => expect(success).toHaveBeenCalledWith("Person A is archived."));
    expect(calls.find((c) => c.method === "PATCH" && c.url === "/api/people/p1")?.body).toEqual({ archived: true });
  });

  it("saves edits, sending the rhythm in days and a blank rhythm as null", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(routes());
    renderWithRouter(ui, "/people/p1");
    const edit = (await screen.findByRole("heading", { name: "Edit" })).closest("section") as HTMLElement;
    fireEvent.change(within(edit).getByLabelText("Catch up every (weeks, blank for none)"), { target: { value: "" } });
    fireEvent.submit(within(edit).getByRole("button", { name: "Save" }).closest("form") as HTMLFormElement);

    await waitFor(() => expect(success).toHaveBeenCalledWith("Saved."));
    expect(calls.find((c) => c.method === "PATCH" && c.url === "/api/people/p1")?.body).toEqual({
      name: "Person A",
      circle: "close",
      every_days: null,
      note: "Likes long walks",
    });
  });

  it("deletes the person only after the confirm", async () => {
    const calls = serve(routes((c) => (c.method === "DELETE" && c.url === "/api/people/p1" ? { status: 204 } : undefined)));
    renderWithRouter(ui, "/people/p1");
    fireEvent.click(await screen.findByRole("button", { name: "Delete Person A" }));

    expect(screen.getByText("Delete Person A?")).toBeInTheDocument();
    expect(calls.some((c) => c.method === "DELETE")).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    expect(await screen.findByText("Circle list")).toBeInTheDocument();
    expect(calls.some((c) => c.method === "DELETE" && c.url === "/api/people/p1")).toBe(true);
  });

  it("says so when the person can't be found", async () => {
    serve(routes((c) => (c.url === "/api/people/p1" ? NOT_FOUND : undefined)));
    renderWithRouter(ui, "/people/p1");
    expect(await screen.findByText("Couldn't load this person.", undefined, { timeout: 4000 })).toBeInTheDocument();
  });
});
