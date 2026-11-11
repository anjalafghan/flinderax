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
    CardBill,
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
                kind: if c.freq == Freq::Yearly { BillKind::Yearly } else { BillKind::Monthly },
                account_id: c.account_id.clone(),
                covered_paise: 0,
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
                });
            }
        }
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
        .filter(|c| c.active && c.freq == Freq::Yearly)
        .filter_map(|c| {
            let horizon = {
                let (y, m) = add_months(today.year(), today.month(), 13);
                Date::from_calendar_date(y, m, 1).ok()?
            };
            let due = commitment_dates(c, today, horizon).into_iter().next()?;
            let months_left = (due.year() - today.year()) * 12 + (due.month() as i32 - today.month() as i32);
            let months_saved = (12 - months_left).clamp(0, 12) as i64;
            Some(SinkingFund { label: c.label.clone(), amount_paise: c.amount, due, saved_paise: c.amount * months_saved / 12 })
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
}
