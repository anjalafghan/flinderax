import type { TestingLibraryMatchers } from "@testing-library/jest-dom/matchers";

// Declaration merging needs an interface; with no members of its own, the empty body is intentional.
declare module "bun:test" {
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  interface Matchers<T> extends TestingLibraryMatchers<unknown, T> {}
  // eslint-disable-next-line @typescript-eslint/no-empty-object-type
  interface AsymmetricMatchers extends TestingLibraryMatchers<unknown, unknown> {}
}
