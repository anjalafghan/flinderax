import { expect, test } from "bun:test";
import { JoyClient, configFromEnv } from "./client.ts";
import { tools } from "./tools.ts";

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

test("only GET, only /api/", async () => {
  const seen: string[] = [];
  const c = new JoyClient({
    baseUrl: "http://x", token: "t",
    fetch: (async (u: URL, init: RequestInit) => { seen.push(`${init.method} ${u.pathname}`); return json({ ok: 1 }); }) as any,
  });
  await c.get("/api/dashboard/summary");
  expect(seen).toEqual(["GET /api/dashboard/summary"]);
  expect(c.get("/healthz")).rejects.toThrow();
});

test("signs in, and again after a 401", async () => {
  let logins = 0;
  const c = new JoyClient({
    baseUrl: "http://x", username: "u", password: "p",
    fetch: (async (u: URL, init: RequestInit) => {
      if (String(u).endsWith("/login")) { logins++; return json({ access_token: `tok${logins}` }); }
      const auth = (init.headers as Record<string, string>).authorization;
      return auth === "Bearer tok2" ? json({ ok: true }) : json({}, 401);
    }) as any,
  });
  expect(await c.get("/api/plan/forecast")).toEqual({ ok: true });
  expect(logins).toBe(2);
});

test("a 404 stays a plain not-found", () => {
  const c = new JoyClient({ baseUrl: "http://x", token: "t", fetch: (async () => json({}, 404)) as any });
  expect(c.get("/api/dates/slips")).rejects.toThrow("Not found.");
});

test("config needs credentials", () => {
  expect(() => configFromEnv({})).toThrow();
  expect(configFromEnv({ JOY_TOKEN: "t" }).baseUrl).toBe("https://joy.fly.dev");
});

test("tool names are unique and paths stay under /api/", () => {
  expect(new Set(tools.map((t) => t.name)).size).toBe(tools.length);
  for (const t of tools) expect(t.path({ year: 2026, month: 1 }).startsWith("/api/")).toBe(true);
});
