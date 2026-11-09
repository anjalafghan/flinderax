//! Bank statement narrations -> structured fields. Pure: no DB, no clock.
//!
//! A narration is the free-text column of a bank statement. It names the rail (UPI, NEFT, IMPS,
//! ...) and the counterparty. For UPI it may also carry the VPA handle, the bank (IFSC prefix), the
//! 12-digit RRN and an app tag ("PAID VIA CRED"). The app tag is never the merchant.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rail { Upi, Ach, Neft, Imps, Rtgs, Salary, Interest, FdClosure, CardFee, Atm, Internal, Other }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Narration {
    pub rail: Rail,
    pub payee: Option<String>,
    pub vpa: Option<String>,
    pub vpa_masked: bool,
    pub handle: Option<String>,
    pub bank: Option<String>,
    pub rrn: Option<String>,
    pub app: Option<&'static str>,
    pub remark: Option<String>,
    pub autopay: bool,
}

/// Reads one statement narration. Never fails: anything unrecognised is `Rail::Other`.
pub fn parse(raw: &str) -> Narration {
    let up = raw.trim().to_ascii_uppercase();
    let rail = detect_rail(&up);
    let mut n = Narration {
        rail,
        payee: None,
        vpa: None,
        vpa_masked: false,
        handle: None,
        bank: None,
        rrn: None,
        app: None,
        remark: None,
        autopay: false,
    };
    match rail {
        Rail::Upi => parse_upi(&up[4..], &mut n),
        Rail::Internal => {
            let tail: Vec<&str> = split_segments(&up[15..]).into_iter().filter(|s| *s != "TPT").collect();
            parse_tail(&tail, &mut n);
        }
        Rail::Ach => parse_ach(&up, &mut n),
        Rail::Neft | Rail::Imps | Rail::Rtgs => parse_transfer(&up, &mut n),
        Rail::Other => n.payee = clean_name(&up),
        Rail::Salary | Rail::Interest | Rail::FdClosure | Rail::CardFee | Rail::Atm => {}
    }
    if n.rrn.is_none() {
        n.rrn = rrn_of(&up);
    }
    n
}

/// Extracts a 12-digit RRN from any text (SMS/email alerts too): first standalone 12-digit run.
pub fn rrn_of(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_digit()).find(|t| t.len() == 12).map(str::to_string)
}

/// Names compared ignoring case, spacing and truncation (HDFC cuts payee names to ~23 chars):
/// true when one is a prefix of the other after normalising and both have >= 2 words.
pub fn same_person(a: &str, b: &str) -> bool {
    let (wa, wb) = (person_words(a), person_words(b));
    if wa.len() < 2 || wb.len() < 2 {
        return false;
    }
    let (ca, cb) = (wa.concat(), wb.concat());
    let (short, long) = if ca.len() <= cb.len() { (ca, cb) } else { (cb, ca) };
    short.len() >= 4 && long.starts_with(short.as_str())
}

pub struct Owner<'a> {
    pub holder_names: &'a [String],
}

/// Category suggestion from structure, before user rules / merchant keywords:
/// - payee matches a holder name (same_person) => "transfer" (own account), both directions
/// - Rail::Imps/Neft with "SELF" => "transfer"
/// - Rail::Ach with "MF" / "MUTUAL" / "BD-MF" => "investments"
/// - Rail::Salary => "income"; Rail::Interest => "income"
/// - Rail::FdClosure => "transfer" (money back from a deposit)
/// - Rail::CardFee => "fees"; Rail::Atm => "cash"
/// - credit whose narration says REFUND/REVERSAL/REVERSED/CASHBACK => "refund"
/// - otherwise None (fall through to user rules + keyword rules on `payee`)
pub fn structural_category(n: &Narration, credit: bool, owner: &Owner) -> Option<&'static str> {
    if n.payee.as_deref().is_some_and(|p| owner.holder_names.iter().any(|h| same_person(p, h))) {
        return Some("transfer");
    }
    let self_transfer = n.remark.as_deref() == Some("SELF");
    match n.rail {
        Rail::Imps | Rail::Neft if self_transfer => return Some("transfer"),
        Rail::FdClosure => return Some("transfer"),
        Rail::Ach if payee_is_mutual_fund(n) => return Some("investments"),
        Rail::Salary | Rail::Interest => return Some("income"),
        _ => {}
    }
    if credit && has_refund_word(n) {
        return Some("refund");
    }
    match n.rail {
        Rail::CardFee => Some("fees"),
        Rail::Atm => Some("cash"),
        _ => None,
    }
}

// ---- rail detection -------------------------------------------------------------------------

fn detect_rail(up: &str) -> Rail {
    if is_internal(up) {
        Rail::Internal
    } else if up.starts_with("UPI-") {
        Rail::Upi
    } else if word_start(up, "ACH") {
        Rail::Ach
    } else if word_start(up, "NEFT") {
        Rail::Neft
    } else if word_start(up, "IMPS") {
        Rail::Imps
    } else if word_start(up, "RTGS") {
        Rail::Rtgs
    } else if up.starts_with("SALARY") {
        Rail::Salary
    } else if up.starts_with("INTEREST PAID") {
        Rail::Interest
    } else if up.starts_with("FD PREMAT") || up.starts_with("IB FD") {
        Rail::FdClosure
    } else if is_card_fee(up) {
        Rail::CardFee
    } else if ["ATM", "NWD", "ATW"].iter().any(|w| word_start(up, w)) {
        Rail::Atm
    } else {
        Rail::Other
    }
}

/// A 14-digit account number, then "-" (usually "-TPT-").
fn is_internal(up: &str) -> bool {
    let b = up.as_bytes();
    b.len() >= 15 && b[..14].iter().all(u8::is_ascii_digit) && b[14] == b'-'
}

fn word_start(up: &str, word: &str) -> bool {
    up.strip_prefix(word).is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_alphanumeric()))
}

fn is_card_fee(up: &str) -> bool {
    up.starts_with("ANNUAL FEE")
        || up.starts_with("ISSUANCE")
        || (up.starts_with("DEBIT CARD") && ["FEE", "ISSUANCE", "ANNUAL", "CHARGE", "GST"].iter().any(|w| up.contains(w)))
}

// ---- per-rail parsing -----------------------------------------------------------------------

/// `body` is the narration after "UPI-". Layout: payee - [extra -] vpa - [ifsc] - [rrn] - remark.
fn parse_upi(body: &str, n: &mut Narration) {
    let segs = split_segments(body);
    let tail_from = match segs.iter().position(|s| s.contains('@')) {
        Some(i) => {
            let vpa = segs[i];
            n.vpa_masked = vpa.contains('*');
            n.vpa = Some(vpa.to_ascii_lowercase());
            if let Some((_, handle)) = vpa.split_once('@') {
                n.handle = Some(handle.to_ascii_lowercase());
                n.app = handle_app(handle);
            }
            // Anything between the payee and the VPA (PAYTM-1234567@PAYTM) is a prefix, not a name.
            if i >= 1 {
                n.payee = clean_name(segs[0]);
            }
            i + 1
        }
        None => {
            n.payee = segs.first().and_then(|s| clean_name(s));
            1
        }
    };
    parse_tail(segs.get(tail_from..).unwrap_or(&[]), n);
}

/// Bank code / RRN / remark, in any order, after the VPA (UPI) or the account number (internal).
fn parse_tail(segs: &[&str], n: &mut Narration) {
    let mut rest = Vec::new();
    for &s in segs {
        if is_ifsc(s) {
            if n.bank.is_none() {
                n.bank = Some(s[..4].to_string());
            }
        } else if is_rrn(s) && n.rrn.is_none() {
            n.rrn = Some(s.to_string());
        } else {
            rest.push(s);
        }
    }
    apply_remark(&rest.join("-"), n);
}

/// "ACH D- BD-MF UTILITIES LUMP-TJAC12345678": the payee is the text, minus the trailing reference.
fn parse_ach(up: &str, n: &mut Narration) {
    let rest = up
        .strip_prefix("ACH D-")
        .or_else(|| up.strip_prefix("ACH C-"))
        .or_else(|| up.strip_prefix("ACH"))
        .unwrap_or(up);
    let mut parts = split_segments(rest);
    if parts.len() > 1 && parts.last().is_some_and(|l| is_reference(l)) {
        parts.pop();
    }
    n.payee = clean_name(&parts.join("-"));
}

/// NEFT / IMPS / RTGS: skip the rail word, then take the name, the bank code and "SELF".
fn parse_transfer(up: &str, n: &mut Narration) {
    let mut rest = Vec::new();
    for s in split_segments(up).into_iter().skip(1) {
        if is_ifsc(s) {
            if n.bank.is_none() {
                n.bank = Some(s[..4].to_string());
            }
            continue;
        }
        if is_rrn(s) {
            if n.rrn.is_none() {
                n.rrn = Some(s.to_string());
            }
            continue;
        }
        if s == "SELF" {
            rest.push(s);
            continue;
        }
        if s.contains('*') || s.contains("XXX") || s.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        if n.payee.is_none() && s.chars().any(char::is_alphabetic) {
            n.payee = clean_name(s);
            continue;
        }
        if n.bank.is_none() && n.payee.is_some() && s.len() == 4 && s.bytes().all(|b| b.is_ascii_alphabetic()) {
            n.bank = Some(s.to_string());
            continue;
        }
        rest.push(s);
    }
    apply_remark(&rest.join("-"), n);
}

/// Sets the app, the autopay flag or the free-text remark from the trailing remark.
fn apply_remark(text: &str, n: &mut Narration) {
    let t = collapse(text);
    if t.is_empty() {
        return;
    }
    if let Some(app) = app_tag(&t) {
        n.app = Some(app);
        return;
    }
    if t.contains("SUBSCRIPTION DEBIT") {
        n.autopay = true;
        return;
    }
    let reference = t == "UPI" || t.starts_with("OIDORD") || t.bytes().all(|b| b.is_ascii_digit());
    if !reference {
        n.remark = Some(t);
    }
}

fn app_tag(t: &str) -> Option<&'static str> {
    if t == "CREDUSER" || t.ends_with(" VIA CRED") || t.ends_with(" ON CRED") {
        Some("CRED")
    } else if t.starts_with("SENT USING PAYTM") {
        Some("Paytm")
    } else if t.starts_with("PAYMENT FROM PHONE") {
        Some("PhonePe")
    } else if t.starts_with("SENT VIA JUPITER") {
        Some("Jupiter")
    } else if t.starts_with("PAID VIA SUPERMONE") {
        Some("Supermoney")
    } else if t.starts_with("PAYMENT FROM AIRTE") {
        Some("Airtel")
    } else {
        None
    }
}

fn handle_app(handle: &str) -> Option<&'static str> {
    match handle.to_ascii_lowercase().as_str() {
        "yescred" => Some("CRED"),
        "paytm" | "pty" | "ptys" | "ptyes" | "pthdfc" | "ptsbi" => Some("Paytm"),
        _ => None,
    }
}

// ---- helpers --------------------------------------------------------------------------------

fn split_segments(s: &str) -> Vec<&str> {
    s.split('-').map(str::trim).filter(|x| !x.is_empty()).collect()
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clean_name(s: &str) -> Option<String> {
    let c = collapse(s);
    if c.chars().any(char::is_alphabetic) { Some(c) } else { None }
}

/// IFSC: 4 letters, "0", 6 alphanumerics. The first four letters are the bank code.
fn is_ifsc(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 11 && b[..4].iter().all(u8::is_ascii_alphabetic) && b[4] == b'0' && b[5..].iter().all(u8::is_ascii_alphanumeric)
}

fn is_rrn(s: &str) -> bool {
    s.len() == 12 && s.bytes().all(|b| b.is_ascii_digit())
}

/// A bare reference token such as "TJAC12345678": no spaces, 8+ chars, at least one digit.
fn is_reference(s: &str) -> bool {
    s.len() >= 8 && !s.contains(' ') && s.bytes().any(|b| b.is_ascii_digit())
}

fn person_words(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|w| w.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_uppercase())
        .filter(|w| !w.is_empty())
        .collect()
}

fn payee_is_mutual_fund(n: &Narration) -> bool {
    n.payee.as_deref().is_some_and(|p| p.split(|c: char| !c.is_ascii_alphanumeric()).any(|t| t == "MF" || t == "MUTUAL"))
}

fn has_refund_word(n: &Narration) -> bool {
    let text = format!("{} {}", n.payee.as_deref().unwrap_or(""), n.remark.as_deref().unwrap_or(""));
    text.split(|c: char| !c.is_ascii_alphanumeric()).any(|t| {
        matches!(t.to_ascii_uppercase().as_str(), "REFUND" | "REFUNDS" | "REVERSAL" | "REVERSED" | "CASHBACK")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holders(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn every_rail_is_detected() {
        let cases: &[(&str, Rail)] = &[
            ("UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED", Rail::Upi),
            ("ACH D- BD-MF UTILITIES LUMP-TJAC12345678", Rail::Ach),
            ("ACH C-SAMPLE CORP-SC123456789", Rail::Ach),
            ("NEFT CR-KKBK0001234-PRIYA SHARMA-SAMPLE", Rail::Neft),
            ("IMPS-612345678908-PRIYA SHARMA-KKBK-XXXXXXX1234-SELF", Rail::Imps),
            ("RTGS-SBIN0005678-ASHA STORES", Rail::Rtgs),
            ("SALARYFORTHEMONTHOFOCT2026", Rail::Salary),
            ("INTEREST PAID TILL 30-SEP-2026", Rail::Interest),
            ("IB FD PREMAT CLOSURE 50012345678901", Rail::FdClosure),
            ("DEBIT CARD ANNUAL FEE", Rail::CardFee),
            ("ATM CASH WDL 0001 MAIN ROAD", Rail::Atm),
            ("NWD-412345XXXX1234", Rail::Atm),
            ("50100123456789-TPT-RENT", Rail::Internal),
            ("POS 412345 AMAZON", Rail::Other),
        ];
        for (raw, rail) in cases {
            assert_eq!(parse(raw).rail, *rail, "rail for {raw}");
        }
    }

    #[test]
    fn upi_cred_payment_fields() {
        let n = parse("UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED");
        assert_eq!(n.rail, Rail::Upi);
        assert_eq!(n.payee.as_deref(), Some("RAVI KUMAR"));
        assert_eq!(n.vpa.as_deref(), Some("ravik123@okaxis"));
        assert!(!n.vpa_masked);
        assert_eq!(n.handle.as_deref(), Some("okaxis"));
        assert_eq!(n.bank.as_deref(), Some("SBIN"));
        assert_eq!(n.rrn.as_deref(), Some("612345678901"));
        assert_eq!(n.app, Some("CRED"));
        assert_eq!(n.remark, None);
        assert!(!n.autopay);
    }

    #[test]
    fn upi_plain_handle_and_upi_remark_gives_no_app_or_remark() {
        let n = parse("UPI-ASHA STORES-Q123456789@YBL-YESB0YBLUPI-612345678902-UPI");
        assert_eq!(n.payee.as_deref(), Some("ASHA STORES"));
        assert_eq!(n.handle.as_deref(), Some("ybl"));
        assert_eq!(n.bank.as_deref(), Some("YESB"));
        assert_eq!(n.rrn.as_deref(), Some("612345678902"));
        assert_eq!(n.app, None);
        assert_eq!(n.remark, None);
    }

    #[test]
    fn upi_payee_with_handle_cred_prefix_and_dotted_vpa() {
        let n = parse("UPI-CREDPAYSWIGGY1-CREDPAY.SWIGGY1@AXISB-UTIB0000114-612345678903-PAYMENT ON CRED");
        assert_eq!(n.payee.as_deref(), Some("CREDPAYSWIGGY1"));
        assert_eq!(n.vpa.as_deref(), Some("credpay.swiggy1@axisb"));
        assert_eq!(n.bank.as_deref(), Some("UTIB"));
        assert_eq!(n.app, Some("CRED"));
    }

    #[test]
    fn masked_vpa_is_flagged() {
        let n = parse("UPI-RAVI KUMAR-*********K123@OKAXIS-SBIN0001234-612345678907-PAID VIA CRED");
        assert!(n.vpa_masked);
        assert_eq!(n.vpa.as_deref(), Some("*********k123@okaxis"));
        assert_eq!(n.payee.as_deref(), Some("RAVI KUMAR"));
        assert_eq!(n.app, Some("CRED"));
    }

    #[test]
    fn paytm_extra_segment_before_vpa_is_skipped() {
        let n = parse("UPI-HELPFUND ONLINE VENTURE-PAYTM-1234567@PAYTM-YESB0PTMUPI-612345678906-OIDORD1234567890");
        assert_eq!(n.payee.as_deref(), Some("HELPFUND ONLINE VENTURE"));
        assert_eq!(n.vpa.as_deref(), Some("1234567@paytm"));
        assert_eq!(n.handle.as_deref(), Some("paytm"));
        assert_eq!(n.app, Some("Paytm"));
        assert_eq!(n.bank.as_deref(), Some("YESB"));
        assert_eq!(n.rrn.as_deref(), Some("612345678906"));
        assert_eq!(n.remark, None, "OIDORD is a reference, not a remark");
    }

    #[test]
    fn upi_autopay_subscription_debit() {
        let n = parse("UPI-STREAMCO-STREAMCO@YBL-YESB0YBLUPI-612345678905-SUBSCRIPTION DEBIT");
        assert!(n.autopay);
        assert_eq!(n.app, None);
        assert_eq!(n.remark, None);
        assert_eq!(n.payee.as_deref(), Some("STREAMCO"));
    }

    #[test]
    fn remark_free_text_is_kept_and_app_tags_are_not() {
        let free = parse("UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-FOR WATCH");
        assert_eq!(free.remark.as_deref(), Some("FOR WATCH"));
        assert_eq!(free.app, None);
        let tagged = parse("UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678901-PAYMENT FROM PHONE");
        assert_eq!(tagged.app, Some("PhonePe"));
        assert_eq!(tagged.remark, None);
    }

    #[test]
    fn app_tags_map_to_apps() {
        let cases = [
            ("UPI-A B-AB@YESCRED-KKBK0001234-612345678950-CREDUSER", "CRED"),
            ("UPI-A B-AB@PTYES-KKBK0001234-612345678951-SENT USING PAYTM U", "Paytm"),
            ("UPI-A B-AB@OKICICI-KKBK0001234-612345678952-SENT VIA JUPITER", "Jupiter"),
            ("UPI-A B-AB@OKICICI-KKBK0001234-612345678953-PAID VIA SUPERMONE", "Supermoney"),
            ("UPI-A B-AB@OKICICI-KKBK0001234-612345678954-PAYMENT FROM AIRTE", "Airtel"),
        ];
        for (raw, app) in cases {
            assert_eq!(parse(raw).app, Some(app), "app for {raw}");
        }
    }

    #[test]
    fn payee_is_normalised() {
        let n = parse("UPI-   ravi    kumar -RAVIK123@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED");
        assert_eq!(n.payee.as_deref(), Some("RAVI KUMAR"));
    }

    #[test]
    fn rrn_found_in_sms_text() {
        let sms = "Rs.450.00 debited from A/c XX1234 on 01-11-26. UPI Ref 612345678901 to ASHA STORES. Avl bal Rs.10,000";
        assert_eq!(rrn_of(sms).as_deref(), Some("612345678901"));
        assert_eq!(rrn_of("A/c XXXX1234 ref 12345 and account 50100123456789"), None);
    }

    #[test]
    fn same_person_ignores_case_spacing_and_truncation() {
        assert!(same_person("PRIYA SHARMA KUMARI GUP", "Priya   Sharma Kumari Gupta"));
        assert!(same_person("priya sharma", "PRIYA SHARMA"));
        assert!(!same_person("PRIYA SHARMA", "ASHA SHARMA"));
        assert!(!same_person("RAVI KUMAR", "RAVIKUMAR"), "a single word is not a name match");
        assert!(!same_person("RAVI", "RAVI KUMAR"));
    }

    #[test]
    fn own_transfer_is_transfer_both_directions() {
        let holder = holders(&["Priya Sharma"]);
        let owner = Owner { holder_names: &holder };
        let n = parse("UPI-PRIYA SHARMA-98XXXXXX12@YESCRED-KKBK0001234-612345678904-PAID VIA CRED");
        assert_eq!(structural_category(&n, false, &owner), Some("transfer"));
        assert_eq!(structural_category(&n, true, &owner), Some("transfer"));
    }

    #[test]
    fn imps_self_is_transfer_without_holder_names() {
        let owner = Owner { holder_names: &[] };
        let n = parse("IMPS-612345678908-PRIYA SHARMA-KKBK-XXXXXXX1234-SELF");
        assert_eq!(n.payee.as_deref(), Some("PRIYA SHARMA"));
        assert_eq!(n.bank.as_deref(), Some("KKBK"));
        assert_eq!(n.rrn.as_deref(), Some("612345678908"));
        assert_eq!(n.remark.as_deref(), Some("SELF"));
        assert_eq!(structural_category(&n, false, &owner), Some("transfer"));
    }

    #[test]
    fn ach_mutual_fund_is_investments() {
        let owner = Owner { holder_names: &[] };
        let n = parse("ACH D- BD-MF UTILITIES LUMP-TJAC12345678");
        assert_eq!(n.rail, Rail::Ach);
        assert_eq!(n.payee.as_deref(), Some("BD-MF UTILITIES LUMP"));
        assert_eq!(structural_category(&n, false, &owner), Some("investments"));
    }

    #[test]
    fn salary_and_interest_are_income() {
        let owner = Owner { holder_names: &[] };
        let salary = parse("SALARYFORTHEMONTHOFOCT2026");
        assert_eq!(structural_category(&salary, true, &owner), Some("income"));
        let interest = parse("INTEREST PAID TILL 30-SEP-2026");
        assert_eq!(structural_category(&interest, true, &owner), Some("income"));
    }

    #[test]
    fn fd_closure_is_transfer() {
        let owner = Owner { holder_names: &[] };
        let n = parse("IB FD PREMAT CLOSURE 50012345678901");
        assert_eq!(n.rail, Rail::FdClosure);
        assert_eq!(structural_category(&n, true, &owner), Some("transfer"));
    }

    #[test]
    fn card_fee_is_fees() {
        let owner = Owner { holder_names: &[] };
        let n = parse("DEBIT CARD ANNUAL FEE");
        assert_eq!(structural_category(&n, false, &owner), Some("fees"));
        let issuance = parse("DEBIT CARD ISSUANCE FEE");
        assert_eq!(structural_category(&issuance, false, &owner), Some("fees"));
    }

    #[test]
    fn refund_credit_is_refund_but_debit_is_not() {
        let owner = Owner { holder_names: &[] };
        let n = parse("UPI-AMAZON-AMAZON@AXISBANK-UTIB0000114-612345678910-REFUND");
        assert_eq!(structural_category(&n, true, &owner), Some("refund"));
        assert_eq!(structural_category(&n, false, &owner), None);
    }

    #[test]
    fn normal_shop_payment_has_no_structural_category() {
        let holder = holders(&["Priya Sharma"]);
        let owner = Owner { holder_names: &holder };
        let upi = parse("UPI-ASHA STORES-Q123456789@YBL-YESB0YBLUPI-612345678902-UPI");
        assert_eq!(structural_category(&upi, false, &owner), None);
        let pos = parse("POS 412345 AMAZON");
        assert_eq!(pos.payee.as_deref(), Some("POS 412345 AMAZON"));
        assert_eq!(structural_category(&pos, false, &owner), None);
    }

    #[test]
    fn internal_transfer_keeps_remark_and_no_payee() {
        let n = parse("50100123456789-TPT-RENT");
        assert_eq!(n.rail, Rail::Internal);
        assert_eq!(n.payee, None);
        assert_eq!(n.remark.as_deref(), Some("RENT"));
    }
}
