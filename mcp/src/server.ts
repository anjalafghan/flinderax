import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { JoyClient, JoyError } from "./client.ts";
import { tools } from "./tools.ts";
import { NEAR_PROMPT, nearSlips, type NearArgs } from "./near.ts";
import { registerCustomDates } from "./custom.ts";
import {
  CATEGORIES,
  SORT_INBOX_PROMPT,
  maskDigits,
  rupeesToPaise,
  shortHash,
  txnKey,
  type NewTxn,
} from "./inbox.ts";

export function scopesForReadTool(name: string): string[] {
  if (name === "get_brief" || name === "money_export_all" || name === "pace_status") {
    return ["money:read", "dates:read"];
  }
  if (name.startsWith("dates_") || name === "preview_draw" || name === "pending_did_it_happen") {
    return ["dates:read"];
  }
  return ["money:read"];
}

export function buildServer(client: JoyClient, allowedScopes?: string[]): McpServer {
  const server = new McpServer({ name: "joy", version: "0.1.0" });
  for (const t of tools) {
    const required = scopesForReadTool(t.name);
    if (allowedScopes && required.some((s) => !allowedScopes.includes(s))) continue;
    server.registerTool(
      t.name,
      {
        description: t.description,
        inputSchema: t.input ?? {},
        annotations: { readOnlyHint: true, openWorldHint: false },
      },
      async (args: Record<string, unknown>) => {
        try {
          const data = await client.get(t.path(args), t.query?.(args));
          return {
            content: [
              { type: "text" as const, text: JSON.stringify(data, null, 2) },
            ],
          };
        } catch (e) {
          const msg = e instanceof JoyError ? e.message : "Request failed.";
          return {
            isError: true,
            content: [{ type: "text" as const, text: msg }],
          };
        }
      },
    );
  }

  if (!allowedScopes || allowedScopes.includes("dates:read")) server.registerTool(
    "dates_near",
    {
      description:
        "Slips that fit where I'll be: pass a station or area ('CSMT', 'Bandra', 'Churchgate', 'Thane'...). Returns a short list grouped as near (walkable), short_hop (a cab or train away) and anywhere, instead of the whole jar. Optional budget in rupees and a max duration in hours. Read-only.",
      inputSchema: {
        area: z.string().min(1).max(60),
        budget_rupees: z.number().int().min(0).optional(),
        max_hours: z.number().min(0.25).max(24).optional(),
        include_far: z.boolean().optional(),
        include_home: z.boolean().optional(),
        limit: z.number().int().min(1).max(40).optional(),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async (args: NearArgs) => {
      try {
        const data = await client.get("/api/dates/slips");
        return {
          content: [
            {
              type: "text" as const,
              text: JSON.stringify(nearSlips(data, args), null, 2),
            },
          ],
        };
      } catch (e) {
        return {
          isError: true,
          content: [
            {
              type: "text" as const,
              text: e instanceof JoyError ? e.message : "Request failed.",
            },
          ],
        };
      }
    },
  );

  if (!allowedScopes || allowedScopes.includes("dates:read")) server.registerPrompt(
    "plan_outing",
    {
      title: "Plan a day out from the slips",
      description:
        "Tell me where you'll be and when; I'll suggest itineraries from the jar.",
      argsSchema: { where: z.string().optional(), when: z.string().optional() },
    },
    ({ where, when }: { where?: string; when?: string }) => ({
      messages: [
        {
          role: "user" as const,
          content: {
            type: "text" as const,
            text: NEAR_PROMPT(where ?? "", when ?? ""),
          },
        },
      ],
    }),
  );

  if (!allowedScopes || (allowedScopes.includes("money:read") && allowedScopes.includes("dates:read"))) {
    server.registerTool(
      "pace_status",
      {
        description: "Compare this month's money spending and Dates spending with the pace so far. Reports ahead, on pace, or behind; amounts are paise.",
        inputSchema: {},
        annotations: { readOnlyHint: true, openWorldHint: false },
      },
      async () => {
        try {
          const b = await client.get("/api/brief") as {
            as_of: string;
            money_allowance_this_month_paise: number;
            money_spent_this_month_paise: number;
            dates_budget_this_month_paise: number | null;
            dates_spent_this_month_paise: number;
          };
          const [year, month, day] = b.as_of.split("-").map(Number);
          const days = new Date(year, month, 0).getDate();
          const ratio = day / days;
          const pace = (spent: number, budget: number | null) => {
            if (budget === null) return { status: "no_budget", spent_paise: spent, expected_by_today_paise: null };
            const expected = Math.floor(budget * ratio);
            return { status: spent > expected ? "ahead" : spent < expected ? "behind" : "on_pace", spent_paise: spent, expected_by_today_paise: expected };
          };
          return {
            content: [{ type: "text" as const, text: JSON.stringify({
              money: pace(b.money_spent_this_month_paise, b.money_allowance_this_month_paise),
              dates: pace(b.dates_spent_this_month_paise, b.dates_budget_this_month_paise),
            }, null, 2) }],
          };
        } catch (e) {
          return failed(e, "Could not check this month's pace.");
        }
      },
    );
  }

  registerInbox(server, client, allowedScopes);
  registerCustomDates(server, client, allowedScopes);

  if (!allowedScopes || allowedScopes.includes("analysis:write")) server.registerTool(
    "save_analysis",
    {
      description:
        "Save a written analysis to the joy app, where it shows on the Review page. Title up to 120 characters, body up to 20,000 characters of plain text (no HTML). Write it in rupees, not paise.",
      inputSchema: {
        title: z.string().min(1).max(120),
        body: z.string().min(1).max(20000),
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ title, body }: { title: string; body: string }) => {
      try {
        const saved = (await client.saveAnalysis(title, body)) as {
          id?: string;
        };
        return {
          content: [
            {
              type: "text" as const,
              text: `Saved. It is now on the Review page (id ${saved.id ?? "unknown"}).`,
            },
          ],
        };
      } catch (e) {
        return {
          isError: true,
          content: [
            {
              type: "text" as const,
              text: e instanceof JoyError ? e.message : "Could not save.",
            },
          ],
        };
      }
    },
  );

  if (!allowedScopes || allowedScopes.includes("analysis:write")) server.registerPrompt(
    "analyse_money",
    {
      title: "Analyse my money and save it to the app",
      description:
        "Read the full export and the forecast, find what matters, and save a short report to the Review page.",
      argsSchema: { focus: z.string().optional() },
    },
    ({ focus }: { focus?: string }) => ({
      messages: [
        {
          role: "user" as const,
          content: { type: "text" as const, text: analysePrompt(focus) },
        },
      ],
    }),
  );
  return server;
}

const text = (t: string) => ({ content: [{ type: "text" as const, text: t }] });
const failed = (e: unknown, fallback: string) => ({
  isError: true,
  content: [
    {
      type: "text" as const,
      text:
        e instanceof JoyError ||
        (e instanceof Error && !(e instanceof TypeError))
          ? e.message
          : fallback,
    },
  ],
});

/** The inbox of bank texts the parser missed: read them masked, post what they say, clear them. */
export function registerInbox(server: McpServer, client: JoyClient, allowedScopes?: string[]) {
  const has = (scope: string) => !allowedScopes || allowedScopes.includes(scope);
  if (has("money:read")) server.registerTool(
    "money_unrecognized",
    {
      description:
        "Bank texts from the last 30 days that joy's parser could not read, newest first, so they can be turned into transactions by hand. Numbers of 8 digits or more are masked to the last 4. Read-only.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      try {
        const inbox = (await client.get("/api/ingest/unrecognized")) as {
          count: number;
          items: { text: string }[];
        };
        const items = inbox.items.map((i) => ({
          ...i,
          text: maskDigits(i.text),
        }));
        return text(JSON.stringify({ count: inbox.count, items }, null, 2));
      } catch (e) {
        return failed(e, "Request failed.");
      }
    },
  );

  if (has("money:write")) server.registerTool(
    "money_add_transaction",
    {
      description:
        "Add one transaction (source 'manual'). Give the amount in rupees as written (\"1,250.50\"); it is turned into paise exactly. Pass alert_id when it comes from money_unrecognized: the alert is cleared once the transaction is saved, and calling again with the same alert_id never adds it twice. Typed by hand, identical calls also add once; for two real, identical spends on one day pass occurrence 2. Ask the person before calling this.",
      inputSchema: {
        occurred_on: z.string().regex(/^\d{4}-\d{2}-\d{2}$/),
        direction: z.enum(["debit", "credit"]),
        amount_rupees: z.string().min(1).max(24),
        merchant: z.string().max(120).optional(),
        category: z.enum(CATEGORIES).optional(),
        alert_id: z.string().min(1).max(80).optional(),
        occurrence: z.number().int().min(1).max(50).optional(),
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async (t: NewTxn) => {
      try {
        const amount_paise = rupeesToPaise(t.amount_rupees);
        const body = {
          occurred_on: t.occurred_on,
          direction: t.direction,
          amount_paise,
          merchant: t.merchant,
          category: t.category ?? "uncategorized",
        };
        const saved = (await client.post(
          "/api/ingest/transactions/create",
          body,
          await txnKey(t, amount_paise),
        )) as { txn_id?: string };
        let note = "";
        if (t.alert_id) {
          try {
            await client.post(
              "/api/ingest/unrecognized/dismiss",
              { alert_id: t.alert_id },
              `mcp-dismiss:${t.alert_id}`.slice(0, 100),
            );
            note = " The alert is cleared from the inbox.";
          } catch {
            note =
              " The alert could not be cleared; dismiss it with money_dismiss_unrecognized.";
          }
        }
        return text(
          `Added (txn ${saved.txn_id ?? "unknown"}, ${amount_paise} paise).${note}`,
        );
      } catch (e) {
        return failed(e, "Could not add the transaction.");
      }
    },
  );

  if (has("inbox:write")) server.registerTool(
    "money_dismiss_unrecognized",
    {
      description:
        "Clear one text from the unrecognised inbox: it is not a transaction (an OTP, a promo), or it was added already. Ask the person first.",
      inputSchema: { alert_id: z.string().min(1).max(80) },
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ alert_id }: { alert_id: string }) => {
      try {
        await client.post(
          "/api/ingest/unrecognized/dismiss",
          { alert_id },
          `mcp-dismiss:${alert_id}`.slice(0, 100),
        );
        return text("Cleared from the inbox.");
      } catch (e) {
        return failed(e, "Could not clear it.");
      }
    },
  );

  if (has("money:write")) server.registerTool(
    "money_categorize_payee",
    {
      description:
        "Give every untagged row of one payee (as money_uncategorized_payees or money_transactions names it) a category. Only blanks are filled: rows already tagged, or locked by a hand correction, are left alone. remember=true also saves a rule so future texts and statements from that payee come in tagged. Answers how many rows changed. Ask the person first.",
      inputSchema: {
        payee: z.string().min(1).max(120),
        category: z.enum(CATEGORIES).exclude(["uncategorized"]),
        remember: z.boolean().optional(),
      },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({
      payee,
      category,
      remember,
    }: {
      payee: string;
      category: (typeof CATEGORIES)[number];
      remember?: boolean;
    }) => {
      try {
        // Keyed by the day too: a retry today is done once, but tagging the same payee again tomorrow
        // (after new rows arrived) is a fresh request rather than a replay of today's answer.
        const day = new Date().toISOString().slice(0, 10);
        const key = `mcp-payee:${await shortHash(`${day}|${payee.trim().toLowerCase()}|${category}|${remember ? 1 : 0}`)}`;
        const res = (await client.post(
          "/api/ingest/payees/categorize",
          { payee, category, remember: !!remember },
          key,
        )) as { updated?: number };
        return text(
          `Tagged ${res.updated ?? 0} row(s) of ${payee} as ${category}.${remember ? " Remembered for future imports." : ""}`,
        );
      } catch (e) {
        return failed(e, "Could not tag that payee.");
      }
    },
  );

  if (has("money:write")) server.registerTool(
    "money_parse_text",
    {
      description:
        "Run joy's own bank-text parser on one message (say a cleaned-up copy of an unrecognised text). If it reads it, the transaction is saved with the card or account matched, exactly as if the phone had forwarded it; the answer says created, duplicate or unrecognized. Sending the same text again does nothing. Ask the person first.",
      inputSchema: { text: z.string().min(1).max(2000) },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ text: msg }: { text: string }) => {
      try {
        const res = (await client.post(
          "/api/ingest/parse",
          { text: msg },
          `mcp-parse:${await shortHash(msg)}`,
        )) as { status?: string };
        return text(JSON.stringify(res, null, 2));
      } catch (e) {
        return failed(e, "Could not parse it.");
      }
    },
  );

  if (has("money:write") && has("inbox:write")) server.registerPrompt(
    "sort_inbox",
    {
      title: "Sort the bank texts joy could not read",
      description:
        "Turn the unrecognised-alert inbox into transactions, with your yes before anything is added.",
    },
    () => ({
      messages: [
        {
          role: "user" as const,
          content: { type: "text" as const, text: SORT_INBOX_PROMPT },
        },
      ],
    }),
  );
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
