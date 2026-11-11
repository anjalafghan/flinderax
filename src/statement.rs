//! Bank / card statement exports -> rows. Pure: no database. Banks differ in column names,
//! date formats, whether debits and credits share a column, and how a card marks payments. The
//! header row is found by looking for a date column plus an amount column. Where that fails, a
//! `Mapping` the user chose (and maybe saved for the layout) says which column is which.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::{Date, Month};

use crate::alerts::Direction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementRow {
    pub date: Date,
    pub direction: Direction,
    pub amount_paise: i64,
    pub narration: String,
    pub reference: Option<String>,
    pub balance_paise: Option<i64>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub rows: Vec<StatementRow>,
    /// Lines under the header that looked like data but could not be read.
    pub skipped: usize,
}

/// How a single amount column says which side a row is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "column", rename_all = "snake_case")]
pub enum SignRule {
    /// Debit and credit are separate columns; whichever is filled gives the side.
    SeparateColumns,
    /// One signed amount: a minus sign is money out, a plain figure is money in.
    NegativeIsDebit,
    /// One amount column and a separate Dr/Cr column, given by its index.
    DrCrColumn(usize),
    /// One amount column of spending: a plain figure is a debit, a "Cr" suffix (or a minus sign) a credit.
    CrSuffix,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateOrder {
    #[default]
    Dmy,
    Mdy,
    Ymd,
}

/// Which column of a file holds what. Column indices count from zero; `header_row` counts every
/// record in the file, blank lines included.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mapping {
    pub header_row: usize,
    pub date: usize,
    pub narration: usize,
    pub debit: Option<usize>,
    pub credit: Option<usize>,
    pub amount: Option<usize>,
    pub sign: SignRule,
    pub balance: Option<usize>,
    pub reference: Option<usize>,
    #[serde(default)]
    pub date_order: DateOrder,
}

/// What the user sees before choosing a mapping: the header and the rows under it, and the
/// mapping auto-detection would use (None when it finds no header).
#[derive(Debug, Serialize)]
pub struct Preview {
    pub header_row: usize,
    pub rows: Vec<Vec<String>>,
    pub guessed: Option<Mapping>,
}

/// How many records the preview shows, the header included.
const PREVIEW_RECORDS: usize = 13;
/// How many records from the top of a file can be the header.
const HEADER_CANDIDATES: usize = 30;

/// The delimiter is the one whose per-line count is most consistent across the first lines. A
/// comma in a narration varies from row to row; the real delimiter appears the same number of
/// times on the header and on the data.
fn delimiter(text: &str) -> char {
    const CANDIDATES: [char; 4] = [',', '\t', ';', '|'];
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).take(20).collect();
    let mut best = (0_usize, 0_usize, ',');
    for d in CANDIDATES {
        let mut freq: BTreeMap<usize, usize> = BTreeMap::new();
        for l in &lines {
            let c = l.matches(d).count();
            if c > 0 {
                *freq.entry(c).or_default() += 1;
            }
        }
        // Most lines agreeing on one count, then the larger count: (lines, count) per delimiter.
        let top = freq.iter().max_by_key(|(c, f)| (**f, **c)).map(|(&count, &lines_with)| (lines_with, count));
        if let Some((lines_with, count)) = top.filter(|&t| t > (best.0, best.1)) {
            best = (lines_with, count, d);
        }
    }
    best.2
}

/// Splits CSV text into records, honouring quotes.
fn split_records(text: &str) -> Vec<Vec<String>> {
    let text = text.trim_start_matches('\u{feff}');
    let delim = delimiter(text);

    let mut records = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            c if c == delim && !quoted => row.push(std::mem::take(&mut field)),
            '\r' if !quoted => {}
            '\n' if !quoted => {
                row.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut row));
            }
            c => field.push(c),
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        records.push(row);
    }
    records
}

fn month_from(s: &str) -> Option<Month> {
    Some(match s.get(..3)?.to_ascii_lowercase().as_str() {
        "jan" => Month::January,
        "feb" => Month::February,
        "mar" => Month::March,
        "apr" => Month::April,
        "may" => Month::May,
        "jun" => Month::June,
        "jul" => Month::July,
        "aug" => Month::August,
        "sep" => Month::September,
        "oct" => Month::October,
        "nov" => Month::November,
        "dec" => Month::December,
        _ => return None,
    })
}

/// 01/11/26, 01-11-2026, 2026-11-01, 1 Nov 2026, 01-Nov-26, Nov 1, 2026, optionally followed by a
/// time. Day first unless the date says otherwise.
pub fn parse_date(raw: &str) -> Option<Date> {
    parse_date_in(raw, DateOrder::Dmy)
}

/// As `parse_date`, but a numeric date is read in the given order (month first for `Mdy`).
pub fn parse_date_in(raw: &str, order: DateOrder) -> Option<Date> {
    let s = raw.trim();
    let s = s.split([' ', 'T']).next().filter(|_| !s.contains(char::is_alphabetic)).unwrap_or(s);
    let parts: Vec<&str> = s.split(['/', '-', '.', ' ', ',']).filter(|p| !p.is_empty()).collect();
    if parts.len() != 3 {
        return None;
    }
    let year = |p: &str| -> Option<i32> {
        let y: i32 = p.parse().ok()?;
        match p.len() {
            2 => Some(2000 + y),
            4 => Some(y),
            _ => None,
        }
    };
    let (y, m, d) = if parts[0].len() == 4 {
        (year(parts[0])?, parts[1], parts[2])
    } else if parts[0].chars().all(char::is_alphabetic) {
        // "Nov 1, 2026": a month name first.
        (year(parts[2])?, parts[0], parts[1])
    } else {
        match order {
            DateOrder::Mdy => (year(parts[2])?, parts[0], parts[1]),
            DateOrder::Dmy => (year(parts[2])?, parts[1], parts[0]),
            DateOrder::Ymd => (year(parts[0])?, parts[1], parts[2]),
        }
    };
    let month = match m.parse::<u8>() {
        Ok(n) => Month::try_from(n).ok()?,
        Err(_) => month_from(m)?,
    };
    Date::from_calendar_date(y, month, d.parse().ok()?).ok()
}

/// "1,23,456.78", "Rs. 450", "-450.00", "Rs. -450", "(450.00)", "450.00 Dr" -> signed paise.
/// A minus sign or an opening bracket before the number makes it negative. Empty / zero -> None.
fn parse_amount(raw: &str) -> Option<i64> {
    // Everything before the first digit is a sign or a currency prefix such as "Rs.", whose dot
    // must not be read as a decimal point.
    let start = raw.find(|c: char| c.is_ascii_digit()).unwrap_or(raw.len());
    let negative = raw[..start].contains('-') || raw[..start].contains('(');
    let cleaned: String = raw[start..].chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    if cleaned.is_empty() {
        return None;
    }
    let (whole, frac) = cleaned.split_once('.').unwrap_or((&cleaned, ""));
    let rupees: i64 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let frac = format!("{:0<2}", frac.chars().take(2).collect::<String>());
    let paise: i64 = frac.parse().ok()?;
    let total = rupees.checked_mul(100)?.checked_add(paise)?;
    let total = if negative { -total } else { total };
    (total != 0).then_some(total)
}

fn flip(side: Direction) -> Direction {
    if matches!(side, Direction::Debit) {
        Direction::Credit
    } else {
        Direction::Debit
    }
}

/// A signed figure in a debit or credit column: a minus sign reverses the side it sits in (a
/// negative withdrawal is money coming back). Returns the side and the positive amount.
fn sided(side: Direction, signed: i64) -> (Direction, i64) {
    if signed >= 0 {
        (side, signed)
    } else {
        (flip(side), -signed)
    }
}

/// The side of a row from its single amount column, as the sign rule reads it.
fn amount_side(sign: SignRule, raw: &str, signed: i64, row: &[String]) -> Direction {
    let lower = raw.to_lowercase();
    let suffix_cr = lower.ends_with("cr") || lower.ends_with("cr.");
    let suffix_dr = lower.ends_with("dr") || lower.ends_with("dr.");
    match sign {
        SignRule::DrCrColumn(i) => {
            let flag = row.get(i).map(|s| s.trim().to_lowercase()).unwrap_or_default();
            if suffix_cr || flag.starts_with("cr") || flag.contains("credit") {
                Direction::Credit
            } else {
                Direction::Debit
            }
        }
        SignRule::CrSuffix => {
            let base = if suffix_cr { Direction::Credit } else { Direction::Debit };
            if signed < 0 {
                flip(base)
            } else {
                base
            }
        }
        SignRule::NegativeIsDebit | SignRule::SeparateColumns => {
            if suffix_cr {
                Direction::Credit
            } else if suffix_dr || signed < 0 {
                Direction::Debit
            } else {
                Direction::Credit
            }
        }
    }
}

#[derive(Default)]
struct Columns {
    date: usize,
    narration: Option<usize>,
    reference: Option<usize>,
    debit: Option<usize>,
    credit: Option<usize>,
    /// One signed or Dr/Cr-flagged amount column.
    amount: Option<usize>,
    flag: Option<usize>,
    balance: Option<usize>,
}

fn find_columns(header: &[String]) -> Option<Columns> {
    let names: Vec<String> = header.iter().map(|h| h.trim().to_lowercase()).collect();
    let find = |pred: &dyn Fn(&str) -> bool| names.iter().position(|n| pred(n));
    // "Debit/Credit", "Debit / Credit" and "Dr/Cr" name one amount column, not two. It holds signed
    // or Dr/Cr-suffixed amounts, so it is read like a single amount column.
    let combined_name = |n: &str| matches!(n.chars().filter(|c| !c.is_whitespace()).collect::<String>().as_str(), "debit/credit" | "dr/cr" | "cr/dr");
    // "value date" is a fallback for banks that print both.
    let date = find(&|n| n == "date" || n == "txn date" || n == "transaction date" || n == "tran date" || n == "posting date")
        .or_else(|| find(&|n| n.contains("date") && !n.contains("value")))
        .or_else(|| find(&|n| n.contains("date")))?;
    let debit = find(&|n| !combined_name(n) && (n.contains("debit") || n.contains("withdrawal") || n == "dr" || n.contains("paid out")));
    let credit = find(&|n| !combined_name(n) && (n.contains("credit") || n.contains("deposit") || n == "cr" || n.contains("paid in")));
    let amount = if debit.is_none() && credit.is_none() {
        find(&|n| n == "amount" || n.starts_with("amount") || n.contains("amt")).or_else(|| find(&combined_name))
    } else {
        None
    };
    if debit.is_none() && credit.is_none() && amount.is_none() {
        return None;
    }
    // A separate Dr/Cr indicator next to an amount column; never the amount column itself. Spacing in
    // the name does not matter ("Dr / Cr"). Some banks print a second one after the balance (always
    // "CR", it says which side the balance is on): the one that belongs to the amount comes first,
    // right after it, and a flag after the balance column is never the amount's.
    let balance = find(&|n| n.contains("balance"));
    let flag_name = |n: &str| {
        let squeezed: String = n.chars().filter(|c| !c.is_whitespace()).collect();
        squeezed == "dr/cr" || squeezed == "cr/dr" || n == "type" || n.contains("transaction type")
    };
    let flags: Vec<usize> = (0..names.len()).filter(|i| flag_name(&names[*i]) && Some(*i) != amount).collect();
    let flag = flags
        .iter()
        .copied()
        .find(|i| amount.is_none_or(|a| *i > a) && balance.is_none_or(|b| *i < b))
        .or_else(|| flags.iter().copied().find(|i| balance.is_none_or(|b| *i < b)))
        .or_else(|| flags.first().copied());
    Some(Columns {
        date,
        narration: find(&|n| n.contains("narration") || n.contains("description") || n.contains("particular") || n.contains("remark") || n.contains("details") || n.contains("merchant")),
        reference: find(&|n| n.contains("ref") || n.contains("chq") || n.contains("cheque") || n.contains("utr")),
        debit,
        credit,
        amount,
        flag,
        balance,
    })
}

/// The mapping auto-detection would use for a header. A card's single amount column is read as
/// spending with "Cr" marking payments and refunds; a bank's as signed.
fn mapping_of(header_row: usize, c: &Columns, header: &[String], card: bool) -> Mapping {
    // With no narration column, the first column nothing else uses is the best guess.
    let used = [Some(c.date), c.debit, c.credit, c.amount, c.flag, c.balance, c.reference, c.narration];
    let narration = c.narration.or_else(|| (0..header.len()).find(|i| !used.contains(&Some(*i)))).unwrap_or(0);
    let sign = match (c.flag, c.debit.is_some() || c.credit.is_some()) {
        (_, true) => SignRule::SeparateColumns,
        (Some(flag), false) => SignRule::DrCrColumn(flag),
        (None, false) if card => SignRule::CrSuffix,
        (None, false) => SignRule::NegativeIsDebit,
    };
    Mapping {
        header_row,
        date: c.date,
        narration,
        debit: c.debit,
        credit: c.credit,
        amount: c.amount,
        sign,
        balance: c.balance,
        reference: c.reference,
        date_order: DateOrder::Dmy,
    }
}

/// The header row auto-detection finds, with the mapping it implies.
fn guess_in(records: &[Vec<String>], card: bool) -> Option<Mapping> {
    let (hi, cols) = records.iter().enumerate().find_map(|(i, r)| find_columns(r).map(|c| (i, c)))?;
    Some(mapping_of(hi, &cols, &records[hi], card))
}

/// Without a detectable header, the record most likely to be one: the one with the most cells
/// that are words rather than dates or amounts (the earliest on a tie).
fn label_header(records: &[Vec<String>]) -> usize {
    let words = |r: &[String]| r.iter().filter(|c| c.chars().any(char::is_alphabetic) && parse_date(c).is_none() && parse_amount(c).is_none()).count();
    records
        .iter()
        .take(40)
        .enumerate()
        .max_by_key(|(i, r)| (words(r), std::cmp::Reverse(*i)))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Reads a statement with the layout auto-detection finds. `None` when no header row (a date column
/// plus an amount column) is found. A bank's single amount column is read as signed.
pub fn parse(text: &str) -> Option<Parsed> {
    parse_for(text, false)
}

/// As `parse`. `card` says the file is a credit card's: a single amount column is then spending,
/// with a "Cr" suffix (or a minus sign) for payments and refunds.
pub fn parse_for(text: &str, card: bool) -> Option<Parsed> {
    let mut records = split_records(text);
    let mapping = guess_in(&records, card)?;
    Some(rows_from(&mut records, &mapping))
}

/// Reads a statement with the columns the user mapped.
pub fn parse_with(text: &str, mapping: &Mapping) -> Parsed {
    let mut records = split_records(text);
    rows_from(&mut records, mapping)
}

/// What the user sees before mapping, for a bank statement. See `preview_for`.
pub fn preview(text: &str) -> Preview {
    preview_for(text, false)
}

/// What the user sees before mapping: the header, the rows under it and the auto-detected mapping.
pub fn preview_for(text: &str, card: bool) -> Preview {
    let records = split_records(text);
    let guessed = guess_in(&records, card);
    let header_row = guessed.as_ref().map_or_else(|| label_header(&records), |m| m.header_row);
    let rows = records
        .iter()
        .skip(header_row)
        .filter(|r| r.iter().any(|c| !c.trim().is_empty()))
        .take(PREVIEW_RECORDS)
        .map(|r| r.iter().map(|c| c.trim().to_string()).collect())
        .collect();
    Preview { header_row, rows, guessed }
}

/// The signature of a header: its lowercased, trimmed column names joined by '|', hashed to 16 hex
/// characters. A saved layout is found again by it, whatever the rows and the file name.
pub fn header_signature(header: &[String]) -> String {
    let joined = header.iter().map(|h| h.trim().to_lowercase()).collect::<Vec<_>>().join("|");
    Sha256::digest(joined.as_bytes()).iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// The signature of each of the first records that could be a header, as (record index, signature).
pub fn header_signatures(text: &str) -> Vec<(usize, String)> {
    split_records(text)
        .iter()
        .enumerate()
        .filter(|(_, r)| r.iter().any(|c| !c.trim().is_empty()))
        .take(HEADER_CANDIDATES)
        .map(|(i, r)| (i, header_signature(r)))
        .collect()
}

/// The signature of the record at `header_row`.
pub fn signature(text: &str, header_row: usize) -> String {
    let records = split_records(text);
    header_signature(records.get(header_row).map_or(&[][..], Vec::as_slice))
}

/// The rows under the header, read with the mapping. A short row never panics: a missing cell is
/// empty. Rows that fail to read are counted as skipped.
fn rows_from(records: &mut [Vec<String>], m: &Mapping) -> Parsed {
    let cell = |r: &[String], i: Option<usize>| -> String { i.and_then(|i| r.get(i)).map(|s| s.trim().to_string()).unwrap_or_default() };
    let header_len = records.get(m.header_row).map_or(0, Vec::len);

    let mut out = Parsed::default();
    for r in records.iter_mut().skip(m.header_row + 1) {
        if r.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        // A narration can contain commas ("UPI-ACME, PUNE-..."), which splits one cell into several.
        // Extra fields beyond the header belong to the narration: join them back so the columns after it line up.
        let extra = r.len().saturating_sub(header_len);
        if extra > 0 && m.narration + extra < r.len() {
            let joined = r[m.narration..=m.narration + extra].join(",");
            r.splice(m.narration..=m.narration + extra, [joined]);
        }
        let Some(date) = r.get(m.date).and_then(|d| parse_date_in(d, m.date_order)) else {
            // Opening/closing balance lines, totals and page footers have no date: not data.
            if r.iter().filter(|c| !c.trim().is_empty()).count() >= 3 && r.iter().any(|c| parse_amount(c).is_some()) && !r.iter().any(|c| c.to_lowercase().contains("total") || c.to_lowercase().contains("balance")) {
                out.skipped += 1;
            }
            continue;
        };
        let debit = m.debit.and_then(|i| r.get(i)).and_then(|s| parse_amount(s));
        let credit = m.credit.and_then(|i| r.get(i)).and_then(|s| parse_amount(s));
        let (direction, amount_paise) = match (debit, credit, m.amount) {
            (Some(d), None, _) => sided(Direction::Debit, d),
            (None, Some(c), _) => sided(Direction::Credit, c),
            (None, None, Some(ai)) => {
                let raw = cell(r, Some(ai));
                let Some(signed) = parse_amount(&raw) else {
                    out.skipped += 1;
                    continue;
                };
                (amount_side(m.sign, &raw, signed, r), signed.abs())
            }
            _ => {
                out.skipped += 1;
                continue;
            }
        };
        let narration = cell(r, Some(m.narration));
        let reference = Some(cell(r, m.reference)).filter(|s| !s.is_empty() && s.chars().any(|c| c.is_ascii_digit()) && !s.chars().all(|c| c == '0'));
        let balance_paise = m.balance.and_then(|i| r.get(i)).and_then(|s| parse_amount(s));
        out.rows.push(StatementRow { date, direction, amount_paise, narration, reference, balance_paise });
    }
    out
}

/// A short merchant name from a narration: "UPI-SWIGGY-swiggy@icici-4305..." -> "SWIGGY".
pub fn merchant_of(narration: &str) -> Option<String> {
    let n = narration.trim();
    if n.is_empty() {
        return None;
    }
    let parts: Vec<&str> = n.split(['-', '/']).map(str::trim).filter(|p| !p.is_empty()).collect();
    let upper = parts.first().map(|p| p.to_ascii_uppercase()).unwrap_or_default();
    let pick = if matches!(upper.as_str(), "UPI" | "POS" | "IMPS" | "NEFT" | "RTGS" | "ACH" | "ECS") && parts.len() > 1 {
        let skip = if parts[1].chars().all(|c| c.is_ascii_digit()) && parts.len() > 2 { 2 } else { 1 };
        parts[skip]
    } else {
        n
    };
    Some(pick.chars().take(60).collect::<String>().trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn dates() {
        assert_eq!(parse_date("01/11/26"), Some(date!(2026 - 11 - 01)));
        assert_eq!(parse_date("01-11-2026"), Some(date!(2026 - 11 - 01)));
        assert_eq!(parse_date("2026-11-01"), Some(date!(2026 - 11 - 01)));
        assert_eq!(parse_date("2026-11-01 13:45:12"), Some(date!(2026 - 11 - 01)));
        assert_eq!(parse_date("01-Nov-2026"), Some(date!(2026 - 11 - 01)));
        assert_eq!(parse_date("1 Nov 26"), Some(date!(2026 - 11 - 01)));
        assert_eq!(parse_date("Narration"), None);
    }

    #[test]
    fn amounts() {
        assert_eq!(parse_amount("1,23,456.78"), Some(12_345_678));
        assert_eq!(parse_amount("Rs. 450"), Some(45_000));
        assert_eq!(parse_amount("450.5 Dr"), Some(45_050));
        assert_eq!(parse_amount(""), None);
        assert_eq!(parse_amount("0.00"), None);
    }

    #[test]
    fn a_dr_cr_column_after_the_amount_is_the_side_and_the_one_after_the_balance_is_ignored() {
        // Newest first, an amount column with a "Dr / Cr" flag, and a second "Dr / Cr" after the balance.
        let csv = "Account Statement\nAccount No.,0000\n\n\"Sl. No.\",\"Transaction Date\",\"Value Date\",\"Description\",\"Chq /Ref No.\",\"Amount\",\"Dr / Cr\",\"Balance\",\"Dr / Cr\"\n\
\"1\",\"02-11-2026 10:00:00\",\"02-11-2026\",\"UPI/SHOP/HDFC/1/Paid\",\"UPI-1\",\"60.00\",\"DR\",\"1,00,026.09\",\"CR\"\n\
\"2\",\"01-11-2026 09:00:00\",\"01-11-2026\",\"UPI/FRIEND/HDFC/2/Paid\",\"UPI-2\",\"1,00,000.00\",\"CR\",\"1,00,086.09\",\"CR\"\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.rows.len(), 2);
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise), (Direction::Debit, 6_000));
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 10_000_000));
        assert_eq!(p.rows[0].balance_paise, Some(10_002_609));
    }

    #[test]
    fn hdfc_style_with_preamble_and_footer() {
        let csv = "Account Statement\nCustomer,ANJAL\n\nDate,Narration,Chq./Ref.No.,Value Dt,Withdrawal Amt.,Deposit Amt.,Closing Balance\n\
01/11/26,UPI-SWIGGY-swiggy@icici-430512345678-Order,0000430512345678,01/11/26,\"1,450.00\",,\"98,550.00\"\n\
02/11/26,NEFT CR-ACME CORP-ANJAL,N123456,02/11/26,,\"1,00,000.00\",\"1,98,550.00\"\n\
\nSTATEMENT SUMMARY,,,,,,\nTotal,,,,1450.00,100000.00,\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.skipped, 0);
        assert_eq!(p.rows.len(), 2);
        assert_eq!(p.rows[0].direction, Direction::Debit);
        assert_eq!(p.rows[0].amount_paise, 145_000);
        assert_eq!(p.rows[0].balance_paise, Some(9_855_000));
        assert_eq!(p.rows[1].direction, Direction::Credit);
        assert_eq!(p.rows[1].amount_paise, 10_000_000);
    }

    #[test]
    fn single_amount_column_with_dr_cr_flag() {
        let csv = "Transaction Date,Description,Amount,Dr/Cr\n03-Nov-2026,AMAZON PAY,\"2,000.00\",Dr\n04-Nov-2026,PAYMENT RECEIVED,\"5,000.00\",Cr\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.rows[0].direction, Direction::Debit);
        assert_eq!(p.rows[1].direction, Direction::Credit);
    }

    #[test]
    fn signed_amounts_and_semicolons() {
        let csv = "Date;Details;Amount\n01/11/2026;Coffee;-120.00\n02/11/2026;Refund;45.00\n";
        let p = parse(csv).unwrap();
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise), (Direction::Debit, 12_000));
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 4_500));
    }

    #[test]
    fn debit_credit_header_is_one_amount_column() {
        let csv = "Date,Narration,Debit/Credit,Balance\n01/11/2026,Coffee,-120.00,1000.00\n02/11/2026,Refund,45.00,1045.00\n03/11/2026,Fee,\"-30.00\",1015.00\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.skipped, 0);
        assert_eq!(p.rows.len(), 3);
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise), (Direction::Debit, 12_000));
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 4_500));
        assert_eq!((p.rows[2].direction, p.rows[2].amount_paise), (Direction::Debit, 3_000));

        // Spaced header, with Dr/Cr suffixes instead of signs.
        let csv = "Transaction Date,Description,Debit / Credit\n03-Nov-2026,AMAZON PAY,\"2,000.00 Dr\"\n04-Nov-2026,PAYMENT RECEIVED,\"5,000.00 Cr\"\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.rows.len(), 2);
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise), (Direction::Debit, 200_000));
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 500_000));
    }

    #[test]
    fn negative_amounts_keep_their_sign() {
        assert_eq!(parse_amount("-450.00"), Some(-45_000));
        assert_eq!(parse_amount("Rs. -1,000"), Some(-100_000));
        assert_eq!(parse_amount("(450.00)"), Some(-45_000));
        assert_eq!(parse_amount("-0.00"), None);
        // A negative withdrawal is money back in; the negative balance is kept as it is.
        let csv = "Date,Narration,Withdrawal,Deposit,Balance\n01/11/2026,Reversal,\"-500.00\",,\"-250.00\"\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.skipped, 0);
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise, p.rows[0].balance_paise), (Direction::Credit, 50_000, Some(-25_000)));
    }

    #[test]
    fn commas_inside_a_narration_are_rejoined() {
        // Seven header fields; the narration holds two commas, so the row splits into nine.
        let csv = "\nDate,Narration,Value Dat,Debit Amount,Credit Amount,Chq/Ref Number,Closing Balance\r\n\
01/11/26,UPI-ACME, PUNE-ACME@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED,01/11/26,250.00,,612345678901,9750.00\r\n\
02/11/26,NEFT CR-ACME CORP-PRIYA,02/11/26,,1000.00,N1,10750.00\r\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.rows.len(), 2);
        assert_eq!(p.rows[0].narration, "UPI-ACME, PUNE-ACME@OKAXIS-SBIN0001234-612345678901-PAID VIA CRED");
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise, p.rows[0].balance_paise), (Direction::Debit, 25_000, Some(975_000)));
        assert_eq!(p.rows[1].narration, "NEFT CR-ACME CORP-PRIYA");
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 100_000));
        assert_eq!(p.skipped, 0);
    }

    #[test]
    fn not_a_statement() {
        assert!(parse("hello,world\n1,2\n").is_none());
    }

    /// A savings statement with a title block and a "Withdrawal (Dr)" / "Deposit (Cr)" pair, as
    /// banks print them. Synthetic figures.
    const KOTAK_LIKE: &str = "Account Statement\n\nTransaction Date,Description,Chq/Ref No,Withdrawal (Dr),Deposit (Cr),Balance\n\
05-11-2026,UPI-ASHA STORES-ASHA@YBL-YESB0YBLUPI-612345678902-UPI,612345678902,120.50,,9629.50\n\
06-11-2026,NEFT CR-ACME CORP-PRIYA,N1234567,,1000.00,10629.50\n";

    fn kotak_mapping() -> Mapping {
        Mapping {
            header_row: 2,
            date: 0,
            narration: 1,
            debit: Some(3),
            credit: Some(4),
            amount: None,
            sign: SignRule::SeparateColumns,
            balance: Some(5),
            reference: Some(2),
            date_order: DateOrder::Dmy,
        }
    }

    #[test]
    fn kotak_style_layout_parses_by_detection_and_by_mapping() {
        let auto = parse(KOTAK_LIKE).unwrap();
        assert_eq!(auto.rows.len(), 2);
        assert_eq!(auto.rows[0].date, date!(2026 - 11 - 05));
        assert_eq!((auto.rows[0].direction, auto.rows[0].amount_paise), (Direction::Debit, 12_050));
        assert_eq!((auto.rows[1].direction, auto.rows[1].amount_paise), (Direction::Credit, 100_000));
        assert_eq!(auto.rows[0].balance_paise, Some(962_950));
        assert_eq!(auto.rows[1].reference.as_deref(), Some("N1234567"));

        let mapped = parse_with(KOTAK_LIKE, &kotak_mapping());
        assert_eq!(mapped, auto);
    }

    #[test]
    fn card_file_reads_cr_as_payments_and_purchases_as_spending() {
        let csv = "Date,Description,Amount\n12-11-2026,AMAZON PAY,\"2,000.00\"\n13-11-2026,PAYMENT RECEIVED,\"5,000.00 Cr\"\n14-11-2026,REFUND FLIPKART,\"-300.00\"\n";
        let p = parse_for(csv, true).unwrap();
        assert_eq!(p.rows.len(), 3);
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise), (Direction::Debit, 200_000));
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 500_000));
        assert_eq!((p.rows[2].direction, p.rows[2].amount_paise), (Direction::Credit, 30_000));
        // The same file read as a bank statement would flip the purchase, which is why the target matters.
        assert_eq!(parse(csv).unwrap().rows[0].direction, Direction::Credit);
    }

    #[test]
    fn unknown_layout_has_no_detected_mapping_and_reads_with_one() {
        // No column name says date or amount, so detection finds nothing.
        let csv = "Posted On,Memo,Out,In\n03/11/2026,Coffee shop,120.00,\n04/11/2026,Salary,,5000.00\n";
        assert!(parse(csv).is_none());
        let pv = preview(csv);
        assert!(pv.guessed.is_none());
        assert_eq!(pv.header_row, 0);
        assert_eq!(pv.rows.len(), 3);
        assert_eq!(pv.rows[0], vec!["Posted On", "Memo", "Out", "In"]);

        let mapping = Mapping {
            header_row: 0,
            date: 0,
            narration: 1,
            debit: Some(2),
            credit: Some(3),
            amount: None,
            sign: SignRule::SeparateColumns,
            balance: None,
            reference: None,
            date_order: DateOrder::Dmy,
        };
        let p = parse_with(csv, &mapping);
        assert_eq!(p.rows.len(), 2);
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 500_000));
    }

    #[test]
    fn semicolon_file_with_a_preamble_that_has_commas() {
        let csv = "Statement for Customer, ANJAL\nDate;Details;Amount\n01/11/2026;Coffee, Pune;-120.00\n02/11/2026;Refund, Pune;45.00\n";
        let p = parse(csv).unwrap();
        assert_eq!(p.rows.len(), 2);
        assert_eq!(p.rows[0].narration, "Coffee, Pune");
        assert_eq!((p.rows[0].direction, p.rows[0].amount_paise), (Direction::Debit, 12_000));
        assert_eq!((p.rows[1].direction, p.rows[1].amount_paise), (Direction::Credit, 4_500));
    }

    #[test]
    fn date_orders_and_month_names() {
        assert_eq!(parse_date_in("11/05/2026", DateOrder::Mdy), Some(date!(2026 - 11 - 05)));
        assert_eq!(parse_date("11/05/2026"), Some(date!(2026 - 05 - 11)));
        assert_eq!(parse_date_in("2026/11/05", DateOrder::Ymd), Some(date!(2026 - 11 - 05)));
        assert_eq!(parse_date_in("26/11/05", DateOrder::Ymd), Some(date!(2026 - 11 - 05)));
        assert_eq!(parse_date("Nov 5, 2026"), Some(date!(2026 - 11 - 05)));
        assert_eq!(parse_date("13/13/2026"), None);
    }

    #[test]
    fn mapping_round_trips_through_json() {
        let card = Mapping { sign: SignRule::DrCrColumn(3), date_order: DateOrder::Mdy, ..kotak_mapping() };
        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("\"kind\":\"dr_cr_column\""), "{json}");
        assert!(json.contains("\"date_order\":\"mdy\""), "{json}");
        assert_eq!(serde_json::from_str::<Mapping>(&json).unwrap(), card);

        let json = r#"{"header_row":0,"date":0,"narration":1,"debit":null,"credit":null,"amount":2,"sign":{"kind":"cr_suffix"},"balance":null,"reference":null}"#;
        let m: Mapping = serde_json::from_str(json).unwrap();
        assert_eq!((m.sign, m.date_order), (SignRule::CrSuffix, DateOrder::Dmy));
    }

    #[test]
    fn signature_ignores_case_and_spacing_but_not_columns() {
        let a = header_signature(&["Date ".to_string(), "Narration".to_string()]);
        assert_eq!(a, header_signature(&["date".to_string(), " narration".to_string()]));
        assert_eq!(a.len(), 16);
        assert_ne!(a, header_signature(&["Date".to_string(), "Details".to_string()]));
        assert_eq!(signature(KOTAK_LIKE, 2), header_signature(&["Transaction Date", "Description", "Chq/Ref No", "Withdrawal (Dr)", "Deposit (Cr)", "Balance"].map(str::to_string)));
    }

    #[test]
    fn short_rows_never_panic() {
        let csv = "Date,Narration,Amount\n01/11/2026,Coffee\n02/11/2026\n";
        let p = parse_with(csv, &Mapping { header_row: 0, date: 0, narration: 1, debit: None, credit: None, amount: Some(2), sign: SignRule::NegativeIsDebit, balance: Some(5), reference: Some(7), date_order: DateOrder::Dmy });
        assert!(p.rows.is_empty());
        assert_eq!(p.skipped, 2);
    }

    #[test]
    fn merchants() {
        assert_eq!(merchant_of("UPI-SWIGGY-swiggy@icici-4305-Order").as_deref(), Some("SWIGGY"));
        assert_eq!(merchant_of("POS 412345 AMAZON").as_deref(), Some("POS 412345 AMAZON"));
        assert_eq!(merchant_of(""), None);
    }
}
