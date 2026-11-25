// dates_near: turns "I'll be around CSMT tomorrow" into a short list of slips, so Claude does not
// have to read the whole jar (about 480 KB). Pure functions over the /api/dates/slips payload.
// Sealed surprises and partner marks are already filtered by the server; nothing here adds them back.

interface RawSlip {
  id: number;
  title: string;
  notes?: string;
  place?: string;
  cost?: string;
  duration?: string;
  setting_key?: string;
  tags?: string[];
  times_done?: number;
  closed_label?: string;
  maps_url?: string | null;
  both_want?: boolean;
}

// Neighbourhood words as they appear after the "·" in a slip's place. `near` is walkable or one
// short hop from the anchor; `hop` is a short cab or train ride away.
const AREAS: Record<
  string,
  { aliases: string[]; near: string[]; hop: string[] }
> = {
  csmt: {
    aliases: [
      "csmt",
      "cst",
      "vt",
      "victoria terminus",
      "chhatrapati shivaji",
      "fort",
      "kala ghoda",
    ],
    near: [
      "fort",
      "kala ghoda",
      "ballard estate",
      "colaba",
      "churchgate",
      "marine lines",
      "marine drive",
      "nariman point",
      "dhobi talao",
      "bhendi bazaar",
      "masjid",
    ],
    hop: [
      "girgaon",
      "chowpatty",
      "byculla",
      "sewri",
      "tardeo",
      "haji ali",
      "malabar hill",
      "mahalaxmi",
      "lower parel",
      "worli",
      "dadar",
      "prabhadevi",
      "elephanta",
    ],
  },
  churchgate: {
    aliases: ["churchgate", "marine drive", "nariman point", "cuffe parade"],
    near: [
      "churchgate",
      "marine drive",
      "marine lines",
      "nariman point",
      "fort",
      "kala ghoda",
      "colaba",
      "girgaon",
      "chowpatty",
    ],
    hop: [
      "malabar hill",
      "tardeo",
      "haji ali",
      "byculla",
      "worli",
      "lower parel",
      "mahalaxmi",
      "dhobi talao",
      "ballard estate",
    ],
  },
  bandra: {
    aliases: ["bandra", "khar", "pali hill", "mahim", "bkc"],
    near: [
      "bandra",
      "khar",
      "pali hill",
      "mahim",
      "bkc",
      "ranwar",
      "mount mary",
    ],
    hop: [
      "juhu",
      "andheri",
      "worli",
      "dadar",
      "lower parel",
      "kurla",
      "matunga",
    ],
  },
  andheri: {
    aliases: ["andheri", "versova", "lokhandwala", "juhu"],
    near: ["andheri", "versova", "lokhandwala", "juhu"],
    hop: ["bandra", "khar", "goregaon", "malad", "powai", "bkc"],
  },
  goregaon: {
    aliases: ["goregaon", "malad", "film city"],
    near: ["goregaon", "malad"],
    hop: ["andheri", "borivali", "powai", "versova"],
  },
  thane: {
    aliases: ["thane", "kalwa", "ghodbunder"],
    near: ["thane", "kalwa", "ghodbunder"],
    hop: ["vashi", "airoli", "kalyan", "powai", "ghatkopar"],
  },
  vashi: {
    aliases: [
      "vashi",
      "navi mumbai",
      "nerul",
      "turbhe",
      "airoli",
      "palm beach",
    ],
    near: ["vashi", "nerul", "turbhe", "airoli", "palm beach", "navi mumbai"],
    hop: ["panvel", "thane", "kurla", "ghatkopar"],
  },
  dadar: {
    aliases: [
      "dadar",
      "matunga",
      "prabhadevi",
      "lower parel",
      "worli",
      "mahalaxmi",
    ],
    near: [
      "dadar",
      "matunga",
      "prabhadevi",
      "lower parel",
      "worli",
      "mahalaxmi",
      "mahim",
    ],
    hop: [
      "girgaon",
      "tardeo",
      "haji ali",
      "byculla",
      "bandra",
      "sewri",
      "malabar hill",
    ],
  },
};

function norm(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9 ]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/** Which area table a free-text query ("CSMT", "around Fort", "Bandra West") belongs to, if any. */
export function resolveArea(query: string): string | undefined {
  const q = norm(query);
  if (!q) return undefined;
  for (const [key, a] of Object.entries(AREAS)) {
    if (a.aliases.some((w) => new RegExp(`(^| )${norm(w)}( |$)`).test(q)))
      return key;
  }
  return undefined;
}

export type Fit = "near" | "hop" | "anywhere";

/** How close a slip's place is to the anchor. Slips without a place (at home, flexible) are "anywhere". */
export function fit(
  place: string | undefined,
  area: string | undefined,
  query: string,
): Fit | undefined {
  if (!place) return "anywhere";
  const p = norm(place.split("·").slice(1).join(" ") || place);
  const full = norm(place);
  // Match the neighbourhood after the "·" only; the venue name can mislead ("Sewri Fort" is not Fort).
  const has = (w: string) => new RegExp(`(^| )${norm(w)}( |$)`).test(p);
  const a = area ? AREAS[area] : undefined;
  if (a?.near.some(has)) return "near";
  const q = norm(query);
  if (q && full.includes(q)) return "near";
  if (a?.hop.some(has)) return "hop";
  return undefined;
}

export interface NearArgs {
  area: string;
  budget_rupees?: number;
  max_hours?: number;
  include_far?: boolean;
  include_home?: boolean;
  limit?: number;
}

function rupees(cost: string | undefined): number | undefined {
  if (!cost || /free/i.test(cost)) return 0;
  const nums = [...cost.replace(/,/g, "").matchAll(/\d+/g)].map((m) =>
    Number(m[0]),
  );
  return nums.length ? Math.max(...nums) : undefined;
}

function hours(duration: string | undefined): number | undefined {
  if (!duration) return undefined;
  const d = duration.toLowerCase();
  if (d.includes("half a day")) return 4;
  if (d.includes("full day") || d.includes("whole day")) return 8;
  const nums = [...d.matchAll(/\d+(?:\.\d+)?/g)].map((m) => Number(m[0]));
  if (!nums.length) return undefined;
  return d.includes("min") && !d.includes("hour")
    ? Math.max(...nums) / 60
    : Math.max(...nums);
}

export function nearSlips(payload: unknown, args: NearArgs) {
  const groups = ((payload as { groups?: { items?: RawSlip[] }[] })?.groups ??
    []) as { items?: RawSlip[] }[];
  const all = groups.flatMap((g) => g.items ?? []);
  const area = resolveArea(args.area);
  const out: Record<Fit, ReturnType<typeof shape>[]> = {
    near: [],
    hop: [],
    anywhere: [],
  };
  for (const s of all) {
    const f = fit(s.place, area, args.area);
    if (!f) continue;
    if (f === "anywhere" && !args.include_home) continue;
    if (args.budget_rupees !== undefined) {
      const c = rupees(s.cost);
      if (c !== undefined && c > args.budget_rupees) continue;
    }
    if (args.max_hours !== undefined) {
      const h = hours(s.duration);
      if (h !== undefined && h > args.max_hours) continue;
    }
    out[f].push(shape(s));
  }
  const limit = args.limit ?? 15;
  // Fresh ideas first: never done, then both-want, then least done.
  const rank = (a: ReturnType<typeof shape>, b: ReturnType<typeof shape>) =>
    Number(b.both_want) - Number(a.both_want) || a.times_done - b.times_done;
  return {
    area_matched:
      area ?? "no known area; matched on the text in the place only",
    total_slips: all.length,
    near: out.near.sort(rank).slice(0, limit),
    short_hop:
      args.include_far === false ? [] : out.hop.sort(rank).slice(0, limit),
    anywhere: out.anywhere.sort(rank).slice(0, 5),
    counts: {
      near: out.near.length,
      short_hop: out.hop.length,
      anywhere: out.anywhere.length,
    },
  };
}

function shape(s: RawSlip) {
  return {
    id: s.id,
    title: s.title,
    place: s.place || "",
    cost: s.cost || "",
    duration: s.duration || "",
    tags: s.tags ?? [],
    times_done: s.times_done ?? 0,
    both_want: !!s.both_want,
    closed: s.closed_label || "",
    notes: (s.notes ?? "").slice(0, 160),
  };
}

export const NEAR_PROMPT = (where: string, when: string) =>
  [
    `I'll be around ${where || "south Mumbai"} ${when || "tomorrow"}. Help me plan the day with slips from the jar.`,
    "1. Call dates_near with that area (add budget_rupees / max_hours if I mention them).",
    "2. Call dates_coming_up and dates_kept so you don't clash with something already booked, and skip anything I did recently.",
    "3. Check closed labels against the day of the week; say when something may be shut.",
    "4. Offer 2 or 3 ordered itineraries (morning to evening) using near slips first, one short hop at most, with rough cost and walking or travel between stops. Ask whether diyaya is joining if it matters.",
    "Do not draw, keep, or change anything; only suggest. Never reveal sealed surprises or the partner's private marks.",
  ].join("\n");
