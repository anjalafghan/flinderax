import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { afterEach, expect } from "bun:test";

// Must run before testing-library is imported so `document` exists.
GlobalRegistrator.register({ url: "http://localhost/" });

const matchers = await import("@testing-library/jest-dom/matchers");
expect.extend(matchers as never);

const { cleanup } = await import("@testing-library/react");
afterEach(() => cleanup());

// A test that reaches the network is a test that depends on the machine it runs on. Tests that
// need `fetch` replace it themselves (and put the original back); anything else fails loudly.
globalThis.fetch = (async (input: RequestInfo | URL) => {
  throw new Error(
    `Unmocked fetch to ${String(input instanceof Request ? input.url : input)}. Replace globalThis.fetch in the test.`,
  );
}) as unknown as typeof fetch;
