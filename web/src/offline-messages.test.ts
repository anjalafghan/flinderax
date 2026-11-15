import { describe, expect, test } from "bun:test";
import { describeFlush } from "./offline";

describe("what to say after sending what was waiting", () => {
  test("nothing sent, nothing said", () => {
    expect(describeFlush({ sent: 0, dropped: [], left: 0 })).toBeNull();
    expect(describeFlush({ sent: 0, dropped: [], left: 2 })).toBeNull();
  });
  test("says how many went", () => {
    expect(describeFlush({ sent: 1, dropped: [], left: 0 })).toBe("Sent 1 saved item.");
    expect(describeFlush({ sent: 3, dropped: [], left: 0 })).toBe("Sent 3 saved items.");
  });
  test("names what the server could not accept", () => {
    expect(describeFlush({ sent: 1, dropped: ["Slip: Picnic"], left: 0 })).toBe("Sent 1 saved item. Couldn't save: Slip: Picnic.");
  });
});
