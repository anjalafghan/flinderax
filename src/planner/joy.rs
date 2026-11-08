//! Joy (the shared date jar) link: your share of the date budget, and which bank-alert
//! transactions were dates. Pure: the handler fetches Joy and the transactions.

use time::Date;

/// How far apart Joy's date and the bank's date may be (a late-night bill posts the next day).
pub const MATCH_DAYS: i64 = 1;

/// Your share of `amount` when the partner covers `partner` of a `budget` (proportional).
/// Without a budget there is nothing to split against, so all of it is yours.
pub fn your_share(amount: i64, budget: Option<i64>, partner: i64) -> i64 {
    match budget {
        Some(b) if b > 0 => amount * (b - partner).clamp(0, b) / b,
        _ => amount,
    }
}

/// A date's spending in Joy.
#[derive(Debug, Clone)]
pub struct DateSpend {
    pub day: Date,
    pub amount: i64,
}

/// A debit that might be one of those dates.
#[derive(Debug, Clone)]
pub struct Debit {
    pub txn_id: String,
    pub on: Date,
    pub amount: i64,
    /// Already labelled a date (by an earlier match): it still uses up its entry.
    pub is_date: bool,
}

fn close(a: i64, b: i64) -> bool {
    (a - b).abs() <= (b.abs() / 20).max(100)
}

/// Transactions to label as dates: each Joy entry takes the closest unclaimed debit within
/// [`MATCH_DAYS`] and 5% of its amount. Debits already labelled are claimed first, so a date
/// never labels a second transaction.
pub fn match_dates(entries: &[DateSpend], debits: &[Debit]) -> Vec<String> {
    let mut claimed = vec![false; debits.len()];
    let mut out = Vec::new();
    let mut order: Vec<&DateSpend> = entries.iter().collect();
    order.sort_by_key(|e| (e.day, e.amount));
    for e in order {
        let best = debits
            .iter()
            .enumerate()
            .filter(|(i, d)| !claimed[*i] && (d.on - e.day).whole_days().abs() <= MATCH_DAYS && close(d.amount, e.amount))
            .min_by_key(|(_, d)| (!d.is_date, (d.amount - e.amount).abs(), (d.on - e.day).whole_days().abs()));
        if let Some((i, d)) = best {
            claimed[i] = true;
            if !d.is_date {
                out.push(d.txn_id.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn debit(id: &str, on: Date, rupees: i64) -> Debit {
        Debit { txn_id: id.into(), on, amount: rupees * 100, is_date: false }
    }

    #[test]
    fn share_is_proportional() {
        // ₹20,000 budget, partner covers ₹6,000: you pay 70%.
        assert_eq!(your_share(1_000_000, Some(2_000_000), 600_000), 700_000);
        assert_eq!(your_share(1_000_000, None, 600_000), 1_000_000);
        assert_eq!(your_share(1_000_000, Some(2_000_000), 5_000_000), 0);
    }

    #[test]
    fn matches_within_a_day_and_five_percent() {
        let e = [DateSpend { day: date!(2026 - 11 - 06), amount: 240_000 }];
        let d = [
            debit("far", date!(2026 - 11 - 09), 2_400),
            debit("off", date!(2026 - 11 - 06), 3_000),
            debit("next-day", date!(2026 - 11 - 07), 2_450),
        ];
        assert_eq!(match_dates(&e, &d), vec!["next-day".to_string()]);
    }

    #[test]
    fn one_transaction_per_date() {
        let e = [
            DateSpend { day: date!(2026 - 11 - 06), amount: 50_000 },
            DateSpend { day: date!(2026 - 11 - 06), amount: 50_000 },
        ];
        let d = [debit("a", date!(2026 - 11 - 06), 500), debit("b", date!(2026 - 11 - 06), 500), debit("c", date!(2026 - 11 - 06), 500)];
        assert_eq!(match_dates(&e, &d).len(), 2);
    }

    #[test]
    fn already_labelled_debits_are_claimed_first() {
        let e = [DateSpend { day: date!(2026 - 11 - 06), amount: 50_000 }];
        let d = [debit("new", date!(2026 - 11 - 06), 500), Debit { is_date: true, ..debit("old", date!(2026 - 11 - 06), 510) }];
        assert!(match_dates(&e, &d).is_empty());
    }
}
