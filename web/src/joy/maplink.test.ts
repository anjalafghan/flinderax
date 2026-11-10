import { describe, expect, test } from "bun:test";
import { parseMapLink } from "./maplink";

describe("parseMapLink", () => {
  test("reads the @lat,lng form", () => {
    expect(parseMapLink("https://www.google.com/maps/place/Carter+Road/@19.0687,72.8226,17z/data=x")).toEqual({ kind: "coords", location: "19.0687, 72.8226" });
  });
  test("reads ?q=lat,lng, also url-encoded", () => {
    expect(parseMapLink("https://maps.google.com/?q=19.13,72.81")).toEqual({ kind: "coords", location: "19.13, 72.81" });
    expect(parseMapLink("https://maps.google.com/?q=19.13%2C72.81")).toEqual({ kind: "coords", location: "19.13, 72.81" });
  });
  test("prefers the pin (!3d!4d) over the map centre", () => {
    const u = "https://www.google.com/maps/place/X/@19.10,72.80,15z/data=!3m1!4b1!4m6!3m5!1s0x0:0x1!8m2!3d19.13!4d72.81";
    expect(parseMapLink(u)).toEqual({ kind: "coords", location: "19.13, 72.81" });
  });
  test("handles negative coordinates", () => {
    expect(parseMapLink("https://www.google.com/maps/@-33.86,151.21,12z")).toEqual({ kind: "coords", location: "-33.86, 151.21" });
  });
  test("short links cannot be opened in the browser", () => {
    expect(parseMapLink("https://maps.app.goo.gl/abc123")).toEqual({ kind: "short" });
  });
  test("rejects out of range and non-links", () => {
    expect(parseMapLink("https://x.test/@95.1,72.8,1z")).toEqual({ kind: "none" });
    expect(parseMapLink("hello")).toEqual({ kind: "none" });
    expect(parseMapLink("  ")).toEqual({ kind: "none" });
  });
});
