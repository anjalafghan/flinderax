import { afterEach, describe, expect, it, mock } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { FollowUpsCard } from "@/components/feature/people/FollowUpsCard";
import { PersonFollowUps } from "@/components/feature/people/PersonFollowUps";
import { followUpWhen, type FollowUp, type FollowUpsOut } from "@/services/people";
import api from "@/services/api";

// Synthetic data only.
const f = (o: Partial<FollowUp> & { id: string; text: string }): FollowUp => ({
  person_id: "p1",
  person_name: "Asha",
  due_on: null,
  done_on: null,
  state: "undated",
  days_away: null,
  ...o,
});

const data: FollowUpsOut = {
  open: [
    f({ id: "a", text: "Ask about the surgery", state: "overdue", days_away: -3, due_on: "2026-11-11" }),
    f({ id: "b", text: "Send the photos", person_id: "p2", person_name: "Ravi", state: "soon", days_away: 2, due_on: "2026-11-16" }),
  ],
  done: [f({ id: "c", text: "Call back", state: "done", done_on: "2026-11-10" })],
};

function wrap(ui: React.ReactElement) {
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(["people", "followups"], data);
  return render(
    <QueryClientProvider client={qc}>
      <MemoryRouter>{ui}</MemoryRouter>
    </QueryClientProvider>,
  );
}

afterEach(() => cleanup());

describe("follow-up wording", () => {
  it("says when each one is due", () => {
    expect(followUpWhen({ state: "overdue", days_away: -1 })).toBe("1 day overdue");
    expect(followUpWhen({ state: "overdue", days_away: -3 })).toBe("3 days overdue");
    expect(followUpWhen({ state: "today", days_away: 0 })).toBe("Today");
    expect(followUpWhen({ state: "soon", days_away: 1 })).toBe("Tomorrow");
    expect(followUpWhen({ state: "later", days_away: 30 })).toBe("In 30 days");
    expect(followUpWhen({ state: "undated", days_away: null })).toBe("No date");
  });
});

describe("the circle's follow-ups card", () => {
  it("lists everyone's open follow-ups with names and how overdue they are", () => {
    wrap(<FollowUpsCard />);
    expect(screen.getByText("Ask about the surgery")).toBeTruthy();
    expect(screen.getByText("Send the photos")).toBeTruthy();
    expect(screen.getByText(/1 overdue\./)).toBeTruthy();
    expect(screen.getByText(/3 days overdue/)).toBeTruthy();
    expect(screen.getByRole("link", { name: "Ravi" }).getAttribute("href")).toBe("/people/p2");
  });
});

describe("one person's follow-ups", () => {
  it("shows only theirs, and the recently done ones folded away", () => {
    wrap(<PersonFollowUps personId="p1" />);
    expect(screen.getByText("Ask about the surgery")).toBeTruthy();
    expect(screen.queryByText("Send the photos")).toBeNull();
    expect(screen.getByText("Done recently (1)")).toBeTruthy();
  });

  it("refuses a blank or too-long follow-up without calling the server, and Reset clears the form", async () => {
    const post = mock(async () => ({ data: {} }));
    api.post = post as unknown as typeof api.post;
    wrap(<PersonFollowUps personId="p1" />);
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Write what to follow up on");
    const box = screen.getByLabelText("Add a follow-up") as HTMLInputElement;
    fireEvent.change(box, { target: { value: "  Ask about the move " } });
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(box.value).toBe("");
    expect(post).not.toHaveBeenCalled();
  });

  it("adds a trimmed follow-up with its due day", async () => {
    const post = mock(async (_url: string, _body: unknown) => ({ data: f({ id: "n", text: "x" }) }));
    api.post = post as unknown as typeof api.post;
    wrap(<PersonFollowUps personId="p1" />);
    fireEvent.change(screen.getByLabelText("Add a follow-up"), { target: { value: "  Ask about the move " } });
    fireEvent.change(screen.getByLabelText("By (optional)"), { target: { value: "2027-01-05" } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    expect(post.mock.calls[0]).toEqual(["/api/people/p1/followups", { text: "Ask about the move", due_on: "2027-01-05" }]);
  });

  it("asks before deleting", () => {
    wrap(<PersonFollowUps personId="p1" />);
    const ask = mock(() => false);
    window.confirm = ask;
    fireEvent.click(screen.getByRole("button", { name: "Delete follow-up: Ask about the surgery" }));
    expect(ask).toHaveBeenCalledTimes(1);
  });
});
