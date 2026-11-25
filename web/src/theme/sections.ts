/**
 * Each tab has its own colour, so the whole app is never just ink on oat.
 * CSS reads `html[data-section]` (see src/theme/color.css); WebGL backgrounds and sparks read the hex lists here.
 */
export type Section = "today" | "money" | "life" | "us" | "review" | "settings" | "auth";

export interface SectionColors {
  /** Three hex colours, light to deep, for backgrounds, gradients and sparks. */
  light: [string, string, string];
  dark: [string, string, string];
}

export const SECTION_COLORS: Record<Section, SectionColors> = Object.fromEntries(
  (["today", "money", "life", "us", "review", "settings", "auth"] as Section[]).map((s) => [
    s,
    // The original palette: warm ink on oat by day, cream on espresso by night.
    { light: ["#3a342c", "#1c1a17", "#6b6257"], dark: ["#efe7d8", "#c9bfae", "#a39888"] },
  ]),
) as Record<Section, SectionColors>;

/** Kept for callers that want a list; all quiet ink tones now. */
export const PARTY = ["#1c1a17", "#3a342c", "#6b6257", "#8a8073", "#1c1a17", "#3a342c", "#6b6257", "#8a8073"];

const under = (pathname: string, p: string) => pathname === p || pathname.startsWith(`${p}/`);

const PREFIXES: [Section, string[]][] = [
  ["money", ["/money", "/cards", "/accounts", "/spending", "/plan", "/working"]],
  ["life", ["/life", "/glance", "/wedding", "/year", "/what-if", "/work", "/health"]],
  ["us", ["/joy", "/people"]],
  ["review", ["/week", "/review", "/checkin"]],
  ["settings", ["/account"]],
  ["auth", ["/auth"]],
];

/** The section a path belongs to. Mirrors the tab prefixes in components/layout/Header.tsx. */
export function sectionFor(pathname: string): Section {
  for (const [s, ps] of PREFIXES) if (ps.some((p) => under(pathname, p))) return s;
  return "today";
}
