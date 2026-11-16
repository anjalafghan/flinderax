//! A regression net for the bank-text parser: anonymised messages with the amount (in paise) and
//! direction each must give. A message that is not a transaction (OTP, promo, failed) must not parse.
//! Add a line to `tests/fixtures/sms/messages.txt` whenever a bank text is misread.

use joy::alerts::{Direction, parse};

#[test]
fn every_fixture_parses_to_its_expected_paise_and_direction() {
    let mut seen = 0;
    for line in include_str!("fixtures/sms/messages.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let mut it = line.splitn(4, '|');
        let (id, _sender, expected, text) = (it.next().unwrap(), it.next().unwrap(), it.next().unwrap(), it.next().unwrap());
        let want = match expected {
            "-" => None,
            e => {
                let (d, p) = e.split_once(':').unwrap();
                Some((if d == "D" { Direction::Debit } else { Direction::Credit }, p.parse::<i64>().unwrap()))
            }
        };
        let got = parse(text).map(|a| (a.direction, a.amount_paise));
        assert_eq!(got, want, "fixture {id}");
        seen += 1;
    }
    assert!(seen >= 25, "only {seen} fixtures");
}
