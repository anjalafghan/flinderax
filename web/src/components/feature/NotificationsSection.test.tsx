import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { urlBase64ToBytes } from "@/utils/push";
import { NotificationsSection } from "./NotificationsSection";

const g = globalThis as unknown as Record<string, unknown>;
let calls: { url: string; body?: unknown }[];
let status: { public_key: string | null; subscribed: boolean };
let permission: NotificationPermission;
let subscription: {
  endpoint: string;
  toJSON: () => unknown;
  unsubscribe: () => Promise<boolean>;
} | null;
const real = {
  fetch: globalThis.fetch,
  notification: g.Notification,
  pm: (window as unknown as Record<string, unknown>).PushManager,
};
const realSw = Object.getOwnPropertyDescriptor(navigator, "serviceWorker");

const view = () =>
  render(
    <QueryClientProvider
      client={
        new QueryClient({ defaultOptions: { queries: { retry: false } } })
      }
    >
      <NotificationsSection />
    </QueryClientProvider>,
  );

beforeEach(() => {
  calls = [];
  status = { public_key: "BAAA", subscribed: false };
  permission = "default";
  subscription = null;
  globalThis.fetch = (async (url: string, init?: RequestInit) => {
    calls.push({
      url: String(url),
      body: init?.body ? JSON.parse(String(init.body)) : undefined,
    });
    if (init?.method === "POST") {
      status = { ...status, subscribed: String(url).endsWith("/subscribe") };
      return new Response(JSON.stringify({ status: true }), { status: 200 });
    }
    return new Response(JSON.stringify(status), { status: 200 });
  }) as unknown as typeof fetch;
  g.Notification = Object.assign(function () {}, {
    permission: "default",
    requestPermission: async () =>
      ((g.Notification as { permission: string }).permission = permission),
  });
  (window as unknown as Record<string, unknown>).PushManager = function () {};
  Object.defineProperty(navigator, "serviceWorker", {
    configurable: true,
    value: {
      ready: Promise.resolve({
        pushManager: {
          getSubscription: async () => subscription,
          subscribe: async () => {
            subscription = {
              endpoint: "https://push.example/abc",
              toJSON: () => ({
                endpoint: "https://push.example/abc",
                keys: { p256dh: "pk", auth: "au" },
              }),
              unsubscribe: async () => ((subscription = null), true),
            };
            return subscription;
          },
        },
      }),
    },
  });
});
afterEach(() => {
  cleanup();
  globalThis.fetch = real.fetch;
  g.Notification = real.notification;
  (window as unknown as Record<string, unknown>).PushManager = real.pm;
  if (realSw) Object.defineProperty(navigator, "serviceWorker", realSw);
  else delete (navigator as unknown as Record<string, unknown>).serviceWorker;
});

describe("urlBase64ToBytes", () => {
  test("reads the server's key the way pushManager.subscribe wants it", () => {
    expect(Array.from(urlBase64ToBytes("AQID"))).toEqual([1, 2, 3]);
    expect(Array.from(urlBase64ToBytes("_-8"))).toEqual([255, 239]);
  });
});

describe("NotificationsSection", () => {
  test("turning them on asks permission, subscribes this browser and tells the server", async () => {
    permission = "granted";
    view();
    fireEvent.click(await screen.findByRole("button", { name: /turn on/i }));
    await waitFor(() =>
      expect(calls.some((c) => c.url.endsWith("/api/push/subscribe"))).toBe(
        true,
      ),
    );
    expect(
      calls.find((c) => c.url.endsWith("/api/push/subscribe"))?.body,
    ).toEqual({
      endpoint: "https://push.example/abc",
      p256dh: "pk",
      auth: "au",
    });
    expect(
      await screen.findByRole("button", { name: /turn off/i }),
    ).toBeTruthy();
  });

  test("saying no to the browser's question leaves them off and explains", async () => {
    permission = "denied";
    view();
    fireEvent.click(await screen.findByRole("button", { name: /turn on/i }));
    expect(await screen.findByText(/blocked/i)).toBeTruthy();
    expect(calls.some((c) => c.url.endsWith("/api/push/subscribe"))).toBe(
      false,
    );
  });

  test("turning them off removes this browser's subscription from the server", async () => {
    status = { public_key: "BAAA", subscribed: true };
    subscription = {
      endpoint: "https://push.example/abc",
      toJSON: () => ({}),
      unsubscribe: async () => ((subscription = null), true),
    };
    view();
    fireEvent.click(await screen.findByRole("button", { name: /turn off/i }));
    await waitFor(() =>
      expect(calls.some((c) => c.url.endsWith("/api/push/unsubscribe"))).toBe(
        true,
      ),
    );
    expect(
      calls.find((c) => c.url.endsWith("/api/push/unsubscribe"))?.body,
    ).toEqual({ endpoint: "https://push.example/abc" });
  });

  test("a server without keys says it is not set up, instead of offering a button that cannot work", async () => {
    status = { public_key: null, subscribed: false };
    view();
    expect(await screen.findByText(/not set up/i)).toBeTruthy();
    expect(screen.queryByRole("button", { name: /turn on/i })).toBeNull();
  });
});
