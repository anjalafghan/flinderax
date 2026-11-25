import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { CoupleGoalsCard } from "./CoupleGoalsCard";

let calls: { url: string; body?: unknown }[];
let payload: unknown;
const realFetch = globalThis.fetch;

const view = () =>
  render(
    <MemoryRouter>
      <QueryClientProvider
        client={
          new QueryClient({ defaultOptions: { queries: { retry: false } } })
        }
      >
        <CoupleGoalsCard />
      </QueryClientProvider>
    </MemoryRouter>,
  );

const goal = {
  goal_id: "g1",
  name: "Wedding",
  target_paise: 5_000_000,
  target_date: "2027-12-01",
  saved_paise: 1_500_000,
  percent: 30,
  monthly_needed_paise: 250_000,
  shares: [
    { name: "asha", paise: 1_000_000 },
    { name: "ravi", paise: 500_000 },
  ],
  contributions: [],
};

beforeEach(() => {
  calls = [];
  payload = { in_couple: true, goals: [goal] };
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({
      url: String(url),
      body: init?.body ? JSON.parse(String(init.body)) : undefined,
    });
    if (init?.method === "POST")
      return new Response(JSON.stringify({ status: true, goal_id: "g2" }), {
        status: 200,
      });
    return new Response(JSON.stringify(payload), { status: 200 });
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

describe("CoupleGoalsCard", () => {
  test("shows each shared goal with progress, what is needed a month, and who has put in what", async () => {
    view();
    expect(await screen.findByText("Wedding")).toBeTruthy();
    expect(screen.getByText(/30%/)).toBeTruthy();
    expect(screen.getByText(/₹15,000 of ₹50,000/)).toBeTruthy();
    expect(screen.getByText(/₹2,500 a month/)).toBeTruthy();
    expect(screen.getByText(/asha ₹10,000/)).toBeTruthy();
    expect(screen.getByText(/ravi ₹5,000/)).toBeTruthy();
  });

  test("each goal links to its budget page", async () => {
    view();
    const link = await screen.findByRole("link", { name: /plan the budget/i });
    expect(link.getAttribute("href")).toBe("/wedding/g1");
  });

  test("adding money sends it in paise to that goal", async () => {
    view();
    fireEvent.change(await screen.findByLabelText(/add to wedding/i), {
      target: { value: "2,000" },
    });
    fireEvent.click(screen.getByRole("button", { name: /put in/i }));
    await waitFor(() =>
      expect(calls.some((c) => c.url.endsWith("/g1/contribute"))).toBe(true),
    );
    expect(calls.find((c) => c.url.endsWith("/g1/contribute"))?.body).toEqual({
      amount_paise: 200_000,
    });
  });

  test("starting a goal together asks for a name, an amount and optionally a date", async () => {
    view();
    fireEvent.click(await screen.findByText(/start a goal together/i));
    fireEvent.change(screen.getByLabelText(/what are you saving for/i), {
      target: { value: "A baby" },
    });
    fireEvent.change(screen.getByLabelText(/how much/i), {
      target: { value: "300000" },
    });
    fireEvent.click(screen.getByRole("button", { name: /start this goal/i }));
    await waitFor(() =>
      expect(
        calls.some(
          (c) => c.body && (c.body as { name?: string }).name === "A baby",
        ),
      ).toBe(true),
    );
    expect(
      calls.find(
        (c) => (c.body as { name?: string } | undefined)?.name === "A baby",
      )?.body,
    ).toEqual({ name: "A baby", target_paise: 30_000_000 });
  });

  test("nothing at all for someone who is not in a couple", async () => {
    payload = { in_couple: false, goals: [] };
    const { container } = view();
    await new Promise((r) => setTimeout(r, 30));
    expect(container.textContent).toBe("");
  });
});
