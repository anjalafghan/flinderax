// A client for the joy API. It reads with GET, and writes only to WRITE_PATHS (a report, a new
// transaction, the parser, dismissing an inbox item, tagging a payee's untagged rows) and WRITE_PATTERNS
// (filling in the blanks of a hand-marked day's slip); no other path can be
// written, whatever a tool asks for. It never deletes a transaction, and only ever fills a blank category
// (the payee endpoint skips rows that are already tagged or locked). Auth is a bearer token: either JOY_TOKEN, or a
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

  /** Save an analysis. Sent with an Idempotency-Key so a retry is stored once. */
  async saveAnalysis(title: string, body: string, key: string = crypto.randomUUID()): Promise<unknown> {
    return this.post(SAVE_PATH, { title, body, source: "claude" }, key);
  }

  /**
   * POST to one of the few paths the connector may write (WRITE_PATHS), always with an Idempotency-Key,
   * so a retry or a second run is done once. Any other path is refused before a request is made.
   */
  async post(path: string, body: unknown, key: string): Promise<unknown> {
    if (!writable(path)) throw new JoyError("That path cannot be written.");
    if (!key) throw new JoyError("A write needs an idempotency key.");
    const send = async (token: string) =>
      this.doFetch(`${this.cfg.baseUrl}${path}`, {
        method: "POST",
        headers: { authorization: `Bearer ${token}`, "content-type": "application/json", "idempotency-key": key },
        body: JSON.stringify(body),
      });
    let res = await send(this.token ?? (await this.login()));
    if (res.status === 401 && this.cfg.username) res = await send(await this.login());
    if (!res.ok) {
      let why = "";
      try {
        why = ((await res.json()) as { error?: string; message?: string }).error ?? "";
      } catch {}
      throw new JoyError(why || `The joy server answered ${res.status}.`, res.status);
    }
    return res.json();
  }
}

const SAVE_PATH = "/api/analyses";
/** Everything the connector can write: a report, a new transaction, the server's own parser, clearing an inbox item,
 * and giving one payee's uncategorised rows a category. */
export const WRITE_PATHS = [
  SAVE_PATH,
  "/api/ingest/transactions/create",
  "/api/ingest/parse",
  "/api/ingest/unrecognized/dismiss",
  "/api/ingest/payees/categorize",
];
/** Writes whose path carries an id: filling in a hand-marked day's slip (the server only fills blanks,
 * and only on a day marked by hand). */
export const WRITE_PATTERNS = [/^\/api\/dates\/slips\/\d+\/dress$/];

export const writable = (path: string) => WRITE_PATHS.includes(path) || WRITE_PATTERNS.some((re) => re.test(path));
