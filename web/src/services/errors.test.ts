import { describe, expect, test } from "bun:test";
import { errorMessageOf } from "./errors";

describe("errorMessageOf", () => {
  test("reads the message out of the server's error body", () => {
    expect(
      errorMessageOf(
        '{"code":"not_found","message":"No such slip","field":null}',
        404,
      ),
    ).toBe("No such slip");
  });
  test("older plain-text bodies and form errors still read", () => {
    expect(errorMessageOf("Please sign in again.", 401)).toBe(
      "Please sign in again.",
    );
    expect(errorMessageOf('{"error":"Give it a name."}', 422)).toBe(
      "Give it a name.",
    );
  });
  test("an empty or odd body falls back to the status", () => {
    expect(errorMessageOf("", 500)).toBe("Request failed with status 500");
    expect(errorMessageOf("{}", 502)).toBe("Request failed with status 502");
  });
});
