import { describe, expect, test } from "bun:test";
import { thumbOf } from "./photo";

describe("thumbOf", () => {
  test("asks for the small copy of a signed date photo", () => {
    expect(thumbOf("/api/public/dates/history/3/photos/9?exp=1&sig=ab")).toBe(
      "/api/public/dates/history/3/photos/9?exp=1&sig=ab&thumb=1",
    );
  });
  test("leaves other pictures alone", () => {
    expect(thumbOf("/api/public/dates/regulars/photo/4?exp=1&sig=ab")).toBe(
      "/api/public/dates/regulars/photo/4?exp=1&sig=ab",
    );
    expect(thumbOf("")).toBe("");
  });
});
