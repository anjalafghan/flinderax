import { describe, expect, test } from "bun:test";
import { screen } from "@testing-library/react";

import { failWith, mockApi, pending, renderPage } from "@/test/page";
import WorkPage from "./WorkPage";

const lead = {
  id: "l1",
  company: "Northwind Labs",
  role: "Backend engineer",
  stage: "interview",
  take_home_paise: 22_000_000,
  next_step: "System design round",
  next_on: null,
  note: "",
  updated_on: "2026-11-20",
};

describe("WorkPage", () => {
  test("groups leads by stage and counts the open ones", async () => {
    mockApi({ "/api/work": { leads: [lead] } });
    renderPage(<WorkPage />);
    expect(await screen.findByText("Northwind Labs")).toBeTruthy();
    expect(screen.getByText("Open leads")).toBeTruthy();
    expect(screen.getByLabelText("Remove Northwind Labs")).toBeTruthy();
  });

  test("says so when there are no leads yet", async () => {
    mockApi({ "/api/work": { leads: [] } });
    renderPage(<WorkPage />);
    expect(await screen.findByText(/No leads yet/)).toBeTruthy();
  });

  test("shows a skeleton while loading", () => {
    mockApi({ "/api/work": pending });
    renderPage(<WorkPage />);
    expect(screen.getByLabelText("Loading your leads")).toBeTruthy();
  });

  test("offers a retry when the leads fail to load", async () => {
    mockApi({ "/api/work": failWith(500) });
    renderPage(<WorkPage />);
    expect(await screen.findByRole("button", { name: "Try again" })).toBeTruthy();
  });
});
