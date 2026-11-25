// Shared look for the people screens. Plain constants and helpers only, so the component files stay component-only.

export const PANEL =
  "glass flex flex-col gap-6 rounded-[28px] px-6 py-8 sm:px-12 sm:py-10";
export const SECTION_TITLE = "m-0 font-display text-[32px] leading-none";
export const NOTE = "m-0 text-sm text-muted-foreground";
export const PILL =
  "inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold";
export const TEXT_LINK = "self-start text-sm font-medium text-primary";
export const NOTE_MAX = 140;
export const NAME_MAX = 40;
export const SHORT_NOTE_MAX = 280;

/** The server's plain-text message when there is one, so 422 and 409 answers read as written. */
export function errorMessage(e: unknown): string {
  return e instanceof Error && e.message
    ? e.message
    : "Something went wrong. Try again.";
}

export const MONTH_NAMES = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

/** Party colours for people (see theme/color.css); a name always gets the same one. */
const AVATAR_HUES = ["rose", "sun", "mint", "sky", "grape", "coral", "teal", "berry"];
export function avatarHue(name: string): string {
  let h = 0;
  for (const ch of name) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return AVATAR_HUES[h % AVATAR_HUES.length]!;
}

/** One or two letters from a name: "Asha Rao" is "AR". */
export function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  const first = parts[0]?.[0] ?? "?";
  const last = parts.length > 1 ? (parts[parts.length - 1]?.[0] ?? "") : "";
  return (first + last).toUpperCase();
}

/** The avatar ring by catch-up state: overdue coral, due soon sun, on track mint. */
export const STATUS_RING = {
  overdue: "ring-coral",
  due_soon: "ring-sun",
  ok: "ring-mint",
} as const;

/** Each kind of date has its colour: birthdays in sun, anniversaries in rose. */
export const KIND_TONE: Record<string, string> = {
  birthday: "bg-sun/25 border-sun/60",
  anniversary: "bg-rose/15 border-rose/50",
  remembrance: "bg-grape/15 border-grape/50",
  other: "bg-sky/15 border-sky/50",
};

/** Each circle has its colour, used for the group dot. */
export const CIRCLE_DOT: Record<string, string> = {
  family: "bg-rose",
  close: "bg-berry",
  friend: "bg-sky",
  work: "bg-teal",
  other: "bg-sun",
};
