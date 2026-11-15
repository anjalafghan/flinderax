/**
 * The 400px copy of a kept date's photo, for lists and grids. Only date photos have one; the
 * signature covers the photo, so the same link works with `thumb=1` added.
 */
export function thumbOf(url: string): string {
  return /\/history\/\d+\/photos\/\d+\?/.test(url) ? `${url}&thumb=1` : url;
}
