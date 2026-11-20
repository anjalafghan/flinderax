# joy-mcp: connector

An MCP server that lets Claude read joy (money and Dates) over the existing API. Its only write is `save_analysis`, which puts a written report on the Review page.
Everything else is GET-only, so nothing else can change. The server's own privacy rules still apply: surprises
sealed for you answer 404, and neither partner's quiet marks nor answers are returned.

## Tools
`money_*`: summary, insights, analytics, forecast, commitments, income, bills, year outflows, goals,
cards, transactions, month review, check-in, `money_export_all` (the full export), `analyses_list`.
`save_analysis` writes a report to the Review page; the prompt `analyse_money` drives a full run. `dates_*`: coming up, jar, slips, packs, kept, regulars,
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
