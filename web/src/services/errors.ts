/**
 * The text to show for a failed request. The server answers errors as
 * `{"code": "...", "message": "...", "field": null}`; a slip form answers `{"error": "..."}`;
 * older paths answered plain text. All three read the same here.
 */
export function errorMessageOf(body: string, status: number): string {
  const text = body.trim();
  if (text.startsWith("{")) {
    try {
      const v = JSON.parse(text) as { message?: unknown; error?: unknown };
      for (const m of [v.message, v.error])
        if (typeof m === "string" && m) return m;
    } catch {
      /* not JSON after all: fall through to the text */
      return text;
    }
    return `Request failed with status ${status}`;
  }
  return text || `Request failed with status ${status}`;
}
