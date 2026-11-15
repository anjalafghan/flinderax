import { afterEach, describe, expect, it, mock } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { PersonFacts } from "@/components/feature/people/PersonFacts";
import api from "@/services/api";

// Synthetic data only.
const facts = [
  { id: "a", person_id: "p1", text: "Allergic to peanuts" },
  { id: "b", person_id: "p2", text: "Loves filter coffee" },
];

function wrap() {
  const qc = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity, retry: false } } });
  qc.setQueryData(["people", "facts"], facts);
  return render(
    <QueryClientProvider client={qc}>
      <PersonFacts personId="p1" />
    </QueryClientProvider>,
  );
}

afterEach(() => cleanup());

describe("things to remember", () => {
  it("shows only this person's notes", () => {
    wrap();
    expect(screen.getByText("Allergic to peanuts")).toBeTruthy();
    expect(screen.queryByText("Loves filter coffee")).toBeNull();
  });

  it("refuses a blank note without calling the server, and Reset clears the form", async () => {
    const post = mock(async () => ({ data: {} }));
    api.post = post as unknown as typeof api.post;
    wrap();
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Write something to remember");
    const box = screen.getByLabelText("Add something to remember") as HTMLInputElement;
    fireEvent.change(box, { target: { value: "half typed" } });
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(box.value).toBe("");
    expect(post).not.toHaveBeenCalled();
  });

  it("adds a trimmed note", async () => {
    const post = mock(async (...args: unknown[]) => ({ data: { id: "n", person_id: "p1", text: String(args.length) } }));
    api.post = post as unknown as typeof api.post;
    wrap();
    fireEvent.change(screen.getByLabelText("Add something to remember"), { target: { value: "  Has two kids " } });
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    await waitFor(() => expect(post).toHaveBeenCalledTimes(1));
    expect(post.mock.calls[0]).toEqual(["/api/people/p1/facts", { text: "Has two kids" }]);
  });

  it("asks before removing", () => {
    wrap();
    const ask = mock(() => false);
    window.confirm = ask;
    fireEvent.click(screen.getByRole("button", { name: "Remove: Allergic to peanuts" }));
    expect(ask).toHaveBeenCalledTimes(1);
  });
});
