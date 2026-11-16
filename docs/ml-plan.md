# Learning plan for joy

Source: `reports/Local ML for joy app.md` (deep research, 2026-11-16). Short version: the M2 Pro can train
anything joy needs, but one household is too little data for neural models to win. Use simple Bayesian
and rule-based methods, keep inference as plain deterministic Rust, and use the Mac only for offline
experiments (embeddings, drafting SMS patterns).

Ground rules (from CLAUDE.md, they bind every phase):

- `taste.rs`, `picker.rs` and anything new in `src/planner/` stay pure: no IO, seeded RNG, `today` passed in.
- Money is integer paise. A partner's quiet marks (`joy_wants`) are never returned or exported; the draw only turns them into weights.
- Anything a user owns is in `GET /api/export` or on the deny list in `tests/lifecycle.rs`.
- Never edit a migration; add one named after the newest file (`20261119…`). After SQL changes: `cargo sqlx prepare -- --bin joy`.
- Every phase is test-first: red test, minimal code, green, then wire in.

## Status after a scope check (2026-11-16)

The first pass over-built. One household is too little data for sampling, artifact stores or
embeddings to pay off, so those were built, judged not worth their upkeep, and **removed**. What stayed
is small and plain:

- [x] **Draw log** (`joy_draws`, `picker::draw_probabilities`): the chance each shown slip had. It is the only way to judge any future taste change, so it keeps collecting.
- [x] **Per-partner taste** (`taste::PartnerTaste`, `combine_partners`): put-backs ("not my thing") and quiet marks feed each partner's own score; the draw sees only the blended factor, and neither partner's score is ever returned.
- [x] **Merchant rules from corrections** (`merchant_rules`, `planner::merchant::merchant_key`, `data/merchant_seed.json`): a correction is remembered per merchant; lookup order is your rule, then the shipped seed, then the old keyword guesses.
- [x] **SMS regression fixtures** (`tests/sms_fixtures.rs`): 30 anonymised bank texts with the exact paise each must give. Add a line whenever a text is misread.

Removed on purpose: the sealed-model artifact store, Thompson sampling, the offline replay estimator, and the
sender-dispatch template layer (none had a caller). They are in this session's history if real data ever calls for them.

## Parked until there is data

- Embeddings for slip text, a learned category suggester, an SMS pattern drafter, and shipping trained weights. Revisit only if, after months of `joy_draws`, a measured gap shows the plain model is losing.
- If revisited: train on the Mac from `GET /api/export`, ship plain JSON weights scored in pure Rust, and judge against the logged draws first.
