//! Per-bank goldens for the bank-alert parser. Each line of `tests/fixtures/sms/banks/*.txt` is
//! `id | expected | merchant | last4 | date | text` (`~` = none). The expected amount (paise),
//! direction, merchant, last four and date must all match; lines marked `-` must not parse.
//! Every name, account tail and reference in the fixtures is invented.

mod common;

use joy::alerts::{Direction, parse};
use serde_json::json;
use time::{Date, Month};

const BANKS: &[(&str, &str)] = &[
    ("hdfc", include_str!("fixtures/sms/banks/hdfc.txt")),
    ("icici", include_str!("fixtures/sms/banks/icici.txt")),
    ("sbi", include_str!("fixtures/sms/banks/sbi.txt")),
    ("axis", include_str!("fixtures/sms/banks/axis.txt")),
    ("kotak", include_str!("fixtures/sms/banks/kotak.txt")),
];

fn opt(s: &str) -> Option<String> {
    (s != "~").then(|| s.to_string())
}

fn date(s: &str) -> Date {
    let mut p = s.split('-').map(|x| x.parse::<i32>().unwrap());
    let (y, m, d) = (p.next().unwrap(), p.next().unwrap(), p.next().unwrap());
    Date::from_calendar_date(y, Month::try_from(m as u8).unwrap(), d as u8).unwrap()
}

/// All fixture lines as (bank, line).
fn lines() -> Vec<(&'static str, &'static str)> {
    BANKS
        .iter()
        .flat_map(|(bank, body)| body.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(move |l| (*bank, l)))
        .collect()
}

#[test]
fn every_bank_message_parses_to_its_golden_fields() {
    let (mut parsed, mut rejected) = (0, 0);
    let mut misses: Vec<String> = Vec::new();
    for (bank, line) in lines() {
        let mut it = line.splitn(6, '|');
        let id = it.next().unwrap();
        let expected = it.next().unwrap();
        let (merchant, last4, day, text) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
        assert!(id.starts_with(bank), "{id} is in the wrong file ({bank})");
        let got = parse(text);
        if expected == "-" {
            if got.is_some() {
                misses.push(format!("{id}: must not parse, got {got:?}"));
            }
            rejected += 1;
            continue;
        }
        let Some(a) = got else {
            misses.push(format!("{id}: did not parse"));
            continue;
        };
        let (d, p) = expected.split_once(':').unwrap();
        let direction = if d == "D" { Direction::Debit } else { Direction::Credit };
        let want = (direction, p.parse::<i64>().unwrap(), opt(merchant), opt(last4), Some(date(day)));
        let have = (a.direction, a.amount_paise, a.merchant, a.last4, a.date);
        if want != have {
            misses.push(format!("{id}:\n   want {want:?}\n   have {have:?}"));
        }
        parsed += 1;
    }
    assert!(misses.is_empty(), "{} mismatches:\n{}", misses.len(), misses.join("\n"));
    assert!(parsed >= 25, "only {parsed} parsed fixtures");
    assert!(rejected >= 8, "only {rejected} must-not-parse fixtures");
}

#[test]
fn balances_and_limits_are_not_mistaken_for_the_amount() {
    let a = parse("INR 1,299.00 spent using ICICI Bank Card XX1234 on 01-Nov-26 on AMAZON. Avl Limit: INR 98,701.00.").unwrap();
    assert_eq!((a.amount_paise, a.available_limit_paise, a.balance_paise), (129_900, Some(9_870_100), None));
    let a = parse("Rs.10,000.00 withdrawn from HDFC Bank ATM XXXX1234 on 04-11-26. Avl bal Rs 40,000.50").unwrap();
    assert_eq!((a.amount_paise, a.balance_paise, a.available_limit_paise), (1_000_000, Some(4_000_050), None));
}

#[test]
fn a_card_bill_payment_is_flagged_and_has_no_merchant() {
    let a = parse("Payment of Rs 15,000.00 received towards your HDFC Bank Credit Card ending 1234 on 05-11-2026. Thank you.").unwrap();
    assert!(a.card_payment);
    assert_eq!((a.direction, a.merchant), (Direction::Credit, None));
}

#[test]
fn a_message_wrapped_over_several_lines_reads_the_same() {
    let a = parse("Spent Rs.89.50\nOn HDFC Bank Card 1234\nAt CHAAYOS On 2026-11-02:10:00:00").unwrap();
    assert_eq!((a.amount_paise, a.merchant.as_deref(), a.last4.as_deref()), (8950, Some("CHAAYOS"), Some("1234")));
    assert_eq!(a.date, Some(date("2026-11-02")));
}

#[test]
fn the_upi_reference_is_read() {
    let a = parse("Sent Rs.250.00 From HDFC Bank A/C *1234 To RAJU STORE On 01/11/26 Ref 430512345678 Not You?").unwrap();
    assert_eq!(a.reference.as_deref(), Some("430512345678"));
}

#[test]
fn empty_and_unrelated_text_do_not_parse() {
    for t in ["", "   ", "Hi, are we still on for dinner tonight?", "Your parcel ships on 12-11-26.", "Rs 500"] {
        assert!(parse(t).is_none(), "{t:?} must not parse");
    }
}

/// The same parser sits behind `POST /api/ingest/parse`; a signed-in web session (cookie plus
/// `X-Joy`) can paste a bank text and get the transaction back.
#[tokio::test]
async fn pasting_a_bank_text_through_the_api_records_it() {
    let t = common::TestApp::new().await;
    let s = t.session("ingest").await;
    let res = t
        .call_cookie(
            axum::http::Method::POST,
            "/api/ingest/parse",
            &s.cookie,
            Some(json!({"text": "Spent Rs.450.00 On HDFC Bank Card 1234 At SWIGGY On 2026-11-01:13:45:12 Not You? Call 18002586161"})),
        )
        .await;
    assert!(res.status.is_success(), "{}", res.text);
    assert!(res.text.contains("45000") || res.text.contains("450"), "{}", res.text);
    // The cookie without X-Joy is refused for a POST; the bearer token still works.
    let no_header = axum::http::Request::post("/api/ingest/parse")
        .header("cookie", &s.cookie)
        .header("fly-client-ip", "10.99.0.1")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(json!({"text": "x"}).to_string()))
        .unwrap();
    use tower::ServiceExt;
    let refused = t.app.clone().oneshot(no_header).await.unwrap();
    assert!(refused.status().is_client_error(), "cookie POST without X-Joy must be refused");
    let ok = t.post("/api/ingest/parse", &s.token, json!({"text": "Rs 500"})).await;
    assert_ne!(ok.status, axum::http::StatusCode::UNAUTHORIZED);
}
