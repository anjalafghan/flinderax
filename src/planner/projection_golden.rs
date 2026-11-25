//! Characterization tests for [`project`]: the whole Projection, pinned across scenarios that reach each
//! branch of the day-by-day simulation. A refactor must leave `projection_golden.json` byte-for-byte
//! alone. Regenerate on purpose with `UPDATE_GOLDEN=1 cargo test projection_golden`.

use super::projection::*;
use super::*;
use time::macros::date;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/projection_golden.json");

fn acct(id: &str, bal: i64, locked: bool, role: Role, today: Date) -> AccountIn {
    AccountIn { id: id.into(), name: id.to_uppercase(), balance: bal, locked, role, updated: Some(today) }
}

fn commit(label: &str, rupees: i64, day: u8, acct: Option<&str>, bucket: Bucket) -> CommitmentIn {
    CommitmentIn {
        id: String::new(),
        waiting_from: None,
        label: label.into(),
        amount: rupees * 100,
        day,
        account_id: acct.map(str::to_string),
        active: true,
        bucket,
        freq: Freq::Monthly,
        due_month: None,
        to_account_id: None,
        start_month: None,
        end_month: None,
    }
}

fn deposit(id: &str, is_rd: bool, rate: i64, maturity: Option<Date>, renewal: Renewal, payout: Option<&str>) -> DepositIn {
    DepositIn { account_id: id.into(), is_rd, rate_bps: rate, maturity, renewal, payout_to: payout.map(str::to_string) }
}

fn base() -> ProjectionInput {
    let today = date!(2026 - 10 - 05);
    let mut rd_sip = commit("RD instalment", 5_000, 12, Some("hdfc"), Bucket::Savings);
    rd_sip.to_account_id = Some("rd1".into());
    ProjectionInput {
        plan: PlanInput {
            today,
            accounts: vec![
                acct("hdfc", 4_301_699, false, Role::Spending, today),
                acct("kotak", 6_914_400, false, Role::Reserve, today),
                acct("fd1", 8_738_500, true, Role::Other, today),
                acct("fd2", 5_000_000, true, Role::Other, today),
                acct("fd3", 3_000_000, true, Role::Other, today),
                acct("rd1", 600_000, true, Role::Other, today),
                acct("misc", 123_400, false, Role::Other, today),
            ],
            incomes: vec![
                IncomeIn { label: "Salary".into(), amount: 7_549_900, day: 30, account_id: Some("hdfc".into()) },
                IncomeIn { label: "Side".into(), amount: 1_000_000, day: 15, account_id: None },
            ],
            commitments: vec![
                commit("House", 4_000, 1, Some("kotak"), Bucket::Needs),
                commit("SIP", 30_000, 10, Some("hdfc"), Bucket::Savings),
                commit("Maid", 3_000, 20, None, Bucket::Needs),
                commit("Stray", 900, 8, Some("nowhere"), Bucket::Wants),
                rd_sip,
            ],
            cards: vec![
                CardIn {
                    card_id: "c1".into(),
                    name: "Main".into(),
                    due_now: 1_200_000,
                    due_date: Some(date!(2026 - 10 - 13)),
                    next_bill_est: 2_306_700,
                    next_due_date: Some(date!(2026 - 11 - 13)),
                    last_updated: Some(today),
                    pay_from: Some("kotak".into()),
                    due_covered: 0,
                    next_covered: 0,
                    spent_this_period: Some(0),
                    tracked_since: Some(today),
                },
                CardIn {
                    card_id: "c2".into(),
                    name: "Second".into(),
                    due_now: 0,
                    due_date: None,
                    next_bill_est: 500_000,
                    next_due_date: None,
                    last_updated: None,
                    pay_from: None,
                    due_covered: 0,
                    next_covered: 0,
                    spent_this_period: None,
                    tracked_since: None,
                },
            ],
            settings: SettingsIn { spending: Some(1_887_500), spending_bps: None, allocation_day: 10, savings_floor: Some(5_000_000), recovery_months: 3 },
            allocation: AllocationState { moved: 0, skipped: 2_662_400 },
            planned_dates: vec![],
        },
        deposits: vec![
            deposit("fd1", false, 710, Some(date!(2026 - 11 - 22)), Renewal::Payout, Some("kotak")),
            deposit("fd2", false, 680, Some(date!(2027 - 01 - 15)), Renewal::RenewPrincipal, None),
            deposit("fd3", false, 650, Some(date!(2027 - 03 - 03)), Renewal::RenewAll, Some("hdfc")),
            deposit("rd1", true, 600, Some(date!(2027 - 02 - 12)), Renewal::Payout, None),
        ],
        emis: vec![EmiIn { monthly: 381_500, months_left: 4 }, EmiIn { monthly: 100_000, months_left: 1 }, EmiIn { monthly: 50_000, months_left: 30 }],
        months: 14,
        spend_pct: 100,
    }
}

fn item(name: &str, kind: ItemKind, amount: i64, start: Date, tenure: Option<i64>, rate: Option<i64>, goal: Option<(i64, Date)>, renewal: Option<Renewal>) -> ScenarioItem {
    ScenarioItem {
        name: name.into(),
        kind,
        amount_paise: amount,
        start,
        tenure_months: tenure,
        rate_bps: rate,
        funded_from: None,
        payout_to: None,
        renewal,
        target_paise: goal.map(|g| g.0),
        target_date: goal.map(|g| g.1),
    }
}

fn scenarios() -> Vec<(&'static str, ProjectionInput, Vec<ScenarioItem>)> {
    let d = |y, m, day| Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), day).unwrap();
    let mut thin = base();
    thin.plan.accounts[0].balance = 50_000;
    thin.plan.accounts[1].balance = 100_000;
    thin.spend_pct = 130;
    let mut no_reserve = base();
    no_reserve.plan.accounts.retain(|a| a.id != "kotak");
    no_reserve.plan.cards[0].pay_from = None;
    let mut two_funded = item("Gadget", ItemKind::OneOff, 8_000_000, d(2026, 12, 3), None, None, None, None);
    two_funded.funded_from = Some("kotak".into());
    let mut paid_out = item("FD to hdfc", ItemKind::Fd, 10_000_000, d(2026, 11, 1), Some(6), Some(700), None, None);
    paid_out.payout_to = Some("hdfc".into());
    let mut huge = base();
    huge.months = 500;
    let mut short = base();
    short.months = 0;
    vec![
        ("baseline_only", base(), vec![]),
        ("rd_with_goal", base(), vec![item("RD", ItemKind::Rd, 2_500_000, d(2026, 11, 1), Some(12), Some(700), Some((40_000_000, d(2027, 11, 1))), None)]),
        ("fd_renew_all_and_oneoff", base(), vec![
            item("Big FD", ItemKind::Fd, 20_000_000, d(2026, 10, 20), Some(24), Some(720), Some((30_000_000, d(2028, 10, 20))), Some(Renewal::RenewAll)),
            item("Wedding", ItemKind::OneOff, 50_000_000, d(2027, 2, 14), None, None, Some((1, d(2027, 2, 14))), None),
        ]),
        ("fd_renew_principal_and_past_start", base(), vec![
            item("Old FD", ItemKind::Fd, 5_000_000, d(2026, 1, 1), Some(15), Some(650), None, Some(Renewal::RenewPrincipal)),
            item("Past spend", ItemKind::OneOff, 100_000, d(2025, 5, 5), None, None, None, None),
        ]),
        ("default_tenure_and_no_rate", base(), vec![item("Plain RD", ItemKind::Rd, 1_000_000, d(2026, 10, 5), None, None, None, None)]),
        ("same_name_twice_and_funding", base(), vec![
            two_funded,
            item("Gadget", ItemKind::OneOff, 1_000_000, d(2026, 10, 6), None, None, None, None),
            paid_out,
        ]),
        ("thin_cash_stress", thin, vec![item("RD", ItemKind::Rd, 4_000_000, d(2026, 10, 25), Some(6), Some(600), None, None)]),
        ("no_reserve_account", no_reserve, vec![item("RD", ItemKind::Rd, 500_000, d(2026, 11, 1), Some(10), Some(650), Some((6_000_000, d(2027, 9, 1))), None)]),
        ("months_clamped_high", huge, vec![item("Long", ItemKind::Fd, 1_000_000, d(2026, 10, 5), Some(900), Some(700), None, None)]),
        ("months_clamped_low", short, vec![]),
    ]
}

fn snapshot() -> String {
    let all: Vec<_> = scenarios().into_iter().map(|(name, input, items)| serde_json::json!({ "name": name, "projection": project(&input, &items) })).collect();
    serde_json::to_string_pretty(&all).unwrap() + "\n"
}

#[test]
fn project_matches_the_pinned_output() {
    let now = snapshot();
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "projection output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
