# joy-mcp: read-only connector

An MCP server that lets Claude read joy (money and Dates) over the existing API. It can only send
GET requests, so it cannot change anything. The server's own privacy rules still apply: surprises
sealed for you answer 404, and neither partner's quiet marks nor answers are returned.

## Tools
`money_*`: summary, insights, analytics, forecast, commitments, income, bills, year outflows, goals,
cards, transactions, month review, check-in. `dates_*`: coming up, jar, slips, packs, kept, regulars,
year, month, summary, settle. Money is integer paise.

## Use with Claude Code / Claude Desktop
```json
{
  "mcpServers": {
    "joy": {
      "command": "bun",
      "args": ["run", "/path/to/flinderax/mcp/src/index.ts"],
      "env": { "JOY_USERNAME": "...", "JOY_PASSWORD": "..." }
    }
  }
}
```
Or `claude mcp add joy -e JOY_USERNAME=... -e JOY_PASSWORD=... -- bun run /path/to/flinderax/mcp/src/index.ts`.

Environment: `JOY_URL` (default https://joy.fly.dev), and either `JOY_TOKEN` (a bearer token) or
`JOY_USERNAME` + `JOY_PASSWORD` (signs in, and again if the session expires).

## Develop
`bun install && bun test` in this folder. Add a tool by adding a row to `src/tools.ts`.
