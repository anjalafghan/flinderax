/**
 * The server hands a session that is past its halfway point a fresh token in `x-refreshed-token`.
 * Keeping it means someone who opens the app now and then never has to sign in again.
 * Only replaces a token that is already there: a signed-out browser stays signed out.
 */
export function adoptRenewedToken(response: Response): void {
  const fresh = response.headers.get("x-refreshed-token");
  if (!fresh) return;
  try {
    if (localStorage.getItem("token")) localStorage.setItem("token", fresh);
  } catch {
    // Storage can be blocked; the current token keeps working until it expires.
  }
}
