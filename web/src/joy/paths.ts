// Where things live under /joy, and how Joy's old server paths map to them. Pure: no React.

export const JOY = "/joy";

/**
 * The server answers redirects as `{ok, to}` with an SPA path. Anything that is not under /joy
 * (or is protocol-relative) falls back to the jar, and `/joy#middle` becomes the Middle page.
 */
export function mapGoTo(to: string | undefined | null): string {
  if (!to || !to.startsWith(JOY) || to.startsWith("//")) return JOY;
  const rest = to.slice(JOY.length);
  if (rest !== "" && !/^[/#?]/.test(rest)) return JOY;
  if (rest === "#middle" || rest === "/#middle") return `${JOY}/middle`;
  return to;
}

export type PageKey = "jar" | "slips" | "kept" | "year" | "month" | "regulars" | "middle" | "settings" | "envelope";

/** The `data-page` Joy's CSS keys the nav underline on. */
export function pageKey(pathname: string): PageKey {
  const p = pathname.replace(/\/+$/, "");
  const seg = p.startsWith(JOY) ? p.slice(JOY.length).split("/").filter(Boolean) : [];
  switch (seg[0]) {
    case undefined:
      return "jar";
    case "slips":
      return "slips";
    case "kept":
      return "kept";
    case "year":
      return seg.length >= 3 ? "month" : "year";
    case "regulars":
      return "regulars";
    case "middle":
      return "middle";
    case "settings":
      return "settings";
    case "e":
      return "envelope";
    default:
      return "jar";
  }
}

/** The nav link (data-nav) that should read as current for a page. */
export function navFor(key: PageKey): "jar" | "slips" | "kept" | "year" | null {
  switch (key) {
    case "jar":
    case "slips":
    case "kept":
      return key;
    case "year":
    case "month":
      return "year";
    default:
      return null;
  }
}
