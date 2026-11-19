import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { JoyClient, JoyError } from "./client.ts";
import { tools } from "./tools.ts";

export function buildServer(client: JoyClient): McpServer {
  const server = new McpServer({ name: "joy", version: "0.1.0" });
  for (const t of tools) {
    server.registerTool(
      t.name,
      { description: t.description, inputSchema: t.input ?? {}, annotations: { readOnlyHint: true, openWorldHint: false } },
      async (args: Record<string, unknown>) => {
        try {
          const data = await client.get(t.path(args), t.query?.(args));
          return { content: [{ type: "text" as const, text: JSON.stringify(data, null, 2) }] };
        } catch (e) {
          const msg = e instanceof JoyError ? e.message : "Request failed.";
          return { isError: true, content: [{ type: "text" as const, text: msg }] };
        }
      },
    );
  }
  return server;
}

export { z };
