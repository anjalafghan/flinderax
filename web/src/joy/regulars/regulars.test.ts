import { describe, expect, test } from "bun:test";
import { ApiError } from "../api";
import { apiMessage, cleanDays, newRegularBody, parseRupees, skipBody, weekdaysLabel, wentElse, wentUsual } from "./days";
import { fitWithin } from "./photo";

describe("weekday labels", () => {
  test("mirrors the server", () => {
    expect(weekdaysLabel("2,4")).toBe("Tuesdays and Thursdays");
    expect(weekdaysLabel("5")).toBe("Fridays");
    expect(weekdaysLabel("1,3,5")).toBe("Mondays, Wednesdays and Fridays");
    expect(weekdaysLabel("")).toBe("");
    expect(weekdaysLabel("9,x")).toBe("");
  });
  test("cleanDays keeps only real weekdays once each, in order", () => {
    expect(cleanDays(["4", "2", "4", "9", "x"])).toEqual([2, 4]);
    expect(cleanDays([])).toEqual([]);
    expect(cleanDays([6, 0, -1, "1.5"])).toEqual([0, 6]);
  });
});

describe("amounts", () => {
  test("commas and spaces are dropped; blank stays blank; junk is refused", () => {
    expect(parseRupees("1,200")).toBe("1200");
    expect(parseRupees(" 1 200 ")).toBe("1200");
    expect(parseRupees("")).toBe("");
    expect(parseRupees("abc")).toBeNull();
    expect(parseRupees("12.5")).toBeNull();
    expect(parseRupees("-3")).toBeNull();
  });
});

describe("payload builders", () => {
  const form = { title: "  Blue Tokai ", place: " Goregaon ", maps_url: "https://maps.app.goo.gl/x", days: [4, 2], cost: "300", since: "" };
  test("a good add form", () => {
    const r = newRegularBody({ ...form, days: [4, 2, 4] });
    expect(r.error).toBe("");
    expect(r.body).toEqual({ title: "Blue Tokai", place: "Goregaon", maps_url: "https://maps.app.goo.gl/x", days: [2, 4], cost: "300", since: "" });
  });
  test("refusals say why", () => {
    expect(newRegularBody({ ...form, title: " " }).error).toBe("Give it a name.");
    expect(newRegularBody({ ...form, days: [] }).error).toBe("Pick at least one day.");
    expect(newRegularBody({ ...form, cost: "lots" }).body).toBeNull();
  });
  test("only a web address is sent as the maps link", () => {
    expect(newRegularBody({ ...form, maps_url: "javascript:alert(1)" }).body?.maps_url).toBe("");
  });
  test("went: blank means the usual", () => {
    expect(wentUsual("2026-11-05")).toEqual({ day: "2026-11-05", amount: "", place: "", back: "" });
    expect(wentElse("2026-11-05", " Elsewhere ", "450").body).toEqual({ day: "2026-11-05", amount: "450", place: "Elsewhere", back: "" });
    expect(wentElse("2026-11-05", "", "x").body).toBeNull();
    expect(wentElse("2026-11-05", "", "").body?.amount).toBe("");
  });
  test("skip", () => expect(skipBody("2026-11-05")).toEqual({ day: "2026-11-05", back: "" }));
});

describe("apiMessage", () => {
  test("prefers the server's {error}", () => {
    expect(apiMessage(new ApiError(422, '{"error":"Give it a name."}', { error: "Give it a name." }))).toBe("Give it a name.");
  });
  test("never shows raw JSON", () => {
    expect(apiMessage(new ApiError(400, '{"x":1}', { x: 1 }))).toBe("That didn’t work. Try again.");
  });
  test("plain text, network and server errors", () => {
    expect(apiMessage(new ApiError(403, "Link not valid", null))).toBe("Link not valid");
    expect(apiMessage(new ApiError(500, "boom", null), "Oops")).toBe("Oops");
    expect(apiMessage(new TypeError("x"))).toBe("No connection. Try again in a moment.");
  });
});

describe("photo shrinking", () => {
  test("fits inside 1600 and never scales up", () => {
    expect(fitWithin(4000, 3000)).toEqual({ width: 1600, height: 1200 });
    expect(fitWithin(3000, 4000)).toEqual({ width: 1200, height: 1600 });
    expect(fitWithin(800, 600)).toEqual({ width: 800, height: 600 });
    expect(fitWithin(0, 10)).toEqual({ width: 0, height: 0 });
  });
});
