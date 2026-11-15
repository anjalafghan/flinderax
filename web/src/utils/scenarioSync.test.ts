import { describe, expect, test } from "bun:test";
import { reconcile } from "./scenarioSync";
import type { ScenarioDraft } from "./scenario";

const d = (id: string) => ({ id }) as unknown as ScenarioDraft;

describe("reconcile what-if drafts", () => {
  test("the server's drafts win when it has any", () => {
    expect(reconcile([d("s")], [d("l")])).toEqual({ drafts: [d("s")], push: false });
  });
  test("old browser-only drafts are uploaded once", () => {
    expect(reconcile([], [d("l")])).toEqual({ drafts: [d("l")], push: true });
  });
  test("nothing anywhere, nothing to do", () => {
    expect(reconcile([], [])).toEqual({ drafts: [], push: false });
  });
});
