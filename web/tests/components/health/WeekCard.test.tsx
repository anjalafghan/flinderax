import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { toast } from "sonner";

import { WeekCard } from "@/components/feature/health/WeekCard";
import { jsonResponse, overview, renderWithProviders, stubFetch } from "./fixture";

describe("WeekCard", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("offers the past eight weeks in the back-fill picker and not the current one twice", () => {
    renderWithProviders(<WeekCard overview={overview} />);
    const pick = screen.getByLabelText("Log a past week") as HTMLSelectElement;
    const options = Array.from(pick.options).map((o) => o.textContent);
    expect(options[0]).toBe("This week");
    expect(options).toHaveLength(9);
    expect(options).not.toContain("Week of 2 Nov");
  });

  it("shows a toast with the server's message when a save fails", async () => {
    stubFetch(() => Promise.resolve(new Response("Pick a day between 0 and 7.", { status: 422 })));
    const error = spyOn(toast, "error");
    renderWithProviders(<WeekCard overview={overview} />);
    fireEvent.click(screen.getByRole("button", { name: "Save week" }));
    await waitFor(() => expect(error).toHaveBeenCalledWith("Pick a day between 0 and 7."));
  });

  it("sends the feel as null when Skip is chosen", async () => {
    const put = stubFetch(() => jsonResponse(overview.week));
    renderWithProviders(<WeekCard overview={overview} />);
    fireEvent.click(screen.getByRole("radio", { name: "Skip" }));
    fireEvent.click(screen.getByRole("button", { name: "Save week" }));
    await waitFor(() => expect(put).toHaveBeenCalled());
    const [, init] = put.mock.calls[0] as unknown as [string, RequestInit];
    expect(JSON.parse(String(init.body)).feel).toBeNull();
  });
});
