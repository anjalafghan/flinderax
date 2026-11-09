import { spyOn } from "bun:test";
import type { ReactNode } from "react";
import { render } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

import { AuthContext } from "@/context/AuthContext";

// Synthetic data only. The fake server answers at the fetch level, so both `api` and the local
// PATCH/PUT/DELETE helper in services/people.ts go through it.

export interface Call {
  method: string;
  url: string;
  body: unknown;
}

export type Reply = { status?: number; json?: unknown; text?: string };

/** Routes every fetch. Each call is recorded so tests can check the exact request body. */
export function serve(route: (call: Call) => Reply): Call[] {
  const calls: Call[] = [];
  spyOn(globalThis, "fetch").mockImplementation((async (input: RequestInfo | URL, init?: RequestInit) => {
    const call: Call = {
      method: init?.method ?? "GET",
      url: String(input),
      body: init?.body ? JSON.parse(String(init.body)) : undefined,
    };
    calls.push(call);
    const reply = route(call);
    const status = reply.status ?? 200;
    const payload = reply.text !== undefined ? reply.text : reply.json !== undefined ? JSON.stringify(reply.json) : null;
    return new Response(status === 204 ? null : payload, {
      status,
      headers: { "Content-Type": reply.text !== undefined ? "text/plain" : "application/json" },
    });
  }) as typeof fetch);
  return calls;
}

export const NOT_FOUND: Reply = { status: 404, text: "Not found." };

export function renderWithRouter(ui: ReactNode, path = "/people") {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } });
  return render(
    <QueryClientProvider client={qc}>
      <AuthContext.Provider value={{ isAuthenticated: true, token: "t", login: () => {}, logout: () => {}, isLoading: false }}>
        <MemoryRouter initialEntries={[path]}>{ui}</MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  );
}
