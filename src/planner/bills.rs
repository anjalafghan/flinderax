//! The bills calendar: every fixed payment and card bill due in a window, and how far each
//! yearly bill's monthly set-aside has got. Pure: built from the same `PlanInput` as Safe to Spend.

use serde::Serialize;
use time::Date;

use super::{commitment_dates, cycle::add_months, Freq, PlanInput};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BillKind {
    Monthly,
    Yearly,
    /// Every few months (2-11): a perfume every four, a service every six.
    Periodic,
    CardBill,
    /// A date planned in Dates, held out of Safe to Spend.
    Date,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bill {
    pub date: Date,
    pub label: String,
    pub amount_paise: i64,
    pub kind: BillKind,
    /// Account it's paid from (card bills: the card's pay-from account).
    pub account_id: Option<String>,
    /// Card bills only: already moved to the reserve for it.
    pub covered_paise: i64,
    /// The fixed payment this is (None for card bills and dates), so it can be put off.
    pub commitment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SinkingFund {
    pub label: String,
    pub amount_paise: i64,
    pub due: Date,
    /// What a twelfth a month has put aside by today since the last time it was paid.
    pub saved_paise: i64,
}

/// Bills dated in `[from, to)`, soonest first.
pub fn calendar(input: &PlanInput, from: Date, to: Date) -> Vec<Bill> {
    let mut out = Vec::new();
    for c in &input.commitments {
        for date in commitment_dates(c, from, to) {
            out.push(Bill {
                date,
                label: c.label.clone(),
                amount_paise: c.amount,
                kind: match c.freq {
                    Freq::Monthly => BillKind::Monthly,
                    Freq::Yearly => BillKind::Yearly,
                    Freq::Every { .. } => BillKind::Periodic,
                },
                account_id: c.account_id.clone(),
                covered_paise: 0,
                commitment_id: Some(c.id.clone()),
            });
        }
    }
    for card in &input.cards {
        let bills = [(card.due_date, card.due_now, card.due_covered), (card.next_due_date, card.next_bill_est, card.next_covered)];
        for (date, amount, covered) in bills {
            if let Some(date) = date.filter(|d| *d >= from && *d < to && amount > 0) {
                out.push(Bill {
                    date,
                    label: format!("{} bill", card.name),
                    amount_paise: amount,
                    kind: BillKind::CardBill,
                    account_id: card.pay_from.clone(),
                    covered_paise: covered.clamp(0, amount),
                    commitment_id: None,
                });
            }
        }
    }
    for d in input.planned_dates.iter().filter(|d| d.date >= from && d.date < to && d.amount > 0) {
        out.push(Bill {
            date: d.date,
            label: d.label.clone(),
            amount_paise: d.amount,
            kind: BillKind::Date,
            account_id: None,
            covered_paise: 0,
            commitment_id: None,
        });
    }
    out.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| b.amount_paise.cmp(&a.amount_paise)));
    out
}

/// Each active yearly bill with a known month: its next due date and the set-aside so far.
pub fn sinking_funds(input: &PlanInput) -> Vec<SinkingFund> {
    let today = input.today;
    let mut out: Vec<SinkingFund> = input
        .commitments
        .iter()
        .filter(|c| c.active && c.freq != Freq::Monthly)
        .filter_map(|c| {
            let horizon = {
                let (y, m) = add_months(today.year(), today.month(), 13);
                Date::from_calendar_date(y, m, 1).ok()?
            };
            let due = commitment_dates(c, today, horizon).into_iter().next()?;
            let today_mi = crate::planner::month_index(today);
            // Put off: the saving still runs to the month it was first due, and is all there from then
            // until it is bought. Putting it off never makes it look less saved.
            let saved_by = match (c.freq, c.waiting_from) {
                (Freq::Every { first, .. }, Some(from)) if today_mi < first => from,
                _ => crate::planner::month_index(due),
            };
            let period = c.freq.period_months();
            // Saving runs for the `period` months before it is due, but never from before its start
            // month ("start saving from"): a bill started late has less put by when it falls due.
            let round_start = saved_by - period as i32;
            let from = c.start_month.map_or(round_start, |s| s.max(round_start));
            let months_saved = i64::from(today_mi - from).clamp(0, period);
            Some(SinkingFund { label: c.label.clone(), amount_paise: c.amount, due, saved_paise: c.amount * months_saved / period })
        })
        .collect();
    out.sort_by_key(|f| f.due);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{AllocationState, Bucket, CardIn, CommitmentIn, SettingsIn};
    use time::macros::date;

    fn commitment(label: &str, rupees: i64, day: u8, freq: Freq, due_month: Option<u8>) -> CommitmentIn {
        CommitmentIn {
            id: String::new(),
            waiting_from: None,
            label: label.into(),
            amount: rupees * 100,
            day,
            account_id: Some("kotak".into()),
            active: true,
            bucket: Bucket::Needs,
            freq,
            due_month,
            to_account_id: None,
            start_month: None,
            end_month: None,
        }
    }

    fn input(today: Date, commitments: Vec<CommitmentIn>, cards: Vec<CardIn>) -> PlanInput {
        PlanInput {
            today,
            accounts: vec![],
            incomes: vec![],
            commitments,
            cards,
            settings: SettingsIn { spending: None, spending_bps: None, allocation_day: 10, savings_floor: None, recovery_months: 3 },
            allocation: AllocationState::default(),
            planned_dates: vec![],
        }
    }

    fn card(due: Date, due_now: i64, covered: i64) -> CardIn {
        CardIn {
            card_id: "c".into(),
            name: "Millennia".into(),
            due_now,
            due_date: Some(due),
            next_bill_est: 0,
            next_due_date: None,
            last_updated: None,
            pay_from: Some("kotak".into()),
            due_covered: covered,
            next_covered: 0,
            spent_this_period: None,
            tracked_since: None,
        }
    }

    #[test]
    fn calendar_lists_fixed_and_card_bills_in_order() {
        let today = date!(2026 - 11 - 08);
        let i = input(
            today,
            vec![commitment("Rent", 15_000, 1, Freq::Monthly, None), commitment("Phone (yearly)", 12_000, 12, Freq::Yearly, Some(11))],
            vec![card(date!(2026 - 11 - 20), 2_400_000, 1_000_000)],
        );
        let c = calendar(&i, today, date!(2026 - 12 - 08));
        let labels: Vec<&str> = c.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(labels, ["Phone (yearly)", "Millennia bill", "Rent"]);
        assert_eq!(c[1].covered_paise, 1_000_000);
        assert_eq!(c[0].kind, BillKind::Yearly);
    }

    #[test]
    fn sinking_fund_grows_a_twelfth_a_month() {
        // Due in March, today is November: 4 months to go, so 8 twelfths are set aside.
        let i = input(date!(2026 - 11 - 08), vec![commitment("Gym", 12_000, 5, Freq::Yearly, Some(3))], vec![]);
        let f = sinking_funds(&i);
        assert_eq!((f[0].due, f[0].saved_paise), (date!(2027 - 03 - 05), 800_000));
    }

    #[test]
    fn sinking_fund_counts_only_from_the_month_saving_starts() {
        // Due in March, saving started in January: by today (February) only January is put by.
        let mut gym = commitment("Gym", 12_000, 5, Freq::Yearly, Some(3));
        gym.start_month = Some(crate::planner::month_index(date!(2027 - 01 - 01)));
        let f = sinking_funds(&input(date!(2027 - 02 - 10), vec![gym.clone()], vec![]));
        assert_eq!((f[0].due, f[0].saved_paise), (date!(2027 - 03 - 05), 100_000));
        // Before the start month nothing is saved yet.
        let f = sinking_funds(&input(date!(2026 - 12 - 10), vec![gym], vec![]));
        assert_eq!(f[0].saved_paise, 0);
    }

    #[test]
    fn planned_dates_sit_in_the_same_calendar_as_bills() {
        let today = date!(2026 - 10 - 05);
        let mut i = input(today, vec![commitment("Rent", 10_000, 8, Freq::Monthly, None)], vec![]);
        i.planned_dates = vec![
            crate::planner::PlannedDateIn { date: date!(2026 - 10 - 07), label: "Dinner".into(), amount: 240_000 },
            crate::planner::PlannedDateIn { date: date!(2027 - 03 - 01), label: "Too far".into(), amount: 100 },
        ];
        let bills = calendar(&i, today, date!(2026 - 11 - 05));
        let kinds: Vec<_> = bills.iter().map(|b| (b.label.as_str(), b.kind.clone())).collect();
        assert_eq!(kinds, [("Dinner", BillKind::Date), ("Rent", BillKind::Monthly)]);
    }

    #[test]
    fn a_bill_every_few_months_is_periodic_on_the_calendar_with_a_fund_that_fills_up() {
        let perfume = CommitmentIn {
            id: String::new(),
            waiting_from: None,
            label: "Perfume".into(),
            amount: 600_000,
            day: 5,
            account_id: None,
            active: true,
            bucket: Bucket::Wants,
            freq: Freq::Every { months: 4, first: crate::planner::month_index(date!(2027 - 03 - 01)) },
            due_month: None,
            to_account_id: None,
            start_month: None,
            end_month: None,
        };
        let i = input(date!(2026 - 12 - 08), vec![perfume], vec![]);
        let bills = calendar(&i, date!(2027 - 03 - 01), date!(2027 - 04 - 01));
        assert_eq!(bills.len(), 1);
        assert_eq!(bills[0].kind, BillKind::Periodic);
        // Due 5 March; December's share is in, so a quarter of it.
        let funds = sinking_funds(&i);
        assert_eq!(funds.len(), 1);
        assert_eq!((funds[0].due, funds[0].saved_paise), (date!(2027 - 03 - 05), 150_000));
    }
}
