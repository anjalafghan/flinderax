use super::*;
use time::macros::date;

fn acct(id: &str, name: &str, rupees: i64, locked: bool, updated: Date) -> AccountIn {
    AccountIn { id: id.into(), name: name.into(), balance: rupees * 100, locked, updated: Some(updated) }
}

fn commitment(label: &str, rupees: i64, day: u8, account: &str) -> CommitmentIn {
    CommitmentIn {
        label: label.into(),
        amount: rupees * 100,
        day,
        account_id: Some(account.into()),
        active: true,
        start_month: None,
        end_month: None,
    }
}

fn salary() -> IncomeIn {
    IncomeIn { label: "Salary".into(), amount: 7_549_900, day: 30, account_id: Some("hdfc".into()) }
}

/// 5 Oct 2026 numbers from the planning session. Not given in the brief, so assumed here:
/// 7420 statement 23rd / due 13th; 2281 statement 1st / due 20th with nothing billed yet;
/// the two ₹30k SIPs are separate commitments paid from HDFC; Ketto is left out (no amount).
fn fixture() -> PlanInput {
    let today = date!(2026 - 10 - 05);
    // 7420: due 0 / outstanding 61,488 / EMI 3,815 (14,739 left) / pending 1,285
    let c7420_next = 6_148_800 - 0 - 1_473_900 + 381_500 + 128_500;
    PlanInput {
        today,
        accounts: vec![
            acct("hdfc", "HDFC", 43_016, false, today),
            acct("kotak", "Kotak", 69_144, false, today),
            acct("fd", "FD", 100_000, true, today),
        ],
        incomes: vec![salary()],
        commitments: vec![
            commitment("SIP 1", 30_000, 10, "hdfc"),
            commitment("SIP 2", 30_000, 10, "hdfc"),
            commitment("House", 4_000, 1, "hdfc"),
        ],
        cards: vec![
            CardIn {
                name: "7420".into(),
                due_now: 0,
                due_date: Some(date!(2026 - 10 - 13)),
                next_bill_est: c7420_next,
                next_due_date: Some(date!(2026 - 11 - 13)),
                last_updated: Some(today),
            },
            CardIn {
                name: "2281".into(),
                due_now: 0,
                due_date: Some(date!(2026 - 10 - 20)),
                next_bill_est: 497_500,
                next_due_date: Some(date!(2026 - 11 - 20)),
                last_updated: Some(today),
            },
        ],
        settings: SettingsIn { cushion: 500_000, next_period_living: 1_800_000 },
    }
}

#[test]
fn card_breakdown_for_7420() {
    let a = card_amounts(CardNumbers {
        total_due: 0,
        outstanding: 6_148_800,
        emi_monthly: 381_500,
        emi_remaining: 1_473_900,
        pending_charges: 128_500,
    });
    // outstanding - due - emi_remaining + emi_monthly + pending
    assert_eq!(a.next_bill_est_paise, 5_184_900);
    assert_eq!(a.loan_not_due_paise, 1_473_900 - 381_500);
    assert_eq!(a.due_now_paise, 0);
    // Without the pending Zomato charge (if the bank already counts it): 61,488 - 14,739 + 3,815
    let b = card_amounts(CardNumbers {
        total_due: 0,
        outstanding: 6_148_800,
        emi_monthly: 381_500,
        emi_remaining: 1_473_900,
        pending_charges: 0,
    });
    assert_eq!(b.next_bill_est_paise, 5_056_400);
}

#[test]
fn card_with_credit_balance_never_goes_negative() {
    let a = card_amounts(CardNumbers { outstanding: -50_000, ..Default::default() });
    assert_eq!(a.next_bill_est_paise, 0);
}

#[test]
fn card_next_bill_estimate_follows_the_breakdown_rule() {
    let s = compute(&fixture());
    let bill = s.upcoming.iter().find(|u| u.label == "7420 next bill (est.)").unwrap();
    assert_eq!(bill.amount_paise, 5_184_900);
    assert_eq!(bill.date, date!(2026 - 11 - 13));
}

#[test]
fn fixture_horizon_and_labels() {
    let s = compute(&fixture());
    assert_eq!(s.until, date!(2026 - 10 - 30));
    assert_eq!(s.next_income.as_ref().unwrap().label, "November money");
    assert_eq!(s.period_label.as_deref(), Some("October money"));
}

#[test]
fn fixture_two_sips_overdraw_hdfc_so_kotak_tops_it_up() {
    let s = compute(&fixture());
    let a = &s.actions[0];
    // HDFC 43,016 vs 60,000 of SIPs on the 10th -> 16,984 short.
    assert_eq!(a.text, "Move ₹16,984 Kotak → HDFC before 10 Oct (SIP 2)");
    assert_eq!(a.severity, "warn");
    // Salary lands in HDFC on the 30th, so the 10 Nov SIPs need no second top-up.
    assert_eq!(s.actions.len(), 1);
}

#[test]
fn fixture_over_committed_month_is_red() {
    // 112,160 + 75,499 - 120,000 (SIPs x2 months) - 4,000 - 51,849 - 4,975 - 18,000 = -11,165
    let s = compute(&fixture());
    assert_eq!(s.lowest_point.amount_paise, -1_116_500);
    assert_eq!(s.lowest_point.date, date!(2026 - 11 - 29));
    assert_eq!(s.safe_to_spend_paise, 0);
    assert_eq!(s.status, Status::Red);
}

#[test]
fn healthy_month_is_green_and_split_per_week() {
    let mut i = fixture();
    i.commitments = vec![commitment("SIP", 10_000, 10, "hdfc"), commitment("House", 4_000, 1, "hdfc")];
    let s = compute(&i);
    // Lowest point is the very end: 112,160 + 75,499 - 20,000 (SIP x2 months) - 4,000
    // - 51,849 - 4,975 - 18,000 = 88,835, so safe = 83,835.
    assert_eq!(s.lowest_point.amount_paise, 8_883_500);
    assert!(s.safe_to_spend_paise > 0);
    assert_eq!(s.safe_to_spend_paise, s.lowest_point.amount_paise - 500_000);
    assert_eq!(s.per_week_paise, s.safe_to_spend_paise / 4); // 25 days -> 4 weeks
    assert_eq!(s.status, Status::Green);
    assert!(s.actions.is_empty());
}

#[test]
fn yellow_when_positive_but_under_two_thousand_a_week() {
    let mut i = fixture();
    i.commitments.clear();
    i.cards.clear();
    i.accounts = vec![acct("hdfc", "HDFC", 13_000, false, i.today)];
    i.incomes = vec![IncomeIn { amount: 0, ..salary() }];
    i.settings.next_period_living = 0;
    // 13,000 - 5,000 cushion = 8,000 over 4 weeks = exactly 2,000/week -> green boundary.
    i.accounts[0].balance = 1_300_000;
    let s = compute(&i);
    assert_eq!(s.safe_to_spend_paise, 800_000);
    assert_eq!(s.per_week_paise, 200_000);
    assert_eq!(s.status, Status::Green);
    i.accounts[0].balance = 1_200_000;
    let s = compute(&i);
    assert_eq!(s.per_week_paise, 175_000);
    assert_eq!(s.status, Status::Yellow);
}

#[test]
fn commitment_before_start_month_is_skipped_and_after_end_month_too() {
    let mut i = fixture();
    i.commitments = vec![CommitmentIn {
        start_month: parse_year_month("2026-12"),
        ..commitment("RD", 10_000, 10, "hdfc")
    }];
    assert!(compute(&i).upcoming.iter().all(|u| u.label != "RD"));
    i.commitments = vec![CommitmentIn {
        end_month: parse_year_month("2026-10"),
        ..commitment("RD", 10_000, 10, "hdfc")
    }];
    let ups: Vec<_> = compute(&i).upcoming.into_iter().filter(|u| u.label == "RD").collect();
    assert_eq!(ups.len(), 1);
    assert_eq!(ups[0].date, date!(2026 - 10 - 10));
}

#[test]
fn stale_balances_and_snapshots_are_flagged() {
    let mut i = fixture();
    i.accounts[1].updated = Some(date!(2026 - 09 - 25));
    i.cards[0].last_updated = Some(date!(2026 - 09 - 20));
    let s = compute(&i);
    assert_eq!(s.stale.len(), 2);
    assert!(s.stale.iter().any(|x| x.what == "Kotak balance" && x.days_old == 10));
    assert!(s.stale.iter().any(|x| x.what == "7420 snapshot" && x.days_old == 15));
    assert!(s.actions.iter().any(|a| a.severity == "info"));
}

#[test]
fn locked_fd_never_counts_and_is_never_stale() {
    let mut i = fixture();
    i.accounts[2].updated = Some(date!(2025 - 01 - 01));
    let s = compute(&i);
    assert!(s.stale.is_empty());
}

#[test]
fn no_income_falls_back_to_a_thirty_day_horizon() {
    let mut i = fixture();
    i.incomes.clear();
    let s = compute(&i);
    assert_eq!(s.until, date!(2026 - 11 - 04));
    assert!(s.next_income.is_none());
}

#[test]
fn overdue_card_bill_lands_today() {
    let mut i = fixture();
    i.cards[0].due_now = 100_000;
    i.cards[0].due_date = Some(date!(2026 - 10 - 01));
    let s = compute(&i);
    let due = s.upcoming.iter().find(|u| u.kind == "card_due").unwrap();
    assert_eq!(due.date, i.today);
}

#[test]
fn rupee_formatting_uses_indian_grouping() {
    assert_eq!(format_inr(0), "₹0");
    assert_eq!(format_inr(99_900), "₹999");
    assert_eq!(format_inr(100_000), "₹1,000");
    assert_eq!(format_inr(12_345_600), "₹1,23,456");
    assert_eq!(format_inr(754_990_000), "₹75,49,900");
    assert_eq!(format_inr(-150), "-₹1.50");
}

/// Same data, later "today": recurring items must roll forward with no new input.
#[test]
fn time_travel_recurring_items_roll_forward() {
    for (today, until, label, period) in [
        (date!(2026 - 11 - 02), date!(2026 - 11 - 30), "December money", "November money"),
        (date!(2026 - 12 - 31), date!(2027 - 01 - 30), "February money", "January money"),
        (date!(2027 - 01 - 05), date!(2027 - 01 - 30), "February money", "January money"),
        (date!(2027 - 02 - 05), date!(2027 - 02 - 28), "March money", "February money"), // 30th clamps to 28th
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
        assert_eq!(s.until, until, "until @ {today}");
        assert_eq!(s.next_income.as_ref().unwrap().label, label, "label @ {today}");
        assert_eq!(s.period_label.as_deref(), Some(period), "period @ {today}");
        assert!(s.upcoming.iter().any(|u| u.label == "SIP 1"), "SIP recurs @ {today}");
    }
}

/// Stale inputs are the real risk over time: nothing updated for 3 months must be loudly flagged.
#[test]
fn three_months_without_updates_is_flagged_everywhere() {
    let mut i = fixture();
    i.today = date!(2027 - 01 - 05);
    let s = compute(&i);
    assert_eq!(s.stale.len(), 4); // HDFC, Kotak, 7420, 2281 (locked FD excluded)
    assert!(s.stale.iter().all(|x| x.days_old >= 90));
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
}
