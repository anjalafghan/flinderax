# joy: guide for agents and new contributors

A private life-planning app for one household: money (Flinderax) plus the date jar (Joy). One Rust (axum 0.8, SQLite via sqlx) server serves the JSON API and the built React 19 SPA in `web/`. Bun bundles and tests the web.

## Layout and layering

- `src/planner/`: pure rules. No IO, `today` is passed in. Keep it that way.
- `src/joy/picker.rs`, `src/joy/taste.rs`: pure and deterministic under a seeded RNG. Keep IO out.
- `src/handlers/`, `src/joy/api/`: HTTP. Newer files still mix SQL and rules; new code should follow `handlers/x.rs` (HTTP), SQL helpers, `planner/x.rs` (rules).
- `src/joy/db.rs`: all Dates SQL.
- `web/src/`: SPA. Dates lives in `web/src/joy/`.

## Rules that must keep holding

- Money on the finance side is integer paise (`*_paise`). Dates still stores whole rupees; `src/planner/joy.rs` converts. Never add float money.
- A slip sealed as a surprise answers **404, never 403**, to the partner it is for. `visible_list`, `visible_get` and `unsealed` are the only doors to a slip by id.
- "Meet in the middle" never reveals one partner's answers.
- Logs record route templates, never real paths (tokens live in paths).
- Enrichment only fills blanks.
- Signed links (photos, calendar, envelopes) use `DATES_URL_KEY`. Set it as a Fly secret; without it the key derives from `PASETO_KEY`, and rotating the login key breaks every link.
- Anything a user owns must be in `GET /api/export` or on the deny list in `tests/lifecycle.rs`; that test fails when a new table is neither.

## Migrations and SQL

- Never edit an existing migration (checksummed). Add a new one.
- **Numbering:** the newest migration is dated ahead of the calendar (`20261119…`). Name new ones after the newest existing file, not today's date, or they sort before ones already applied.
- After changing any SQL: `cargo sqlx prepare -- --bin joy` and commit `.sqlx/`.

## Checks (all must be clean before merging)

```
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
bun run typecheck
bun run --cwd web lint
bun run --cwd web test
```

`cargo fmt` is not enforced: the code base is not rustfmt-formatted, so do not run it over files you are not otherwise changing.

## Release order

Commit to `develop`, merge to `master`, then deploy by hand: `fly deploy --depot=false` (Fly app `joy`, https://joy.fly.dev). CI runs on push; the deploy workflow only runs after green CI and needs the `FLY_API_TOKEN` secret. `/healthz` runs `SELECT 1` and is the Fly health check.

## Known open work

See `findings_s_tier.md` (local, not in git) for the prioritised list.
