import { afterEach } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement } from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";

const realFetch = globalThis.fetch;

/** What a mocked endpoint answers: a body (200), or `failWith(status)` for an error. */
export type Answer = unknown;

export const failWith = (status: number, body: unknown = {}): Answer => ({
  __status: status,
  body,
});

/** An endpoint that never answers, to hold a page in its loading state. */
export const pending: Answer = { __pending: true };

/**
 * Replaces `fetch` with a router over exact paths (query string ignored). Unlisted paths answer
 * 404 and are recorded in `unmocked`, so a page that starts asking for something new is easy to
 * spot. Restored automatically after each test.
 */
export function mockApi(answers: Record<string, Answer>) {
  const unmocked: string[] = [];
  const requested: string[] = [];
  globalThis.fetch = (async (input: RequestInfo | URL) => {
    const url = new URL(
      String(input instanceof Request ? input.url : input),
      "http://x",
    );
    requested.push(url.pathname);
    const hit = answers[url.pathname];
    if (hit === undefined) {
      unmocked.push(url.pathname);
      return new Response("{}", { status: 404 });
    }
    if (typeof hit === "object" && hit !== null && "__pending" in hit) {
      return new Promise<Response>(() => {});
    }
    if (typeof hit === "object" && hit !== null && "__status" in hit) {
      const f = hit as { __status: number; body?: unknown };
      return new Response(JSON.stringify(f.body ?? {}), { status: f.__status });
    }
    return new Response(JSON.stringify(hit), { status: 200 });
  }) as unknown as typeof fetch;
  return { unmocked, requested };
}

afterEach(() => {
  globalThis.fetch = realFetch;
});

/** Renders a page inside a fresh query client (no retries) and a memory router. */
export function renderPage(
  ui: ReactElement,
  opts: { path?: string; route?: string } = {},
) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, retryDelay: 0, gcTime: 0 } },
  });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={[opts.path ?? "/"]}>
        {opts.route ? (
          <Routes>
            <Route path={opts.route} element={ui} />
          </Routes>
        ) : (
          ui
        )}
      </MemoryRouter>
    </QueryClientProvider>,
  );
}
