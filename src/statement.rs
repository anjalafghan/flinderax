//! Bank / card statement CSV exports -> rows. Pure: no database. Banks differ in column names,
//! date formats and whether debits and credits share a column, so the header row is found by
//! looking for a date column plus an amount column, and everything else is optional.

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

/// Splits CSV text into records, honouring quotes. The delimiter is comma, tab, semicolon or pipe,
/// whichever the header line uses most.
fn split_records(text: &str) -> Vec<Vec<String>> {
    let text = text.trim_start_matches('\u{feff}');
    let first = text.lines().find(|l| l.contains("ate")).unwrap_or("");
    let delim = [',', '\t', ';', '|']
        .into_iter()
        .max_by_key(|d| first.matches(*d).count())
        .filter(|d| first.contains(*d))
        .unwrap_or(',');

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

/// 01/11/26, 01-11-2026, 2026-11-01, 1 Nov 2026, 01-Nov-26, optionally followed by a time.
pub fn parse_date(raw: &str) -> Option<Date> {
    let s = raw.trim();
    let s = s.split([' ', 'T']).next().filter(|_| !s.contains(char::is_alphabetic)).unwrap_or(s);
    let parts: Vec<&str> = s.split(['/', '-', '.', ' ']).filter(|p| !p.is_empty()).collect();
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
    } else {
        (year(parts[2])?, parts[1], parts[0])
    };
    let month = match m.parse::<u8>() {
        Ok(n) => Month::try_from(n).ok()?,
        Err(_) => month_from(m)?,
    };
    Date::from_calendar_date(y, month, d.parse().ok()?).ok()
}

/// "1,23,456.78", "Rs. 450", "450.00 Dr" -> paise. Empty / zero -> None.
fn parse_amount(raw: &str) -> Option<i64> {
    // Skip a currency prefix such as "Rs." so its dot isn't read as a decimal point.
    let raw = &raw[raw.find(|c: char| c.is_ascii_digit()).unwrap_or(raw.len())..];
    let cleaned: String = raw.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    if cleaned.is_empty() {
        return None;
    }
    let (whole, frac) = cleaned.split_once('.').unwrap_or((&cleaned, ""));
    let rupees: i64 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let frac = format!("{:0<2}", frac.chars().take(2).collect::<String>());
    let paise: i64 = frac.parse().ok()?;
    let total = rupees.checked_mul(100)?.checked_add(paise)?;
    (total > 0).then_some(total)
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
    // "value date" is a fallback for banks that print both.
    let date = find(&|n| n == "date" || n == "txn date" || n == "transaction date" || n == "tran date" || n == "posting date")
        .or_else(|| find(&|n| n.contains("date") && !n.contains("value")))
        .or_else(|| find(&|n| n.contains("date")))?;
    let debit = find(&|n| n.contains("debit") || n.contains("withdrawal") || n == "dr" || n.contains("paid out"));
    let credit = find(&|n| n.contains("credit") || n.contains("deposit") || n == "cr" || n.contains("paid in"));
    let amount = if debit.is_none() && credit.is_none() {
        find(&|n| n == "amount" || n.starts_with("amount") || n.contains("amt"))
    } else {
        None
    };
    if debit.is_none() && credit.is_none() && amount.is_none() {
        return None;
    }
    Some(Columns {
        date,
        narration: find(&|n| n.contains("narration") || n.contains("description") || n.contains("particular") || n.contains("remark") || n.contains("details") || n.contains("merchant")),
        reference: find(&|n| n.contains("ref") || n.contains("chq") || n.contains("cheque") || n.contains("utr")),
        debit,
        credit,
        amount,
        flag: find(&|n| n == "dr/cr" || n == "cr/dr" || n == "type" || n.contains("transaction type") || n == "debit/credit"),
        balance: find(&|n| n.contains("balance")),
    })
}

/// Reads a statement. `None` when no header row (a date column plus an amount column) is found.
pub fn parse(text: &str) -> Option<Parsed> {
    let records = split_records(text);
    let (hi, cols) = records.iter().enumerate().find_map(|(i, r)| find_columns(r).map(|c| (i, c)))?;
    let cell = |r: &[String], i: Option<usize>| -> String { i.and_then(|i| r.get(i)).map(|s| s.trim().to_string()).unwrap_or_default() };

    let mut out = Parsed::default();
    for r in &records[hi + 1..] {
        if r.iter().all(|c| c.trim().is_empty()) {
            continue;
        }
        let Some(date) = r.get(cols.date).and_then(|d| parse_date(d)) else {
            // Opening/closing balance lines, totals and page footers have no date: not data.
            if r.iter().filter(|c| !c.trim().is_empty()).count() >= 3 && r.iter().any(|c| parse_amount(c).is_some()) && !r.iter().any(|c| c.to_lowercase().contains("total") || c.to_lowercase().contains("balance")) {
                out.skipped += 1;
            }
            continue;
        };
        let debit = cols.debit.and_then(|i| r.get(i)).and_then(|s| parse_amount(s));
        let credit = cols.credit.and_then(|i| r.get(i)).and_then(|s| parse_amount(s));
        let (direction, amount_paise) = match (debit, credit, cols.amount) {
            (Some(d), None, _) => (Direction::Debit, d),
            (None, Some(c), _) => (Direction::Credit, c),
            (None, None, Some(ai)) => {
                let raw = r.get(ai).map(|s| s.trim()).unwrap_or("");
                let Some(a) = parse_amount(raw) else {
                    out.skipped += 1;
                    continue;
                };
                let flag = cell(r, cols.flag).to_lowercase();
                let lower = raw.to_lowercase();
                let credit = flag.starts_with("cr") || flag.contains("credit") || lower.ends_with("cr") || lower.ends_with("cr.");
                let debit = flag.starts_with("dr") || flag.contains("debit") || lower.ends_with("dr") || lower.ends_with("dr.") || raw.starts_with('-') || raw.starts_with('(');
                // Signed columns: a leading minus is money out. With no flag at all, positive is money in.
                if credit || (!debit && cols.flag.is_none()) {
                    (Direction::Credit, a)
                } else {
                    (Direction::Debit, a)
                }
            }
            _ => {
                out.skipped += 1;
                continue;
            }
        };
        let narration = cell(r, cols.narration);
        let reference = Some(cell(r, cols.reference)).filter(|s| !s.is_empty() && s.chars().any(|c| c.is_ascii_digit()) && !s.chars().all(|c| c == '0'));
        let balance_paise = cols.balance.and_then(|i| r.get(i)).and_then(|s| parse_amount(s));
        out.rows.push(StatementRow { date, direction, amount_paise, narration, reference, balance_paise });
    }
    Some(out)
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
    fn not_a_statement() {
        assert!(parse("hello,world\n1,2\n").is_none());
    }

    #[test]
    fn merchants() {
        assert_eq!(merchant_of("UPI-SWIGGY-swiggy@icici-4305-Order").as_deref(), Some("SWIGGY"));
        assert_eq!(merchant_of("POS 412345 AMAZON").as_deref(), Some("POS 412345 AMAZON"));
        assert_eq!(merchant_of(""), None);
    }
}
