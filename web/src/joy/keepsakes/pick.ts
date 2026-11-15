// Which keepsake a kept date is drawn as, on the Month and Year pages. Pure: the date's setting
// and its slip's tags in, one kind out. A trip, a day out or a night apart is drawn by where we were;
// otherwise by what we did, by how strongly a tag says what we'd have come home with.

export type Keepsake =
  | "ticket" // around town, anything else: a ticket stub
  | "boarding" // travel
  | "postcard" // a day trip
  | "airmail" // long distance
  | "recipe" // at home
  | "gig" // music, nightlife
  | "arcade" // games
  | "admission" // cultural
  | "swingtag" // creative: a workshop's tag
  | "trail" // outdoors, adventurous, active
  | "coaster"; // foodie

export const KEEPSAKES: readonly Keepsake[] = ["ticket", "boarding", "postcard", "airmail", "recipe", "gig", "arcade", "admission", "swingtag", "trail", "coaster"];

/** Where we were decides first when the place is the memory: a trip, a day out, a call apart. */
const BY_SETTING: Record<string, Keepsake> = { travel: "boarding", day_trip: "postcard", apart: "airmail" };

/** Then what we did, strongest first: food beats "cultural" (a dinner is a dinner), music beats food. */
const BY_TAG: [string, Keepsake][] = [
  ["music", "gig"],
  ["foodie", "coaster"],
  ["nightlife", "gig"],
  ["games", "arcade"],
  ["cultural", "admission"],
  ["creative", "swingtag"],
  ["outdoors", "trail"],
  ["adventurous", "trail"],
  ["active", "trail"],
];

export function keepsakeFor(setting: string, tags: readonly string[]): Keepsake {
  const bySetting = BY_SETTING[setting];
  if (bySetting) return bySetting;
  const have = new Set(tags.map((t) => t.trim().toLowerCase()));
  // At home, cooking (or nothing louder than a cosy night in) is a recipe card.
  if (setting === "home" && (have.has("foodie") || !BY_TAG.some(([t]) => have.has(t)))) return "recipe";
  for (const [tag, k] of BY_TAG) if (have.has(tag)) return k;
  return "ticket";
}

/** A three-letter "airport code" for a boarding pass: the first word of the place (or title) with letters only. */
export function codeOf(place: string, title: string): string {
  const word = (s: string) => (s.match(/\p{L}+/gu) ?? []).find((w) => w.length >= 3) ?? "";
  const w = word(place) || word(title) || "JOY";
  return w.slice(0, 3).toUpperCase();
}
