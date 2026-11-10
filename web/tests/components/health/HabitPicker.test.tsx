import { afterEach, describe, expect, it, mock, spyOn } from "bun:test";
import { cleanup, fireEvent, screen, waitFor } from "@testing-library/react";
import { toast } from "sonner";

import { HabitPicker } from "@/components/feature/health/HabitPicker";
import api from "@/services/api";
import { homeDinner, overview, renderWithProviders, respond, walk, stubFetch } from "./fixture";

describe("HabitPicker", () => {
  afterEach(() => {
    cleanup();
    mock.restore();
  });

  it("offers only the presets not already tracked", () => {
    renderWithProviders(<HabitPicker overview={overview} />);
    expect(screen.getByRole("button", { name: "No sugary drinks, 6 days" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Fruit or salad every day, 5 days" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Home-cooked dinner, 5 days" })).toBeNull();
  });

  it("adds a preset with its key and target", async () => {
    const post = spyOn(api, "post").mockImplementation(() => respond({ id: "h3" }));
    const success = spyOn(toast, "success");
    renderWithProviders(<HabitPicker overview={overview} />);
    fireEvent.click(screen.getByRole("button", { name: "No sugary drinks, 6 days" }));
    await waitFor(() => expect(success).toHaveBeenCalledWith("Now tracking No sugary drinks."));
    expect(post).toHaveBeenCalledWith("/api/diet/habits", {
      label: "No sugary drinks",
      preset_key: "no_sugary",
      target_days: 6,
    });
  });

  it("shows the server's message when one more habit would be over the limit", async () => {
    spyOn(api, "post").mockImplementation(() => Promise.reject(new Error("You can track up to five at a time.")));
    const error = spyOn(toast, "error");
    renderWithProviders(<HabitPicker overview={overview} />);
    fireEvent.click(screen.getByRole("button", { name: "No sugary drinks, 6 days" }));
    await waitFor(() => expect(error).toHaveBeenCalledWith("You can track up to five at a time."));
  });

  it("replaces the add forms with the limit note when five are tracked", () => {
    const more = ["Fruit or salad every day", "Workout", "10,000 steps"].map((label, i) => ({ id: `h${i + 3}`, label, preset_key: null, target_days: 5, active: true }));
    renderWithProviders(<HabitPicker overview={{ ...overview, habits: [homeDinner, walk, ...more] }} />);
    expect(screen.getByText("You can track up to five at a time. Stop one to add another.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Add habit" })).toBeNull();
  });

  it("adds a custom habit with its days, and refuses an empty name", async () => {
    const post = spyOn(api, "post").mockImplementation(() => respond({ id: "h4" }));
    spyOn(api, "get").mockImplementation(() => respond(overview));
    renderWithProviders(<HabitPicker overview={overview} />);
    fireEvent.click(screen.getByRole("button", { name: "Add habit" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Give the habit a name of up to 60 letters.");
    expect(post).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText("Or your own habit"), { target: { value: "  Stretch after work " } });
    fireEvent.change(screen.getByLabelText("Days a week", { selector: "#habit-custom-target" }), { target: { value: "3" } });
    fireEvent.click(screen.getByRole("button", { name: "Add habit" }));
    await waitFor(() => expect(post).toHaveBeenCalledWith("/api/diet/habits", { label: "Stretch after work", target_days: 3 }));
  });

  it("stops tracking a habit with a DELETE", async () => {
    const del = stubFetch(() => Promise.resolve(new Response(null, { status: 204 })));
    renderWithProviders(<HabitPicker overview={overview} />);
    fireEvent.click(screen.getByRole("button", { name: "Stop tracking Walk 30 minutes" }));
    await waitFor(() => expect(del).toHaveBeenCalled());
    const [url, init] = del.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe("/api/diet/habits/h2");
    expect(init.method).toBe("DELETE");
  });
});
