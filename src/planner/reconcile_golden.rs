//! Characterization tests for [`reconcile`]: the whole Report (and headline) pinned across scenarios
//! that reach each rule. A refactor must leave `reconcile_golden.json` byte-for-byte alone.
//! Regenerate on purpose with `UPDATE_GOLDEN=1 cargo test reconcile_golden`.

use super::reconcile::*;
use time::{macros::date, Date};

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/reconcile_golden.json");
const TODAY: Date = date!(2026 - 11 - 20);

fn bank(id: &str) -> AccountIn {
    AccountIn { account_id: id.into(), name: format!("{id} savings"), kind: "bank".into(), locked: false }
}

fn snap(id: &str, d: Date, bal: i64, source: SnapSource, seq: i64) -> SnapshotIn {
    SnapshotIn { account_id: id.into(), as_of: d, balance_paise: bal, source, seq }
}

fn st(id: &str, d: Date, bal: i64) -> SnapshotIn {
    snap(id, d, bal, SnapSource::Statement, 1)
}

fn al(id: &str, d: Date, bal: i64) -> SnapshotIn {
    snap(id, d, bal, SnapSource::Alert, 1)
}

fn tx(id: &str, d: Date, amt: i64) -> TxnIn {
    TxnIn { on: d, amount_paise: amt, account_id: Some(id.into()), card_id: None, from_alert: false }
}

fn alert_tx(id: &str, d: Date, amt: i64) -> TxnIn {
    TxnIn { from_alert: true, ..tx(id, d, amt) }
}

fn adj(id: &str, d: Date, amt: i64, kind: AdjustKind) -> AdjustIn {
    AdjustIn { account_id: id.into(), on: d, amount_paise: amt, kind }
}

fn input(accounts: Vec<AccountIn>, snapshots: Vec<SnapshotIn>, txns: Vec<TxnIn>, adjustments: Vec<AdjustIn>) -> ReconcileIn {
    ReconcileIn { today: TODAY, accounts, snapshots, txns, adjustments }
}

fn snapshot() -> String {
    let mut out = Vec::new();
    let mut run = |name: &str, i: ReconcileIn| {
        let report = reconcile(&i);
        let headlines: Vec<Option<String>> = report.accounts.iter().map(headline).collect();
        out.push(serde_json::json!({ "name": name, "report": report, "headlines": headlines }));
    };
    let (nov1, nov5, nov10) = (date!(2026 - 11 - 01), date!(2026 - 11 - 05), date!(2026 - 11 - 10));

    run("clean_window", input(vec![bank("a")], vec![st("a", nov1, 100_000), st("a", nov10, 70_000)], vec![tx("a", nov5, -30_000)], vec![]));
    run("missing_debit", input(vec![bank("a")], vec![st("a", nov1, 100_000), st("a", nov10, 40_000)], vec![tx("a", nov5, -30_000)], vec![]));
    run("missing_credit", input(vec![bank("a")], vec![st("a", nov1, 100_000), st("a", nov10, 150_000)], vec![tx("a", nov5, 10_000)], vec![]));

    // Same flows, balance dropped 30,000. Flows: -20,000 on day 0, -10,000 between, -5,000 on day 1.
    let boundary_txns = vec![alert_tx("a", nov1, -20_000), alert_tx("a", nov5, -10_000), alert_tx("a", nov10, -5_000)];
    run("boundary_alert_midday_not_flagged", input(vec![bank("a")], vec![al("a", nov1, 100_000), al("a", nov10, 70_000)], boundary_txns.clone(), vec![]));
    run("boundary_statement_statement_flagged", input(vec![bank("a")], vec![st("a", nov1, 100_000), st("a", nov10, 70_000)], boundary_txns.clone(), vec![]));
    run("boundary_statement_then_alert_flagged", input(vec![bank("a")], vec![st("a", nov1, 100_000), al("a", nov10, 70_000)], boundary_txns, vec![]));

    run(
        "autopost_inside_window_counted",
        input(vec![bank("a")], vec![st("a", nov1, 100_000), st("a", nov10, 5_100_000)], vec![], vec![adj("a", date!(2026 - 11 - 03), 5_000_000, AdjustKind::AutoPost)]),
    );
    run(
        "autopost_also_in_statement_not_doubled",
        input(
            vec![bank("a")],
            vec![st("a", nov1, 100_000), st("a", nov10, 5_090_000)],
            vec![tx("a", nov5, 4_990_000)],
            vec![adj("a", date!(2026 - 11 - 03), 5_000_000, AdjustKind::AutoPost)],
        ),
    );
    run(
        "one_row_covers_one_adjustment_only",
        input(
            vec![bank("a")],
            vec![st("a", nov1, 100_000), st("a", nov10, 300_000)],
            vec![tx("a", nov5, 100_000)],
            vec![adj("a", date!(2026 - 11 - 04), 100_000, AdjustKind::AutoPost), adj("a", date!(2026 - 11 - 06), 100_000, AdjustKind::AutoPost)],
        ),
    );
    run(
        "move_inside_window_both_legs",
        input(
            vec![bank("a"), bank("b")],
            vec![st("a", nov1, 1_000_000), st("a", nov10, 800_000), st("b", nov1, 50_000), st("b", nov10, 250_000)],
            vec![],
            vec![adj("a", date!(2026 - 11 - 04), -200_000, AdjustKind::Move), adj("b", date!(2026 - 11 - 04), 200_000, AdjustKind::Move)],
        ),
    );
    run(
        "transfer_txn_legs_counted",
        input(
            vec![bank("a"), bank("b")],
            vec![st("a", nov1, 500_000), st("a", nov10, 400_000), st("b", nov1, 0), st("b", nov10, 100_000)],
            vec![tx("a", nov5, -100_000), tx("b", nov5, 100_000)],
            vec![],
        ),
    );
    run(
        "transfer_one_leg_missing",
        input(
            vec![bank("a"), bank("b")],
            vec![st("a", nov1, 500_000), st("a", nov10, 400_000), st("b", nov1, 0), st("b", nov10, 100_000)],
            vec![tx("a", nov5, -100_000)],
            vec![],
        ),
    );
    run(
        "null_account_rows_are_a_hint",
        input(
            vec![bank("a")],
            vec![st("a", nov1, 100_000), st("a", nov10, 40_000)],
            vec![
                TxnIn { on: nov5, amount_paise: -60_000, account_id: None, card_id: None, from_alert: false },
                TxnIn { on: date!(2026 - 11 - 11), amount_paise: -1_000, account_id: None, card_id: None, from_alert: false },
                TxnIn { on: date!(2026 - 11 - 06), amount_paise: -999, account_id: None, card_id: Some("c".into()), from_alert: false },
            ],
            vec![],
        ),
    );
    run("single_snapshot_is_never", input(vec![bank("a")], vec![st("a", nov1, 100_000)], vec![tx("a", nov5, -30_000)], vec![]));
    run("no_snapshots_is_never", input(vec![bank("a")], vec![], vec![], vec![]));
    run(
        "locked_and_non_bank_ignored",
        input(
            vec![
                AccountIn { locked: true, ..bank("l") },
                AccountIn { kind: "wallet".into(), ..bank("w") },
                bank("a"),
            ],
            vec![st("l", nov1, 100), st("l", nov10, 999_999), st("w", nov1, 100), st("w", nov10, 999_999), st("a", nov1, 100_000), st("a", nov10, 70_000)],
            vec![tx("a", nov5, -30_000)],
            vec![],
        ),
    );
    run(
        "old_gap_dropped_after_clean_window",
        input(vec![bank("a")], vec![st("a", date!(2026 - 08 - 01), 100_000), st("a", date!(2026 - 08 - 21), 50_000), st("a", nov10, 50_000)], vec![], vec![]),
    );
    run(
        "gap_ending_on_the_cutoff_day_kept",
        input(vec![bank("a")], vec![st("a", date!(2026 - 08 - 12), 100_000), st("a", date!(2026 - 08 - 22), 50_000)], vec![], vec![]),
    );
    run(
        "gap_one_day_before_the_cutoff_dropped",
        input(vec![bank("a")], vec![st("a", date!(2026 - 08 - 11), 100_000), st("a", date!(2026 - 08 - 21), 50_000)], vec![], vec![]),
    );
    run(
        "two_gaps_newest_first",
        input(
            vec![bank("a")],
            vec![
                st("a", date!(2026 - 10 - 01), 100_000),
                st("a", date!(2026 - 10 - 10), 100_000),
                st("a", date!(2026 - 10 - 20), 50_000),
                st("a", nov5, 50_000),
                st("a", date!(2026 - 11 - 15), 0),
            ],
            vec![],
            vec![],
        ),
    );
    run(
        "same_day_snapshots_statement_then_highest_seq",
        input(
            vec![bank("a")],
            vec![
                snap("a", nov1, 5, SnapSource::Alert, 1),
                snap("a", nov1, 100_000, SnapSource::Typed, 2),
                snap("a", nov10, 1, SnapSource::Alert, 9),
                snap("a", nov10, 70_000, SnapSource::Statement, 3),
                snap("a", nov10, 2, SnapSource::Statement, 2),
            ],
            vec![tx("a", nov5, -30_000)],
            vec![],
        ),
    );
    serde_json::to_string_pretty(&out).unwrap() + "\n"
}

#[test]
fn reconcile_matches_the_pinned_output() {
    let now = snapshot();
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "reconcile output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
