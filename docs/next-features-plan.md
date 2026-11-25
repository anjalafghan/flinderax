# Next features: detailed plan

Written 16 Nov 2026. Goal: features that help the two of us reach the wedding and the child, and stop surprise bills. No new breadth for its own sake.

## 0. Correction to my earlier suggestion

I said there was no wedding-specific code. That was too strong. Already in the tree:

| Exists                                                                  | Where                                                                                                                                    | Gap for our goals                                                                                                                         |
| ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Shared goals with contributions per person, `monthly_needed`, `percent` | `src/planner/couple_goals.rs`, `src/handlers/couple_goals.rs`, `web/src/components/feature/CoupleGoalsCard.tsx` (shown on the Dashboard) | One lump target. No line items, no vendor milestones, no "on track" verdict, no link to real transactions                                 |
| Bills calendar with yearly sinking funds                                | `src/planner/bills.rs`, `src/handlers/bills.rs`                                                                                          | Only fixed commitments. One-off or irregular lumpy outflows (insurance, advance tax, wedding advances, FD maturities) are not first-class |
| "Coming up" timeline (bills, dates, birthdays, follow-ups)              | `src/planner/coming.rs`                                                                                                                  | No goal milestones or tax dates in it                                                                                                     |
| Life line with `Child`, `House`, `BigSpend` choices                     | `src/planner/lifeline.rs`                                                                                                                | No education-cost model, no "top-up needed" answer                                                                                        |
| Subscriptions and repeat-payment detector                               | `src/planner/recurring.rs`                                                                                                               | Fine as is                                                                                                                                |
| Push digests (counts only)                                              | `src/push.rs`                                                                                                                            | No weekly decision digest                                                                                                                 |

So the work is closing gaps, not building from scratch. Every item below reuses these modules.

## 1. Wedding budget (extends couple goals)

**What it does.** Turn the wedding goal from a single number into a budget: categories (venue, catering, clothes, jewellery, photography, travel, misc), each with an estimate, a quote, what is paid, and dated payment milestones. One verdict line: "On track" or "Short by ₹X by <date>".

**Data (new migration; name it after the newest file, `20261119000022_…` or later, not today's date).**

- `goal_items(item_id, goal_id → couple_goals, name, category, estimate_paise, quote_paise NULL, vendor TEXT, notes, archived)`.
- `goal_milestones(milestone_id, item_id → goal_items, due_on, amount_paise, paid_on NULL, paid_by NULL, transaction_id NULL)`.
- All money in paise. Both new tables are user-owned: add to `export::USER_TABLES` parent-before-child, and extend the round-trip test in `tests/auth_integration.rs`.

**Rules (pure, `src/planner/wedding.rs`).**

- `budget_summary(items, milestones, today)`: totals by category, committed vs estimated, paid, remaining.
- `verdict(goal, saved, upcoming_milestones, monthly_surplus, today)`: for each milestone, can saved plus expected monthly contribution cover it by its date? Returns on-track, tight (within 10%), or short with the gap. Reuse `monthly_needed` and `months_until`.
- `next_due(milestones, today)` for the timeline.
- Unit tests first: overdue milestone, no date, over-funded, quote lower than estimate, milestone with no item.

**HTTP.** `handlers/wedding.rs`: CRUD for items and milestones, `POST …/milestones/{id}/pay` (accepts `idempotency_key`, creates a contribution row so existing shares keep working). Amounts typed in rupees go through `validate::paise_from_text`; form fields filled with `validate::plain_rupees`.

**Web.** A Wedding page under the goal (item list grouped by category, milestone list, verdict banner). Reuse `CoupleGoalsCard` as the entry point.

**Integration.**

- Milestones feed `coming.rs` as a new `Kind::Milestone`.
- Optional: link a paid milestone to a bank-text transaction (same pattern as `20261119000014_date_transaction_link.sql`).
- Push digest: add "N payments due in 7 days" (count only, never names or amounts).

**Done when:** tests cover every rule; the verdict changes when you change a quote; export and import round-trip includes the new tables; checks in CLAUDE.md are clean.

## 2. Annual money calendar (lumpy outflows)

**What it does.** A 12-month view of every large known outflow, with each month's safe-to-spend shown after them, so a March premium is visible in January.

**Sources it merges (no new concept, one new input):**

1. Yearly commitments (`bills.rs`, already there).
2. Wedding milestones (item 1).
3. Tax dates from `tax.rs` (advance tax instalments, if the user is liable).
4. New: `planned_outflows(outflow_id, couple_id/user_id, label, month, amount_paise, kind)` for insurance premiums, annual card fees, and FD or RD maturities (inflows allowed, negative sign or a `direction` column; pick one, keep it integer paise).

**Rules (`src/planner/year_ahead.rs`).** `year_view(plan_input, outflows, milestones, today)` returns 12 month rows: income, fixed, lumpy, projected safe-to-spend, and a flag for any month that goes negative. Build it on `forecast.rs` so it cannot disagree with Safe to Spend.

**Web.** A Year page: a month strip with the lumpy items stacked, red where a month is short. Tapping a month opens the bills calendar for it.

**Check first.** Read `forecast.rs` and `PlanInput` to confirm whether lumpy items can enter the existing projection or need a small extension. That decides the size of this item.

**Done when:** a premium added for March lowers the January and February sinking-fund advice; a negative month is flagged; unit tests for month rollover and leap years.

## 3. Child and education fund (extends life line and goals)

**What it does.** Answers "how much per month for the child's education, and what does that do to everything else?"

**Rules (`src/planner/education.rs`).**

- Inputs: child's birth year (from the existing `Choice::Child`), education start age(s), today's cost estimate in paise, education inflation in basis points (default higher than general inflation, user-editable), expected return in basis points on the matching holdings.
- Output: future cost at each stage, current corpus earmarked, monthly top-up needed (closed-form SIP formula in integer arithmetic, rounded up; no floats for money).
- Hook into `lifeline::simulate_with` as an optional earmark so the life line shows the effect of the top-up.

**Data.** Prefer storing assumptions in a small `education_plan` row per couple (paise and bps only). Add to export and the round-trip test.

**Web.** A card on the Life page: cost at 18, monthly top-up, and how much the top-up changes the life line (use `compare`).

**Risk.** Return and inflation assumptions are guesses; the UI must say so and show a low/base/high range rather than one confident number. This is planning arithmetic, not investment advice.

## 4. Weekly decision digest

A single screen (and a push with counts only) listing: unreviewed transactions, goals behind pace, expiring envelope links, upcoming birthdays with no gift planned, unbooked dates, milestones due within 14 days. Built as a read-only view over existing handlers (`review`, `coming`, `goals`, `couple_goals`), plus a scheduled round in `push.rs` that keeps to the counts-only rule in CLAUDE.md. No new tables.

## 5. Merchant categorisation (finish the in-flight work)

The working tree already has `src/merchant_rules.rs`, `src/planner/merchant.rs`, `src/planner/merchant_seed.rs`, migrations `…20` and `…21`, and `docs/ml-plan.md`. Before anything else:

1. Finish and test it, including `tests/sms_fixtures.rs`.
2. Run `cargo sqlx prepare -- --bin joy` against a scratch database and commit `.sqlx/` (one `.sqlx/query-19ba…json` is currently deleted).
3. Confirm the new tables are in export or on the `tests/lifecycle.rs` deny list.
4. Commit to `develop` as its own change so the features above start from a clean tree.

## 6. Backup and restore drill

- Scheduled copy of the SQLite database off the Fly volume (the code has backups; verify where they land and that they leave the machine).
- A documented, tested restore: stop, replace file, start, `/healthz`. Run it once against a scratch machine and record the steps in `docs/`.
- Keep the 30-day `unrecognized_alerts` retention in mind when restoring.

## Order and sizing

| Step | Item                                       | Rough size | Depends on                          |
| ---- | ------------------------------------------ | ---------- | ----------------------------------- |
| 1    | Item 5: land the merchant work, clean tree | small      | none                                |
| 2    | Item 6: backup and restore drill           | small      | none                                |
| 3    | Item 1: wedding budget                     | medium     | 1                                   |
| 4    | Item 2: annual calendar                    | medium     | 3 (uses milestones), forecast check |
| 5    | Item 3: education fund                     | medium     | none, but best after 2              |
| 6    | Item 4: weekly digest                      | small      | 3 and 4 give it content             |

Steps 1 and 2 are independent and could run in parallel. 3 to 6 touch overlapping files (`coming.rs`, `export.rs`, migrations) so do them one at a time.

## Rules to hold throughout (from CLAUDE.md)

- Integer paise everywhere; no float money, including the SIP formula.
- Pure planner modules: no IO, `today` passed in.
- New POSTs accept `idempotency_key`.
- New tables in `export::USER_TABLES` (parent before child) and the round-trip test.
- Never edit an existing migration; name new ones after the newest file.
- After SQL changes: `cargo sqlx prepare -- --bin joy`.
- Push notifications: counts only.
- Checks: `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, `bun run typecheck`, `bun run --cwd web lint`, `bun run --cwd web test`.
- Commit to `develop`, merge to `master`, then `fly deploy --depot=false`.

## Open questions for you

1. Wedding date and rough total budget, so defaults and the verdict can be tested against real numbers.
2. Are you liable for advance tax (affects item 2)?
3. Education: one stage (graduation) or several (school, graduation, abroad)?
4. Should the partner see milestone amounts, or only totals? (Nothing here is secret like a surprise slip, but confirm.)
