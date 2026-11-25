/** The party colours from the theme. */
export type Hue =
  | "sun"
  | "coral"
  | "mint"
  | "sky"
  | "grape"
  | "rose"
  | "teal"
  | "berry";

/** A party colour as CSS, optionally see-through: hue("rose", 0.15). */
export const hue = (h: Hue, alpha?: number) =>
  alpha === undefined
    ? `hsl(var(--c-${h}))`
    : `hsl(var(--c-${h}) / ${alpha})`;

/** A party colour for text: mixed toward the text colour, so it stays readable in light and dark. */
export const ink = (h: Hue) =>
  `color-mix(in oklab, hsl(var(--c-${h})) 68%, hsl(var(--foreground)))`;
