#!/usr/bin/env bun
import { WebStandardStreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/webStandardStreamableHttp.js";
import { JoyClient } from "./client.ts";
import { buildServer } from "./server.ts";

const port = Number(process.env.PORT ?? 3001);
const baseUrl = (process.env.JOY_URL ?? "https://joy.fly.dev").replace(/\/+$/, "");
const hosts = (process.env.JOY_MCP_HOSTS ?? "localhost,127.0.0.1")
  .split(",").map((x) => x.trim()).filter(Boolean);

// Joy starts this server itself (src/mcp_proxy.rs), and it calls Joy back on loopback, so plain
// HTTP to 127.0.0.1 never leaves the machine. Anything else must be HTTPS in production.
const loopback = /^http:\/\/(127\.0\.0\.1|localhost)(:\d+)?$/.test(baseUrl);
if (!/^https:\/\//.test(baseUrl) && !loopback && process.env.NODE_ENV === "production") {
  throw new Error("JOY_URL must use HTTPS in production.");
}
const bind = process.env.JOY_MCP_BIND ?? "0.0.0.0";

const unauthorized = () => new Response("Unauthorized", { status: 401, headers: { "www-authenticate": "Bearer" } });

Bun.serve({
  port,
  hostname: bind,
  async fetch(request) {
    const url = new URL(request.url);
    if (url.pathname === "/healthz") return new Response("ok");
    if (url.pathname !== "/mcp") return new Response("Not found", { status: 404 });
    if (request.method !== "POST" && request.method !== "GET" && request.method !== "DELETE") {
      return new Response("Method not allowed", { status: 405, headers: { allow: "GET, POST, DELETE" } });
    }
    const authorization = request.headers.get("authorization") ?? "";
    const match = /^Bearer (joy_[A-Za-z0-9_-]{40,})$/.exec(authorization);
    if (!match) return unauthorized();
    if (Number(request.headers.get("content-length") ?? 0) > 1_000_000) return new Response("Request too large", { status: 413 });

    // Validate on every request, including initialize and tools/list. Revocation and expiry then
    // take effect for an already connected Claude client without relying on an MCP session cache.
    const check = await fetch(`${baseUrl}/api/token-info`, {
      headers: { authorization: `Bearer ${match[1]}`, accept: "application/json" },
    }).catch(() => null);
    if (!check) return new Response("Joy is unavailable", { status: 502 });
    if (check.status === 401) return unauthorized();
    if (!check.ok) return new Response("Token validation failed", { status: check.status });
    const info = await check.json() as { scopes?: string[] };
    if (!Array.isArray(info.scopes)) return unauthorized();

    const client = new JoyClient({ baseUrl, token: match[1] });
    const server = buildServer(client, info.scopes);
    const transport = new WebStandardStreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true,
      allowedHosts: hosts,
      enableDnsRebindingProtection: true,
    });
    try {
      await server.connect(transport);
      return await transport.handleRequest(request);
    } catch {
      return new Response("MCP request failed", { status: 500 });
    } finally {
      await server.close().catch(() => undefined);
    }
  },
});

console.info(`Joy MCP listening on ${bind}:${port}`);
