import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { toast } from "sonner";

import PeoplePage from "@/pages/PeoplePage";
import type { GiftPlan, PeopleOverview, PersonRow, PeopleSettings, Upcoming } from "@/services/people";
import { NOT_FOUND, renderWithRouter, serve, type Call, type Reply } from "../components/people/fake";

// Synthetic data only: Person A, Aunt B, fixed 2026 dates.

const person = (over: Partial<PersonRow>): PersonRow => ({
  id: "p1",
  name: "Person A",
  circle: "family",
  every_days: 28,
  last_contact_on: "2026-10-20",
  next_due_on: "2026-11-17",
  status: "ok",
  note: "",
  archived: false,
  ...over,
});

const upcoming: Upcoming = {
  date_id: "d1",
  person_id: "p1",
  name: "Person A",
  kind: "birthday",
  label: "",
  on: "2026-11-14",
  days_away: 5,
  turns: 60,
  gift_budget_paise: 200_000,
  gift_status: "none",
};

const overview: PeopleOverview = {
  people: [
    person({ id: "p1", name: "Person A", circle: "family", status: "due_soon" }),
    person({ id: "p2", name: "Aunt B", circle: "friend", every_days: null, last_contact_on: null, next_due_on: null, status: null }),
  ],
  archived: [person({ id: "p3", name: "Person C", circle: "work", archived: true })],
  upcoming: [upcoming],
  catch_up: [person({ id: "p1", name: "Person A", circle: "family", status: "due_soon" })],
};

const months = Array.from({ length: 12 }, (_, i) => {
  const m = ((10 + i) % 12) + 1;
  const y = 2026 + Math.floor((10 + i) / 12);
  return { month: `${y}-${String(m).padStart(2, "0")}`, budget_paise: 100_000 + i * 1_000, occasions: 1 };
});

const plan = (over: Partial<GiftPlan> = {}): GiftPlan => ({
  months,
  total_paise: 1_400_000,
  monthly_paise: 116_667,
  given_this_year_paise: 48_000,
  goal_id: null,
  ...over,
});

const settings: PeopleSettings = { feed_names: false };

function standardRoutes(extra: (c: Call) => Reply | undefined = () => undefined) {
  return (c: Call): Reply => {
    const custom = extra(c);
    if (custom) return custom;
    if (c.method === "GET" && c.url === "/api/people") return { json: overview };
    if (c.method === "GET" && c.url === "/api/people/gift-plan") return { json: plan() };
    if (c.method === "GET" && c.url === "/api/people/seasons") return { json: { seasons: [] } };
    if (c.method === "GET" && c.url === "/api/people/settings") return { json: settings };
    if (c.method === "POST" && c.url === "/api/people/p1/touches") return { json: person({ id: "p1" }) };
    if (c.method === "POST" && c.url === "/api/people") return { json: person({ id: "p9", name: "Aunt D" }) };
    if (c.method === "POST" && c.url === "/api/goals/create") return { json: { goal_id: "g9", status: true } };
    if (c.method === "PUT" && c.url === "/api/people/settings") return { json: c.body };
    return NOT_FOUND;
  };
}

describe("PeoplePage", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("shows the upcoming dates, the catch-up list, the circle grouped by circle and the archived people", async () => {
    serve(standardRoutes());
    renderWithRouter(<PeoplePage />);

    expect(await screen.findByRole("heading", { level: 1, name: "Circle" })).toBeInTheDocument();
    const upcomingSection = screen.getByRole("region", { name: "Upcoming" });
    expect(within(upcomingSection).getByText("Person A's birthday")).toBeInTheDocument();
    expect(within(upcomingSection).getByText("No gift yet")).toBeInTheDocument();
    expect(within(upcomingSection).getByText("Sat 14 Nov · in 5 days · turns 60")).toBeInTheDocument();

    const catchUp = screen.getByRole("region", { name: "Catch up" });
    expect(within(catchUp).getByRole("link", { name: "Person A" })).toHaveAttribute("href", "/people/p1");
    expect(within(catchUp).getByText("Due soon")).toBeInTheDocument();

    const circle = screen.getByRole("region", { name: "Your people" });
    expect(within(circle).getByRole("heading", { level: 3, name: "Family" })).toBeInTheDocument();
    expect(within(circle).getByRole("heading", { level: 3, name: "Friends" })).toBeInTheDocument();
    expect(within(circle).getByText("Archived (1)")).toBeInTheDocument();
    expect(within(circle).getByText("No catch-up logged yet.")).toBeInTheDocument();
  });

  it("logs a one-tap catch-up as a call with no day, so the server picks today", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    fireEvent.click(await screen.findByRole("button", { name: "Caught up" }));

    await waitFor(() => expect(success).toHaveBeenCalledWith("Caught up with Person A."));
    const post = calls.find((c) => c.method === "POST" && c.url === "/api/people/p1/touches");
    expect(post?.body).toEqual({ kind: "call" });
  });

  it("opens the other-ways form and sends a visit with its cost", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    fireEvent.click(await screen.findByRole("button", { name: "Log another way" }));

    const form = screen.getByRole("form", { name: "Log a catch-up" });
    fireEvent.change(within(form).getByLabelText("How did you catch up?"), { target: { value: "visit" } });
    fireEvent.change(within(form).getByLabelText("What did the visit cost? (₹, optional)"), { target: { value: "450" } });
    fireEvent.change(within(form).getByLabelText("Note (optional)"), { target: { value: "Lunch" } });
    fireEvent.submit(form);

    await waitFor(() => expect(success).toHaveBeenCalledWith("Catch-up logged."));
    const post = calls.find((c) => c.method === "POST" && c.url === "/api/people/p1/touches");
    expect(post?.body).toEqual({ kind: "visit", cost_paise: 45_000, note: "Lunch" });
  });

  it("rejects a visit cost that is not an amount, without sending", async () => {
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    fireEvent.click(await screen.findByRole("button", { name: "Log another way" }));
    const form = screen.getByRole("form", { name: "Log a catch-up" });
    fireEvent.change(within(form).getByLabelText("How did you catch up?"), { target: { value: "visit" } });
    fireEvent.change(within(form).getByLabelText("What did the visit cost? (₹, optional)"), { target: { value: "lots" } });
    fireEvent.submit(form);

    expect(await within(form).findByText("Enter the cost as rupees, like 450 or 450.50.")).toBeInTheDocument();
    expect(calls.some((c) => c.method === "POST" && c.url.includes("/touches"))).toBe(false);
  });

  it("adds someone with a catch-up rhythm in weeks, sent as days", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    const section = await screen.findByRole("region", { name: "Add someone" });

    fireEvent.change(within(section).getByLabelText("Name"), { target: { value: "Aunt D" } });
    fireEvent.change(within(section).getByLabelText("Circle"), { target: { value: "close" } });
    fireEvent.change(within(section).getByLabelText("Catch up every (weeks, optional)"), { target: { value: "4" } });
    fireEvent.submit(within(section).getByRole("button", { name: "Add to circle" }).closest("form") as HTMLFormElement);

    await waitFor(() => expect(success).toHaveBeenCalledWith("Aunt D is in your circle."));
    const post = calls.find((c) => c.method === "POST" && c.url === "/api/people");
    expect(post?.body).toEqual({ name: "Aunt D", circle: "close", every_days: 28, note: "" });
  });

  it("sends no rhythm when the weeks are left blank", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    const section = await screen.findByRole("region", { name: "Add someone" });
    fireEvent.change(within(section).getByLabelText("Name"), { target: { value: "Aunt D" } });
    fireEvent.submit(within(section).getByRole("button", { name: "Add to circle" }).closest("form") as HTMLFormElement);

    await waitFor(() => expect(success).toHaveBeenCalledWith("Aunt D is in your circle."));
    const post = calls.find((c) => c.method === "POST" && c.url === "/api/people");
    expect(post?.body).toMatchObject({ every_days: null });
  });

  it("shows the gift plan and makes a Gifts goal from its total", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    const card = await screen.findByRole("region", { name: "Gift plan" });

    expect(await within(card).findByText("About ₹1,167 a month")).toBeInTheDocument();
    expect(within(card).getByRole("img", { name: /Gift budget by month/ })).toBeInTheDocument();
    fireEvent.click(await within(card).findByRole("button", { name: "Make a Gifts goal" }));

    await waitFor(() => expect(success).toHaveBeenCalledWith("Gifts goal made."));
    const post = calls.find((c) => c.method === "POST" && c.url === "/api/goals/create");
    expect(post?.body).toEqual({ name: "Gifts", kind: "custom", target_paise: 1_400_000 });
  });

  it("links to the plan instead of offering a second goal when one exists", async () => {
    serve(standardRoutes((c) => (c.url === "/api/people/gift-plan" ? { json: plan({ goal_id: "g9" }) } : undefined)));
    renderWithRouter(<PeoplePage />);
    const card = await screen.findByRole("region", { name: "Gift plan" });
    expect(await within(card).findByRole("link", { name: "Gifts goal set" })).toHaveAttribute("href", "/plan");
    expect(within(card).queryByRole("button", { name: "Make a Gifts goal" })).not.toBeInTheDocument();
  });

  it("hides the goal button when there is no gift budget at all", async () => {
    serve(standardRoutes((c) => (c.url === "/api/people/gift-plan" ? { json: plan({ total_paise: 0, monthly_paise: 0 }) } : undefined)));
    renderWithRouter(<PeoplePage />);
    const card = await screen.findByRole("region", { name: "Gift plan" });
    expect(within(card).queryByRole("button", { name: "Make a Gifts goal" })).not.toBeInTheDocument();
  });

  it("saves the calendar feed setting with names off or on", async () => {
    const calls = serve(standardRoutes());
    renderWithRouter(<PeoplePage />);
    const box = (await screen.findByLabelText("Show details in my calendar feed")) as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);

    await waitForCall(calls, "PUT", "/api/people/settings");
    expect(calls.find((c) => c.method === "PUT")?.body).toEqual({ feed_names: true });
  });

  it("offers a retry when the circle can't be loaded", async () => {
    const calls = serve(standardRoutes((c) => (c.url === "/api/people" ? NOT_FOUND : undefined)));
    renderWithRouter(<PeoplePage />);
    // The query retries once before it gives up, so allow for that.
    expect(await screen.findByText("Couldn't load your circle.", undefined, { timeout: 4000 })).toBeInTheDocument();
    const before = calls.filter((c) => c.url === "/api/people").length;
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    await waitForCount(calls, () => calls.filter((c) => c.url === "/api/people").length > before);
  });

  it("reports a failed catch-up with the server's words", async () => {
    const error = spyOn(toast, "error");
    serve(standardRoutes((c) => (c.method === "POST" && c.url === "/api/people/p1/touches" ? { status: 422, text: "Pick a day that has already come." } : undefined)));
    renderWithRouter(<PeoplePage />);
    fireEvent.click(await screen.findByRole("button", { name: "Caught up" }));
    await waitForCount(error.mock.calls, () => error.mock.calls.length > 0);
    expect(error).toHaveBeenCalledWith("Pick a day that has already come.");
  });

  it("links to the plan when a goal exists even though the total is zero", async () => {
    serve(standardRoutes((c) => (c.url === "/api/people/gift-plan" ? { json: plan({ total_paise: 0, monthly_paise: 0, goal_id: "g9" }) } : undefined)));
    renderWithRouter(<PeoplePage />);
    const card = await screen.findByRole("region", { name: "Gift plan" });
    expect(await within(card).findByRole("link", { name: "Gifts goal set" })).toHaveAttribute("href", "/plan");
  });

  it("adds, edits and deletes a festival season, sending budgets in paise", async () => {
    const success = spyOn(toast, "success");
    const calls = serve(
      standardRoutes((c) => {
        if (c.method === "GET" && c.url === "/api/people/seasons") {
          return { json: { seasons: [{ id: "s1", label: "Diwali", month: 10, budget_paise: 500_000 }] } };
        }
        if (c.method === "POST" && c.url === "/api/people/seasons") return { json: { id: "s2", ...(c.body as object) } };
        if (c.method === "PATCH" && c.url === "/api/people/seasons/s1") return { json: { id: "s1", ...(c.body as object) } };
        if (c.method === "DELETE" && c.url === "/api/people/seasons/s1") return { status: 204 };
        return undefined;
      }),
    );
    renderWithRouter(<PeoplePage />);
    const card = await screen.findByRole("region", { name: "Gift plan" });
    expect(await within(card).findByText("October · budget ₹5,000")).toBeInTheDocument();

    fireEvent.click(within(card).getByRole("button", { name: "Add a season" }));
    const add = within(card).getByRole("form", { name: "Add a season" });
    fireEvent.change(within(add).getByLabelText("Season"), { target: { value: "Eid" } });
    fireEvent.change(within(add).getByLabelText("Month"), { target: { value: "3" } });
    fireEvent.change(within(add).getByLabelText("Budget (₹)"), { target: { value: "2500.5" } });
    fireEvent.submit(add);
    await waitFor(() => expect(success).toHaveBeenCalledWith("Season added."));
    expect(calls.find((c) => c.method === "POST" && c.url === "/api/people/seasons")?.body).toEqual({
      label: "Eid",
      month: 3,
      budget_paise: 250_050,
    });

    fireEvent.click(within(card).getByRole("button", { name: "Edit Diwali" }));
    const edit = within(card).getByRole("form", { name: "Edit Diwali" });
    expect(within(edit).getByLabelText("Budget (₹)")).toHaveValue("5000");
    fireEvent.change(within(edit).getByLabelText("Budget (₹)"), { target: { value: "7500.50" } });
    fireEvent.submit(edit);
    await waitFor(() => expect(success).toHaveBeenCalledWith("Season updated."));
    expect(calls.find((c) => c.method === "PATCH" && c.url === "/api/people/seasons/s1")?.body).toEqual({
      label: "Diwali",
      month: 10,
      budget_paise: 750_050,
    });

    // Deleting a season asks first.
    const realConfirm = window.confirm;
    window.confirm = () => true;
    fireEvent.click(within(card).getByRole("button", { name: "Delete Diwali" }));
    window.confirm = realConfirm;
    await waitFor(() => expect(success).toHaveBeenCalledWith("Diwali removed."));
    expect(calls.some((c) => c.method === "DELETE" && c.url === "/api/people/seasons/s1")).toBe(true);
  });
});

async function waitForCall(calls: Call[], method: string, url: string) {
  await waitForCount(calls, () => calls.some((c) => c.method === method && c.url === url));
}

async function waitForCount(_calls: unknown, ok: () => boolean) {
  for (let i = 0; i < 100 && !ok(); i++) {
    await new Promise((r) => setTimeout(r, 5));
  }
  if (!ok()) throw new Error("condition not met in time");
}
