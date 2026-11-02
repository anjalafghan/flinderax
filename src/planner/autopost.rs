//! Automatic posting: salary and fixed payments land in balances on their dates, so a balance
//! only has to be typed in again when it has drifted. Pure: the caller supplies the sources and
//! the date each balance was last typed in (its *baseline*).
//!
//! The dates follow the Safe-to-Spend engine's convention, so nothing is counted twice:
//! * income dated on the baseline day is assumed to be in the typed balance already, so income
//!   posts for days in `(baseline, today]`;
//! * a fixed payment dated today is still "upcoming" for the engine, so payments post for days
//!   in `[baseline, today)`.

use std::collections::HashMap;

use time::{Date, Duration};

use super::{commitment_dates, occurrences, CommitmentIn, IncomeIn};

/// Never reach back further than this, however old a baseline is.
pub const MAX_LOOKBACK_DAYS: i64 = 92;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Income,
    Commitment,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Income => "income",
            SourceKind::Commitment => "commitment",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    pub kind: SourceKind,
    pub source_id: String,
    pub account_id: String,
    pub on: Date,
    pub label: String,
    /// Signed: + credit, - debit.
    pub amount: i64,
}

/// Every posting due as of `today`. `baselines` maps account id -> date its balance was typed
/// in; accounts missing from it are never touched.
pub fn due_postings(
    incomes: &[(String, IncomeIn)],
    commitments: &[(String, CommitmentIn)],
    baselines: &HashMap<String, Date>,
    today: Date,
) -> Vec<Posting> {
    let floor = today - Duration::days(MAX_LOOKBACK_DAYS);
    let since = |account: &Option<String>| -> Option<(String, Date)> {
        let id = account.as_ref()?;
        baselines.get(id).map(|b| (id.clone(), (*b).max(floor)))
    };
    let mut out = Vec::new();

    for (id, inc) in incomes {
        if let Some((acct, base)) = since(&inc.account_id) {
            for on in occurrences(inc.day, base + Duration::days(1), today + Duration::days(1)) {
                out.push(Posting {
                    kind: SourceKind::Income,
                    source_id: id.clone(),
                    account_id: acct.clone(),
                    on,
                    label: inc.label.clone(),
                    amount: inc.amount,
                });
            }
        }
    }

    for (id, c) in commitments {
        let legs = [(&c.account_id, -c.amount), (&c.to_account_id, c.amount)];
        for (account, amount) in legs {
            if let Some((acct, base)) = since(account) {
                for on in commitment_dates(c, base, today) {
                    out.push(Posting {
                        kind: SourceKind::Commitment,
                        source_id: id.clone(),
                        account_id: acct.clone(),
                        on,
                        label: c.label.clone(),
                        amount,
                    });
                }
            }
        }
    }

    out.sort_by(|a, b| a.on.cmp(&b.on).then_with(|| a.source_id.cmp(&b.source_id)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::{Bucket, Freq};
    use time::macros::date;

    fn income(day: u8, rupees: i64, account: &str) -> (String, IncomeIn) {
        ("inc".into(), IncomeIn { label: "Salary".into(), amount: rupees * 100, day, account_id: Some(account.into()) })
    }

    fn sip(day: u8, rupees: i64, from: &str, to: Option<&str>) -> (String, CommitmentIn) {
        (
            "sip".into(),
            CommitmentIn {
                label: "SIP".into(),
                amount: rupees * 100,
                day,
                account_id: Some(from.into()),
                active: true,
                bucket: Bucket::Savings,
                freq: Freq::Monthly,
                due_month: None,
                to_account_id: to.map(str::to_string),
                start_month: None,
                end_month: None,
            },
        )
    }

    fn base(pairs: &[(&str, Date)]) -> HashMap<String, Date> {
        pairs.iter().map(|(k, d)| (k.to_string(), *d)).collect()
    }

    #[test]
    fn income_posts_after_baseline_up_to_today() {
        let b = base(&[("hdfc", date!(2026 - 10 - 20))]);
        let p = due_postings(&[income(1, 90_000, "hdfc")], &[], &b, date!(2026 - 11 - 01));
        assert_eq!(p.len(), 1);
        assert_eq!((p[0].on, p[0].amount), (date!(2026 - 11 - 01), 9_000_000));
    }

    #[test]
    fn income_on_baseline_day_is_already_in_the_balance() {
        let b = base(&[("hdfc", date!(2026 - 11 - 01))]);
        assert!(due_postings(&[income(1, 90_000, "hdfc")], &[], &b, date!(2026 - 11 - 05)).is_empty());
    }

    #[test]
    fn payment_due_today_is_left_to_the_planner() {
        let b = base(&[("hdfc", date!(2026 - 10 - 20))]);
        let today = date!(2026 - 11 - 05);
        assert!(due_postings(&[], &[sip(5, 10_000, "hdfc", None)], &b, today).is_empty());
        let p = due_postings(&[], &[sip(5, 10_000, "hdfc", None)], &b, today + Duration::days(1));
        assert_eq!((p[0].on, p[0].amount), (date!(2026 - 11 - 05), -1_000_000));
    }

    #[test]
    fn payment_on_baseline_day_still_posts() {
        let b = base(&[("hdfc", date!(2026 - 11 - 05))]);
        let p = due_postings(&[], &[sip(5, 10_000, "hdfc", None)], &b, date!(2026 - 11 - 06));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn transfer_debits_one_account_and_credits_the_other() {
        let b = base(&[("hdfc", date!(2026 - 10 - 01)), ("rd", date!(2026 - 10 - 01))]);
        let p = due_postings(&[], &[sip(10, 5_000, "hdfc", Some("rd"))], &b, date!(2026 - 10 - 15));
        let mut amounts: Vec<(String, i64)> = p.iter().map(|x| (x.account_id.clone(), x.amount)).collect();
        amounts.sort();
        assert_eq!(amounts, vec![("hdfc".into(), -500_000), ("rd".into(), 500_000)]);
    }

    #[test]
    fn accounts_without_a_baseline_are_untouched() {
        let p = due_postings(&[income(1, 90_000, "hdfc")], &[], &HashMap::new(), date!(2026 - 11 - 05));
        assert!(p.is_empty());
    }

    #[test]
    fn lookback_is_capped() {
        let b = base(&[("hdfc", date!(2025 - 01 - 01))]);
        let p = due_postings(&[income(1, 1, "hdfc")], &[], &b, date!(2026 - 11 - 05));
        assert!(p.len() <= 4, "{p:?}");
        assert!(p.iter().all(|x| x.on >= date!(2026 - 11 - 05) - Duration::days(MAX_LOOKBACK_DAYS)));
    }
}
