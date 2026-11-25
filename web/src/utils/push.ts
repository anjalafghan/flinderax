/** The server's base64url key as the bytes `pushManager.subscribe` wants. */
export function urlBase64ToBytes(b64url: string): Uint8Array {
  const padded = b64url
    .replace(/-/g, "+")
    .replace(/_/g, "/")
    .padEnd(Math.ceil(b64url.length / 4) * 4, "=");
  const raw = atob(padded);
  return Uint8Array.from(raw, (c) => c.charCodeAt(0));
}
