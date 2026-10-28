import { describe, expect, it } from "bun:test";
import { defaultBucket } from "@/utils/buckets";

describe("defaultBucket", () => {
  it("matches the backend defaults", () => {
    expect(defaultBucket("sip")).toBe("savings");
    expect(defaultBucket("rd")).toBe("savings");
    expect(defaultBucket("subscription")).toBe("wants");
    expect(defaultBucket("donation")).toBe("wants");
    expect(defaultBucket("bills")).toBe("needs");
    expect(defaultBucket("gym")).toBe("needs");
    expect(defaultBucket("other")).toBe("needs");
  });
});
