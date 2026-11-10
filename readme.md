# joy

Your life from now to 90, and what each choice does to it. joy is one app for one household: it draws your money year by year to the age you plan to, lets you switch a choice on (a new job, a home, a child, retiring earlier) and says in plain sentences what changes. Everything else in the app feeds that line or keeps it honest.

It works at four time scales:

| When | Where | What you do |
|------|-------|-------------|
| **Decades** | Life → Life line | See net worth by age, when work becomes optional and how long the money lasts. Try a choice, keep it in the plan or drop it. |
| **Each quarter** | Review → check-in | Upload statements, confirm balances, read the report. It says how the life line moved since last time. |
| **Each week** | Health → This week | Five minutes: habit days (food and exercise), who to call, the next step in the job search, what is safe to spend. |
| **Each day** | Today | **Safe to Spend** until payday, card bills, the short do-this-today list. Bank alerts and statements come in on their own. |

The parts:

- **Life line** (`src/planner/lifeline.rs`). Money is computed. Later-life medical costs, the cost of raising a child and the effect of staying active are estimates, labelled as such and editable.
- **Money.** A planner whose home screen answers one question: **how much can I spend until my next salary without missing anything?** There is no bank integration: balances follow your salary and bills, bank alerts your phone forwards become categorised transactions, and you correct a number only when it drifts.
- **Work.** A short list of job leads, one next step each. A lead with a salary opens on the life line as a job switch.
- **People.** Dates for the two of you (a jar of ideas that picks **one**), and a circle of people with their birthdays, catch-ups and gifts.
- **Health.** Up to five weekly habits, food money from your statements, weight and the yearly check-up.

It was built by merging two apps, the old Flinderax (money) and Joy (the date jar, now **People → Us**, `src/joy/` and `web/src/joy/`). See [Moving from the old Joy and Flinderax apps](#moving-from-the-old-joy-and-flinderax-apps).

## One server

The Rust (axum) server is the only server. It serves the JSON API under `/api/*` **and** the built web app (`web/dist`), with SPA fallback. Bun is only the package manager, bundler and test runner.

## Run locally

```
bun install            # installs web/ (workspace) and tooling
bun run build          # bundle the web app into web/dist
cargo run              # API + web app on http://localhost:3000
```

Development: `bun run dev` rebuilds `web/dist` on every change while the Rust server runs; refresh the page to see edits.

Configuration (env or `.env`):

| Variable | Meaning |
|----------|---------|
| `DATABASE_URL` | SQLite, e.g. `sqlite://flinderax.db` locally (the local file name is unchanged), `sqlite:///data/joy-app.db?mode=rwc` on Fly |
| `PASETO_KEY` | base64 signing key for login tokens |
| `PORT` | default 3000 |
| `STATIC_DIR` | built web app, default `web/dist` |
| `PUBLIC_URL` | where envelope links point, e.g. `https://joy-jar.fly.dev` |
| `JOY_PHOTOS_DIR` | where Dates photos are stored |
| `TZ` | local time zone for dates (`Asia/Kolkata`) |
| `FLINDERAX_UTC_OFFSET_MINUTES` | what "today" is for the money side (330 = IST). The name is kept from the old app |
| `CORS_ORIGINS` | optional, comma-separated; off by default since the server serves the app itself |
| `REDIS_URL` | optional cache |

```
bun run test           # cargo unit tests + web tests (bun test)
bun run typecheck      # tsc --noEmit for web/
```

Changed an SQL query? Run `cargo sqlx prepare -- --bin joy` and commit `.sqlx/` (the Docker build compiles offline).

## The tabs

| Tab | What it does |
|-----|--------------|
| **Today** | Safe to Spend, the life line in one row, the do-this-today list, cards, bills due soon, the net-worth chart |
| **Money** | Accounts, spending (bank-alert transactions, statements), goals |
| **Life** | **Life line** (to 90, with choices), **Budget** (income, fixed payments, settings), **What-if** (the next 12-60 months), **Work** (job leads) |
| **People** | **Us** (the date jar, see below) and **Circle** (people, dates, catch-ups, gifts) |
| **Health** | **This week** (the five-minute screen) and **Health** (habits, food money, body) |
| **Review** | Quarterly check-in, month close, tax entries by financial year, export |

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

## Tracking with almost no typing

**Balances move on their own.** Salary is added and fixed payments are taken off the linked account on their dates (`src/planner/autopost.rs`, applied lazily whenever balances are read, never twice). Typing a balance resets the starting point. Turn it off in Plan settings. Every balance change is kept in `balance_history`, which feeds the net-worth chart on the dashboard (`GET /api/dashboard/history`).

**Bank alerts become transactions.** Generate a key on the Spending page, then have your phone forward bank SMS/emails to `POST /api/hook/ingest` with header `X-Ingest-Key: <key>` and body `{"text": "<message>", "source": "sms"}`. `src/alerts.rs` reads the amount, direction, last digits, merchant, date and "Avl Bal"; OTPs, reminders and offers are ignored; the same alert is stored once. Set each bank account's last 4 digits (Accounts page) so alerts update its balance and auto-posting skips what an alert already recorded. Re-categorise a transaction with "Remember" ticked and every future one from that merchant follows.

**Statements.** Spending page > *Upload a statement* takes a bank CSV export (`POST /api/ingest/statement` with `{csv, account_id?, card_id?}`, `src/statement.rs`). It finds the header row, handles separate Debit/Credit columns or one signed / Dr-Cr amount column, and dd/mm/yy, dd-Mon-yyyy and ISO dates. Re-importing, or overlapping files, never double-count, and rows a bank alert already recorded (same day, direction, amount) are matched instead of added. Picking an account sets its balance from the newest line unless a newer balance exists.

- **iPhone:** Shortcuts → Automation → Message, sender contains your bank's ID (HDFCBK, KOTAKB...) → Run Immediately → *Get Contents of URL* (POST, header `X-Ingest-Key`, JSON body `text` = Shortcut Input, `source` = `sms`).
- **Android:** any SMS-to-webhook forwarder with the same URL, header and body.
- **Gmail:** filter bank alerts into a label `bank-alerts`, then in Google Apps Script (run every 15 minutes with a time trigger):

```js
const URL = "https://your-app.fly.dev/api/hook/ingest", KEY = "fx_...";
function forwardAlerts() {
  for (const t of GmailApp.search("label:bank-alerts is:unread", 0, 50)) {
    for (const m of t.getMessages().filter((m) => m.isUnread())) {
      UrlFetchApp.fetch(URL, { method: "post", contentType: "application/json",
        headers: { "X-Ingest-Key": KEY }, payload: JSON.stringify({ text: m.getPlainBody().slice(0, 2000), source: "email" }) });
      m.markRead();
    }
  }
}
```

## Dates

Dates is a **date jar**: every idea is a slip, you draw one, you can put it back three times, and the one you keep is stamped *IT'S A DATE* and goes into the keepsake box (Kept). Two joy users form a **couple** (one starts it, the other joins with an invite), and everything in the `joy_*` tables belongs to that couple.

- **Picking.** Hard filters: cost within budget, duration within the time you have, setting, distance (from either home, or whoever is farther), vibe and energy. At-home ideas always pass the distance filter. *At home, together* is for the same room; *Long distance* is for different cities. Weighting: ideas you've never done are about 3x as likely; something you just did starts at 1x and climbs back over about two months; anything rerolled in the last 3 days is halved. When nothing fits, it says which single filter to loosen. The logic is unit tested.
- **Meet in the middle.** Each of you checks in on your own phone (energy, spend, time, how far, optionally a vibe). Once you both have, **Draw from the middle** picks only from slips that fit both of you: the lower budget, shorter time, lower energy and nearer distance, and a slip that matches either vibe. Answers stay private (neither of you sees the other's) and last for the local day; "Take mine back" removes yours at once.
- **The day plan.** Set the schedule (date days, office, latest times, commutes) and the jar draws for a real date. Weeknights start at the office and only fit if you can get there, do it and still be home on time; weekends start from each home. Stamping a slip plans the day: the time, a timeline (when to leave, when you're each home), a **Plan B** (indoors, close by, one tap to swap), somewhere to eat after, and the jobs (book it, bring cash) dealt to whoever has had fewer. The weather forecast is checked over HTTPS.
- **Envelopes.** A date can be sent as a sealed envelope, a link `PUBLIC_URL/e/<token>` (signed, expiring) you share on WhatsApp. It opens without logging in, so keep `PUBLIC_URL` pointing at the real address.
- **Surprises.** A surprise date stays sealed: its planner sees everything, the partner sees only their own times, and a sealed surprise shows its amount in spending but not its name.
- **Privacy.** Your answers in the middle are never shown to your partner; the shared state is the couple's, not other users'. A small service worker keeps a read-only copy of the jar, Kept and year pages (up to 7 days, plus up to 25 MB of photos) for offline viewing; copies are per person and wiped on logout. Personal pages are `no-store`, and every response carries `Referrer-Policy: no-referrer`, `nosniff`, `X-Frame-Options: DENY` and the other usual headers.
- **Spending.** Date spending counts toward a monthly Dates budget with rollover; your share of it (proportional to what each of you covers) shows on the Today dashboard, and bank-alert debits within a day and 5% of a Dates entry are labelled **Dates** (`src/planner/joy.rs`) unless you picked the category by hand.

Edit ideas **in the app**, not in the SQL files: migrations are checksummed.

## Adding and removing things

Everything is data, nothing is hard-coded: accounts (`bank`, `cash`, `fd`, `rd`), fixed payments (SIPs, rent, an RD instalment with optional first/last month), income sources, card EMIs and pending charges all have create/update/delete endpoints and screens (Accounts, Plan, card page).

## API (all POST + JSON unless noted; everything except `common/*`, `hook/*` and `public/*` needs a PASETO bearer token)

| Area | Endpoints |
|------|-----------|
| Auth | `/api/common/register`, `/api/common/login` |
| Cards | `/api/card/{create,update,delete,get_card,history,reset,insert_transaction,delete_transaction,defer_*}`, `GET /api/card/get_all_cards` (protobuf) |
| Card cycles | `/api/card/snapshot`, `/api/card/breakdown`, `/api/card/emi/{create,update,delete}`, `/api/card/pending/{create,clear}` |
| Accounts | `/api/account/{create,update,update_balance,delete}`, `GET /api/account/list` |
| Plan | `/api/plan/commitment/{create,update,delete}`, `/api/plan/income/{create,update,delete}`, `/api/plan/settings/update`; `GET` `.../commitment/list`, `.../income/list`, `settings/get` |
| Dashboard | `GET /api/dashboard/summary`, `GET /api/dashboard/history?months=12` |
| Ledger | `/api/moves/{record,undo}`, `GET /api/moves/list` |
| Projection | `POST /api/plan/project` (what-if items in, baseline + scenario out) |
| Afford | `POST /api/plan/afford` ("can I afford this?") |
| Life line | `GET /api/plan/lifeline`, `POST /api/plan/lifeline/try` (`{base, with}` choices in, both lines and the consequences out), `POST /api/plan/lifeline/choices`, `PATCH`/`DELETE /api/plan/lifeline/choices/{id}` |
| Work | `GET`/`POST /api/work`, `PUT`/`DELETE /api/work/{id}` |
| Bills | `GET /api/bills`, `GET /api/bills/suggestions`, `POST /api/bills/suggestions/dismiss` |
| Goals | `GET /api/goals`, `POST /api/goals/{create,update,archive,contribute}` |
| Review | `GET /api/review/month`, `GET /api/review/tax`, `POST /api/review/tax/{entry,entry/delete,commitment}` |
| Export | `GET /api/export` (everything as JSON) |
| Spending | `/api/ingest/{parse,statement,transactions/create,transactions/update,transactions/delete,rules/delete,key/rotate}`; `GET` `/api/ingest/{transactions?month=,summary?months=,rules,key/status}` |
| Couple | `GET /api/couple`, `POST /api/couple/{create,invite,join,leave}` |
| Dates | `/api/dates/*`: the jar, `pick`, `match`, `accept/{id}`, `middle`, `slips` (list, create, edit, archive, restore, delete, unveto, plan), plus Kept, regulars, the year and settings |
| Public Dates | `GET /api/public/dates/e/{token}` (the envelope; signed link, no login) |
| Dates spending | `/api/joy/{summary,settle}` and `POST /api/joy/{connect,partner,disconnect}` (the Dates share on the dashboard) |
| Phone hook | `POST /api/hook/ingest` (header `X-Ingest-Key`, no bearer token) |

## Deploy

One Fly app, `joy-jar` (region `sin`), one machine, one volume `joy_data` mounted at `/data`. The database is `/data/joy-app.db` and photos go to `/data/joy-photos`; see the comment at the top of `fly.toml`. Secrets (`PASETO_KEY`, `DATES_URL_KEY`, ...) are set with `fly secrets set`, never in `fly.toml`.

```
fly deploy --depot=false      # from a clean checkout
```

The Dockerfile builds the web app with Bun, the server with cargo-chef (offline `.sqlx` data), and ships `joy` with `tzdata` and `ca-certificates`. It sleeps when nobody is using it (`suspend`).

## Moving from the old Joy and Flinderax apps

The old Joy database (`/data/joy.db`) stays untouched on the volume as the archive; joy starts a new database and imports into it. The steps, in order, with the import command and the checks, are in [docs/cutover.md](docs/cutover.md).

