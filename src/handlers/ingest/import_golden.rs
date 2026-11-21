//! Characterization test for [`import_statement`]: one database, a sequence of imports (bank and card,
//! repeats, overlaps, alert confirmation, errors), the results and the stored rows pinned. A refactor
//! must leave `import_golden.json` byte-for-byte alone. Regenerate on purpose with
//! `UPDATE_GOLDEN=1 cargo test import_golden`.

use super::*;
use crate::statement::{DateOrder, Mapping, SignRule};
use sqlx::sqlite::SqlitePoolOptions;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/handlers/ingest/import_golden.json");
const USER: &str = "u1";

const HEADER: &str = "\r\nDate,Narration,Value Dat,Debit Amount,Credit Amount,Chq/Ref Number,Closing Balance\r\n";

async fn seeded() -> sqlx::SqlitePool {
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let run = |sql: &'static str| {
        let db = db.clone();
        async move { sqlx::query(sql).execute(&db).await.unwrap() }
    };
    run("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u1', 'u1', 'x', 'user'), ('u2', 'u2', 'x', 'user')").await;
    run("INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, last_4_digits) VALUES ('a1', 'u1', 'HDFC', 'bank', 100, '1234')").await;
    run("INSERT INTO accounts (account_id, user_id, name, kind, balance_paise, last_4_digits, balance_set_on) VALUES ('a2', 'u1', 'Kotak', 'bank', 777, '5678', '2030-01-01')").await;
    run("INSERT INTO accounts (account_id, user_id, name, kind) VALUES ('other', 'u2', 'Not yours', 'bank')").await;
    run("INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color, last_4_digits) VALUES ('c1', 'u1', 'Visa', 'HDFC', 1, 2, '9999')").await;
    run("INSERT INTO merchant_rules (user_id, pattern, category) VALUES ('u1', 'asha stores', 'groceries')").await;
    // Bank alerts waiting for a statement: one with a matching reference, one matching on amount and
    // date only, one with a locked category, one the statement never mentions, one on another account.
    let alerts = [
        ("t_ref", "2026-11-02", "debit", 12050, "Asha Stores", "uncategorized", "a1", "612345678902", 0),
        ("t_plain", "2026-11-01", "debit", 25000, "Ravi", "uncategorized", "a1", "", 0),
        ("t_locked", "2026-11-03", "debit", 100000, "Acme", "shopping", "a1", "", 1),
        ("t_orphan", "2026-11-04", "debit", 777700, "Mystery", "uncategorized", "a1", "", 0),
        ("t_a2", "2026-11-01", "debit", 25000, "Ravi", "uncategorized", "a2", "", 0),
    ];
    for (id, on, dir, amt, merchant, cat, acct, rrn, locked) in alerts {
        sqlx::query(
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, account_id, source, raw_text, dedupe_key, rrn, category_locked)
             VALUES (?, 'u1', ?, ?, ?, ?, ?, ?, 'sms', 'alert text', ?, NULLIF(?, ''), ?)",
        )
        .bind(id).bind(on).bind(dir).bind(amt).bind(merchant).bind(cat).bind(acct).bind(format!("alert:{id}")).bind(rrn).bind(locked)
        .execute(&db)
        .await
        .unwrap();
    }
    // A card alert for a payment, filed as income: the statement must not leave it there.
    sqlx::query(
        "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, card_id, source, raw_text, dedupe_key)
         VALUES ('t_card', 'u1', '2026-11-02', 'credit', 500000, 'Payment', 'income', 'c1', 'sms', 'alert text', 'alert:t_card')",
    )
    .execute(&db)
    .await
    .unwrap();
    // A card alert for a purchase the user filed themselves: the statement keeps the user's category.
    sqlx::query(
        "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, card_id, source, raw_text, dedupe_key)
         VALUES ('t_card_debit', 'u1', '2026-11-01', 'debit', 45000, 'Swiggy', 'shopping', 'c1', 'sms', 'alert text', 'alert:t_card_debit')",
    )
    .execute(&db)
    .await
    .unwrap();
    db
}

fn bank_csv(rows: &[&str]) -> String {
    format!("{HEADER}{}", rows.join("\r\n"))
}

const R1: &str = "01/11/26,UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED,01/11/26,250.00,,612345678901,9750.00";
const R2: &str = "02/11/26,UPI-ASHA STORES-Q123456789@YBL-YESB0YBLUPI-612345678902-UPI,02/11/26,120.50,,612345678902,9629.50";
const R3: &str = "03/11/26,UPI-ACME PUNE-ACME@OKAXIS-SBIN0001234-612345678903-PAYMENT ON CRED,03/11/26,1000.00,,612345678903,8629.50";
const R4: &str = "04/11/26,SALARYFORTHEMONTHOFOCT2026,04/11/26,,50000.00,,58629.50";
const R5: &str = "05/11/26,UPI-ASHA RAO-98XXXXXX12@YESCRED-KKBK0001234-612345678904-PAID VIA CRED,05/11/26,6000.00,,612345678904,52629.50";
const R6: &str = "05/11/26,UPI-ASHA RAO-98XXXXXX12@YESCRED-KKBK0001234-612345678905-PAID VIA CRED,05/11/26,,6500.00,,59129.50";
const R7: &str = "06/11/26,ATM WDL SELF,06/11/26,2000.00,,,57129.50";
const R8: &str = "06/11/26,ATM WDL SELF,06/11/26,2000.00,,,55129.50";

fn card_mapping() -> Mapping {
    Mapping { header_row: 0, date: 0, narration: 1, debit: None, credit: None, amount: Some(2), sign: SignRule::CrSuffix, balance: None, reference: None, date_order: DateOrder::Dmy }
}

fn input(csv: String, account: Option<&str>, card: Option<&str>, mapping: Option<Mapping>) -> StatementInput {
    StatementInput { csv, account_id: account.map(String::from), card_id: card.map(String::from), mapping }
}

async fn run_step(db: &sqlx::SqlitePool, name: &str, user: &str, input: StatementInput) -> serde_json::Value {
    match import_statement(db, user, input).await {
        Ok(res) => serde_json::json!({ "step": name, "ok": res }),
        Err(AppError(status, msg)) => serde_json::json!({ "step": name, "error": { "status": status.as_u16(), "message": msg } }),
    }
}

async fn snapshot() -> String {
    let db = seeded().await;
    let mut out = Vec::new();
    let newest_first = bank_csv(&[R8, R7, R6, R5, R4, R3, R2, R1]);
    let steps: Vec<(&str, &str, StatementInput)> = vec![
        ("bank_first_import_chain_ok", USER, input(bank_csv(&[R1, R2, R3, R4, R5, R6, R7, R8]), Some("a1"), None, None)),
        ("same_file_again_all_duplicates", USER, input(bank_csv(&[R1, R2, R3, R4, R5, R6, R7, R8]), Some("a1"), None, None)),
        ("newest_first_export_is_still_duplicates", USER, input(newest_first, Some("a1"), None, None)),
        ("overlap_adds_one_new_row", USER, input(bank_csv(&[R7, R8, "07/11/26,SWIGGY ORDER,07/11/26,300.00,,,54829.50"]), Some("a1"), None, None)),
        ("older_balance_kept_on_dated_account", USER, input(bank_csv(&[R1, R2]), Some("a2"), None, None)),
        ("no_target_matches_any_alert", USER, input(bank_csv(&["01/11/26,UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-X,01/11/26,250.00,,612345678901,1.00"]), None, None, None)),
        ("card_statement_with_payment", USER, input("Date,Description,Amount\n01/11/26,SWIGGY,450.00\n02/11/26,PAYMENT RECEIVED,5000.00 Cr\n03/11/26,AMAZON,1299.00\n".into(), None, Some("c1"), Some(card_mapping()))),
        ("card_statement_again", USER, input("Date,Description,Amount\n01/11/26,SWIGGY,450.00\n02/11/26,PAYMENT RECEIVED,5000.00 Cr\n".into(), None, Some("c1"), Some(card_mapping()))),
        ("someone_elses_account", USER, input(bank_csv(&[R1]), Some("other"), None, None)),
        ("unknown_card", USER, input(bank_csv(&[R1]), None, Some("nope"), None)),
        ("not_a_statement", USER, input("nothing,useful\n1,2\n".into(), None, None, None)),
        ("mapping_reads_nothing", USER, input("Date,Description,Amount\nnot a date,X,1\n".into(), None, Some("c1"), Some(card_mapping()))),
        ("empty_header_only", USER, input(HEADER.into(), Some("a1"), None, None)),
        ("other_user_same_file_is_all_new", "u2", input(bank_csv(&[R1, R2]), None, None, None)),
    ];
    for (name, user, inp) in steps {
        out.push(run_step(&db, name, user, inp).await);
    }
    // Holder names set: no suggestion is offered.
    sqlx::query("INSERT INTO user_settings (user_id, holder_names) VALUES ('u1', 'ASHA RAO')").execute(&db).await.unwrap();
    out.push(run_step(&db, "holder_names_saved", USER, input(bank_csv(&[R5, R6, "08/11/26,CHEQUE DEPOSIT,08/11/26,,100.00,,55229.50"]), Some("a1"), None, None)).await);

    let txns: Vec<(String, String, i64, Option<String>, String, Option<String>, Option<String>, Option<String>, String, Option<String>, String, Option<String>, i64, i64)> = sqlx::query_as(
        "SELECT occurred_on, direction, amount_paise, merchant, category, card_id, account_id, last4, source, raw_text, dedupe_key, rrn, on_statement, category_locked
         FROM transactions ORDER BY user_id, occurred_on, dedupe_key, amount_paise",
    )
    .fetch_all(&db)
    .await
    .unwrap();
    let accounts: Vec<(String, i64, Option<String>)> = sqlx::query_as("SELECT account_id, balance_paise, balance_set_on FROM accounts ORDER BY account_id").fetch_all(&db).await.unwrap();
    let imports: Vec<(String, Option<String>, Option<String>, String, String, i64, Option<i64>, Option<i64>, i64)> = sqlx::query_as(
        "SELECT user_id, account_id, card_id, first_date, last_date, rows, opening_paise, closing_paise, chain_ok FROM statement_imports ORDER BY first_date, last_date, user_id, rows",
    )
    .fetch_all(&db)
    .await
    .unwrap();
    serde_json::to_string_pretty(&serde_json::json!({ "steps": out, "transactions": txns, "accounts": accounts, "imports": imports })).unwrap() + "\n"
}

#[tokio::test]
async fn import_statement_matches_the_pinned_output() {
    let now = snapshot().await;
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "statement import changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
