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

  server.registerTool(
    "save_analysis",
    {
      description:
        "Save a written analysis to the joy app, where it shows on the Review page. This is the only tool that writes. Title up to 120 characters, body up to 20,000 characters of plain text (no HTML). Write it in rupees, not paise.",
      inputSchema: { title: z.string().min(1).max(120), body: z.string().min(1).max(20000) },
      annotations: { readOnlyHint: false, destructiveHint: false, idempotentHint: true, openWorldHint: false },
    },
    async ({ title, body }: { title: string; body: string }) => {
      try {
        const saved = (await client.saveAnalysis(title, body)) as { id?: string };
        return { content: [{ type: "text" as const, text: `Saved. It is now on the Review page (id ${saved.id ?? "unknown"}).` }] };
      } catch (e) {
        return { isError: true, content: [{ type: "text" as const, text: e instanceof JoyError ? e.message : "Could not save." }] };
      }
    },
  );

  server.registerPrompt(
    "analyse_money",
    {
      title: "Analyse my money and save it to the app",
      description: "Read the full export and the forecast, find what matters, and save a short report to the Review page.",
      argsSchema: { focus: z.string().optional() },
    },
    ({ focus }: { focus?: string }) => ({
      messages: [{ role: "user" as const, content: { type: "text" as const, text: analysePrompt(focus) } }],
    }),
  );
  return server;
}

export function analysePrompt(focus?: string): string {
  return [
    "Analyse my money in joy and save what you find.",
    focus ? `Focus on: ${focus}.` : "",
    "1. Call analyses_list first, so you don't repeat an earlier report.",
    "2. Call money_export_all for the full history, and money_forecast and money_commitments for what is ahead.",
    "3. Work out the numbers carefully from the rows (amounts are integer paise; 100 paise = 1 rupee). Show the figures you rely on, and say so when the data is too thin to conclude.",
    "4. Find: where spending changed and why, recurring leaks, what the next few months look like against commitments and income, and one to three concrete actions. Weigh them against my goals (a wedding, one child).",
    "5. Call save_analysis once with a plain-text report: a short title and a body of a few short sections. Use rupees. No HTML.",
    "Do not change anything else. Do not reveal my partner's private answers or any sealed surprise.",
  ]
    .filter(Boolean)
    .join("\n");
}

export { z };
