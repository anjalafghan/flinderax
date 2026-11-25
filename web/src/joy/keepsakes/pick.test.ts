import { describe, expect, test } from "bun:test";
import { codeOf, keepsakeFor } from "./pick";

describe("keepsakeFor", () => {
  test("a trip, a day out or a night apart is drawn by where we were", () => {
    expect(keepsakeFor("travel", ["foodie", "music"])).toBe("boarding");
    expect(keepsakeFor("day_trip", ["foodie"])).toBe("postcard");
    expect(keepsakeFor("apart", ["games"])).toBe("airmail");
  });

  test("around town, by what we did, strongest tag first", () => {
    expect(keepsakeFor("local", ["nightlife", "music", "games"])).toBe("gig"); // karaoke
    expect(keepsakeFor("local", ["foodie", "cultural", "romantic"])).toBe(
      "coaster",
    ); // a dinner is a dinner
    expect(keepsakeFor("local", ["foodie", "outdoors", "games"])).toBe(
      "coaster",
    ); // a food-stall hop
    expect(keepsakeFor("local", ["romantic", "nightlife", "chill"])).toBe(
      "gig",
    ); // a speakeasy
    expect(keepsakeFor("local", ["games", "chill"])).toBe("arcade");
    expect(keepsakeFor("local", ["cultural"])).toBe("admission");
    expect(keepsakeFor("local", ["creative", "cozy"])).toBe("swingtag");
    expect(keepsakeFor("local", ["adventurous", "chill"])).toBe("trail");
    expect(keepsakeFor("local", ["romantic", "chill"])).toBe("ticket");
    expect(keepsakeFor("local", [])).toBe("ticket");
  });

  test("at home: cooking or a quiet night in is a recipe card, anything louder is itself", () => {
    expect(keepsakeFor("home", ["foodie", "cozy"])).toBe("recipe");
    expect(keepsakeFor("home", ["cozy", "romantic"])).toBe("recipe");
    expect(keepsakeFor("home", [])).toBe("recipe");
    expect(keepsakeFor("home", ["games", "chill"])).toBe("arcade");
    expect(keepsakeFor("home", ["music"])).toBe("gig");
  });

  test("tags are matched loosely", () => {
    expect(keepsakeFor("local", [" Music "])).toBe("gig");
  });
});

describe("codeOf", () => {
  test("three letters from the place, else the title", () => {
    expect(codeOf("Alibaug", "A slow weekend")).toBe("ALI");
    expect(codeOf("", "Vineyard weekend in Nashik")).toBe("VIN");
    expect(codeOf("", "")).toBe("JOY");
    expect(codeOf("Ajanta & Ellora", "")).toBe("AJA");
  });
});
