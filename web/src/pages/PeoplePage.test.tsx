import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import PeoplePage from "./PeoplePage";

const person = {
  id: "p1",
  name: "Asha Verma",
  circle: "family",
  every_days: 14,
  last_contact_on: "2026-11-01",
  next_due_on: "2026-11-15",
  status: "overdue",
  note: "",
  archived: false,
};

describe("PeoplePage", () => {
  test("lists the circle and who is due for a catch-up", async () => {
    mockApi({
      "/api/people": {
        people: [person],
        archived: [],
        upcoming: [],
        catch_up: [person],
      },
    });
    renderPage(<PeoplePage />);
    expect(await screen.findByRole("heading", { name: "Circle" })).toBeTruthy();
    expect((await screen.findAllByText(/Asha Verma/)).length).toBeGreaterThan(0);
  });

  test("shows a skeleton while loading", () => {
    mockApi({ "/api/people": pending });
    renderPage(<PeoplePage />);
    expect(screen.getByLabelText("Loading your circle")).toBeTruthy();
  });

  test("offers a retry when the circle cannot load", async () => {
    mockApi({ "/api/people": failWith(500, { error: "boom" }) });
    renderPage(<PeoplePage />);
    expect(await screen.findByText(/Couldn't load your circle/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Try again" })).toBeTruthy();
  });
});
