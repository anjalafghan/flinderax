//! Reconciliation: between two observed balances of a bank account, does the money that moved
//! match what joy recorded? A gap means joy missed data (a bank row, a transfer, a bill).
//! Pure: the caller supplies snapshots, transactions and balance adjustments; `today` is passed in.
//!
//! The boundary days are the catch. A statement balance is the end of its day, so all of that
//! day's flows are inside it. An alert or typed balance is read at an unknown moment, so any
//! subset of that day's flows may be inside it: the day contributes an interval (all its debits
//! to all its credits) instead of a number, and a window is flagged only when the observed change
//! falls outside the whole interval, beyond a small tolerance.

use serde::Serialize;
use time::{Date, Duration};

use super::{format_inr, short_date};

/// A difference this small (or smaller) is rounding, not a gap.
pub const TOLERANCE_PAISE: i64 = 100;
/// Gaps older than this (by their end date) are not listed.
pub const GAP_LOOKBACK_DAYS: i64 = 90;
/// How far apart an adjustment and a real row may be and still be the same money.
const MATCH_WINDOW_DAYS: i64 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapSource {
    Alert,
    Statement,
    Typed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotIn {
    pub account_id: String,
    pub as_of: Date,
    pub balance_paise: i64,
    pub source: SnapSource,
    /// Insertion order; higher is later.
    pub seq: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountIn {
    pub account_id: String,
    pub name: String,
    pub kind: String,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxnIn {
    pub on: Date,
    /// Signed: + credit, - debit.
    pub amount_paise: i64,
    pub account_id: Option<String>,
    pub card_id: Option<String>,
    pub from_alert: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustKind {
    AutoPost,
    Move,
}

/// A balance change joy made without a transaction row (one per account leg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustIn {
    pub account_id: String,
    pub on: Date,
    /// Signed: + credit, - debit.
    pub amount_paise: i64,
    pub kind: AdjustKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileIn {
    pub today: Date,
    pub accounts: Vec<AccountIn>,
    pub snapshots: Vec<SnapshotIn>,
    pub txns: Vec<TxnIn>,
    pub adjustments: Vec<AdjustIn>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gap {
    pub from: Date,
    pub to: Date,
    pub observed_paise: i64,
    pub expected_low_paise: i64,
    pub expected_high_paise: i64,
    pub gap_paise: i64,
    /// "missing_credit" (money arrived unseen) or "missing_debit" (money left unseen).
    pub direction: &'static str,
    pub unattributed_count: usize,
    pub unattributed_paise: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountReport {
    pub account_id: String,
    pub name: String,
    /// "gap" | "reconciled" | "never"
    pub status: &'static str,
    pub reconciled_through: Option<Date>,
    pub snapshot_days: usize,
    pub last_snapshot_on: Option<Date>,
    pub last_alert_on: Option<Date>,
    pub gaps: Vec<Gap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub accounts: Vec<AccountReport>,
}

/// One snapshot per day: a statement wins, otherwise the latest inserted. Sorted by day.
fn keep_snapshots(account_id: &str, all: &[SnapshotIn]) -> Vec<SnapshotIn> {
    let mut mine: Vec<&SnapshotIn> = all.iter().filter(|s| s.account_id == account_id).collect();
    mine.sort_by_key(|s| (s.as_of, s.source == SnapSource::Statement, s.seq));
    let mut kept: Vec<SnapshotIn> = Vec::new();
    for s in mine {
        match kept.last_mut() {
            Some(last) if last.as_of == s.as_of => *last = s.clone(),
            _ => kept.push(s.clone()),
        }
    }
    kept
}

/// Adjustments no real row already covers. Same rule as autopost's `already_seen`, but a row may
/// cover only one adjustment.
fn counted_adjustments(account_id: &str, adjs: &[AdjustIn], txns: &[&TxnIn]) -> Vec<(Date, i64)> {
    let mut mine: Vec<&AdjustIn> = adjs.iter().filter(|a| a.account_id == account_id).collect();
    mine.sort_by_key(|a| (a.on, a.amount_paise));
    let mut used = vec![false; txns.len()];
    let mut out = Vec::new();
    for a in mine {
        let tol = (a.amount_paise.abs() / 20).max(100);
        let hit = txns
            .iter()
            .enumerate()
            .filter(|(i, t)| {
                !used[*i]
                    && t.amount_paise.signum() == a.amount_paise.signum()
                    && (t.amount_paise - a.amount_paise).abs() <= tol
                    && (t.on - a.on).whole_days().abs() <= MATCH_WINDOW_DAYS
            })
            .min_by_key(|(_, t)| ((t.on - a.on).whole_days().abs(), (t.amount_paise - a.amount_paise).abs()))
            .map(|(i, _)| i);
        match hit {
            Some(i) => used[i] = true,
            None => out.push((a.on, a.amount_paise)),
        }
    }
    out
}

/// (negative sum, positive sum) of the flows on one day.
fn day_interval(flows: &[(Date, i64)], day: Date) -> (i64, i64) {
    flows.iter().filter(|(d, _)| *d == day).fold((0, 0), |(lo, hi), (_, a)| {
        if *a < 0 {
            (lo + a, hi)
        } else {
            (lo, hi + a)
        }
    })
}

pub fn reconcile(input: &ReconcileIn) -> Report {
    let mut accounts = Vec::new();
    for acct in input.accounts.iter().filter(|a| a.kind == "bank" && !a.locked) {
        let id = acct.account_id.as_str();
        let snaps = keep_snapshots(id, &input.snapshots);
        let own_txns: Vec<&TxnIn> = input.txns.iter().filter(|t| t.account_id.as_deref() == Some(id)).collect();
        let mut flows: Vec<(Date, i64)> = own_txns.iter().map(|t| (t.on, t.amount_paise)).collect();
        flows.extend(counted_adjustments(id, &input.adjustments, &own_txns));

        let mut windows: Vec<Option<Gap>> = Vec::new();
        for w in snaps.windows(2) {
            let (s0, s1) = (&w[0], &w[1]);
            let (d0, d1) = (s0.as_of, s1.as_of);
            let certain: i64 = flows.iter().filter(|(d, _)| *d > d0 && *d < d1).map(|(_, a)| a).sum();
            let (lo0, hi0) = if s0.source == SnapSource::Statement { (0, 0) } else { day_interval(&flows, d0) };
            let (lo1, hi1) = if s1.source == SnapSource::Statement {
                let exact: i64 = flows.iter().filter(|(d, _)| *d == d1).map(|(_, a)| a).sum();
                (exact, exact)
            } else {
                day_interval(&flows, d1)
            };
            let (low, high) = (certain + lo0 + lo1, certain + hi0 + hi1);
            let observed = s1.balance_paise - s0.balance_paise;
            let (gap, direction) = if observed > high + TOLERANCE_PAISE {
                (observed - high, "missing_credit")
            } else if observed < low - TOLERANCE_PAISE {
                (observed - low, "missing_debit")
            } else {
                windows.push(None);
                continue;
            };
            let loose: Vec<&TxnIn> = input
                .txns
                .iter()
                .filter(|t| t.account_id.is_none() && t.card_id.is_none() && t.on >= d0 && t.on <= d1)
                .collect();
            windows.push(Some(Gap {
                from: d0,
                to: d1,
                observed_paise: observed,
                expected_low_paise: low,
                expected_high_paise: high,
                gap_paise: gap,
                direction,
                unattributed_count: loose.len(),
                unattributed_paise: loose.iter().map(|t| t.amount_paise).sum(),
            }));
        }

        let status = if snaps.len() < 2 {
            "never"
        } else if windows.last().is_some_and(|w| w.is_some()) {
            "gap"
        } else {
            "reconciled"
        };
        let reconciled_through = windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, w)| w.is_none())
            .map(|(i, _)| snaps[i + 1].as_of);
        let cutoff = input.today - Duration::days(GAP_LOOKBACK_DAYS);
        let gaps: Vec<Gap> = windows.into_iter().rev().flatten().filter(|g| g.to >= cutoff).collect();

        accounts.push(AccountReport {
            account_id: acct.account_id.clone(),
            name: acct.name.clone(),
            status,
            reconciled_through,
            snapshot_days: snaps.len(),
            last_snapshot_on: snaps.last().map(|s| s.as_of),
            last_alert_on: own_txns.iter().filter(|t| t.from_alert).map(|t| t.on).max(),
            gaps,
        });
    }
    Report { accounts }
}

/// A one-line action for an account whose newest window is a gap; `None` otherwise.
pub fn headline(a: &AccountReport) -> Option<String> {
    if a.status != "gap" {
        return None;
    }
    let g = a.gaps.first()?;
    let how = if g.gap_paise < 0 { "went out" } else { "came in" };
    Some(format!(
        "{}: {} {} that joy didn't see between {} and {}. Upload a statement to fill it in.",
        a.name,
        format_inr(g.gap_paise.abs()),
        how,
        short_date(g.from),
        short_date(g.to)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn acct(id: &str) -> AccountIn {
        AccountIn { account_id: id.into(), name: format!("{id} savings"), kind: "bank".into(), locked: false }
    }
    fn snap(d: Date, bal: i64, source: SnapSource) -> SnapshotIn {
        SnapshotIn { account_id: "a".into(), as_of: d, balance_paise: bal, source, seq: 1 }
    }
    fn txn(d: Date, amt: i64) -> TxnIn {
        TxnIn { on: d, amount_paise: amt, account_id: Some("a".into()), card_id: None, from_alert: false }
    }
    fn run(snaps: Vec<SnapshotIn>, txns: Vec<TxnIn>) -> AccountReport {
        let input = ReconcileIn {
            today: date!(2026 - 11 - 20),
            accounts: vec![acct("a")],
            snapshots: snaps,
            txns,
            adjustments: vec![],
        };
        reconcile(&input).accounts.remove(0)
    }

    #[test]
    fn tolerance_edges() {
        let s = |off: i64| {
            run(
                vec![snap(date!(2026 - 11 - 01), 1_000_000, SnapSource::Statement), snap(date!(2026 - 11 - 10), 1_000_000 - 50_000 - off, SnapSource::Statement)],
                vec![txn(date!(2026 - 11 - 05), -50_000)],
            )
        };
        assert_eq!(s(100).status, "reconciled");
        assert_eq!(s(101).status, "gap");
        assert_eq!(s(101).gaps[0].gap_paise, -101);
        assert_eq!(s(-100).status, "reconciled");
        assert_eq!(s(-101).gaps[0].direction, "missing_credit");
    }

    #[test]
    fn headline_text() {
        let r = run(
            vec![snap(date!(2026 - 11 - 02), 1_000_000, SnapSource::Statement), snap(date!(2026 - 11 - 14), 580_000, SnapSource::Statement)],
            vec![],
        );
        assert_eq!(
            headline(&r).unwrap(),
            "a savings: ₹4,200 went out that joy didn't see between 2 Nov and 14 Nov. Upload a statement to fill it in."
        );
        let r = run(
            vec![snap(date!(2026 - 11 - 02), 1_000_000, SnapSource::Statement), snap(date!(2026 - 11 - 14), 1_000_050, SnapSource::Statement)],
            vec![],
        );
        assert!(r.status == "reconciled" && headline(&r).is_none());
        let r = run(
            vec![snap(date!(2026 - 11 - 02), 1_000_000, SnapSource::Statement), snap(date!(2026 - 11 - 14), 1_050_050, SnapSource::Statement)],
            vec![],
        );
        assert!(headline(&r).unwrap().contains("₹500.50 came in that joy didn't see"));
    }
}
