# Flinderax

A manual-entry money planner. Its home screen answers one question: **how much can I spend until my next salary without missing anything?**

It shows one big *Safe to Spend* number (plus a per-week figure), a plain-English breakdown of each credit card, and a short "do this today" list (e.g. "Move ₹8,000 Kotak → HDFC before 10 Oct"). There is no bank integration; you copy numbers from your bank apps.

## One server

The Rust (axum) server is the only server. It serves the JSON API under `/api/*` **and** the built web app (`web/dist`), with SPA fallback. Bun is used only as the package manager, bundler and test runner.

```
bun install            # installs web/ (workspace) and tooling
bun run build          # bundle the web app into web/dist
cargo run              # API + web app on http://localhost:3000
```

Development: `bun run dev` rebuilds `web/dist` on every change while the Rust server runs; refresh the page to see edits.

Configuration (env or `.env`): `DATABASE_URL`, `PASETO_KEY` (base64), optional `REDIS_URL`, `PORT` (3000), `STATIC_DIR` (`web/dist`), `FLINDERAX_UTC_OFFSET_MINUTES` (330 = IST; decides what "today" is).

```
bun run test           # cargo unit tests + web tests (bun test)
bun run typecheck      # tsc --noEmit for web/
```

Changed an SQL query? Run `cargo sqlx prepare -- --bin flinderax` and commit `.sqlx/` (the Docker build compiles offline).

## How the numbers work

Money is **integer paise** in all new tables and APIs (`*_paise`). The older `REAL` card columns are unchanged.

Per card (from the four numbers the bank app shows, plus EMIs and pending charges):

```
due_now        = total_due                       -> pay by the current due date
next_bill_est  = outstanding - total_due - emi_remaining + emi_monthly + Σ pending charges
loan_not_due   = emi_remaining - emi_monthly
```

`unbilled_spent` / `unbilled_credit` are stored for reference only, so duplicates and refunds can't skew anything.

Safe to Spend (`src/planner/`, pure and unit-tested): take every unlocked account's balance, add incomes and subtract commitments, card bills and next period's living costs on their dates until the pay day *after* the next one, find the lowest balance, subtract the cushion. Locked accounts (an FD) never count. A statement day counts as passed on the day itself; days are clamped to month length (31 -> 28/30).

## Adding and removing things

Everything is data, nothing is hard-coded: accounts (`bank`, `cash`, `fd`, `rd`), fixed payments (SIPs, rent, an RD instalment with optional first/last month), income sources, card EMIs and pending charges all have create/update/delete endpoints and screens (Accounts, Plan, card page).

## API (all POST + JSON unless noted; everything except `common/*` needs a PASETO bearer token)

| Area | Endpoints |
|------|-----------|
| Auth | `/api/common/register`, `/api/common/login` |
| Cards | `/api/card/{create,update,delete,get_card,history,reset,insert_transaction,delete_transaction,defer_*}`, `GET /api/card/get_all_cards` (protobuf) |
| Card cycles | `/api/card/snapshot`, `/api/card/breakdown`, `/api/card/emi/{create,update,delete}`, `/api/card/pending/{create,clear}` |
| Accounts | `/api/account/{create,update,update_balance,delete}`, `GET /api/account/list` |
| Plan | `/api/plan/commitment/{create,update,delete}`, `/api/plan/income/{create,update,delete}`, `/api/plan/settings/update`; `GET` `.../commitment/list`, `.../income/list`, `settings/get` |
| Dashboard | `GET /api/dashboard/summary` |
