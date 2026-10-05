import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { afterEach, expect } from "bun:test";

// Must run before testing-library is imported so `document` exists.
GlobalRegistrator.register({ url: "http://localhost/" });

const matchers = await import("@testing-library/jest-dom/matchers");
expect.extend(matchers as never);

const { cleanup } = await import("@testing-library/react");
afterEach(() => cleanup());
