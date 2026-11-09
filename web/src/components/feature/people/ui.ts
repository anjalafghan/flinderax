// Shared look for the people screens. Plain constants and helpers only, so the component files stay component-only.

export const PANEL = "flex flex-col gap-6 rounded-[28px] border border-border bg-card px-6 py-8 sm:px-12 sm:py-10";
export const SECTION_TITLE = "m-0 font-display text-[32px] leading-none";
export const NOTE = "m-0 text-sm text-muted-foreground";
export const PILL = "inline-flex min-h-7 shrink-0 items-center whitespace-nowrap rounded-full px-3 text-[13px] font-semibold";
export const TEXT_LINK = "self-start text-sm font-medium text-primary";
export const NOTE_MAX = 140;
export const NAME_MAX = 40;
export const SHORT_NOTE_MAX = 280;

/** The server's plain-text message when there is one, so 422 and 409 answers read as written. */
export function errorMessage(e: unknown): string {
  return e instanceof Error && e.message ? e.message : "Something went wrong. Try again.";
}

export const MONTH_NAMES = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
