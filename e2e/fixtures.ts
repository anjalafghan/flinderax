import { test as base, expect } from "@playwright/test";

let n = 0;

/**
 * Every test gets its own browser context and its own client address. The server rate-limits sign-ups
 * per address (five a minute), and a whole suite from one address would hit that.
 */
export const test = base.extend({
  context: async ({ browser }, use) => {
    n += 1;
    const ip = `10.77.${(n >> 8) & 255}.${n & 255}`;
    const context = await browser.newContext({ baseURL: "http://127.0.0.1:3123", extraHTTPHeaders: { "fly-client-ip": ip } });
    await use(context);
    await context.close();
  },
});

export { expect };
