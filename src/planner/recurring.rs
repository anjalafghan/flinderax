//! Spots subscriptions and other repeating payments in transactions, so they can be added as
//! fixed payments with one tap, and notices when one gets more expensive. Pure: no DB, no clock.

use serde::Serialize;
use time::Date;

/// A debit as the detector needs it.
#[derive(Debug, Clone)]
pub struct Seen {
    /// Normalised merchant (see `alerts::normalize`).
    pub merchant: String,
    /// The merchant as shown.
    pub label: String,
    pub on: Date,
    pub amount: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Every {
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recurring {
    pub merchant: String,
    pub label: String,
    pub every: Every,
    /// The latest amount.
    pub amount_paise: i64,
    /// Day of the month it usually comes out.
    pub day: u8,
    /// For yearly ones, the month (1-12).
    pub month: Option<u8>,
    pub last_on: Date,
    pub times: usize,
    /// The amount before the latest one, when the latest is more than 2% higher.
    pub was_paise: Option<i64>,
}

const MONTH_DAYS: (i64, i64) = (26, 35);
const YEAR_DAYS: (i64, i64) = (350, 380);
/// Repeats within this share of each other (25%) are the same bill, so a price rise doesn't
/// break the chain.
const SAME_AMOUNT_BPS: i64 = 2_500;
const RISE_BPS: i64 = 200;

fn similar(a: i64, b: i64) -> bool {
    (a - b).abs() * 10_000 <= a.max(b) * SAME_AMOUNT_BPS
}

/// Repeating payments in `debits` that aren't already covered by one of `known` (normalised
/// labels of existing fixed payments). Monthly needs 2 gaps of about a month (3 payments);
/// yearly needs 1 gap of about a year.
pub fn detect(debits: &[Seen], known: &[String]) -> Vec<Recurring> {
    let mut by_merchant: std::collections::BTreeMap<&str, Vec<&Seen>> = Default::default();
    for d in debits.iter().filter(|d| !d.merchant.is_empty() && d.amount > 0) {
        by_merchant.entry(d.merchant.as_str()).or_default().push(d);
    }
    let mut out = Vec::new();
    for (merchant, mut list) in by_merchant {
        if known.iter().any(|k| !k.is_empty() && (merchant.contains(k.as_str()) || k.contains(merchant))) {
            continue;
        }
        list.sort_by_key(|d| d.on);
        let last = *list.last().expect("non-empty");
        // Walk back from the latest payment through similar amounts at a steady gap.
        for (every, (lo, hi), needed) in [(Every::Monthly, MONTH_DAYS, 2usize), (Every::Yearly, YEAR_DAYS, 1)] {
            let mut chain = vec![last];
            for d in list.iter().rev().skip(1) {
                let prev = *chain.last().expect("non-empty");
                let gap = (prev.on - d.on).whole_days();
                if gap < lo {
                    continue; // a second charge in the same cycle (a top-up): skip it
                }
                if gap > hi || !similar(d.amount, prev.amount) {
                    break;
                }
                chain.push(d);
            }
            if chain.len() > needed {
                let was = chain.get(1).map(|p| p.amount).filter(|p| (last.amount - p) * 10_000 > p * RISE_BPS);
                out.push(Recurring {
                    merchant: merchant.to_string(),
                    label: last.label.clone(),
                    every,
                    amount_paise: last.amount,
                    day: last.on.day(),
                    month: (every == Every::Yearly).then(|| last.on.month() as u8),
                    last_on: last.on,
                    times: chain.len(),
                    was_paise: was,
                });
                break;
            }
        }
    }
    out.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn s(m: &str, on: Date, rupees: i64) -> Seen {
        Seen { merchant: m.into(), label: m.to_uppercase(), on, amount: rupees * 100 }
    }

    #[test]
    fn monthly_needs_three_payments_a_month_apart() {
        let two = [s("netflix", date!(2026 - 09 - 05), 649), s("netflix", date!(2026 - 10 - 05), 649)];
        assert!(detect(&two, &[]).is_empty());
        let three = [two[0].clone(), two[1].clone(), s("netflix", date!(2026 - 11 - 04), 649)];
        let r = detect(&three, &[]);
        assert_eq!(r.len(), 1);
        assert_eq!((r[0].every, r[0].day, r[0].times, r[0].was_paise), (Every::Monthly, 4, 3, None));
    }

    #[test]
    fn notices_a_price_rise() {
        let d = [s("netflix", date!(2026 - 08 - 05), 649), s("netflix", date!(2026 - 09 - 05), 649), s("netflix", date!(2026 - 10 - 05), 699)];
        let r = detect(&d, &[]);
        assert_eq!((r[0].amount_paise, r[0].was_paise), (69_900, Some(64_900)));
    }

    #[test]
    fn a_big_rise_still_counts_as_the_same_bill() {
        let d = [s("netflix", date!(2026 - 08 - 05), 649), s("netflix", date!(2026 - 09 - 05), 649), s("netflix", date!(2026 - 10 - 05), 799)];
        assert_eq!(detect(&d, &[])[0].was_paise, Some(64_900));
    }

    #[test]
    fn yearly_from_two_payments_a_year_apart() {
        let d = [s("airtel", date!(2025 - 11 - 02), 3_599), s("airtel", date!(2026 - 11 - 01), 3_599)];
        let r = detect(&d, &[]);
        assert_eq!((r[0].every, r[0].month), (Every::Yearly, Some(11)));
    }

    #[test]
    fn irregular_or_known_payments_are_ignored() {
        let swiggy = [
            s("swiggy", date!(2026 - 09 - 01), 450),
            s("swiggy", date!(2026 - 09 - 09), 220),
            s("swiggy", date!(2026 - 10 - 20), 900),
            s("swiggy", date!(2026 - 11 - 02), 300),
        ];
        assert!(detect(&swiggy, &[]).is_empty());
        let gym = [s("cult fit", date!(2026 - 09 - 01), 1_500), s("cult fit", date!(2026 - 10 - 01), 1_500), s("cult fit", date!(2026 - 11 - 01), 1_500)];
        assert!(detect(&gym, &["cult".into()]).is_empty());
    }
}
