# joy-mcp: connector

An MCP server that lets Claude read joy (money and Dates) over the existing API. It writes to five paths only
(`WRITE_PATHS` in `src/client.ts`): a report on the Review page, a new transaction, joy's own text parser, clearing
an item from the unrecognised-alert inbox, and tagging a payee's untagged rows. It never deletes a transaction and only
ever fills a blank category (rows already tagged or locked by hand are skipped), and every write carries an
`Idempotency-Key`, so a retry or a second run is done once. The server's own privacy rules still apply: surprises
sealed for you answer 404, and neither partner's quiet marks nor answers are returned.

## Tools

`money_*`: summary, insights, analytics, forecast, commitments, income, bills, year outflows, goals,
cards, transactions, month review, check-in, `money_reconcile` (which bank balances moved by money joy never recorded), `money_export_all` (the full export), `analyses_list`.
`save_analysis` writes a report to the Review page; the prompt `analyse_money` drives a full run. `dates_*`: coming up, jar, slips, packs, kept, regulars,
year, month, summary, settle. Money is integer paise.

## Sorting the bank texts joy could not read

Texts the regexes miss wait 30 days in the inbox. Claude becomes the fallback parser, with no API key or LLM in the server:

- `money_unrecognized` lists them, with numbers of 8+ digits masked to the last 4.
- `money_add_transaction` takes rupees as text ("1,250.50"), converts them to paise exactly, and saves the transaction.
  With an `alert_id` it is keyed on that alert, so it can't post twice, and it clears the alert once the transaction is saved.
- `money_parse_text` sends a cleaned-up text through joy's own parser, which also matches the card or account.
- `money_dismiss_unrecognized` clears an OTP, a promo and the like.
- The `sort_inbox` prompt runs the whole pass and shows you a table to approve before it adds anything. Run it weekly.

## Tagging untagged spending

- `money_uncategorized_payees` lists payees with untagged debits, biggest first, with joy's suggestion where it has one.
- `money_categorize_payee` gives one payee's untagged rows a category (`remember: true` also saves a rule for future
  imports). It calls `POST /api/ingest/payees/categorize`, which fills blanks only.

## Planning a day out

`dates_near` takes a station or area ("CSMT", "Bandra"...) plus an optional budget and max hours, and returns a short
list of slips grouped as near, short hop and anywhere. The `plan_outing` prompt wraps it: say "I'll be at CSMT tomorrow"
and Claude builds itineraries, checking `dates_coming_up` and `dates_kept` for clashes. Add areas in `src/near.ts`.

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
