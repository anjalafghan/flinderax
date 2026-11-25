import { describe, expect, test } from "bun:test";
import { defaultBucket } from "./buckets";

// Must agree with `default_bucket` in src/planner/mod.rs (see categories_map_to_default_buckets).
describe("defaultBucket", () => {
  test("savings categories", () => {
    for (const c of ["sip", "rd", "savings", "investment"])
      expect(defaultBucket(c)).toBe("savings");
  });
  test("spending categories", () => {
    for (const c of [
      "subscription",
      "donation",
      "entertainment",
      "shopping",
      "dates",
      "gifts",
      "travel",
    ])
      expect(defaultBucket(c)).toBe("wants");
  });
  test("everything else is a need", () => {
    for (const c of [
      "bills",
      "emi",
      "house",
      "rent",
      "family",
      "phone",
      "internet",
      "transport",
      "car",
      "insurance",
      "health",
      "gym",
      "fees",
      "other",
    ])
      expect(defaultBucket(c)).toBe("needs");
  });
});
