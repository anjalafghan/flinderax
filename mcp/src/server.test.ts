import { describe, expect, test } from "bun:test";
import { scopesForReadTool } from "./server.ts";

describe("remote MCP read scopes", () => {
  test("combined summaries need both money and dates access", () => {
    expect(scopesForReadTool("get_brief")).toEqual(["money:read", "dates:read"]);
    expect(scopesForReadTool("pace_status")).toEqual(["money:read", "dates:read"]);
  });

  test("date tools never appear for a money-only token", () => {
    expect(scopesForReadTool("dates_jar")).toEqual(["dates:read"]);
    expect(scopesForReadTool("preview_draw")).toEqual(["dates:read"]);
    expect(scopesForReadTool("pending_did_it_happen")).toEqual(["dates:read"]);
  });

  test("money tools stay on the money scope", () => {
    expect(scopesForReadTool("money_summary")).toEqual(["money:read"]);
  });
});
