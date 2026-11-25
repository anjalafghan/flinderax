// Each spending category keeps one party colour everywhere it shows (bars, dots, sorter stamps),
// so "Food" is always coral and "Travel" always sky. Colours are the tokens in theme/color.css.

export type PartyColor =
  | "sun"
  | "coral"
  | "mint"
  | "sky"
  | "grape"
  | "rose"
  | "teal"
  | "berry";

/** The eight party colours in a pleasing order, for series without a fixed meaning. */
export const PARTY: readonly PartyColor[] = [
  "mint",
  "sky",
  "grape",
  "coral",
  "sun",
  "rose",
  "teal",
  "berry",
];

const CATEGORY_COLOR: Record<string, PartyColor> = {
  food: "coral",
  groceries: "mint",
  shopping: "berry",
  transport: "sky",
  fuel: "sun",
  bills: "teal",
  entertainment: "grape",
  health: "rose",
  travel: "sky",
  rent: "grape",
  emi: "coral",
  cash: "sun",
  dates: "rose",
  people: "berry",
  transfer: "teal",
  income: "mint",
  investments: "sky",
  fees: "coral",
  refund: "mint",
  other: "teal",
  uncategorized: "sun",
};

/** The party colour name for a category; unknown ids get a stable colour from their name. */
export function categoryParty(id: string): PartyColor {
  const known = CATEGORY_COLOR[id];
  if (known) return known;
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) >>> 0;
  return PARTY[h % PARTY.length];
}

/** A CSS colour for a party colour, optionally see-through: partyColor("mint", 0.2). */
export function partyColor(name: PartyColor, alpha = 1): string {
  return alpha >= 1
    ? `hsl(var(--c-${name}))`
    : `hsl(var(--c-${name}) / ${alpha})`;
}

/** A CSS colour for a category: categoryColor("food") -> coral. */
export function categoryColor(id: string, alpha = 1): string {
  return partyColor(categoryParty(id), alpha);
}

/** Hex versions for canvas-style bits (SloshGauge) that can't read CSS variables. */
export const PARTY_HEX: Record<PartyColor, string> = {
  sun: "#8a8073",
  coral: "#8a8073",
  mint: "#8a8073",
  sky: "#8a8073",
  grape: "#8a8073",
  rose: "#8a8073",
  teal: "#8a8073",
  berry: "#8a8073",
};

/** A stable party colour for anything with an id (a goal, a pot): same id, same colour. */
export function colorForId(id: string | number): PartyColor {
  return categoryParty(`#${id}`);
}
