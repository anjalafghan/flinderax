//! Reads Indian bank / card alert messages (SMS or email text) into a transaction, and picks a
//! category for a merchant. Pure: no DB, no clock.
//!
//! Formats differ by bank and change often, so this is deliberately heuristic: it looks for an
//! amount, a direction word, the last digits of the account/card, a merchant, a date and an
//! "available balance". Anything that looks like an OTP, a reminder or an offer is rejected.

use std::sync::OnceLock;

use regex::Regex;
use time::{Date, Month};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Debit,
    Credit,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Debit => "debit",
            Direction::Credit => "credit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    pub direction: Direction,
    pub amount_paise: i64,
    /// Last 3-4 digits of the account or card, as printed.
    pub last4: Option<String>,
    pub merchant: Option<String>,
    pub date: Option<Date>,
    /// "Avl Bal" after the transaction (bank accounts).
    pub balance_paise: Option<i64>,
    /// "Avl Limit" after the transaction (cards).
    pub available_limit_paise: Option<i64>,
    pub reference: Option<String>,
    /// A payment *towards* a credit card (a bill payment, not income or spending).
    pub card_payment: bool,
}

pub const CATEGORIES: &[&str] = &[
    "food", "groceries", "shopping", "transport", "fuel", "bills", "entertainment", "health", "travel", "rent",
    "emi", "cash", "transfer", "income", "other", "uncategorized",
];

/// Categories that are not spending.
pub fn is_spending(category: &str) -> bool {
    !matches!(category, "transfer" | "income")
}

fn re(cell: &'static OnceLock<Regex>, pattern: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("valid regex"))
}

macro_rules! regex {
    ($p:expr) => {{
        static CELL: OnceLock<Regex> = OnceLock::new();
        re(&CELL, $p)
    }};
}

const AMOUNT: &str = r"(?:rs\.?|inr|₹)\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)";

/// "1,00,000.50" -> 10000050 paise.
fn parse_amount(s: &str) -> Option<i64> {
    let clean: String = s.chars().filter(|c| *c != ',').collect();
    let (whole, frac) = clean.split_once('.').unwrap_or((&clean, ""));
    let whole: i64 = whole.parse().ok()?;
    let frac: i64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().ok()? * 10,
        _ => frac.get(..2)?.parse().ok()?,
    };
    whole.checked_mul(100)?.checked_add(frac)
}

fn month_from(s: &str) -> Option<Month> {
    let m = match s.get(..3)?.to_ascii_lowercase().as_str() {
        "jan" => 1, "feb" => 2, "mar" => 3, "apr" => 4, "may" => 5, "jun" => 6,
        "jul" => 7, "aug" => 8, "sep" => 9, "oct" => 10, "nov" => 11, "dec" => 12,
        _ => return None,
    };
    Month::try_from(m).ok()
}

fn year_from(s: &str) -> Option<i32> {
    let y: i32 = s.parse().ok()?;
    Some(if s.len() == 2 { 2000 + y } else { y })
}

fn find_date(text: &str) -> Option<Date> {
    let iso = regex!(r"\b(20\d{2})-(\d{2})-(\d{2})");
    if let Some(c) = iso.captures(text) {
        let m = Month::try_from(c[2].parse::<u8>().ok()?).ok()?;
        return Date::from_calendar_date(c[1].parse().ok()?, m, c[3].parse().ok()?).ok();
    }
    let named = regex!(r"(?i)\b(\d{1,2})[\s\-/]?(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*[\s\-/,]?\s?(\d{4}|\d{2})\b");
    if let Some(c) = named.captures(text) {
        return Date::from_calendar_date(year_from(&c[3])?, month_from(&c[2])?, c[1].parse().ok()?).ok();
    }
    let numeric = regex!(r"\b(\d{1,2})[\-/.](\d{1,2})[\-/.](\d{4}|\d{2})\b");
    if let Some(c) = numeric.captures(text) {
        let m = Month::try_from(c[2].parse::<u8>().ok()?).ok()?;
        return Date::from_calendar_date(year_from(&c[3])?, m, c[1].parse().ok()?).ok();
    }
    None
}

/// Strips UPI handles and noise: "swiggy.in@axisbank" -> "swiggy.in", "PAYTM-12345678@paytm" -> "PAYTM".
fn clean_merchant(raw: &str) -> Option<String> {
    let mut m = raw.trim().trim_matches(|c: char| c == '.' || c == ',' || c == '-' || c == ':' || c == ';').trim();
    if let Some((handle, _)) = m.split_once('@') {
        m = handle;
    }
    let m = regex!(r"[-_.]?\d{5,}$").replace(m, "");
    let m = regex!(r"\s+").replace_all(m.trim(), " ");
    let m = m.trim();
    let generic = ["your", "you", "a/c", "ac", "account", "vpa", "upi", "neft", "imps"];
    if m.len() < 2 || m.chars().all(|c| c.is_ascii_digit()) || generic.contains(&m.to_ascii_lowercase().as_str()) {
        None
    } else {
        Some(m.to_string())
    }
}

fn find_merchant(text: &str, direction: Direction) -> Option<String> {
    let stop = r"(?:\s+on\b|\s+ref(?:no)?\b|\s+rrn\b|\s*\(|\s*\.\s|\s*\.$|\s+upi\b|\s+avl\b|\s+not you|\s*;|,|$)";
    let debit_patterns = [
        format!(r"(?i)\bat\s+(.{{2,40}}?){stop}"),
        format!(r"(?i)\btrf to\s+(.{{2,40}}?){stop}"),
        format!(r"(?i)\bto\s+(?:vpa\s+)?(.{{2,40}}?){stop}"),
        format!(r"(?i)\bon\s+([A-Z][A-Za-z0-9&'.* \-]{{1,30}}?)\.\s*avl"),
        r"(?i);\s*([A-Za-z0-9&'.* \-]{2,40}?)\s+credited".to_string(),
        format!(r"(?i)\bfor\s+(?:upi|neft|imps)?[\s\-/]*(.{{2,40}}?){stop}"),
    ];
    let credit_patterns = [
        format!(r"(?i)\bfrom\s+(?:vpa\s+)?(.{{2,40}}?){stop}"),
        format!(r"(?i)\bby\s+(?:vpa\s+)?(.{{2,40}}?){stop}"),
        format!(r"(?i)\bfor\s+(?:neft|imps|rtgs|upi)\s*(?:cr)?[\s\-/]*(.{{2,40}}?){stop}"),
    ];
    let patterns: &[String] = if direction == Direction::Debit { &debit_patterns } else { &credit_patterns };
    for p in patterns {
        let r = Regex::new(p).expect("valid regex");
        for c in r.captures_iter(text) {
            let cand = &c[1];
            // "to HDFC Bank Card" / "from HDFC Bank A/c" describe the user's own instrument.
            if regex!(r"(?i)^(your\b|.*\b(a/?c|acct|account|card)\b)").is_match(cand) {
                continue;
            }
            if let Some(m) = clean_merchant(cand) {
                return Some(m);
            }
        }
    }
    None
}

/// Reads one alert. `None` when the text is not a completed transaction.
pub fn parse(raw: &str) -> Option<Alert> {
    let text = regex!(r"\s+").replace_all(raw, " ").trim().to_string();
    if text.is_empty() {
        return None;
    }
    let reject = regex!(
        r"(?i)\botp\b|one[\s-]?time[\s-]?password|verification code|\bwill be (debited|charged|deducted)|has requested|\brequest(ed)? (of|for)\b|\bdue (date|on|by)\b|minimum (amount )?due|\bpre-?approved\b|\boffer\b|\bapply now\b|\bfailed\b|\bdeclined\b|\bunsuccessful\b"
    );
    if reject.is_match(&text) {
        return None;
    }

    // Available balance / limit first, so their amounts aren't mistaken for the transaction.
    let avl = regex!(
        r"(?i)(?:avl\.?|available|avbl\.?|aval\.?)\s*(bal(?:ance)?|lmt|limit|cr\.? lmt|credit limit)\s*(?:is|:|-)?\s*(?:rs\.?|inr|₹)?\s*([0-9][0-9,]*(?:\.[0-9]{1,2})?)"
    );
    let mut balance = None;
    let mut limit = None;
    let mut skip_spans = Vec::new();
    for c in avl.captures_iter(&text) {
        let whole = c.get(0).expect("match");
        skip_spans.push(whole.range());
        let v = parse_amount(&c[2]);
        if c[1].to_ascii_lowercase().starts_with("bal") {
            balance = balance.or(v);
        } else {
            limit = limit.or(v);
        }
    }

    let amount_re = regex!(&format!("(?i){AMOUNT}"));
    let amount = amount_re
        .captures_iter(&text)
        .find(|c| {
            let r = c.get(0).expect("match").range();
            !skip_spans.iter().any(|s| s.contains(&r.start))
        })
        .and_then(|c| parse_amount(&c[1]))
        .or_else(|| {
            regex!(r"(?i)(?:debited|credited)\s+(?:by|for|with)\s+([0-9][0-9,]*(?:\.[0-9]{1,2})?)")
                .captures(&text)
                .and_then(|c| parse_amount(&c[1]))
        })?;
    if amount <= 0 {
        return None;
    }

    let debit = regex!(r"(?i)\b(debited|spent|sent|paid|withdrawn|withdrawal|purchase|txn of|transaction of|used)\b");
    let credit = regex!(r"(?i)\b(credited|received|deposited|refund(ed)?|reversed|reversal|cashback)\b");
    let first = |r: &Regex| r.find(&text).map(|m| m.start());
    let direction = match (first(debit), first(credit)) {
        (Some(d), Some(c)) => if d <= c { Direction::Debit } else { Direction::Credit },
        (Some(_), None) => Direction::Debit,
        (None, Some(_)) => Direction::Credit,
        (None, None) => return None,
    };

    let card_payment = direction == Direction::Credit
        && regex!(r"(?i)(payment|paid).{0,60}(towards|to|on|for) (your )?.{0,30}credit card|credit card.{0,40}payment.{0,20}received")
            .is_match(&text);

    let last4 = regex!(r"(?i)(?:\bx+|\*+|\bending\s+(?:in\s+)?|\bcard\s+(?:no\.?\s*)?|\ba/?c\s+(?:no\.?\s*)?)(\d{3,4})\b")
        .captures(&text)
        .map(|c| c[1].to_string());

    let reference = regex!(r"(?i)\b(?:upi\s*ref(?:\s*no)?|ref(?:\s*no)?|refno|rrn|upi)[\s:.#\-]*(\d{6,})")
        .captures(&text)
        .map(|c| c[1].to_string());

    Some(Alert {
        direction,
        amount_paise: amount,
        last4,
        merchant: if card_payment { None } else { find_merchant(&text, direction) },
        date: find_date(&text),
        balance_paise: balance,
        available_limit_paise: limit,
        reference,
        card_payment,
    })
}

/// Lower-case letters and digits separated by single spaces: the form rules are matched on.
pub fn normalize(merchant: &str) -> String {
    let mut out = String::new();
    for word in merchant.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
        if !word.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(word);
        }
    }
    out
}

const DEFAULT_RULES: &[(&str, &[&str])] = &[
    ("transfer", &["credit card", "cc payment", "cred club", "cred"]),
    ("food", &["swiggy", "zomato", "domino", "mcdonald", "kfc", "starbucks", "pizza", "burger", "cafe", "restaurant", "eatsure", "box8", "chaayos", "haldiram"]),
    ("groceries", &["blinkit", "zepto", "bigbasket", "instamart", "dmart", "jiomart", "grofers", "more retail", "nature s basket", "ratnadeep", "milkbasket", "kirana", "supermarket"]),
    ("shopping", &["amazon", "flipkart", "myntra", "ajio", "nykaa", "meesho", "croma", "reliance digital", "decathlon", "ikea", "tata cliq", "lenskart"]),
    ("transport", &["uber", "ola", "rapido", "namma yatri", "blusmart", "metro", "fastag", "redbus"]),
    ("fuel", &["petrol", "fuel", "hpcl", "bpcl", "iocl", "indian oil", "shell", "hp pay"]),
    ("bills", &["airtel", "jio", "vodafone", "bescom", "electricity", "tata power", "broadband", "act fibernet", "bbps", "recharge", "gas", "water", "insurance", "lic"]),
    ("entertainment", &["netflix", "spotify", "hotstar", "prime video", "bookmyshow", "pvr", "inox", "youtube", "apple com", "sonyliv", "zee5"]),
    ("health", &["pharmacy", "medical", "chemist", "apollo", "medplus", "1mg", "pharmeasy", "netmeds", "hospital", "clinic", "practo", "cult", "diagnostic"]),
    ("travel", &["makemytrip", "goibibo", "irctc", "indigo", "air india", "vistara", "akasa", "cleartrip", "yatra", "oyo", "airbnb", "ixigo"]),
    ("cash", &["atm", "cash withdrawal"]),
    ("emi", &["emi", "loan"]),
];

/// The category for a transaction: the user's longest matching rule, then the built-in list.
pub fn categorize(merchant: Option<&str>, direction: Direction, card_payment: bool, user_rules: &[(String, String)]) -> String {
    if card_payment {
        return "transfer".into();
    }
    if let Some(m) = merchant {
        let norm = normalize(m);
        let padded = format!(" {norm} ");
        let contains = |pat: &str| padded.contains(&format!(" {pat} ")) || (pat.len() >= 5 && norm.contains(pat));
        if let Some((_, cat)) = user_rules.iter().filter(|(p, _)| contains(p)).max_by_key(|(p, _)| p.len()) {
            return cat.clone();
        }
        if direction == Direction::Debit {
            for (cat, pats) in DEFAULT_RULES {
                if pats.iter().any(|p| contains(p)) {
                    return (*cat).into();
                }
            }
        }
    }
    match direction {
        Direction::Credit => "income".into(),
        Direction::Debit => "uncategorized".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn p(s: &str) -> Alert {
        parse(s).unwrap_or_else(|| panic!("did not parse: {s}"))
    }

    #[test]
    fn hdfc_card_spend() {
        let a = p("Spent Rs.450.00 On HDFC Bank Card 1234 At SWIGGY On 2026-11-01:13:45:12 Not You? Call 18002586161/SMS BLOCK CC 1234 to 7308080808");
        assert_eq!(a.direction, Direction::Debit);
        assert_eq!(a.amount_paise, 45_000);
        assert_eq!(a.last4.as_deref(), Some("1234"));
        assert_eq!(a.merchant.as_deref(), Some("SWIGGY"));
        assert_eq!(a.date, Some(date!(2026 - 11 - 01)));
    }

    #[test]
    fn hdfc_upi_sent_multiline() {
        let a = p("Sent Rs.250.00\nFrom HDFC Bank A/C *1234\nTo RAJU STORE\nOn 01/11/26\nRef 430512345678\nNot You?\nCall 18002586161/SMS BLOCK UPI to 7308080808");
        assert_eq!((a.direction, a.amount_paise), (Direction::Debit, 25_000));
        assert_eq!(a.last4.as_deref(), Some("1234"));
        assert_eq!(a.merchant.as_deref(), Some("RAJU STORE"));
        assert_eq!(a.date, Some(date!(2026 - 11 - 01)));
        assert_eq!(a.reference.as_deref(), Some("430512345678"));
    }

    #[test]
    fn hdfc_vpa_debit() {
        let a = p("Rs.5000.00 debited from a/c **1234 on 01-11-26 to VPA swiggy.in@axisbank (UPI Ref No 430512345678). Not you? Call 18002586161");
        assert_eq!(a.amount_paise, 500_000);
        assert_eq!(a.merchant.as_deref(), Some("swiggy.in"));
        assert_eq!(categorize(a.merchant.as_deref(), a.direction, false, &[]), "food");
    }

    #[test]
    fn hdfc_salary_with_balance() {
        let a = p("Update! INR 1,00,000.00 deposited in HDFC Bank A/c XX1234 on 30-OCT-26 for NEFT Cr-ACME CORP PVT LTD. Avl bal INR 1,25,000.50. Cheque deposits in A/C are subject to clearing");
        assert_eq!((a.direction, a.amount_paise), (Direction::Credit, 10_000_000));
        assert_eq!(a.balance_paise, Some(12_500_050));
        assert_eq!(a.date, Some(date!(2026 - 10 - 30)));
        assert_eq!(a.last4.as_deref(), Some("1234"));
        assert_eq!(categorize(a.merchant.as_deref(), a.direction, false, &[]), "income");
    }

    #[test]
    fn kotak_sent_and_received() {
        let a = p("Sent Rs.120.00 from Kotak Bank AC X1234 to merchant.zepto@ybl on 01-11-26.UPI Ref 430512345678. Not you, https://kotak.com/KBANKT/Fraud");
        assert_eq!((a.direction, a.amount_paise), (Direction::Debit, 12_000));
        assert_eq!(a.merchant.as_deref(), Some("merchant.zepto"));
        assert_eq!(categorize(a.merchant.as_deref(), a.direction, false, &[]), "groceries");
        let b = p("Received Rs.500.00 in your Kotak Bank AC X1234 from friend@okaxis on 01-11-26.UPI Ref:430512345679.");
        assert_eq!((b.direction, b.amount_paise), (Direction::Credit, 50_000));
        assert_eq!(b.merchant.as_deref(), Some("friend"));
    }

    #[test]
    fn icici_card_with_limit() {
        let a = p("INR 1,299.00 spent using ICICI Bank Card XX1234 on 01-Nov-26 on AMAZON. Avl Limit: INR 98,701.00. If not you, call 1800 2662/SMS BLOCK 1234 to 9215676766");
        assert_eq!((a.direction, a.amount_paise), (Direction::Debit, 129_900));
        assert_eq!(a.available_limit_paise, Some(9_870_100));
        assert_eq!(a.balance_paise, None);
        assert_eq!(a.merchant.as_deref(), Some("AMAZON"));
        assert_eq!(a.date, Some(date!(2026 - 11 - 01)));
    }

    #[test]
    fn icici_account_debit() {
        let a = p("ICICI Bank Acct XX234 debited for Rs 500.00 on 01-Nov-26; ZOMATO credited. UPI:430512345678. Call 18002662 for dispute. SMS BLOCK 234 to 9215676766.");
        assert_eq!((a.direction, a.amount_paise), (Direction::Debit, 50_000));
        assert_eq!(a.last4.as_deref(), Some("234"));
        assert_eq!(a.merchant.as_deref(), Some("ZOMATO"));
    }

    #[test]
    fn sbi_formats() {
        let a = p("Dear UPI user A/C X1234 debited by 250.0 on date 01Nov26 trf to SWIGGY Refno 430512345678. If not u? call 1800111109. -SBI");
        assert_eq!((a.direction, a.amount_paise), (Direction::Debit, 25_000));
        assert_eq!(a.merchant.as_deref(), Some("SWIGGY"));
        assert_eq!(a.date, Some(date!(2026 - 11 - 01)));
        let b = p("Dear SBI User, your A/c X1234-credited by Rs.5000 on 01Nov26 transfer from JOHN DOE Ref No 430512345678 -SBI");
        assert_eq!((b.direction, b.amount_paise), (Direction::Credit, 500_000));
        assert_eq!(b.merchant.as_deref(), Some("JOHN DOE"));
    }

    #[test]
    fn axis_card() {
        let a = p("INR 750.00 spent on Axis Bank Card no. XX1234 at UBER INDIA on 01-11-26 10:22:11 IST. Avl Limit: INR 49,250.00. Not you? SMS BLOCK 1234 to 919951860002");
        assert_eq!(a.merchant.as_deref(), Some("UBER INDIA"));
        assert_eq!(categorize(a.merchant.as_deref(), a.direction, false, &[]), "transport");
        assert_eq!(a.last4.as_deref(), Some("1234"));
    }

    #[test]
    fn card_bill_payment_is_a_transfer() {
        let a = p("Payment of Rs 15,000.00 received towards your HDFC Bank Credit Card ending 1234 on 05-11-2026. Thank you.");
        assert_eq!(a.direction, Direction::Credit);
        assert!(a.card_payment);
        assert_eq!(categorize(a.merchant.as_deref(), a.direction, a.card_payment, &[]), "transfer");
        assert_eq!(a.last4.as_deref(), Some("1234"));
    }

    #[test]
    fn rejects_non_transactions() {
        for s in [
            "123456 is your OTP for txn of Rs 500.00 at AMAZON on HDFC Bank Card 1234. Valid for 10 mins.",
            "Get a pre-approved personal loan of Rs 5,00,000 at 10.5%. Apply now!",
            "Your HDFC Bank Credit Card 1234 statement: Total due Rs 12,000, minimum due Rs 600, due by 15-Nov-26",
            "abc@okicici has requested money of Rs 200 from you on Google Pay.",
            "Rs 499 will be debited from your account on 05-11-26 for Netflix mandate.",
            "Txn of Rs 2000 on card 1234 declined due to insufficient limit.",
            "Hello, how are you?",
        ] {
            assert!(parse(s).is_none(), "should reject: {s}");
        }
    }

    #[test]
    fn amounts() {
        assert_eq!(parse_amount("1,00,000.5"), Some(10_000_050));
        assert_eq!(parse_amount("12"), Some(1200));
        assert_eq!(parse_amount("12.345"), Some(1234));
    }

    #[test]
    fn user_rules_win_and_longest_match_wins() {
        let rules = vec![("raju".to_string(), "groceries".to_string()), ("raju store".to_string(), "food".to_string())];
        assert_eq!(categorize(Some("RAJU STORE"), Direction::Debit, false, &rules), "food");
        assert_eq!(categorize(Some("Amazon Pay"), Direction::Debit, false, &[("amazon pay".into(), "bills".into())]), "bills");
        assert_eq!(categorize(Some("Unknown Shop"), Direction::Debit, false, &[]), "uncategorized");
        assert_eq!(categorize(Some("RAJU KIRANA STORE"), Direction::Debit, false, &[]), "groceries");
        assert_eq!(categorize(Some("sharma.medicals"), Direction::Debit, false, &[]), "health");
        // Short patterns only match whole words: "ola" must not match "Coca Cola".
        assert_eq!(categorize(Some("Coca Cola"), Direction::Debit, false, &[]), "uncategorized");
    }
}
