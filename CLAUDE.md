# joy: guide for agents and new contributors

A private life-planning app for one household: money (Flinderax) plus the date jar (Joy). One Rust (axum 0.8, SQLite via sqlx) server serves the JSON API and the built React 19 SPA in `web/`. Bun bundles and tests the web.

## Layout and layering

- `src/planner/`: pure rules. No IO, `today` is passed in. Keep it that way.
- `src/joy/picker.rs`, `src/joy/taste.rs`: pure and deterministic under a seeded RNG. Keep IO out.
- `src/handlers/`, `src/joy/api/`: HTTP. Newer files still mix SQL and rules; new code should follow `handlers/x.rs` (HTTP), SQL helpers, `planner/x.rs` (rules).
- `src/joy/db.rs`: all Dates SQL.
- `web/src/`: SPA. Dates lives in `web/src/joy/`.

## Rules that must keep holding

- Money is integer paise everywhere, Dates included. Rupees typed into a form become paise only through `validate::paise_from_text`; form fields are filled with `validate::plain_rupees`; `Ctx::money` formats for display. Never add float money.
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

## How the draw and the history work (read before touching slips)

- Putting a slip back is unlimited. Each one is a row in `joy_peeks` (optional reason; only `not_my_thing` teaches taste), never in `joy_history`.
- A kept date is `joy_history.outcome = 'did_it'` from the moment it is kept, but it only *counts* once its day has passed (`times_done` is derived in `db::COLS`). `last_done_at` includes planned dates so a booked slip is not drawn again. "Did it happen?" (`POST /history/{id}/happened`) confirms or cancels (`outcome = 'cancelled'`).
- A slip can be kept or planned only through `db::drawable_get` (not archived, not passed on, not sealed).
- Envelope links expire (`share_expires_at`); `unshare` clears them.
- Money-changing POSTs accept an `idempotency_key` (moves do now); add it to new ones.
- A commitment every few months (a perfume every 4) is `frequency = 'monthly'` with `every_months` 2-11 and a `first_month`; the `frequency` CHECK is not widened. Read it through `Freq::from_columns`, never by comparing the strings, so the planner, bills, year view, auto-post and tax all agree.
- Quiet marks (`joy_wants`): a person only ever sees their own, plus `both_want`. Never return or export the partner's level; the draw only turns them into weights.
- `POST /api/import` loads an export back (money and people; Dates is left alone). New user tables join `export::USER_TABLES` in parent-before-child order, and the round-trip test in `tests/auth_integration.rs` should cover them.
- Web sign-in is an HttpOnly `joy_session` cookie (SameSite=Strict). Unsafe requests authenticated by the cookie must carry `X-Joy: 1`. The page keeps only a harmless `s_…` id under `localStorage.token` (it names the offline cache; never the token). Bearer tokens still work for scripts and older logins.
- Notifications (`src/push.rs`, `docs/notifications.md`): a morning and an evening digest of counts only (never names or amounts), sent by a scheduled call to `/api/internal/push/run`. New nudge kinds must keep that rule.
- Every authenticated POST accepts an `Idempotency-Key` and is then done once; the web outbox (`web/src/offline.ts`) relies on it.
- Bank texts the parser can't read are kept 30 days in `unrecognized_alerts`.

## SQL, offline data and local databases

`cargo sqlx prepare` needs a migrated database. Use a scratch one, not your dev database:
`DATABASE_URL=sqlite:///tmp/prep.db?mode=rwc cargo sqlx database setup && DATABASE_URL=... cargo sqlx prepare -- --bin joy`.
After new migrations, run tests with `SQLX_OFFLINE=true` until your `.env` database is migrated too.

## Checks (all must be clean before merging)

```
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
bun run typecheck
bun run --cwd web lint
bun run --cwd web test
```

Browser tests: `bun run e2e` (Playwright; builds the web app and the server and drives them against a throw-away database). Each test gets its own client address because sign-ups are rate-limited per address.

`cargo fmt` is not enforced: the code base is not rustfmt-formatted, so do not run it over files you are not otherwise changing.

## Release order

Commit to `develop`, merge to `master`, then deploy by hand: `fly deploy --depot=false` (Fly app `joy`, https://joy.fly.dev). CI runs on push; the deploy workflow only runs after green CI and needs the `FLY_API_TOKEN` secret. `/healthz` runs `SELECT 1` and is the Fly health check.

## Known open work

See `findings_s_tier.md` (local, not in git) for the prioritised list.
