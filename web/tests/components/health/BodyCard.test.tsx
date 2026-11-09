import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { toast } from "sonner";

import { BodyCard } from "@/components/feature/health/BodyCard";
import api from "@/services/api";
import { overview, marksFor, renderWithProviders, respond, stubFetch } from "./fixture";

const marks = () =>
  spyOn(api, "get").mockImplementation((url: string) => {
    const kind = new URL(url, "http://x").searchParams.get("kind") ?? "";
    return respond({ marks: marksFor(kind) });
  });

describe("BodyCard", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("shows the latest weight, the change since the last reading, and the check-up dates", async () => {
    marks();
    renderWithProviders(<BodyCard body={overview.body} />);
    expect(screen.getByText("70.5 kg on 12 Oct")).toBeInTheDocument();
    expect(await screen.findByText("−1.2 kg since 12 Aug")).toBeInTheDocument();
    expect(screen.getByText("Last 12 Mar 2026")).toBeInTheDocument();
    expect(screen.getByText("Next due 12 Mar 2027")).toBeInTheDocument();
    expect(screen.getByText("Waist").parentElement).toHaveTextContent("Not recorded");
    expect(screen.getByText(/Kept private to you/)).toBeInTheDocument();
    expect(await screen.findByText("Weight on 12 Aug")).toBeInTheDocument();
  });

  it("falls back to a plain phrase when the earlier reading is not in the recent list", () => {
    spyOn(api, "get").mockImplementation(() => respond({ marks: [] }));
    renderWithProviders(<BodyCard body={overview.body} />);
    expect(screen.getByText("−1.2 kg since your last weigh-in")).toBeInTheDocument();
  });

  it("says when there is no check-up or weight yet", () => {
    marks();
    renderWithProviders(
      <BodyCard body={{ ...overview.body, weight_x10: null, weight_on: null, prev_weight_x10: null, last_checkup_on: null, checkup_due_on: null }} />,
    );
    expect(screen.getByText("Not recorded yet")).toBeInTheDocument();
    expect(screen.getByText("No check-up recorded yet.")).toBeInTheDocument();
  });

  it("rejects a weight outside the range or with two decimals, without saving", () => {
    marks();
    const post = spyOn(api, "post");
    renderWithProviders(<BodyCard body={overview.body} />);
    fireEvent.click(screen.getByRole("button", { name: "Add weight" }));
    fireEvent.change(screen.getByLabelText("Weight (kg)"), { target: { value: "70.55" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Enter a weight from 20 to 300 kg, with one decimal at most.");
    expect(post).not.toHaveBeenCalled();
  });

  it("rejects a date in the future", () => {
    marks();
    const post = spyOn(api, "post");
    renderWithProviders(<BodyCard body={overview.body} />);
    fireEvent.click(screen.getByRole("button", { name: "Add check-up date" }));
    fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2999-01-01" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Pick today or an earlier date.");
    expect(post).not.toHaveBeenCalled();
  });

  it("adds a weight in tenths of a kilo and toasts", async () => {
    marks();
    const post = spyOn(api, "post").mockImplementation(() => respond({ id: "m9", kind: "weight", measured_on: "2026-11-01", value_x10: 704, note: "" }));
    const success = spyOn(toast, "success");
    renderWithProviders(<BodyCard body={overview.body} />);
    fireEvent.click(screen.getByRole("button", { name: "Add weight" }));
    fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2026-11-01" } });
    fireEvent.change(screen.getByLabelText("Weight (kg)"), { target: { value: "70.4" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    await waitFor(() => expect(success).toHaveBeenCalledWith("Saved."));
    expect(post).toHaveBeenCalledWith("/api/diet/marks", { kind: "weight", measured_on: "2026-11-01", value_x10: 704 });
  });

  it("lists readings with a delete for each", async () => {
    marks();
    const del = stubFetch(() => Promise.resolve(new Response(null, { status: 204 })));
    renderWithProviders(<BodyCard body={overview.body} />);
    const list = await screen.findByRole("list");
    expect(within(list).getByText("70.5 kg")).toBeInTheDocument();
    fireEvent.click(within(list).getByRole("button", { name: "Delete weight from 12 Oct" }));
    await waitFor(() => expect(del).toHaveBeenCalled());
    const [url, init] = del.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/diet/marks/m1");
    expect(init.method).toBe("DELETE");
  });
});
