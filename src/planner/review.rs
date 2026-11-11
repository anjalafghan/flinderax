//! Month close: a plain-language look back at one calendar month. Pure: the handler loads the
//! facts, this turns them into numbers and sentences. Money is INTEGER paise.

use serde::Serialize;
use time::{Date, Month};

use super::{cycle::add_months, format_inr, month_index, parse_year_month};

pub const NO_TRANSACTIONS: &str = "No transactions recorded this month. Forward bank alerts or import a statement.";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CategoryAmount {
    pub category: String,
    pub amount_paise: i64,
}

/// Joy's date spending for the month (see `handlers::joy`), in paise.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DatesFacts {
    pub spent_paise: i64,
    pub budget_paise: Option<i64>,
    pub your_share_paise: i64,
    /// Your own debits labelled as dates in this month's transactions: what the bank side saw.
    pub in_accounts_paise: i64,
}

#[derive(Debug, Clone)]
pub struct MonthFacts {
    pub has_transactions: bool,
    pub income_paise: i64,
    pub spent_paise: i64,
    pub allowance_paise: i64,
    pub top_categories: Vec<CategoryAmount>,
    pub savings_rate_bps: Option<i64>,
    pub dates: Option<DatesFacts>,
    pub net_worth_change_paise: Option<i64>,
}

/// `'2026-10'` -> (2026, October).
pub fn parse_month(s: &str) -> Option<(i32, Month)> {
    let idx = parse_year_month(s)?;
    let month = Month::try_from((idx.rem_euclid(12) + 1) as u8).ok()?;
    Some((idx.div_euclid(12), month))
}

/// First day of the month and of the one after it; `None` for an absurd year.
pub fn month_range(year: i32, month: Month) -> Option<(Date, Date)> {
    let (ny, nm) = add_months(year, month, 1);
    Some((Date::from_calendar_date(year, month, 1).ok()?, Date::from_calendar_date(ny, nm, 1).ok()?))
}

/// `YYYY-MM` of the month before `today`'s.
pub fn last_month(today: Date) -> String {
    let (y, m) = add_months(today.year(), today.month(), -1);
    month_key(y, m)
}

pub fn month_key(year: i32, month: Month) -> String {
    format!("{year:04}-{:02}", month as u8)
}

/// Whether `year`/`month` is after `today`'s month.
pub fn is_future(year: i32, month: Month, today: Date) -> bool {
    year * 12 + (month as i32 - 1) > month_index(today)
}

/// Share of income left after spending and needs, in basis points; negative when overspent.
/// `None` without income.
pub fn savings_rate_bps(income: i64, spent: i64, needs: i64) -> Option<i64> {
    (income > 0).then(|| (income - spent - needs) * 10_000 / income)
}

/// The biggest `n` categories, largest first (ties by name).
pub fn top_categories(totals: &[(String, i64)], n: usize) -> Vec<CategoryAmount> {
    let mut v: Vec<CategoryAmount> = totals
        .iter()
        .filter(|(_, a)| *a > 0)
        .map(|(c, a)| CategoryAmount { category: c.clone(), amount_paise: *a })
        .collect();
    v.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise).then_with(|| a.category.cmp(&b.category)));
    v.truncate(n);
    v
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

/// 3 to 5 one-liners for someone who won't read a table.
pub fn sentences(f: &MonthFacts) -> Vec<String> {
    if !f.has_transactions {
        return vec![NO_TRANSACTIONS.to_string()];
    }
    let mut out = Vec::new();
    out.push(if f.allowance_paise <= 0 {
        format!("You spent {}.", format_inr(f.spent_paise))
    } else if f.spent_paise > f.allowance_paise {
        format!(
            "You spent {}, which is {} over your {} allowance.",
            format_inr(f.spent_paise),
            format_inr(f.spent_paise - f.allowance_paise),
            format_inr(f.allowance_paise)
        )
    } else {
        format!("You spent {} of {}. Within your allowance, well done.", format_inr(f.spent_paise), format_inr(f.allowance_paise))
    });
    if let Some(top) = f.top_categories.first() {
        out.push(format!("{} was your biggest category at {}.", capitalise(&top.category), format_inr(top.amount_paise)));
    }
    out.push(match f.savings_rate_bps {
        Some(bps) if bps >= 1_000 => format!("You saved {}% of your income. Nicely done.", (bps + 50) / 100),
        Some(bps) if bps >= 0 => format!("You saved {}% of your income.", (bps + 50) / 100),
        Some(_) => "You spent more than you earned this month. Small steps next month will turn it around.".to_string(),
        None => "No income was recorded this month.".to_string(),
    });
    if let Some(change) = f.net_worth_change_paise {
        out.push(match change {
            0 => "Net worth stayed the same.".to_string(),
            c if c > 0 => format!("Net worth went up {}. Lovely.", format_inr(c)),
            c => format!("Net worth went down {}. One month is not a trend.", format_inr(-c)),
        });
    }
    if let Some(d) = f.dates.as_ref().filter(|d| d.spent_paise > 0) {
        let seen = if d.in_accounts_paise > 0 {
            format!("Your accounts show {} of it labelled as dates.", format_inr(d.in_accounts_paise))
        } else {
            "None of it is labelled as dates in your accounts.".to_string()
        };
        out.push(format!("Dates took {}, of which your share was {}. {seen}", format_inr(d.spent_paise), format_inr(d.your_share_paise)));
    }
    out.truncate(5);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn facts() -> MonthFacts {
        MonthFacts {
            has_transactions: true,
            income_paise: 100_000_00,
            spent_paise: 18_200_00,
            allowance_paise: 20_000_00,
            top_categories: vec![CategoryAmount { category: "food".into(), amount_paise: 6_100_00 }],
            savings_rate_bps: Some(3_100),
            dates: None,
            net_worth_change_paise: Some(12_000_00),
        }
    }

    #[test]
    fn the_usual_month_reads_naturally() {
        assert_eq!(
            sentences(&facts()),
            [
                "You spent ₹18,200 of ₹20,000. Within your allowance, well done.",
                "Food was your biggest category at ₹6,100.",
                "You saved 31% of your income. Nicely done.",
                "Net worth went up ₹12,000. Lovely.",
            ]
        );
    }

    #[test]
    fn over_budget_overspent_and_down() {
        let f = MonthFacts {
            spent_paise: 22_000_00,
            savings_rate_bps: Some(-500),
            net_worth_change_paise: Some(-3_000_00),
            dates: Some(DatesFacts { spent_paise: 3_000_00, budget_paise: Some(20_000_00), your_share_paise: 2_100_00, in_accounts_paise: 1_500_00 }),
            ..facts()
        };
        let s = sentences(&f);
        assert_eq!(s[0], "You spent ₹22,000, which is ₹2,000 over your ₹20,000 allowance.");
        assert_eq!(s[2], "You spent more than you earned this month. Small steps next month will turn it around.");
        assert_eq!(s[3], "Net worth went down ₹3,000. One month is not a trend.");
        assert_eq!(s.len(), 5);
        assert_eq!(s[4], "Dates took ₹3,000, of which your share was ₹2,100. Your accounts show ₹1,500 of it labelled as dates.");
    }

    #[test]
    fn no_income_and_no_net_worth_still_gives_three() {
        let f = MonthFacts { savings_rate_bps: None, net_worth_change_paise: None, income_paise: 0, ..facts() };
        let s = sentences(&f);
        assert_eq!(s.len(), 3);
        assert_eq!(s[2], "No income was recorded this month.");
    }

    #[test]
    fn empty_month() {
        let f = MonthFacts { has_transactions: false, spent_paise: 0, top_categories: vec![], ..facts() };
        assert_eq!(sentences(&f), [NO_TRANSACTIONS]);
    }

    #[test]
    fn savings_rate() {
        assert_eq!(savings_rate_bps(100_000, 40_000, 29_000), Some(3_100));
        assert_eq!(savings_rate_bps(100_000, 80_000, 30_000), Some(-1_000));
        assert_eq!(savings_rate_bps(0, 1, 1), None);
    }

    #[test]
    fn top_three_largest_first() {
        let t = vec![("food".to_string(), 5), ("fuel".to_string(), 9), ("rent".to_string(), 5), ("bills".to_string(), 1), ("zero".to_string(), 0)];
        let top: Vec<_> = top_categories(&t, 3).into_iter().map(|c| c.category).collect();
        assert_eq!(top, ["fuel", "food", "rent"]);
    }

    #[test]
    fn month_maths() {
        assert_eq!(parse_month("2026-10"), Some((2026, Month::October)));
        assert_eq!(parse_month("2026-13"), None);
        assert_eq!(parse_month("nope"), None);
        assert_eq!(month_range(2026, Month::December), Some((date!(2026 - 12 - 01), date!(2027 - 01 - 01))));
        assert_eq!(last_month(date!(2026 - 01 - 15)), "2025-12");
        assert_eq!(last_month(date!(2026 - 11 - 08)), "2026-10");
        assert!(is_future(2026, Month::December, date!(2026 - 11 - 08)));
        assert!(!is_future(2026, Month::November, date!(2026 - 11 - 08)));
    }
}
