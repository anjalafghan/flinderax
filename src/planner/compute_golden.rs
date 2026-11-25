//! Characterization tests for [`compute`]: the whole Summary, pinned across scenarios that reach each
//! branch. A refactor must leave `compute_golden.json` byte-for-byte alone. Regenerate on purpose with
//! `UPDATE_GOLDEN=1 cargo test compute_golden`.

use super::*;
use time::macros::date;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/compute_golden.json");

fn acct(id: &str, balance: i64, locked: bool, role: Role, updated: Option<Date>) -> AccountIn {
    AccountIn { id: id.into(), name: id.to_uppercase(), balance, locked, role, updated }
}

fn commit(label: &str, rupees: i64, day: u8, account: Option<&str>, bucket: Bucket) -> CommitmentIn {
    CommitmentIn {
        id: String::new(),
        waiting_from: None,
        label: label.into(),
        amount: rupees * 100,
        day,
        account_id: account.map(str::to_string),
        active: true,
        bucket,
        freq: Freq::Monthly,
        due_month: None,
        to_account_id: None,
        start_month: None,
        end_month: None,
    }
}

fn income(amount_rupees: i64, day: u8) -> IncomeIn {
    IncomeIn { label: "Salary".into(), amount: amount_rupees * 100, day, account_id: None }
}

fn card(id: &str, due: i64, due_date: Option<Date>, next: i64, next_date: Option<Date>, covered: (i64, i64), spent: Option<i64>, updated: Option<Date>) -> CardIn {
    CardIn {
        card_id: id.into(),
        name: id.to_uppercase(),
        due_now: due * 100,
        due_date,
        next_bill_est: next * 100,
        next_due_date: next_date,
        last_updated: updated,
        pay_from: if id == "visa" { Some("kotak".into()) } else { None },
        due_covered: covered.0 * 100,
        next_covered: covered.1 * 100,
        spent_this_period: spent.map(|s| s * 100),
        tracked_since: Some(date!(2026 - 10 - 08)),
    }
}

fn settings() -> SettingsIn {
    SettingsIn { spending: None, spending_bps: None, allocation_day: 2, savings_floor: None, recovery_months: 3 }
}

fn busy_accounts(today: Date) -> Vec<AccountIn> {
    vec![
        acct("hdfc", 4_301_699, false, Role::Spending, Some(today)),
        acct("kotak", 9_500_000, false, Role::Reserve, Some(date!(2026 - 09 - 20))),
        acct("fd", 50_000_000, true, Role::Other, None),
        acct("rd", 12_000_000, true, Role::Other, None),
    ]
}

fn busy_commitments() -> Vec<CommitmentIn> {
    let mut yearly = commit("Insurance", 24_000, 12, Some("kotak"), Bucket::Needs);
    yearly.freq = Freq::Yearly;
    yearly.due_month = Some(10);
    let mut perfume = commit("Perfume", 4_000, 18, Some("hdfc"), Bucket::Wants);
    perfume.freq = Freq::Every { months: 4, first: month_index(date!(2026 - 10 - 01)) };
    let mut stopped = commit("Old gym", 1_500, 9, None, Bucket::Wants);
    stopped.active = false;
    let mut ranged = commit("Course", 3_000, 22, Some("hdfc"), Bucket::Needs);
    ranged.start_month = Some(month_index(date!(2026 - 11 - 01)));
    ranged.end_month = Some(month_index(date!(2027 - 02 - 01)));
    vec![
        commit("Rent", 22_000, 3, Some("kotak"), Bucket::Needs),
        commit("Maid", 3_000, 25, None, Bucket::Needs),
        commit("SIP", 30_000, 7, Some("hdfc"), Bucket::Savings),
        commit("Spotify", 119, 14, Some("hdfc"), Bucket::Wants),
        commit("Donation", 1_000, 28, Some("hdfc"), Bucket::Needs),
        yearly,
        perfume,
        stopped,
        ranged,
    ]
}

fn busy(today: Date) -> PlanInput {
    PlanInput {
        today,
        accounts: busy_accounts(today),
        incomes: vec![income(95_000, 30), income(40_000, 5)],
        commitments: busy_commitments(),
        cards: vec![
            card("visa", 18_000, Some(date!(2026 - 10 - 13)), 23_067, Some(date!(2026 - 11 - 13)), (6_000, 0), Some(9_000), Some(date!(2026 - 10 - 01))),
            card("upi", 0, None, 4_975, Some(date!(2026 - 10 - 28)), (0, 4_975), Some(2_500), Some(today)),
        ],
        settings: SettingsIn { savings_floor: Some(8_000_000), ..settings() },
        allocation: AllocationState { moved: 3_000_000, skipped: 0 },
        planned_dates: vec![
            PlannedDateIn { date: date!(2026 - 10 - 18), label: "Dinner".into(), amount: 450_000 },
            PlannedDateIn { date: date!(2026 - 10 - 19), label: "Covered trip".into(), amount: 0 },
            PlannedDateIn { date: date!(2026 - 12 - 25), label: "Far away".into(), amount: 900_000 },
        ],
    }
}

fn scenarios() -> Vec<(&'static str, PlanInput)> {
    let oct5 = date!(2026 - 10 - 05);
    let mut no_reserve = busy(oct5);
    no_reserve.accounts = vec![acct("hdfc", 2_000_000, false, Role::Other, Some(oct5)), acct("fd", 9_000_000, true, Role::Other, None)];
    let mut overspend = busy(oct5);
    overspend.cards[0].spent_this_period = Some(95_000 * 100);
    overspend.accounts[0].balance = 150_000;
    overspend.allocation = AllocationState { moved: 0, skipped: 500_000 };
    overspend.settings.savings_floor = Some(50_000_000);
    let mut untracked = busy(oct5);
    untracked.cards[1].spent_this_period = None;
    untracked.settings = SettingsIn { spending_bps: Some(2_500), recovery_months: 0, ..settings() };
    let mut fixed_allowance = busy(oct5);
    fixed_allowance.settings.spending = Some(3_000_000);
    fixed_allowance.settings.allocation_day = 31;
    let mut empty = busy(oct5);
    (empty.accounts, empty.incomes, empty.commitments, empty.cards, empty.planned_dates) = Default::default();
    let mut rich = busy(oct5);
    rich.accounts[0].balance = 90_000_000;
    rich.allocation = AllocationState { moved: 99_000_000, skipped: 0 };
    let mut funded = busy(oct5);
    funded.accounts[0].balance = 90_000_000;
    funded.allocation = AllocationState { moved: 0, skipped: 0 };
    let mut nothing_free = busy(oct5);
    nothing_free.accounts[0].balance = 10_000;
    nothing_free.allocation = AllocationState { moved: 0, skipped: 0 };
    vec![
        ("busy_mid_period", busy(oct5)),
        ("busy_on_a_payday", busy(date!(2026 - 10 - 30))),
        ("busy_day_before_payday", busy(date!(2026 - 10 - 29))),
        ("busy_month_end_year_end", busy(date!(2026 - 12 - 31))),
        ("busy_after_late_stale", busy(date!(2027 - 02 - 28))),
        ("no_reserve_account", no_reserve),
        ("overspent_and_short", overspend),
        ("untracked_cards_bps_allowance", untracked),
        ("fixed_allowance_late_alloc_day", fixed_allowance),
        ("nothing_set_up", empty),
        ("plenty_of_cash", rich),
        ("top_up_fully_funded", funded),
        ("top_up_nothing_free", nothing_free),
    ]
}

fn snapshot() -> String {
    let all: Vec<_> = scenarios().into_iter().map(|(name, input)| serde_json::json!({ "name": name, "summary": compute(&input) })).collect();
    serde_json::to_string_pretty(&all).unwrap() + "\n"
}

#[test]
fn compute_matches_the_pinned_output() {
    let now = snapshot();
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "compute output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
