//! The year ahead: twelve months of income, fixed payments and the large one-off outflows (premiums,
//! tax, wedding payments, maturities), with the money you would hold at the end of each month.
//! Pure: the handler gathers the pieces and passes `today`. All money is integer paise.

use serde::Serialize;
use time::{Date, Month};

use super::{
    commitment_dates,
    cycle::{add_months, day_in_month_offset},
    occurrences, CommitmentIn, IncomeIn,
};

/// How many months the view covers, this one included.
pub const MONTHS: usize = 12;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Out,
    In,
}

/// A dated amount that is not a fixed payment: a premium, an advance-tax instalment, a wedding
/// milestone, an FD maturing. `yearly` repeats it on the same day each year.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lumpy {
    pub on: Date,
    pub label: String,
    pub amount_paise: i64,
    pub direction: Direction,
    pub yearly: bool,
    /// Where it comes from, e.g. "premium", "tax", "wedding", "maturity", "other".
    pub kind: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct LumpyOut {
    pub on: Date,
    pub label: String,
    pub amount_paise: i64,
    pub direction: Direction,
    pub kind: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct MonthRow {
    /// "YYYY-MM".
    pub month: String,
    pub income_paise: i64,
    /// Fixed payments falling in the month (monthly ones and yearly ones due then).
    pub fixed_paise: i64,
    pub lumpy_out_paise: i64,
    pub lumpy_in_paise: i64,
    pub net_paise: i64,
    /// What you would hold at the end of the month if nothing else were spent.
    pub closing_paise: i64,
    /// The month ends below zero.
    pub short: bool,
    pub lumpy: Vec<LumpyOut>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct YearView {
    pub months: Vec<MonthRow>,
    /// First month that ends below zero, if any.
    pub first_short_month: Option<String>,
    /// The lowest closing balance over the year.
    pub lowest_paise: i64,
    pub total_lumpy_out_paise: i64,
}

fn first_of_month(y: i32, m: Month) -> Date {
    Date::from_calendar_date(y, m, 1).expect("day 1 exists in every month")
}

/// Dates in `[from, to)` on which `l` falls (a yearly one repeats; a Feb 29 falls on Feb 28 in
/// other years).
fn lumpy_dates(l: &Lumpy, from: Date, to: Date) -> Vec<Date> {
    if !l.yearly {
        return if l.on >= from && l.on < to { vec![l.on] } else { Vec::new() };
    }
    let mut out = Vec::new();
    for year in from.year() - 1..=to.year() {
        let d = Date::from_calendar_date(year, l.on.month(), l.on.day())
            .or_else(|_| Date::from_calendar_date(year, l.on.month(), 28));
        if let Ok(d) = d {
            if d >= l.on && d >= from && d < to {
                out.push(d);
            }
        }
    }
    out
}

/// Twelve months from the month of `today`. The first month counts only from today on. `opening` is
/// what you hold now. Fixed payments of `commitments` and `incomes` are expanded by their day of month.
pub fn year_view(today: Date, opening_paise: i64, incomes: &[IncomeIn], commitments: &[CommitmentIn], lumpy: &[Lumpy]) -> YearView {
    let mut months = Vec::with_capacity(MONTHS);
    let mut balance = opening_paise;
    let mut lowest = opening_paise;
    let mut first_short = None;
    let mut total_out = 0i64;

    for i in 0..MONTHS {
        let (y, m) = add_months(today.year(), today.month(), i as i32);
        let start = if i == 0 { today } else { first_of_month(y, m) };
        let (ny, nm) = add_months(today.year(), today.month(), i as i32 + 1);
        let end = first_of_month(ny, nm);

        let income: i64 = incomes.iter().map(|inc| inc.amount.saturating_mul(occurrences(inc.day, start, end).len() as i64)).sum();
        let fixed: i64 = commitments
            .iter()
            .map(|c| c.amount.saturating_mul(commitment_dates(c, start, end).len() as i64))
            .sum();

        let mut items: Vec<LumpyOut> = Vec::new();
        for l in lumpy {
            for on in lumpy_dates(l, start, end) {
                items.push(LumpyOut { on, label: l.label.clone(), amount_paise: l.amount_paise, direction: l.direction, kind: l.kind.clone() });
            }
        }
        items.sort_by(|a, b| a.on.cmp(&b.on).then_with(|| a.label.cmp(&b.label)));
        let out: i64 = items.iter().filter(|x| x.direction == Direction::Out).map(|x| x.amount_paise).sum();
        let inn: i64 = items.iter().filter(|x| x.direction == Direction::In).map(|x| x.amount_paise).sum();

        let net = income + inn - fixed - out;
        balance = balance.saturating_add(net);
        lowest = lowest.min(balance);
        total_out += out;
        let short = balance < 0;
        let month = format!("{:04}-{:02}", y, m as u8);
        if short && first_short.is_none() {
            first_short = Some(month.clone());
        }
        months.push(MonthRow {
            month,
            income_paise: income,
            fixed_paise: fixed,
            lumpy_out_paise: out,
            lumpy_in_paise: inn,
            net_paise: net,
            closing_paise: balance,
            short,
            lumpy: items,
        });
    }
    YearView { months, first_short_month: first_short, lowest_paise: lowest, total_lumpy_out_paise: total_out }
}

/// What to set aside each month, from now until `on`, for a one-off outflow (rounded up). 0 when
/// it is today or past.
pub fn set_aside_monthly(amount_paise: i64, today: Date, on: Date) -> i64 {
    if amount_paise <= 0 || on <= today {
        return 0;
    }
    let months = ((on.year() - today.year()) as i64 * 12 + (on.month() as i64 - today.month() as i64)).max(1);
    (amount_paise - 1) / months + 1
}

/// The next date a yearly lumpy item falls on or after `today`.
pub fn next_occurrence(l: &Lumpy, today: Date) -> Option<Date> {
    if !l.yearly {
        return (l.on >= today).then_some(l.on);
    }
    let end = day_in_month_offset(today, 13, 1);
    lumpy_dates(l, today, end).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{Bucket, Freq};
    use time::macros::date;

    const TODAY: Date = date!(2026 - 11 - 16);

    fn income(amount: i64, day: u8) -> IncomeIn {
        IncomeIn { label: "Pay".into(), amount, day, account_id: None }
    }

    fn commitment(amount: i64, day: u8, freq: Freq, due_month: Option<u8>) -> CommitmentIn {
        CommitmentIn {
            label: "Rent".into(),
            amount,
            day,
            account_id: None,
            active: true,
            bucket: Bucket::Needs,
            freq,
            due_month,
            to_account_id: None,
            start_month: None,
            end_month: None,
        }
    }

    fn lumpy(on: Date, amount: i64, yearly: bool) -> Lumpy {
        Lumpy { on, label: "Premium".into(), amount_paise: amount, direction: Direction::Out, yearly, kind: "premium".into() }
    }

    #[test]
    fn covers_twelve_months_starting_this_one() {
        let v = year_view(TODAY, 0, &[], &[], &[]);
        assert_eq!(v.months.len(), 12);
        assert_eq!(v.months[0].month, "2026-11");
        assert_eq!(v.months[11].month, "2027-10");
    }

    #[test]
    fn the_first_month_counts_only_from_today() {
        // Pay on the 1st has passed this month; the next one is 1 Dec.
        let v = year_view(TODAY, 0, &[income(100_000, 1)], &[], &[]);
        assert_eq!(v.months[0].income_paise, 0);
        assert_eq!(v.months[1].income_paise, 100_000);
    }

    #[test]
    fn a_large_one_off_shows_in_its_month_and_flags_a_shortfall() {
        let v = year_view(TODAY, 100_000, &[income(50_000, 1)], &[], &[lumpy(date!(2027 - 03 - 10), 400_000, false)]);
        let march = v.months.iter().find(|m| m.month == "2027-03").unwrap();
        assert_eq!(march.lumpy_out_paise, 400_000);
        assert_eq!(march.lumpy.len(), 1);
        assert!(march.short, "100k + 4 months of 50k is 300k, below the 400k premium");
        assert_eq!(v.first_short_month.as_deref(), Some("2027-03"));
        assert_eq!(v.total_lumpy_out_paise, 400_000);
    }

    #[test]
    fn closing_balance_carries_forward() {
        let v = year_view(TODAY, 10_000, &[income(5_000, 1)], &[commitment(2_000, 5, Freq::Monthly, None)], &[]);
        assert_eq!(v.months[0].closing_paise, 10_000, "1 and 5 Nov already passed");
        assert_eq!(v.months[1].closing_paise, 10_000 + 3_000);
        assert_eq!(v.months[2].closing_paise, 10_000 + 6_000);
        assert_eq!(v.lowest_paise, 10_000);
    }

    #[test]
    fn a_yearly_commitment_only_lands_in_its_due_month() {
        let v = year_view(TODAY, 0, &[], &[commitment(120_000, 10, Freq::Yearly, Some(3))], &[]);
        let hit: Vec<&str> = v.months.iter().filter(|m| m.fixed_paise > 0).map(|m| m.month.as_str()).collect();
        assert_eq!(hit, vec!["2027-03"]);
    }

    #[test]
    fn a_yearly_lumpy_item_repeats_each_year_but_only_once_in_the_window() {
        let l = lumpy(date!(2025 - 12 - 20), 30_000, true);
        let v = year_view(TODAY, 0, &[], &[], &[l]);
        let hits: Vec<&str> = v.months.iter().filter(|m| m.lumpy_out_paise > 0).map(|m| m.month.as_str()).collect();
        assert_eq!(hits, vec!["2026-12"]);
    }

    #[test]
    fn a_yearly_item_not_started_yet_waits_for_its_first_date() {
        let l = lumpy(date!(2028 - 01 - 15), 30_000, true);
        let v = year_view(TODAY, 0, &[], &[], &[l]);
        assert_eq!(v.total_lumpy_out_paise, 0);
    }

    #[test]
    fn money_coming_in_lifts_the_month() {
        let mut l = lumpy(date!(2027 - 02 - 01), 500_000, false);
        l.direction = Direction::In;
        l.kind = "maturity".into();
        let v = year_view(TODAY, 0, &[], &[], &[l]);
        let feb = v.months.iter().find(|m| m.month == "2027-02").unwrap();
        assert_eq!(feb.lumpy_in_paise, 500_000);
        assert_eq!(feb.net_paise, 500_000);
        assert_eq!(v.total_lumpy_out_paise, 0, "an inflow is not an outflow");
    }

    #[test]
    fn a_past_one_off_is_ignored() {
        let v = year_view(TODAY, 0, &[], &[], &[lumpy(date!(2026 - 01 - 01), 99, false)]);
        assert_eq!(v.total_lumpy_out_paise, 0);
    }

    #[test]
    fn set_aside_rounds_up_and_counts_months() {
        assert_eq!(set_aside_monthly(100_001, TODAY, date!(2027 - 03 - 10)), 25_001, "4 months");
        assert_eq!(set_aside_monthly(100_000, TODAY, date!(2026 - 11 - 30)), 100_000, "this month: all of it");
        assert_eq!(set_aside_monthly(100_000, TODAY, date!(2026 - 11 - 01)), 0, "already past");
        assert_eq!(set_aside_monthly(0, TODAY, date!(2027 - 03 - 10)), 0);
    }

    #[test]
    fn next_occurrence_of_a_yearly_item() {
        let l = lumpy(date!(2024 - 06 - 05), 1, true);
        assert_eq!(next_occurrence(&l, TODAY), Some(date!(2027 - 06 - 05)));
        let once = lumpy(date!(2026 - 01 - 01), 1, false);
        assert_eq!(next_occurrence(&once, TODAY), None);
    }

    #[test]
    fn feb_29_falls_on_the_28th_in_other_years() {
        let l = lumpy(date!(2024 - 02 - 29), 1, true);
        assert_eq!(next_occurrence(&l, TODAY), Some(date!(2027 - 02 - 28)));
    }
}
