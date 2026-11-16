//! Merchant key: a stable lookup key for a user-corrections table, derived from the raw payee
//! string of a bank text, statement line or UPI payment. Pure: no IO, no clock.
//!
//! An empty result means "no key" (all noise, or a person-to-person phone-number VPA); callers
//! must not store or look up corrections under it.

/// Payment-rail words that start a bank narration (`UPI/...`, `POS ...`, `ACH D-...`). Only
/// stripped from the front, and repeatedly, so `UPI-POS X` and `X` agree.
const CHANNELS: &[&str] = &["upi", "pos", "neft", "imps", "rtgs", "vps", "ecom", "nach", "mmt"];

/// Company-form and filler words, stripped only from the END (repeatedly). Conservative on
/// purpose:
/// * `pvt`, `ltd`, `limited`, `private`: legal suffixes, never the distinguishing part.
/// * `india`: trailing only, so `uber india` -> `uber` but `india gate` keeps its name and
///   `uber india systems` stays distinct from `uber`.
/// * `payment` / `payments`: trailing only (`phonepe payments` -> `phonepe`).
/// * `pay` is NEVER stripped: `amazon pay` (wallet/bill pay) and `amazon` (shopping) are
///   different things to categorise, and `google pay`, `bharat pay` etc. would collide.
const TRAILING_NOISE: &[&str] = &["pvt", "ltd", "limited", "private", "india", "payment", "payments"];

/// Normalise a payee string to a stable key. Empty means "no key" (all noise, or a person
/// identified only by a phone-number VPA such as `9876543210@ybl`).
///
/// Steps, in order:
/// 1. Lowercase; `/` and whitespace separate chunks. In each chunk, a VPA handle (`@okaxis`,
///    `@ybl`, any `@bank`) is cut off and the local part kept (`john.doe@okhdfcbank` -> `john doe`).
/// 2. Split on every non-alphanumeric char, so runs of separators collapse.
/// 3. Remove digit runs of 4+ (reference / account / phone numbers) wherever they sit, but keep
///    short digits inside words (`7eleven`). This is also why a pure phone-number VPA yields "".
/// 4. Drop leading channel words (and `ach d` / `ach c` debit/credit markers).
/// 5. Drop trailing noise words and trailing short store numbers (`swiggy bangalore 44`).
///    Short numbers in the middle stay (`route 66 cafe`).
pub fn merchant_key(raw: &str) -> String {
    let lower = raw.to_lowercase();
    let mut tokens: Vec<String> = Vec::new();
    for chunk in lower.split(|c: char| c == '/' || c.is_whitespace()) {
        let local = chunk.split('@').next().unwrap_or("");
        for tok in local.split(|c: char| !c.is_alphanumeric()) {
            let tok = strip_long_digit_runs(tok);
            if !tok.is_empty() {
                tokens.push(tok);
            }
        }
    }
    // Both strips run to a fixpoint so the result is idempotent.
    loop {
        let before = tokens.len();
        while let Some(first) = tokens.first() {
            if CHANNELS.contains(&first.as_str()) {
                tokens.remove(0);
            } else if first == "ach" && matches!(tokens.get(1).map(String::as_str), Some("d" | "c")) {
                tokens.drain(0..2);
            } else {
                break;
            }
        }
        while let Some(last) = tokens.last() {
            let short_number = last.len() <= 3 && last.chars().all(|c| c.is_ascii_digit());
            if short_number || TRAILING_NOISE.contains(&last.as_str()) {
                tokens.pop();
            } else {
                break;
            }
        }
        if tokens.len() == before {
            break;
        }
    }
    tokens.join(" ")
}

/// Remove every maximal run of 4 or more ASCII digits from a token.
fn strip_long_digit_runs(tok: &str) -> String {
    let mut out = String::new();
    let mut run = String::new();
    for c in tok.chars() {
        if c.is_ascii_digit() {
            run.push(c);
        } else {
            if run.len() < 4 {
                out.push_str(&run);
            }
            run.clear();
            out.push(c);
        }
    }
    if run.len() < 4 {
        out.push_str(&run);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn realistic_indian_examples() {
        let cases: &[(&str, &str)] = &[
            ("SWIGGY@icici", "swiggy"),
            ("john.doe@okhdfcbank", "john doe"),
            ("UPI/SWIGGY/412345678901/Payment", "swiggy"),
            ("SWIGGY BANGALORE 4412", "swiggy bangalore"),
            ("SWIGGY BANGALORE 44", "swiggy bangalore"),
            ("ZOMATO LTD", "zomato"),
            ("zomato@ybl", "zomato"),
            ("Amazon Pay India Pvt Ltd", "amazon pay"),
            ("AMAZON", "amazon"),
            ("UBER INDIA", "uber"),
            ("UPI-UBER INDIA SYSTEMS PVT LTD", "uber india systems"),
            ("IRCTC", "irctc"),
            ("IRCTC Limited", "irctc"),
            ("UPI/BIGBASKET@okaxis/Groceries", "bigbasket groceries"),
            ("BIGBASKET@paytm", "bigbasket"),
            ("UPI-BLINKIT-GROFERS INDIA PVT LTD", "blinkit grofers"),
            ("Zepto Marketplace Private Limited", "zepto marketplace"),
            ("POS 7ELEVEN MUMBAI", "7eleven mumbai"),
            ("IMPS-412345678901-RAHUL SHARMA", "rahul sharma"),
            ("NEFT-ACME  CORP", "acme corp"),
            ("ACH D- BAJAJ FINANCE LTD 889", "bajaj finance"),
            ("VPS/PHONEPE PAYMENTS", "phonepe"),
            ("paytm-12345@paytm", "paytm"),
            ("  Big\t Basket \n", "big basket"),
            ("amazon pay", "amazon pay"),
            ("amazon", "amazon"),
            ("route 66 cafe", "route 66 cafe"),
        ];
        for (raw, want) in cases {
            assert_eq!(merchant_key(raw), *want, "input: {raw:?}");
        }
    }

    #[test]
    fn amazon_pay_and_amazon_do_not_collide() {
        assert_ne!(merchant_key("Amazon Pay"), merchant_key("Amazon"));
        assert_ne!(merchant_key("AMAZON PAY INDIA PVT LTD"), merchant_key("AMAZON INDIA"));
    }

    #[test]
    fn person_phone_vpas_have_no_key() {
        for raw in ["9876543210@ybl", "UPI/9876543210@okaxis", "919876543210@paytm", "9876543210"] {
            assert_eq!(merchant_key(raw), "", "input: {raw:?}");
        }
    }

    #[test]
    fn empty_and_all_noise_have_no_key() {
        for raw in ["", "   ", "UPI/", "INDIA PVT LTD", "POS 123456", "@ybl", "---", "ACH D-"] {
            assert_eq!(merchant_key(raw), "", "input: {raw:?}");
        }
    }

    proptest! {
        #[test]
        fn idempotent(s in any::<String>()) {
            let k = merchant_key(&s);
            prop_assert_eq!(merchant_key(&k), k);
        }

        #[test]
        fn idempotent_on_bank_like_text(s in "[A-Za-z0-9@/ .&_-]{0,40}") {
            let k = merchant_key(&s);
            prop_assert_eq!(merchant_key(&k), k);
        }

        #[test]
        fn output_is_tidy(s in any::<String>()) {
            let k = merchant_key(&s);
            // Some "uppercase" scripts (e.g. mathematical letters) have no lowercase form, so
            // the property is "lowercasing changes nothing".
            prop_assert_eq!(k.to_lowercase(), k.clone());
            prop_assert_eq!(k.trim(), k.as_str());
            prop_assert!(!k.contains("  "));
        }
    }
}
