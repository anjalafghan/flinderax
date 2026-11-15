import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { CloseOut } from "./CloseOut";

let calls: { url: string; body: unknown }[];
const realFetch = globalThis.fetch;

beforeEach(() => {
  calls = [];
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({ url, body: init?.body ? JSON.parse(String(init.body)) : undefined });
    return new Response(JSON.stringify({ ok: true, to: "/joy/kept" }), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("CloseOut: did it happen?", () => {
  const item = { id: 7, title: "Cook together", when: "14 Nov" };

  test("asks about the date by name", () => {
    render(<CloseOut item={item} onDone={() => {}} />);
    expect(screen.getByText(/Cook together/)).toBeTruthy();
    expect(screen.getByText(/14 Nov/)).toBeTruthy();
  });

  test("yes confirms it and tells the page", async () => {
    let done = 0;
    render(<CloseOut item={item} onDone={() => done++} />);
    fireEvent.click(screen.getByRole("button", { name: /yes, we went/i }));
    await waitFor(() => expect(done).toBe(1));
    expect(calls[0]?.url).toContain("/history/7/happened");
    expect(calls[0]?.body).toEqual({ happened: "1" });
  });

  test("no cancels it", async () => {
    let done = 0;
    render(<CloseOut item={item} onDone={() => done++} />);
    fireEvent.click(screen.getByRole("button", { name: /it didn't happen/i }));
    await waitFor(() => expect(done).toBe(1));
    expect(calls[0]?.body).toEqual({ happened: "0" });
  });
});
