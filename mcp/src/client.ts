// A read-only client for the joy API. It can only issue GET requests, so a connector built on it
// cannot change anything, whatever a tool asks for. Auth is a bearer token: either JOY_TOKEN, or a
// sign-in with JOY_USERNAME / JOY_PASSWORD (renewed automatically when the server answers 401).
// Nothing here logs a path, a token or a password.

export interface ClientConfig {
  baseUrl: string;
  token?: string;
  username?: string;
  password?: string;
  fetch?: typeof fetch;
}

export class JoyError extends Error {
  constructor(
    message: string,
    readonly status?: number,
  ) {
    super(message);
  }
}

export function configFromEnv(env: Record<string, string | undefined> = process.env): ClientConfig {
  const baseUrl = (env.JOY_URL ?? "https://joy.fly.dev").replace(/\/+$/, "");
  const cfg: ClientConfig = { baseUrl, token: env.JOY_TOKEN, username: env.JOY_USERNAME, password: env.JOY_PASSWORD };
  if (!cfg.token && !(cfg.username && cfg.password)) {
    throw new JoyError("Set JOY_TOKEN, or JOY_USERNAME and JOY_PASSWORD (and optionally JOY_URL).");
  }
  return cfg;
}

export class JoyClient {
  private token?: string;
  private readonly doFetch: typeof fetch;

  constructor(private readonly cfg: ClientConfig) {
    this.token = cfg.token;
    this.doFetch = cfg.fetch ?? fetch;
  }

  private async login(): Promise<string> {
    const { username, password } = this.cfg;
    if (!username || !password) throw new JoyError("Session expired and no JOY_USERNAME/JOY_PASSWORD to sign in again.", 401);
    const res = await this.doFetch(`${this.cfg.baseUrl}/api/common/login`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ user_name: username, user_password: password }),
    });
    if (!res.ok) throw new JoyError(`Sign-in failed (${res.status}).`, res.status);
    const body = (await res.json()) as { access_token?: string };
    if (!body.access_token) throw new JoyError("Sign-in returned no token.");
    this.token = body.access_token;
    return this.token;
  }

  /** GET `path` (must start with /api/) and return the parsed JSON. */
  async get(path: string, query?: Record<string, string | number | undefined>): Promise<unknown> {
    if (!path.startsWith("/api/")) throw new JoyError("Only /api/ paths can be read.");
    const url = new URL(this.cfg.baseUrl + path);
    for (const [k, v] of Object.entries(query ?? {})) if (v !== undefined) url.searchParams.set(k, String(v));

    const send = async (token: string) =>
      this.doFetch(url, { method: "GET", headers: { authorization: `Bearer ${token}`, accept: "application/json" } });

    let res = await send(this.token ?? (await this.login()));
    if (res.status === 401 && this.cfg.username) res = await send(await this.login());
    if (res.status === 404) throw new JoyError("Not found.", 404);
    if (!res.ok) throw new JoyError(`The joy server answered ${res.status}.`, res.status);
    return res.json();
  }
}
