import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { BackupSection } from "./BackupSection";

let calls: { url: string; method?: string; body?: unknown }[];
let reply: (url: string) => Response;
const realFetch = globalThis.fetch;

beforeEach(() => {
  calls = [];
  reply = () =>
    new Response(JSON.stringify({ status: true, rows: 12 }), { status: 200 });
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({
      url: String(url),
      method: init?.method,
      body: init?.body ? JSON.parse(String(init.body)) : undefined,
    });
    return reply(String(url));
  }) as unknown as typeof fetch;
});
afterEach(() => {
  cleanup();
  globalThis.fetch = realFetch;
});

const pick = (content: string) => {
  const input = screen.getByLabelText(/backup file/i) as HTMLInputElement;
  const file = new File([content], "joy-2026-11-15.json", {
    type: "application/json",
  });
  fireEvent.change(input, { target: { files: [file] } });
};

describe("BackupSection", () => {
  test("choosing a file asks first and sends nothing yet", async () => {
    render(<BackupSection />);
    pick(JSON.stringify({ data: { accounts: [] } }));
    expect(
      await screen.findByText(/replace your money and people data/i),
    ).toBeTruthy();
    expect(calls).toHaveLength(0);
  });

  test("confirming sends the file and says how many rows came back", async () => {
    render(<BackupSection />);
    pick(JSON.stringify({ data: { accounts: [] } }));
    fireEvent.click(
      await screen.findByRole("button", { name: /restore from this file/i }),
    );
    await waitFor(() =>
      expect(screen.getByRole("status").textContent).toContain("12"),
    );
    expect(calls[0]?.url).toBe("/api/import");
    expect(calls[0]?.body).toEqual({ data: { accounts: [] } });
  });

  test("a file that isn't JSON is refused without asking the server", async () => {
    render(<BackupSection />);
    pick("this is not json");
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(calls).toHaveLength(0);
  });

  test("the server's refusal is shown", async () => {
    reply = () =>
      new Response("A row in “accounts” already belongs to another account.", {
        status: 409,
      });
    render(<BackupSection />);
    pick(JSON.stringify({ data: {} }));
    fireEvent.click(
      await screen.findByRole("button", { name: /restore from this file/i }),
    );
    expect((await screen.findByRole("alert")).textContent).toContain(
      "already belongs",
    );
  });
});
