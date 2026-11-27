// Days marked by hand ("the day we met", a night that wasn't a slip): list them, and fill in their
// slips so they read like any slip from the jar. The server fills blanks only and never renames a
// slip, because its illustrated keepsake is found by title.
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { JoyError, type JoyClient } from "./client.ts";
import { shortHash } from "./inbox.ts";

const text = (t: string) => ({ content: [{ type: "text" as const, text: t }] });
const failed = (e: unknown, fallback: string) => ({
  isError: true,
  content: [
    {
      type: "text" as const,
      text: e instanceof JoyError ? e.message : fallback,
    },
  ],
});

export const DRESS_FIELDS = {
  notes: z.string().max(2000).optional(),
  venue: z.string().max(120).optional(),
  area: z.string().max(120).optional(),
  location: z
    .string()
    .regex(/^-?\d{1,2}(\.\d+)?,\s*-?\d{1,3}(\.\d+)?$/)
    .optional(),
  tags: z.string().max(400).optional(),
  energy: z.enum(["low", "med", "high"]).optional(),
  duration: z.enum(["short", "half_day", "full_day", "weekend"]).optional(),
  best_time: z.enum(["morning", "afternoon", "evening", "night"]).optional(),
  minutes: z.number().int().min(10).max(10080).optional(),
  hours: z.string().max(200).optional(),
  cover: z.enum(["indoor", "outdoor", "mixed"]).optional(),
  bring: z.string().max(200).optional(),
  station: z.string().max(60).optional(),
  station_walk_min: z.number().int().min(0).max(120).optional(),
  url: z.string().url().max(300).optional(),
  in_jar: z.boolean().optional(),
};
type DressArgs = { slip_id: number } & {
  [K in keyof typeof DRESS_FIELDS]?: z.infer<(typeof DRESS_FIELDS)[K]>;
};

export const DRESS_PROMPT = [
  "Go through the days we marked by hand in joy (dates that never were slips) and make each one as good as a slip from the jar.",
  "1. Call dates_custom. Work on the ones with dressed: false, oldest first. The title is fixed: never suggest renaming it.",
  "2. For each, work out what it was from the title, the place and the day (look the place up if it is a real venue: what it is known for, the area, nearest station, hours, a link). Ask me what you can't know (what we did, how it felt) rather than inventing it.",
  "3. Show me, per day, the details you would add: notes (two or three warm, specific sentences in our voice, the way our other slips read: call dates_slips for examples), area, location, tags, energy, duration, best time, how long, indoor/outdoor, what to bring, station, link. And whether it should go in the jar to be drawn again: yes for a place or activity we'd repeat, no for a one-time milestone like the day we met.",
  "4. Only after my yes, call dates_dress_custom once per day. It fills blanks only; report what it filled and what it left alone.",
  "5. The illustrated keepsake is code, not data: when you are in the flinderax repository, offer to draw one per day with the keepsake pipeline in tools/keepsakes (a short slip module using kit.py, keyed by the slip's title lower-cased, built with python3 tools/keepsakes/build.py, checked on the contact sheet), to the same bar as the KOKO keepsake. Outside the repository, list the days that still need one.",
  "Never touch slips that were drawn from the jar, and never reveal a sealed surprise or my partner's private answers.",
].join("\n");

export function registerCustomDates(server: McpServer, client: JoyClient, allowedScopes?: string[]) {
  const has = (scope: string) => !allowedScopes || allowedScopes.includes(scope);
  if (has("dates:read")) server.registerTool(
    "dates_custom",
    {
      description:
        "Days we marked by hand rather than drew from the jar (the day we met, an evening that wasn't a slip), newest first: the one-off slip behind each (slip_id, title, kept_on days, place, kind) with whatever details it has, and dressed: false while it has no notes yet. Slips that were joined to the jar no longer show here. Amounts in paise. Read-only.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      try {
        return text(
          JSON.stringify(await client.get("/api/dates/custom"), null, 2),
        );
      } catch (e) {
        return failed(e, "Request failed.");
      }
    },
  );

  if (has("dates:write")) server.registerTool(
    "dates_dress_custom",
    {
      description:
        'Fill in the slip of one hand-marked day (slip_id from dates_custom) so it reads like a jar slip: notes, place, map location ("19.06, 72.83"), tags, energy, duration, best time, minutes, hours ("tue-sun 12:00-23:30"), indoor/outdoor, what to bring, nearest station and walk, a link. Only blank fields are filled (energy and duration may be set the first time, while the slip has no notes); the title never changes. in_jar: true also puts it in the jar to be drawn again (not for one-time milestones). Answers which fields were filled and which were left alone. Ask the person first.',
      inputSchema: { slip_id: z.number().int().positive(), ...DRESS_FIELDS },
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async ({ slip_id, ...fields }: DressArgs) => {
      try {
        const body = Object.fromEntries(
          Object.entries(fields).filter(([, v]) => v !== undefined && v !== ""),
        );
        const key = `mcp-dress:${slip_id}:${await shortHash(JSON.stringify(body))}`;
        return text(
          JSON.stringify(
            await client.post(`/api/dates/slips/${slip_id}/dress`, body, key),
            null,
            2,
          ),
        );
      } catch (e) {
        return failed(e, "Could not fill in that slip.");
      }
    },
  );

  if (has("dates:write")) server.registerPrompt(
    "dress_custom_dates",
    {
      title: "Turn hand-marked days into proper slips",
      description:
        "Find the days we marked by hand and fill in their slips (and keepsakes) so they look as good as the jar's, with your yes first.",
    },
    () => ({
      messages: [
        {
          role: "user" as const,
          content: { type: "text" as const, text: DRESS_PROMPT },
        },
      ],
    }),
  );
}
