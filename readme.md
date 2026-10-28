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

Configuration (env or `.env`): `DATABASE_URL`, `PASETO_KEY` (base64), optional `CORS_ORIGINS` (comma-separated; off by default since the server serves the app itself), optional `REDIS_URL`, `PORT` (3000), `STATIC_DIR` (`web/dist`), `FLINDERAX_UTC_OFFSET_MINUTES` (330 = IST; decides what "today" is).

```
bun run test           # cargo unit tests + web tests (bun test)
bun run typecheck      # tsc --noEmit for web/
```

Changed an SQL query? Run `cargo sqlx prepare -- --bin flinderax` and commit `.sqlx/` (the Docker build compiles offline).

## Demo data

`bun scripts/seed-demo.ts` creates a fresh account full of round numbers (needs a running server) and prints a click-by-click demo script. Options: `--url https://your-app.fly.dev --user demo2 --pass secret123`. It refuses to touch an account that already has data.

## How the numbers work

Money is **integer paise** in all new tables and APIs (`*_paise`). The older `REAL` card columns are unchanged.

Per card (from the four numbers the bank app shows, plus EMIs and pending charges):

```
due_now        = total_due                       -> pay by the current due date
next_bill_est  = outstanding - total_due - emi_remaining + emi_monthly + Σ pending charges
loan_not_due   = emi_remaining - emi_monthly
```

`unbilled_spent` / `unbilled_credit` are stored for reference only, so duplicates and refunds can't skew anything.

**The model: two pots and a ledger.**
- **Spending account** (e.g. HDFC): salary lands here, the SIP debits here, and what is left is your spending pot.
- **Reserve account** (e.g. Kotak): your savings; it pays card bills and needs (house, phone, internet, EMI).
- **FDs/RDs** are locked and never counted as spendable.
- Every card swipe should be matched by moving that amount spending → reserve. The dashboard shows the move as a button; you move the money in your bank, then click **I moved it** (balances adjust), **Already in the reserve**, or **Skip this month** (savings top-up only). Clicks are kept in a ledger (`moves`) and can be undone.

**Safe to Spend** (`src/planner/`, pure and unit-tested) is for the **current pay period only**: today up to, not including, the next payday. The next salary is never used.

```
  spending-account balance
- fixed payments leaving it before payday (the SIP...)
- card bills still to move to the reserve   (the ledger says how much is already covered)
- this month's savings top-up to move (needs money + the rest of savings; only from cash BEYOND your spending pot, and skippable)
= free in the spending account
  safe to spend = the smaller of that and what is left of your spending allowance
```

Spending so far is inferred from card snapshots, so nothing is logged by hand: `total due + unbilled spent - unbilled credit` (or `outstanding - EMI balance` when unbilled isn't entered), compared with a baseline. A card's first snapshot is either a *starting balance* (owed before tracking began, not spending) or *new spending* (the card was at zero). Days before tracking began are assumed to follow the allowance's even pace, so the remaining allowance is `allowance - pace so far - actually spent since`. A card's next bill is the bank's `unbilled spent - unbilled credit`; the rest of `outstanding` is shown as loan, not due.

**Budget:** needs = your listed bills (yearly ones at a twelfth a month), spending = the allowance you set (an amount like `18875` or a percentage of income like `25%`, which follows your income), savings = whatever is left (SIP + the rest, which goes to the reserve). The dashboard stays clean; the full step-by-step lives on **/working**.

**Overspending:** a shortfall or skipped top-up becomes "rebuild ₹X a month for N months", so no FD needs to be touched.

**What-if and projections (`/what-if`, `src/planner/projection.rs`):** add an RD, an FD or a one-off spend and the page recomputes live: balances for the next 12-60 months with and without it, which pot absorbs the cost (savings first vs spending first) and at what amount each starts to hurt, months where an account would fall short, and a goal check (target amount and date). FDs/RDs compound quarterly; an FD can pay out, renew the principal only, or renew principal and interest. A statement day counts as passed on the day itself; days are clamped to month length (31 -> 28/30).

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
| Ledger | `/api/moves/{record,undo}`, `GET /api/moves/list` |
| Projection | `POST /api/plan/project` (what-if items in, baseline + scenario out) |
