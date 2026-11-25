use super::*;
use time::macros::date;

const HDFC: &str = "hdfc";
const KOTAK: &str = "kotak";

fn acct(id: &str, name: &str, paise: i64, locked: bool, role: Role, updated: Date) -> AccountIn {
    AccountIn { id: id.into(), name: name.into(), balance: paise, locked, role, updated: Some(updated) }
}

fn commitment(label: &str, rupees: i64, day: u8, account: Option<&str>, bucket: Bucket) -> CommitmentIn {
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

fn card(id: &str, name: &str, next_bill: i64) -> CardIn {
    CardIn {
        card_id: id.into(),
        name: name.into(),
        due_now: 0,
        due_date: Some(date!(2026 - 10 - 13)),
        next_bill_est: next_bill,
        next_due_date: Some(date!(2026 - 11 - 13)),
        last_updated: Some(date!(2026 - 10 - 05)),
        pay_from: Some(KOTAK.into()),
        due_covered: 0,
        next_covered: 0,
        spent_this_period: Some(0),
        tracked_since: Some(date!(2026 - 10 - 05)),
    }
}

fn line<'a>(s: &'a Summary, label_starts: &str) -> &'a Line {
    s.breakdown
        .iter()
        .find(|l| l.label.starts_with(label_starts))
        .unwrap_or_else(|| panic!("no breakdown line starting with {label_starts:?}"))
}

/// 5 Oct 2026, the real data. HDFC (spending) and Kotak (reserve); two locked FDs; salary
/// ₹75,499 on the 30th. Needs: house 4,000, office 2,000 (paid from Kotak), donation 1,000;
/// SIP 30,000 and Spotify 119 from HDFC. Spending allowance ₹18,875. Main HDFC's next bill is
/// the bank's unbilled net ₹23,067, the UPI card's is ₹4,975, both due 13 Nov and paid from Kotak.
fn fixture() -> PlanInput {
    let today = date!(2026 - 10 - 05);
    PlanInput {
        today,
        accounts: vec![
            acct(HDFC, "HDFC", 4_301_699, false, Role::Spending, today),
            acct(KOTAK, "Kotak", 6_914_400, false, Role::Reserve, today),
            acct("fd1", "Emergency Fund", 12_165_900, true, Role::Other, today),
            acct("fd2", "iPhone saving", 8_738_500, true, Role::Other, today),
        ],
        incomes: vec![IncomeIn { label: "Salary".into(), amount: 7_549_900, day: 30, account_id: Some(HDFC.into()) }],
        commitments: vec![
            commitment("House", 4_000, 1, Some(KOTAK), Bucket::Needs),
            commitment("Office travel", 2_000, 20, Some(KOTAK), Bucket::Needs),
            commitment("Donation", 1_000, 31, Some(KOTAK), Bucket::Needs),
            commitment("Spotify", 119, 1, Some(HDFC), Bucket::Wants),
            commitment("SIP", 30_000, 10, Some(HDFC), Bucket::Savings),
        ],
        cards: vec![card("c1", "Main HDFC", 2_306_700), card("c2", "HDFC UPI", 497_500)],
        settings: SettingsIn { spending: Some(1_887_500), spending_bps: None, allocation_day: 10, savings_floor: Some(4_110_200), recovery_months: 3 },
        allocation: AllocationState::default(),
        planned_dates: vec![],
    }
}

/// Everything the ledger can say once you've dealt with this period: both bills already in
/// Kotak and this month's savings top-up skipped.
fn settled() -> PlanInput {
    let mut i = fixture();
    i.cards[0].next_covered = 2_306_700;
    i.cards[1].next_covered = 497_500;
    i.allocation = AllocationState { moved: 0, skipped: 2_662_400 };
    i
}

// ---------- the budget in amounts ----------

#[test]
fn budget_is_needs_plus_spending_plus_savings() {
    let b = compute(&fixture()).budget;
    assert_eq!(b.income_paise, 7_549_900);
    assert_eq!(b.needs_paise, 700_000); // 4,000 + 2,000 + 1,000
    assert_eq!(b.spending_paise, 1_887_500);
    assert_eq!(b.savings_paise, 4_962_400); // the rest
    assert_eq!(b.sip_paise, 3_000_000);
    assert_eq!(b.rest_savings_paise, 1_962_400);
    assert_eq!(b.allocation_paise, 2_662_400); // needs money + rest of savings -> Kotak
    assert_eq!(b.needs_paise + b.spending_paise + b.savings_paise, b.income_paise);
}

#[test]
fn spending_allowance_defaults_to_twenty_percent() {
    let mut i = fixture();
    i.settings.spending = None;
    assert_eq!(compute(&i).budget.spending_paise, 1_509_980);
}

#[test]
fn spending_allowance_can_be_a_percentage_of_income() {
    let mut i = fixture();
    i.settings.spending = None;
    i.settings.spending_bps = Some(2_500); // 25% of 75,499
    let b = compute(&i).budget;
    assert_eq!(b.spending_paise, 7_549_900 * 25 / 100);
    assert_eq!(b.savings_paise, b.income_paise - b.needs_paise - b.spending_paise);
    // it follows the income when the salary changes
    i.incomes[0].amount = 10_000_000;
    assert_eq!(compute(&i).budget.spending_paise, 2_500_000);
}

#[test]
fn a_fixed_amount_wins_over_a_percentage() {
    let mut i = fixture();
    i.settings.spending_bps = Some(5_000);
    assert_eq!(compute(&i).budget.spending_paise, 1_887_500); // the amount from the fixture
}

#[test]
fn yearly_bills_are_set_aside_monthly_and_due_once() {
    let mut yearly = commitment("Phone", 12_000, 12, Some(KOTAK), Bucket::Needs);
    yearly.freq = Freq::Yearly;
    yearly.due_month = Some(12);
    assert_eq!(monthly_equivalent(&yearly, month_index(date!(2026 - 10 - 05))), 100_000);
    assert_eq!(commitment_dates(&yearly, date!(2026 - 10 - 01), date!(2027 - 01 - 01)), vec![date!(2026 - 12 - 12)]);
    yearly.due_month = None; // month not known: still set aside, but nothing scheduled
    assert!(commitment_dates(&yearly, date!(2026 - 10 - 01), date!(2027 - 01 - 01)).is_empty());
    assert_eq!(monthly_equivalent(&yearly, month_index(date!(2026 - 10 - 05))), 100_000);

    let mut i = fixture();
    let mut phone = commitment("Phone", 12_000, 12, Some(KOTAK), Bucket::Needs);
    phone.freq = Freq::Yearly;
    phone.due_month = Some(12);
    i.commitments.push(phone);
    assert_eq!(compute(&i).budget.needs_paise, 800_000);
}

// ---------- the current pay period ----------

#[test]
fn pay_period_runs_from_last_payday_to_the_next() {
    let p = pay_period(date!(2026 - 10 - 05), &fixture().incomes);
    assert_eq!((p.start, p.payday1, p.payday2), (date!(2026 - 09 - 30), date!(2026 - 10 - 30), date!(2026 - 11 - 30)));
    let s = compute(&fixture());
    assert_eq!((s.until, s.days_left, s.period_start), (date!(2026 - 10 - 30), 25, date!(2026 - 09 - 30)));
}

#[test]
fn untouched_period_is_red_and_lists_every_move() {
    let s = compute(&fixture());
    // HDFC 43,016.99 - SIP 30,000 - 28,042 of card bills still to move: not even the bills are covered
    assert_eq!(line(&s, "HDFC balance").amount_paise, 4_301_699);
    assert_eq!(line(&s, "Fixed payments from it").amount_paise, -3_000_000);
    assert_eq!(line(&s, "Card bills still to move").amount_paise, -2_804_200);
    assert_eq!(line(&s, "Free in HDFC").amount_paise, -1_502_501);
    // the savings top-up can't be funded, and it never reduces the pot (there is none)
    assert!(s.breakdown.iter().all(|l| !l.label.starts_with("Savings top-up to move")));
    assert!(line(&s, "Savings top-up you can't fund").note.as_deref().unwrap().contains("₹26,624"));
    assert_eq!(s.short_by_paise, 1_502_501);
    assert_eq!((s.safe_to_spend_paise, s.status), (0, Status::Red));
    assert_eq!(s.recovery.unwrap().owed_paise, 1_502_501 + 2_662_400);
}

#[test]
fn card_bills_become_move_buttons() {
    let s = compute(&fixture());
    let moves: Vec<&LedgerAction> = s.actions.iter().filter_map(|a| a.ledger.as_ref()).filter(|l| l.kind == "card_bill").collect();
    assert_eq!(moves.len(), 2);
    assert_eq!(moves[0].amount_paise, 2_306_700);
    assert_eq!(moves[0].key, "c1:2026-11-13");
    assert_eq!(moves[0].from_account_id.as_deref(), Some(HDFC));
    assert_eq!(moves[0].to_account_id.as_deref(), Some(KOTAK));
    assert!(s.actions.iter().any(|a| a.text == "Move ₹23,067 HDFC → Kotak for Main HDFC's next bill, pay by 13 Nov"));
}

#[test]
fn partly_covered_bill_only_asks_for_the_rest() {
    let mut i = fixture();
    i.cards[0].next_covered = 500_000;
    let s = compute(&i);
    let m = s.actions.iter().filter_map(|a| a.ledger.as_ref()).find(|l| l.key == "c1:2026-11-13").unwrap();
    assert_eq!(m.amount_paise, 1_806_700);
    // once covered in full there is no button at all
    let s = compute(&settled());
    assert!(s.actions.iter().filter_map(|a| a.ledger.as_ref()).all(|l| l.kind != "card_bill"));
}

#[test]
fn growing_bill_asks_only_for_the_increase() {
    let ask = |i: &PlanInput| {
        compute(i).actions.iter().filter_map(|a| a.ledger.as_ref()).find(|l| l.key == "c1:2026-11-13").map(|l| l.amount_paise)
    };
    let mut i = fixture();
    // bill is 14,000: move all of it
    i.cards[0].next_bill_est = 1_400_000;
    assert_eq!(ask(&i), Some(1_400_000));
    // after moving it, nothing is asked
    i.cards[0].next_covered = 1_400_000;
    assert_eq!(ask(&i), None);
    // bill grows to 15,000: only the extra 1,000 is asked
    i.cards[0].next_bill_est = 1_500_000;
    assert_eq!(ask(&i), Some(100_000));
    // moved again (total 15,000), then the bill shrinks to 14,500: no refund request
    i.cards[0].next_covered = 1_500_000;
    i.cards[0].next_bill_est = 1_450_000;
    assert_eq!(ask(&i), None);
    // statement generated: the same bill is now "current" with the same key and covered amount,
    // and the fresh next bill starts uncovered
    i.cards[0].due_now = 1_450_000;
    i.cards[0].due_date = i.cards[0].next_due_date;
    i.cards[0].due_covered = 1_500_000;
    i.cards[0].next_bill_est = 300_000;
    i.cards[0].next_due_date = i.cards[0].next_due_date.map(|d| d.replace_month(time::Month::December).unwrap());
    i.cards[0].next_covered = 0;
    let keys: Vec<(String, i64)> = compute(&i)
        .actions
        .iter()
        .filter_map(|a| a.ledger.as_ref())
        .filter(|l| l.kind == "card_bill" && l.key.starts_with("c1:"))
        .map(|l| (l.key.clone(), l.amount_paise))
        .collect();
    assert_eq!(keys, vec![("c1:2026-12-13".to_string(), 300_000)]);
}

#[test]
fn savings_top_up_you_cannot_afford_can_be_skipped() {
    let s = compute(&fixture());
    let a = s.actions.iter().find(|a| a.ledger.as_ref().is_some_and(|l| l.kind == "allocation")).unwrap();
    let l = a.ledger.as_ref().unwrap();
    assert_eq!(l.amount_paise, 0); // nothing free in HDFC
    assert_eq!(l.skippable_paise, 2_662_400);
    assert_eq!(l.key, "2026-09-30");
    assert!(a.text.contains("Skip it this month"));
}

#[test]
fn settled_period_leaves_the_hdfc_pot() {
    let s = compute(&settled());
    // HDFC 43,016.99 - SIP 30,000 = 13,016.99, within the 18,756 left of the allowance
    assert_eq!(line(&s, "Free in HDFC").amount_paise, 1_301_699);
    assert_eq!(s.safe_to_spend_paise, 1_301_699);
    assert_eq!(s.per_week_paise, 1_301_699 / 4);
    assert_eq!(s.status, Status::Green);
    assert_eq!(s.short_by_paise, 0);
}

#[test]
fn skipping_the_top_up_creates_a_recovery_plan() {
    let r = compute(&settled()).recovery.unwrap();
    assert_eq!(r.owed_paise, 2_662_400);
    assert_eq!(r.months, 3);
    assert_eq!(r.per_month_paise, 887_467); // rounded up
}

#[test]
fn moving_money_in_the_ledger_lowers_what_is_free_and_what_is_owed() {
    // Moved 28,042 HDFC -> Kotak: HDFC balance drops (the handler does that), bills are covered.
    let mut i = settled();
    i.accounts[0].balance -= 2_804_200;
    i.accounts[1].balance += 2_804_200;
    let s = compute(&i);
    assert_eq!(s.pots.spending.free_paise, 1_301_699 - 2_804_200);
    assert_eq!(s.pots.reserve.free_savings_paise, 3_910_200 + 2_804_200);
}

// ---------- spending allowance ----------

#[test]
fn spending_so_far_caps_safe_to_spend() {
    let mut i = settled();
    i.cards[0].spent_this_period = Some(1_500_000);
    let s = compute(&i);
    // allowance 18,875 - spotify 119 - 5 days of pace before tracking began (30 Sep -> 5 Oct) - spent 15,000
    let pace = 1_887_500 * 5 / 30;
    assert_eq!(s.spending.before_tracking_paise, pace);
    assert_eq!(s.spending.left_paise, 1_887_500 - 11_900 - pace - 1_500_000);
    assert_eq!(s.safe_to_spend_paise, s.spending.left_paise);
    assert!(line(&s, "Held back").amount_paise < 0);
}

#[test]
fn overspending_is_reported_and_leaves_nothing_to_spend() {
    let mut i = settled();
    i.cards[0].spent_this_period = Some(2_000_000);
    let s = compute(&i);
    assert_eq!(s.spending.overspend_paise, 2_000_000 + 11_900 + 1_887_500 * 5 / 30 - 1_887_500);
    assert_eq!((s.spending.left_paise, s.safe_to_spend_paise), (0, 0));
}

#[test]
fn untracked_spending_falls_back_to_a_pro_rated_allowance() {
    let mut i = settled();
    i.cards[0].spent_this_period = None;
    let s = compute(&i);
    assert!(!s.spending.tracked);
    assert_eq!(s.spending.left_paise, 1_887_500 * 25 / 30);
}

#[test]
fn spending_pace_before_tracking_vanishes_when_tracking_starts_at_the_period_start() {
    let mut i = settled();
    i.cards[0].tracked_since = Some(date!(2026 - 09 - 30));
    i.cards[0].spent_this_period = Some(100_000);
    let s = compute(&i);
    assert_eq!(s.spending.before_tracking_paise, 0);
    assert_eq!(s.spending.left_paise, 1_887_500 - 11_900 - 100_000);
}

/// The new_anjal scenario: one account with ₹1,00,000, salary ₹1,00,000 on the 1st, a card that
/// started at zero and now owes ₹10,000. Spending went from "pace" to "10,000 actually spent".
#[test]
fn ten_thousand_spent_takes_ten_thousand_off_the_remaining_allowance() {
    let today = date!(2026 - 10 - 05);
    let mut input = PlanInput {
        today,
        accounts: vec![acct("hdfc", "HDFC savings", 10_000_000, false, Role::Spending, today)],
        incomes: vec![IncomeIn { label: "Salary".into(), amount: 10_000_000, day: 1, account_id: Some("hdfc".into()) }],
        commitments: vec![],
        cards: vec![CardIn {
            card_id: "c1".into(),
            name: "My Card".into(),
            due_now: 0,
            due_date: None,
            next_bill_est: 0,
            next_due_date: None,
            last_updated: Some(today),
            pay_from: None,
            due_covered: 0,
            next_covered: 0,
            spent_this_period: None,
            tracked_since: None,
        }],
        settings: SettingsIn { spending: None, spending_bps: None, allocation_day: 10, savings_floor: None, recovery_months: 3 },
        allocation: AllocationState::default(),
        planned_dates: vec![],
    };
    // before: no spend tracked, so the allowance (20% = ₹20,000) runs at an even pace: 27 of 31 days left
    let before = compute(&input);
    assert_eq!(before.safe_to_spend_paise, 2_000_000 * 27 / 31); // ₹17,419
    assert_eq!(before.until, date!(2026 - 11 - 01));

    // after: the card started at zero on 5 Oct (4 days into the period) and now holds ₹10,000
    input.cards[0].due_now = 1_000_000;
    input.cards[0].spent_this_period = Some(1_000_000);
    input.cards[0].tracked_since = Some(date!(2026 - 10 - 05));
    let after = compute(&input);
    assert_eq!(after.spending.before_tracking_paise, 2_000_000 * 4 / 31);
    // 20,000 - 2,580.64 (4 days at pace) - 10,000 = ₹7,419.36
    assert_eq!(after.safe_to_spend_paise, 741_936);
    assert!((before.safe_to_spend_paise - after.safe_to_spend_paise - 1_000_000).abs() <= 1);
}

#[test]
fn the_savings_top_up_never_eats_the_spending_pot() {
    // HDFC has far more than the 18,756 left of the allowance: only the excess may move
    let mut i = settled();
    i.allocation = AllocationState::default();
    i.accounts[0].balance = 10_000_000;
    let s = compute(&i);
    let l = s.actions.iter().filter_map(|a| a.ledger.as_ref()).find(|l| l.kind == "allocation").unwrap();
    let free_before = 10_000_000 - 3_000_000;
    let pot = s.spending.left_paise; // allowance - spotify - pace - 0 spent
    assert_eq!(l.amount_paise, (free_before - pot).min(2_662_400));
    assert!(s.safe_to_spend_paise >= pot - 1); // the pot is intact after the move
    assert_eq!(s.safe_to_spend_paise, pot);
}

#[test]
fn a_tight_month_only_offers_to_skip_the_top_up() {
    // 13,016.99 free is inside the spending allowance: nothing may move, the whole plan is skippable
    let mut i = fixture();
    i.cards[0].next_covered = 2_306_700;
    i.cards[1].next_covered = 497_500;
    let s = compute(&i);
    let l = s.actions.iter().filter_map(|a| a.ledger.as_ref()).find(|l| l.kind == "allocation").unwrap();
    assert_eq!(l.amount_paise, 0);
    assert_eq!(l.skippable_paise, 2_662_400);
    assert_eq!(s.safe_to_spend_paise, 1_301_699); // the pot is untouched: ₹13,016.99
    assert_eq!(s.recovery.unwrap().owed_paise, 2_662_400); // the missing savings, even before you click skip
}

// ---------- the reserve ----------

#[test]
fn reserve_free_savings_and_floor() {
    let r = compute(&fixture()).pots.reserve;
    // Kotak 69,144 - cards 28,042 - office travel 2,000 before payday
    assert_eq!(r.required_paise, 2_804_200 + 200_000);
    assert_eq!(r.free_savings_paise, 3_910_200);
    assert!(r.below_floor); // floor 41,102
    let mut i = fixture();
    i.settings.savings_floor = Some(3_000_000);
    assert!(!compute(&i).pots.reserve.below_floor);
    assert!(compute(&fixture()).actions.iter().any(|a| a.text.contains("below your")));
}

#[test]
fn locked_accounts_are_a_separate_pot() {
    let l = compute(&fixture()).pots.locked;
    assert_eq!((l.count, l.total_paise), (2, 12_165_900 + 8_738_500));
}

// ---------- without roles, like before ----------

#[test]
fn accounts_without_roles_are_one_pool_with_no_ledger() {
    let mut i = fixture();
    i.accounts[0].role = Role::Other;
    i.accounts[1].role = Role::Other;
    let s = compute(&i);
    // 112,160.99 - SIP 30,000 - office travel 2,000 (one pool: nothing is paid from a reserve) - both bills 28,042
    assert_eq!(s.pots.spending.free_paise, 11_216_099 - 3_200_000 - 2_804_200);
    assert!(s.actions.iter().all(|a| a.ledger.is_none()));
    assert_eq!(s.pots.reserve.account_name, None);
}

// ---------- heads-up, labels, stale data ----------

#[test]
fn next_period_heads_up() {
    let n = compute(&fixture()).next_period.unwrap();
    assert_eq!((n.label.as_str(), n.income_paise), ("November money", 7_549_900));
    // donation 31 Oct, house + spotify 1 Nov, SIP 10 Nov, office travel 20 Nov
    assert_eq!(n.committed_paise, 100_000 + 400_000 + 11_900 + 3_000_000 + 200_000);
}

#[test]
fn labels_follow_the_pay_period() {
    let s = compute(&fixture());
    assert_eq!(s.next_income.as_ref().unwrap().label, "November money");
    assert_eq!(s.period_label.as_deref(), Some("October money"));
}

#[test]
fn stale_balances_and_snapshots_are_flagged() {
    let mut i = fixture();
    i.accounts[1].updated = Some(date!(2026 - 09 - 25));
    i.cards[0].last_updated = Some(date!(2026 - 09 - 20));
    let s = compute(&i);
    assert_eq!(s.stale.len(), 2);
    assert!(s.stale.iter().any(|x| x.what == "Kotak balance" && x.days_old == 10));
    assert!(s.stale.iter().any(|x| x.what == "Main HDFC snapshot" && x.days_old == 15));
    assert!(s.actions.iter().any(|a| a.severity == "info"));
}

#[test]
fn locked_accounts_are_never_stale() {
    let mut i = fixture();
    i.accounts[2].updated = Some(date!(2025 - 01 - 01));
    assert!(compute(&i).stale.is_empty());
}

#[test]
fn no_income_falls_back_to_a_thirty_day_period() {
    let mut i = fixture();
    i.incomes.clear();
    let s = compute(&i);
    assert_eq!(s.until, date!(2026 - 11 - 04));
    assert!(s.next_income.is_none() && s.next_period.is_none());
}

#[test]
fn time_travel_recurring_items_roll_forward() {
    for (today, until, days_left, label, period) in [
        (date!(2026 - 11 - 02), date!(2026 - 11 - 30), 28, "December money", "November money"),
        (date!(2026 - 12 - 31), date!(2027 - 01 - 30), 30, "February money", "January money"),
        (date!(2027 - 02 - 05), date!(2027 - 02 - 28), 23, "March money", "February money"),
    ] {
        let mut i = fixture();
        i.today = today;
        for a in &mut i.accounts {
            a.updated = Some(today);
        }
        for c in &mut i.cards {
            c.last_updated = Some(today);
            c.due_date = None;
            c.next_due_date = None;
        }
        let s = compute(&i);
        assert_eq!((s.until, s.days_left), (until, days_left), "{today}");
        assert_eq!(s.next_income.as_ref().unwrap().label, label, "{today}");
        assert_eq!(s.period_label.as_deref(), Some(period), "{today}");
        assert!(s.upcoming.iter().any(|u| u.label == "SIP"), "SIP recurs @ {today}");
    }
}

#[test]
fn categories_map_to_default_buckets() {
    assert_eq!(default_bucket("sip"), Bucket::Savings);
    assert_eq!(default_bucket("rd"), Bucket::Savings);
    assert_eq!(default_bucket("subscription"), Bucket::Wants);
    assert_eq!(default_bucket("bills"), Bucket::Needs);
    assert_eq!(default_bucket("gym"), Bucket::Needs);
    for c in ["emi", "rent", "family", "phone", "internet", "transport", "car", "insurance", "health", "fees"] {
        assert_eq!(default_bucket(c), Bucket::Needs, "{c}");
    }
    for c in ["shopping", "dates", "gifts", "travel"] {
        assert_eq!(default_bucket(c), Bucket::Wants, "{c}");
    }
    assert_eq!(default_bucket("investment"), Bucket::Savings);
    assert_eq!(Bucket::parse("wants"), Some(Bucket::Wants));
    assert_eq!(Bucket::parse("nope"), None);
}

// ---------- card maths ----------

#[test]
fn next_bill_comes_from_the_banks_unbilled_numbers() {
    // Main HDFC 5 Oct: outstanding 85,487, due 0, unbilled 79,790 - credit 56,723
    let a = card_amounts(CardNumbers {
        total_due: 0,
        outstanding: 8_548_700,
        unbilled_net: Some(7_979_000 - 5_672_300),
        ..Default::default()
    });
    assert_eq!(a.next_bill_est_paise, 2_306_700);
    assert_eq!(a.loan_not_due_paise, 8_548_700 - 2_306_700); // ₹62,420 of loan, never counted as due
    assert_eq!(a.due_now_paise, 0);
}

#[test]
fn pending_charges_add_to_the_unbilled_based_bill() {
    let a = card_amounts(CardNumbers { unbilled_net: Some(100_000), pending_charges: 25_000, ..Default::default() });
    assert_eq!(a.next_bill_est_paise, 125_000);
}

#[test]
fn without_unbilled_numbers_the_emi_formula_is_used() {
    let n = CardNumbers {
        total_due: 0,
        outstanding: 6_148_800,
        emi_monthly: 381_500,
        emi_remaining: 1_473_900,
        pending_charges: 128_500,
        unbilled_net: None,
    };
    let a = card_amounts(n);
    // outstanding - due - emi_remaining + emi_monthly + pending
    assert_eq!(a.next_bill_est_paise, 5_184_900);
    assert_eq!(a.loan_not_due_paise, 1_473_900 - 381_500);
    assert_eq!(card_amounts(CardNumbers { pending_charges: 0, ..n }).next_bill_est_paise, 5_056_400);
}

#[test]
fn card_with_credit_balance_never_goes_negative() {
    assert_eq!(card_amounts(CardNumbers { outstanding: -50_000, ..Default::default() }).next_bill_est_paise, 0);
    assert_eq!(card_amounts(CardNumbers { unbilled_net: Some(-9_000), ..Default::default() }).next_bill_est_paise, 0);
}

#[test]
fn emis_age_with_the_calendar() {
    // 4 months left in Oct 2026 (idx), ₹3,815/month, ₹14,739 remaining
    let oct = 2026 * 12 + 9;
    assert_eq!(emi_effective(381_500, 1_473_900, 4, oct, oct), (381_500, 1_473_900, 4));
    assert_eq!(emi_effective(381_500, 1_473_900, 4, oct, oct + 1), (381_500, 1_105_425, 3));
    assert_eq!(emi_effective(381_500, 1_473_900, 4, oct, oct + 3), (381_500, 368_475, 1));
    // finished: stops adding to the next bill
    assert_eq!(emi_effective(381_500, 1_473_900, 4, oct, oct + 4), (0, 0, 0));
    assert_eq!(emi_effective(381_500, 1_473_900, 4, oct, oct + 20), (0, 0, 0));
    // entered with 0 months left, or an as-of month in the future: no panic, no negative
    assert_eq!(emi_effective(100, 100, 0, oct, oct), (0, 0, 0));
    assert_eq!(emi_effective(381_500, 1_473_900, 4, oct + 2, oct), (381_500, 1_473_900, 4));
    // Even legacy rows at extreme i64 values must not overflow projection arithmetic.
    assert_eq!(emi_effective(0, i64::MAX, 2, oct, oct + 1), (0, i64::MAX / 2, 1));
}

#[test]
fn rupee_formatting_uses_indian_grouping() {
    assert_eq!(format_inr(0), "₹0");
    assert_eq!(format_inr(99_900), "₹999");
    assert_eq!(format_inr(100_000), "₹1,000");
    assert_eq!(format_inr(12_345_600), "₹1,23,456");
    assert_eq!(format_inr(754_990_000), "₹75,49,900");
    assert_eq!(format_inr(-150), "-₹1.50");
    assert_eq!(format_inr(-100), "-₹1");
    assert_eq!(format_inr(5), "₹0.05");
    assert_eq!(format_inr(i64::MAX), "₹92,23,37,20,36,85,47,758.07");
    assert_eq!(format_inr(i64::MIN), "-₹92,23,37,20,36,85,47,758.08");
}

// ---------- planned dates ----------

#[test]
fn a_planned_date_before_payday_is_held_back_and_listed() {
    let before = compute(&settled());
    let mut i = settled();
    i.planned_dates = vec![PlannedDateIn { date: date!(2026 - 10 - 10), label: "Dinner".into(), amount: 240_000 }];
    let s = compute(&i);
    assert_eq!(s.safe_to_spend_paise, before.safe_to_spend_paise - 240_000);
    assert_eq!(line(&s, "Held back for planned dates").amount_paise, -240_000);
    assert!(s.upcoming.iter().any(|u| u.kind == "planned_date" && u.label == "Dinner" && u.amount_paise == 240_000));
}

#[test]
fn a_planned_date_after_payday_is_not_held_yet_and_the_hold_never_goes_below_zero() {
    let before = compute(&settled());
    let mut i = settled();
    i.planned_dates = vec![PlannedDateIn { date: date!(2026 - 12 - 20), label: "Trip".into(), amount: 900_000 }];
    assert_eq!(compute(&i).safe_to_spend_paise, before.safe_to_spend_paise);
    i.planned_dates = vec![PlannedDateIn { date: date!(2026 - 10 - 10), label: "Big".into(), amount: i64::MAX }];
    assert_eq!(compute(&i).safe_to_spend_paise, 0);
}

// ---------- bills every few months ----------

fn every(months: u8, first: Date) -> CommitmentIn {
    let mut c = commitment("Perfume", 6_000, 5, Some(KOTAK), Bucket::Wants);
    c.freq = Freq::Every { months, first: month_index(first) };
    c
}

#[test]
fn a_bill_every_four_months_falls_due_in_every_fourth_month_from_its_first() {
    let c = every(4, date!(2026 - 11 - 01));
    // November, March, July, then November again; nothing in between.
    let got = commitment_dates(&c, date!(2026 - 11 - 01), date!(2027 - 12 - 01));
    assert_eq!(got, vec![date!(2026 - 11 - 05), date!(2027 - 03 - 05), date!(2027 - 07 - 05), date!(2027 - 11 - 05)]);
}

#[test]
fn a_bill_every_few_months_is_never_due_before_its_first_month() {
    // The pattern runs on in both directions, but the first month is the first payment.
    let c = every(4, date!(2027 - 03 - 01));
    assert!(commitment_dates(&c, date!(2026 - 07 - 01), date!(2027 - 03 - 01)).is_empty());
    assert_eq!(commitment_dates(&c, date!(2026 - 07 - 01), date!(2027 - 04 - 01)), vec![date!(2027 - 03 - 05)]);
}

#[test]
fn a_bill_every_few_months_counts_across_a_year_boundary() {
    let c = every(5, date!(2026 - 10 - 01));
    // October, March, August.
    let months: Vec<u32> = commitment_dates(&c, date!(2026 - 10 - 01), date!(2027 - 09 - 01))
        .iter()
        .map(|d| d.month() as u32)
        .collect();
    assert_eq!(months, vec![10, 3, 8]);
}

#[test]
fn a_bill_every_few_months_honours_its_first_and_last_month_and_pause() {
    let mut c = every(2, date!(2026 - 01 - 01));
    c.start_month = Some(month_index(date!(2026 - 07 - 01)));
    c.end_month = Some(month_index(date!(2026 - 11 - 01)));
    // Phase is January, so odd months: start July, end November.
    let months: Vec<u32> = commitment_dates(&c, date!(2026 - 01 - 01), date!(2027 - 01 - 01))
        .iter()
        .map(|d| d.month() as u32)
        .collect();
    assert_eq!(months, vec![7, 9, 11]);
    c.active = false;
    assert!(commitment_dates(&c, date!(2026 - 01 - 01), date!(2027 - 01 - 01)).is_empty());
    assert_eq!(monthly_equivalent(&c, month_index(date!(2026 - 08 - 01))), 0);
}

#[test]
fn a_bill_every_few_months_on_the_31st_lands_on_the_last_day_of_short_months() {
    let mut c = every(3, date!(2026 - 11 - 01));
    c.day = 31;
    let got = commitment_dates(&c, date!(2026 - 11 - 01), date!(2027 - 09 - 01));
    assert_eq!(got, vec![date!(2026 - 11 - 30), date!(2027 - 02 - 28), date!(2027 - 05 - 31), date!(2027 - 08 - 31)]);
}

#[test]
fn a_bill_every_few_months_is_set_aside_a_share_a_month_inside_the_wants_budget() {
    let c = every(4, date!(2026 - 11 - 01));
    // 6,000 rupees every four months is 1,500 a month, and it is set aside before it falls due.
    assert_eq!(monthly_equivalent(&c, month_index(date!(2026 - 10 - 05))), 150_000);
    let mut i = fixture();
    let before = compute(&i).budget;
    i.commitments.push(c);
    let after = compute(&i).budget;
    assert_eq!(after.needs_paise, before.needs_paise, "a want is not a need");
    // The share comes out of the monthly allowance's room, not the bills.
    assert_eq!(after.spending_paise, before.spending_paise);
}

#[test]
fn amounts_that_do_not_divide_evenly_round_down_by_less_than_a_paise_a_month_per_month() {
    let mut c = every(3, date!(2026 - 11 - 01));
    c.amount = 10_000; // 100 rupees
    assert_eq!(monthly_equivalent(&c, month_index(date!(2026 - 11 - 01))), 3_333);
}

#[test]
fn freq_from_columns_reads_the_stored_cadence_and_falls_back_to_the_safe_side() {
    use super::Freq as F;
    assert_eq!(F::from_columns("monthly", 1, None), F::Monthly);
    assert_eq!(F::from_columns("yearly", 1, None), F::Yearly);
    assert_eq!(F::from_columns("monthly", 4, Some("2026-11")), F::Every { months: 4, first: 2026 * 12 + 10 });
    // A gap with no readable first month, or a nonsense gap: plain monthly (over-plans).
    assert_eq!(F::from_columns("monthly", 4, None), F::Monthly);
    assert_eq!(F::from_columns("monthly", 4, Some("soon")), F::Monthly);
    assert_eq!(F::from_columns("monthly", 12, Some("2026-11")), F::Monthly);
    assert_eq!(F::from_columns("monthly", 0, Some("2026-11")), F::Monthly);
    assert_eq!(F::from_columns("monthly", -3, Some("2026-11")), F::Monthly);
    // A yearly bill ignores a leftover gap.
    assert_eq!(F::from_columns("yearly", 4, Some("2026-11")), F::Yearly);
}

// ---------- "not this month" ----------

/// A perfume every four months, first due 10 Dec 2026 (the screenshot's case), seen on 25 Nov.
fn perfume() -> CommitmentIn {
    let mut c = every(4, date!(2026 - 12 - 01));
    c.day = 10;
    c.amount = 400_000;
    c
}

#[test]
fn putting_off_moves_the_next_one_a_month_and_the_schedule_counts_on_from_there() {
    let mut c = perfume();
    let today = date!(2026 - 11 - 25);
    let (first, waiting) = put_off(&c, today).unwrap();
    assert_eq!((year_month_of(first), year_month_of(waiting)), ("2027-01".to_string(), "2026-12".to_string()));
    c.freq = Freq::Every { months: 4, first };
    c.waiting_from = Some(waiting);
    // January, then every four months from January.
    assert_eq!(commitment_dates(&c, today, date!(2027 - 10 - 01)), vec![date!(2027 - 01 - 10), date!(2027 - 05 - 10), date!(2027 - 09 - 10)]);

    // Put off again: still waiting since December.
    let (first, waiting) = put_off(&c, today).unwrap();
    assert_eq!((year_month_of(first), year_month_of(waiting)), ("2027-02".to_string(), "2026-12".to_string()));
}

#[test]
fn nothing_more_is_set_aside_while_a_put_off_one_waits_and_its_fund_shows_full() {
    let mut c = perfume();
    c.freq = Freq::Every { months: 4, first: month_index(date!(2027 - 02 - 01)) };
    c.waiting_from = Some(month_index(date!(2026 - 12 - 01)));
    assert_eq!(monthly_equivalent(&c, month_index(date!(2026 - 11 - 01))), 100_000, "saving for it");
    assert_eq!(monthly_equivalent(&c, month_index(date!(2026 - 12 - 01))), 0, "waiting");
    assert_eq!(monthly_equivalent(&c, month_index(date!(2027 - 01 - 01))), 0, "still waiting");
    assert_eq!(monthly_equivalent(&c, month_index(date!(2027 - 02 - 01))), 100_000, "bought: saving for the next");

    let mut i = fixture();
    i.today = date!(2027 - 01 - 05);
    i.commitments = vec![c];
    let f = &bills::sinking_funds(&i)[0];
    assert_eq!((f.due, f.saved_paise), (date!(2027 - 02 - 10), 400_000));
}

#[test]
fn undo_brings_it_back_a_month_and_stops_waiting_when_it_is_back_where_it_was() {
    let mut c = perfume();
    c.freq = Freq::Every { months: 4, first: month_index(date!(2027 - 02 - 01)) };
    c.waiting_from = Some(month_index(date!(2026 - 12 - 01)));
    let (first, waiting) = undo_put_off(&c).unwrap();
    assert_eq!((year_month_of(first), waiting.map(year_month_of)), ("2027-01".to_string(), Some("2026-12".to_string())));
    c.freq = Freq::Every { months: 4, first };
    c.waiting_from = waiting;
    let (first, waiting) = undo_put_off(&c).unwrap();
    assert_eq!((year_month_of(first), waiting), ("2026-12".to_string(), None));
    c.freq = Freq::Every { months: 4, first };
    c.waiting_from = waiting;
    assert_eq!(undo_put_off(&c), None);
}

#[test]
fn only_a_payment_every_few_months_with_one_to_come_can_be_put_off() {
    let today = date!(2026 - 11 - 25);
    let monthly = commitment("SIP", 25_000, 10, Some(KOTAK), Bucket::Savings);
    assert_eq!(put_off(&monthly, today), Err(PutOffRefusal::NotEveryFewMonths));
    let mut c = perfume();
    c.end_month = Some(month_index(date!(2026 - 12 - 01)));
    assert_eq!(put_off(&c, today), Err(PutOffRefusal::PastEnd));
    c.end_month = None;
    c.active = false;
    assert_eq!(put_off(&c, today), Err(PutOffRefusal::NothingDue));
}

#[test]
fn a_new_cycle_put_off_later_starts_waiting_afresh() {
    // Put off in December, bought in January; in May the next one is put off: waiting from May.
    let mut c = perfume();
    c.freq = Freq::Every { months: 4, first: month_index(date!(2027 - 01 - 01)) };
    c.waiting_from = Some(month_index(date!(2026 - 12 - 01)));
    let (first, waiting) = put_off(&c, date!(2027 - 04 - 20)).unwrap();
    assert_eq!((year_month_of(first), year_month_of(waiting)), ("2027-06".to_string(), "2027-05".to_string()));
}

#[test]
fn putting_off_ahead_of_time_never_makes_it_look_less_saved() {
    // Clothes every 3 months, due 10 Dec, seen on 25 Nov: two thirds saved. Put off to January,
    // it is still two thirds saved; the plan sets the last third aside in November as before.
    let mut c = perfume();
    c.label = "new clothes".into();
    c.amount = 220_000;
    c.freq = Freq::Every { months: 3, first: month_index(date!(2026 - 12 - 01)) };
    let mut i = fixture();
    i.today = date!(2026 - 11 - 25);
    i.commitments = vec![c.clone()];
    let before = bills::sinking_funds(&i)[0].saved_paise;
    let (first, waiting) = put_off(&c, i.today).unwrap();
    c.freq = Freq::Every { months: 3, first };
    c.waiting_from = Some(waiting);
    i.commitments = vec![c.clone()];
    let after = &bills::sinking_funds(&i)[0];
    assert_eq!((after.due, after.saved_paise), (date!(2027 - 01 - 10), before));
    assert_eq!(before, 146_666);
    assert_eq!(monthly_equivalent(&c, month_index(date!(2026 - 11 - 01))), 73_333, "November still saves for it");
}
