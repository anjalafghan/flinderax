//! Personal analytics from the user's own transactions: monthly money in and out, savings rate,
//! where spending goes, salary rhythm, SIPs, AutoPay, own transfers and a few plain notes.
//! Pure: no DB, no clock. The caller passes `today` and the statement coverage in. Money is paise.
//!
//! Spend is debits outside transfers, investments and refunds. Refunds are credits that net
//! against spend and are reported on their own.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use time::{Date, Duration, Month};

use crate::narration::{self, Narration, Rail};
use crate::planner::{format_inr, month_index};

const TRANSFER: &str = "transfer";
const INVESTMENTS: &str = "investments";
const REFUND: &str = "refund";
const INCOME: &str = "income";

/// Window for categories, top payees, small tickets and the weekday pattern.
const RECENT_DAYS: i64 = 90;
/// Window length in months for the monthly table and the yearly lists.
const MONTHS: i64 = 12;
const SMALL_TICKET_PAISE: i64 = 20_000;
const FREQUENT_MIN_COUNT: usize = 8;
const FREQUENT_MAX_PAISE: i64 = 50_000;
const SHOP_MIN_COUNT: usize = 3;
const SHOP_MEDIAN_PAISE: i64 = 50_000;
const TOP_PAYEES: usize = 10;
const NOTE_LIMIT: usize = 5;

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
    "November", "December",
];
const MERCHANT_WORDS: &[&str] = &[
    "PVT", "LTD", "LLP", "STORE", "STORES", "MART", "FOODS", "RESTAURANT", "HOTEL", "PHARMA", "PHARMACY",
    "MEDICAL", "CAFE", "SERVICES", "ENTERPRISES", "BAZAAR", "TECHNOLOGIES",
    // HDFC cuts payee names at ~20 characters, so these are matched as word prefixes too.
    "PRIVATE", "LIMITED", "GARAGE", "ACCESSORIES", "MOTORS", "AUTOMOBILES", "DESIGNS", "PROJECT", "SEVA",
    "TRADERS", "TRADING", "AGENCY", "AGENCIES", "SOLUTIONS", "ONLINE", "VENTURE", "VENTURES", "INDIA",
    "CENTRE", "CENTER", "CLINIC", "HOSPITAL", "SALON", "STUDIO", "BAKERY", "SWEETS", "KIRANA",
    "SUPERMARKET", "ELECTRONICS", "MOBILES", "TRAVELS", "TOURS", "PETROLEUM", "FILLING", "DAIRY",
    "CHEMIST", "OPTICALS", "FOUNDATION", "TRUST", "SCHOOL", "COLLEGE", "INSURANCE", "FINANCE",
];
const KNOWN_BRANDS: &[&str] = &[
    "SWIGGY", "ZOMATO", "AMAZON", "FLIPKART", "NETFLIX", "SPOTIFY", "UBER", "OLA", "IRCTC", "BLINKIT", "ZEPTO",
    "BIGBASKET", "JIO", "AIRTEL", "MYNTRA", "APPLE", "GOOGLE", "YOUTUBE", "HOTSTAR", "BOOKMYSHOW",
];
/// Payment gateways: a payment to one of these handles is a merchant, whatever the name says.
const MERCHANT_HANDLES: &[&str] = &["paytm", "razorpay", "payu", "cashfree", "billdesk", "juspay", "ccavenue"];

/// A transaction as the analytics need it.
#[derive(Debug, Clone, PartialEq)]
pub struct Txn {
    pub on: Date,
    pub debit: bool,
    pub amount: i64,
    pub merchant: Option<String>,
    pub category: String,
    pub raw_text: Option<String>,
}

/// The span of imported statements, from `statement_imports`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoverageIn {
    pub first: Date,
    pub last: Date,
    pub imports: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CoverageOut {
    pub first_date: String,
    pub last_date: String,
    pub imports: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MonthOut {
    pub month: String,
    pub income_paise: i64,
    pub spend_paise: i64,
    pub invested_paise: i64,
    pub transfers_paise: i64,
    pub refunds_paise: i64,
    pub net_paise: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CategoryOut {
    pub category: String,
    pub paise: i64,
    pub share_bps: i64,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PayeeOut {
    pub name: String,
    pub paise: i64,
    pub count: i64,
    pub kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SmallTicketOut {
    pub per_month_count: i64,
    pub per_month_paise: i64,
    pub threshold_paise: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SalaryOut {
    pub typical_days: Vec<u8>,
    /// Short form of the pay window: "Month-end (29th–2nd)", "Around the 7th", "7th–12th".
    pub usual: String,
    pub avg_paise: i64,
    pub last_paise: i64,
    pub last_on: String,
    pub missing_months: Vec<String>,
    pub next_expected: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SipOut {
    pub label: String,
    pub amount_paise: i64,
    pub usual_day: u8,
    pub months_seen: i64,
    pub last_on: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AutopayOut {
    pub name: String,
    pub amount_paise: i64,
    pub last_on: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OwnTransfersOut {
    pub count: i64,
    pub out_paise: i64,
    #[serde(rename = "in_paise")]
    pub inflow_paise: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrequentOut {
    pub name: String,
    pub count: i64,
    pub paise: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnalyticsOut {
    pub coverage: Option<CoverageOut>,
    pub months: Vec<MonthOut>,
    pub savings_rate_bps: Option<i64>,
    pub categories: Vec<CategoryOut>,
    pub top_payees: Vec<PayeeOut>,
    pub small_ticket: SmallTicketOut,
    pub salary: Option<SalaryOut>,
    pub sips: Vec<SipOut>,
    pub autopay: Vec<AutopayOut>,
    pub own_transfers: Option<OwnTransfersOut>,
    pub frequent_small: Vec<FrequentOut>,
    pub weekday_spend_paise: [i64; 7],
    pub notes: Vec<String>,
}

/// A transaction with its narration read once.
#[derive(Debug, Clone)]
struct Row {
    on: Date,
    debit: bool,
    amount: i64,
    category: String,
    payee: Option<String>,
    n: Narration,
}

impl Row {
    fn new(t: &Txn) -> Self {
        let n = narration::parse(t.raw_text.as_deref().unwrap_or(""));
        // The stored merchant wins: alert SMS text is not a narration, so its parse can be wrong.
        let payee = t
            .merchant
            .as_deref()
            .or(n.payee.as_deref())
            .map(normalise)
            .filter(|p| !p.is_empty());
        Row { on: t.on, debit: t.debit, amount: t.amount, category: t.category.clone(), payee, n }
    }

    fn is_spend(&self) -> bool {
        self.debit && !matches!(self.category.as_str(), TRANSFER | INVESTMENTS | REFUND)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Acc {
    income: i64,
    spend: i64,
    invested: i64,
    transfers: i64,
    refunds: i64,
}

/// Builds every analytics field. `today` bounds the data: later transactions are ignored.
pub fn build(txns: &[Txn], coverage: Option<CoverageIn>, today: Date) -> AnalyticsOut {
    let rows: Vec<Row> = txns.iter().filter(|t| t.on <= today && t.amount > 0).map(Row::new).collect();
    let now = month_index(today);
    let first = now - MONTHS as i32 + 1;
    let recent_from = today - Duration::days(RECENT_DAYS);

    let acc = monthly(&rows, first, now);
    // Months before the first statement aren't "₹0 months": they're simply not covered.
    let shown_from = coverage.map(|c| month_index(c.first).max(first)).unwrap_or(first);
    let savings_rate_bps = savings_rate(&acc, now);
    let categories = categories(&rows, recent_from);
    let top_payees = top_payees(&rows, recent_from);
    let small_ticket = small_ticket(&rows, recent_from);
    let salary = salary_out(&rows, coverage, first, now, today);
    let sips = sips(&rows);
    let autopay = autopay(&rows);
    let own_transfers = own_transfers(&rows, first);
    let frequent_small = frequent_small(&rows, first);
    let weekday_spend_paise = weekday_spend(&rows, recent_from);
    let notes = notes(&salary, savings_rate_bps, &categories, &small_ticket, &sips);

    AnalyticsOut {
        coverage: coverage.map(|c| CoverageOut {
            first_date: c.first.to_string(),
            last_date: c.last.to_string(),
            imports: c.imports,
        }),
        months: acc
            .iter()
            .filter(|(i, _)| **i >= shown_from)
            .map(|(i, a)| MonthOut {
                month: month_label(*i),
                income_paise: a.income,
                spend_paise: a.spend,
                invested_paise: a.invested,
                transfers_paise: a.transfers,
                refunds_paise: a.refunds,
                net_paise: a.income + a.refunds - a.spend - a.invested,
            })
            .collect(),
        savings_rate_bps,
        categories,
        top_payees,
        small_ticket,
        salary,
        sips,
        autopay,
        own_transfers,
        frequent_small,
        weekday_spend_paise,
        notes,
    }
}

fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase()
}

fn monthly(rows: &[Row], first: i32, now: i32) -> BTreeMap<i32, Acc> {
    let mut acc: BTreeMap<i32, Acc> = (first..=now).map(|i| (i, Acc::default())).collect();
    for r in rows {
        let Some(a) = acc.get_mut(&month_index(r.on)) else { continue };
        match (r.debit, r.category.as_str()) {
            (true, INVESTMENTS) => a.invested += r.amount,
            (true, TRANSFER) => a.transfers += r.amount,
            (true, REFUND) => {}
            (true, _) => a.spend += r.amount,
            (false, INCOME) => a.income += r.amount,
            (false, REFUND) => a.refunds += r.amount,
            (false, _) => {}
        }
    }
    acc
}

/// (income − spend) / income over the last three complete months (the current month is left out).
fn savings_rate(acc: &BTreeMap<i32, Acc>, now: i32) -> Option<i64> {
    let (income, spend) = (now - 3..now)
        .filter_map(|i| acc.get(&i))
        .fold((0i64, 0i64), |(inc, sp), a| (inc + a.income, sp + a.spend));
    (income > 0).then(|| (income - spend) * 10_000 / income)
}

fn share_bps(part: i64, total: i64) -> i64 {
    if total > 0 { part * 10_000 / total } else { 0 }
}

fn categories(rows: &[Row], recent_from: Date) -> Vec<CategoryOut> {
    let mut by: BTreeMap<&str, (i64, i64)> = BTreeMap::new();
    for r in rows.iter().filter(|r| r.is_spend() && r.on >= recent_from) {
        let e = by.entry(r.category.as_str()).or_default();
        e.0 += r.amount;
        e.1 += 1;
    }
    let total: i64 = by.values().map(|v| v.0).sum();
    let mut out: Vec<CategoryOut> = by
        .into_iter()
        .map(|(c, (paise, count))| CategoryOut {
            category: c.to_string(),
            paise,
            share_bps: share_bps(paise, total),
            count,
        })
        .collect();
    out.sort_by(|a, b| b.paise.cmp(&a.paise).then_with(|| a.category.cmp(&b.category)));
    out
}

fn by_payee<'a>(rows: impl Iterator<Item = &'a Row>) -> BTreeMap<String, Vec<&'a Row>> {
    let mut m: BTreeMap<String, Vec<&Row>> = BTreeMap::new();
    for r in rows {
        if let Some(p) = &r.payee {
            m.entry(p.clone()).or_default().push(r);
        }
    }
    m
}

fn median(mut values: Vec<i64>) -> i64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let n = values.len();
    if n % 2 == 1 { values[n / 2] } else { (values[n / 2 - 1] + values[n / 2]) / 2 }
}

/// Words and brands in the payee name make it a business.
fn is_business(name: &str) -> bool {
    let words: Vec<&str> = name.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).collect();
    let last = words.len().saturating_sub(1);
    words.iter().enumerate().any(|(i, w)| {
        MERCHANT_WORDS.contains(w)
            || KNOWN_BRANDS.contains(w)
            // A truncated last word ("ACCESSORI", "PROJEC", "PRIVA") still counts.
            || (i == last && w.len() >= 4 && MERCHANT_WORDS.iter().any(|m| m.len() > w.len() && m.starts_with(w)))
    })
}

fn payee_kind(name: &str, count: usize, median_paise: i64, merchant_handle: bool) -> &'static str {
    if merchant_handle || is_business(name) {
        "merchant"
    } else if count >= SHOP_MIN_COUNT && median_paise <= SHOP_MEDIAN_PAISE {
        "shop"
    } else {
        "person"
    }
}

fn top_payees(rows: &[Row], recent_from: Date) -> Vec<PayeeOut> {
    let mut out: Vec<PayeeOut> = by_payee(rows.iter().filter(|r| r.is_spend() && r.on >= recent_from))
        .into_iter()
        .map(|(name, rs)| {
            let amounts: Vec<i64> = rs.iter().map(|r| r.amount).collect();
            let paise: i64 = amounts.iter().sum();
            let merchant_handle = rs
                .iter()
                .any(|r| r.n.handle.as_deref().is_some_and(|h| MERCHANT_HANDLES.contains(&h)));
            let kind = payee_kind(&name, rs.len(), median(amounts), merchant_handle);
            PayeeOut { name, paise, count: rs.len() as i64, kind }
        })
        .collect();
    out.sort_by(|a, b| b.paise.cmp(&a.paise).then_with(|| a.name.cmp(&b.name)));
    out.truncate(TOP_PAYEES);
    out
}

/// Rounds x / d to the nearest whole number, halves up (x and d positive).
fn round_div(x: i64, d: i64) -> i64 {
    (x * 2 + d) / (2 * d)
}

fn small_ticket(rows: &[Row], recent_from: Date) -> SmallTicketOut {
    let small: Vec<&Row> = rows
        .iter()
        .filter(|r| r.is_spend() && r.on >= recent_from && r.amount < SMALL_TICKET_PAISE)
        .collect();
    let paise: i64 = small.iter().map(|r| r.amount).sum();
    SmallTicketOut {
        per_month_count: round_div(small.len() as i64, 3),
        per_month_paise: round_div(paise, 3),
        threshold_paise: SMALL_TICKET_PAISE,
    }
}

fn year_month(idx: i32) -> (i32, Month) {
    let m = Month::try_from((idx.rem_euclid(12) + 1) as u8).unwrap_or(Month::January);
    (idx.div_euclid(12), m)
}

/// "2026-03" style label for a month index.
fn month_label(idx: i32) -> String {
    format!("{:04}-{:02}", idx.div_euclid(12), idx.rem_euclid(12) + 1)
}

fn last_day(y: i32, m: Month) -> u8 {
    (28..=31u8).rev().find(|&d| Date::from_calendar_date(y, m, d).is_ok()).unwrap_or(28)
}

/// The given day of a month, clamped to the month's length (31 becomes 28 or 30 where needed).
fn on_day(idx: i32, day: u8) -> Date {
    let (y, m) = year_month(idx);
    let d = day.min(last_day(y, m));
    Date::from_calendar_date(y, m, d).unwrap_or(Date::MIN)
}

/// The most common day of the month; ties go to the earlier day.
fn mode_day(days: impl Iterator<Item = u8>) -> Option<u8> {
    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    for d in days {
        *counts.entry(d).or_default() += 1;
    }
    counts.into_iter().max_by_key(|(d, c)| (*c, Reverse(*d))).map(|(d, _)| d)
}

/// Salary rows: narration says salary. Without any, the payee that credits income in the most
/// months (at least two) counts as recurring, and its largest credit of each month is used.
fn salary_rows(rows: &[Row]) -> Vec<&Row> {
    let salary: Vec<&Row> = rows
        .iter()
        .filter(|r| !r.debit && r.category == INCOME && r.n.rail == Rail::Salary)
        .collect();
    if !salary.is_empty() {
        return salary;
    }

    let income: Vec<(&str, &Row)> = rows
        .iter()
        .filter(|r| !r.debit && r.category == INCOME)
        .filter_map(|r| r.payee.as_deref().map(|p| (p, r)))
        .collect();
    let mut months: BTreeMap<&str, BTreeSet<i32>> = BTreeMap::new();
    for (p, r) in &income {
        months.entry(p).or_default().insert(month_index(r.on));
    }
    let best: Option<&str> = months
        .iter()
        .filter(|(_, m)| m.len() >= 2)
        .max_by_key(|(p, m)| (m.len(), Reverse(**p)))
        .map(|(p, _)| *p);
    let Some(best) = best else { return Vec::new() };

    let mut per_month: BTreeMap<i32, &Row> = BTreeMap::new();
    for (p, r) in income {
        if p != best {
            continue;
        }
        let e = per_month.entry(month_index(r.on)).or_insert(r);
        if r.amount > e.amount {
            *e = r;
        }
    }
    per_month.into_values().collect()
}

/// Months in the statement span with no salary. The first month is skipped when the statement
/// starts after the 10th, the last when it ends before the 10th: the salary may not be in yet.
fn missing_months(c: CoverageIn, salary_months: &BTreeSet<i32>) -> Vec<String> {
    let (a, b) = (month_index(c.first), month_index(c.last));
    (a..=b)
        .filter(|i| !(*i == a && c.first.day() > 10))
        .filter(|i| !(*i == b && c.last.day() < 10))
        .filter(|i| !salary_months.contains(i))
        .map(month_label)
        .collect()
}

fn next_expected(usual: u8, now: i32, paid_this_month: bool, today: Date) -> Date {
    let this = on_day(now, usual);
    if !paid_this_month && this >= today { this } else { on_day(now + 1, usual) }
}

fn salary_out(rows: &[Row], coverage: Option<CoverageIn>, first: i32, now: i32, today: Date) -> Option<SalaryOut> {
    let salary = salary_rows(rows);
    let in_year: Vec<&Row> = salary.iter().copied().filter(|r| month_index(r.on) >= first).collect();
    if in_year.is_empty() {
        return None;
    }
    let last = in_year.iter().copied().max_by_key(|r| r.on)?;
    let typical_days: Vec<u8> = in_year.iter().map(|r| r.on.day()).collect::<BTreeSet<_>>().into_iter().collect();
    let avg_paise = in_year.iter().map(|r| r.amount).sum::<i64>() / in_year.len() as i64;
    let usual = mode_day(in_year.iter().map(|r| r.on.day()))?;
    let months: BTreeSet<i32> = salary.iter().map(|r| month_index(r.on)).collect();
    Some(SalaryOut {
        usual: salary_window(&typical_days).unwrap_or_default(),
        typical_days,
        avg_paise,
        last_paise: last.amount,
        last_on: last.on.to_string(),
        missing_months: coverage.map(|c| missing_months(c, &months)).unwrap_or_default(),
        next_expected: next_expected(usual, now, months.contains(&now), today).to_string(),
    })
}

fn sips(rows: &[Row]) -> Vec<SipOut> {
    let mut by: BTreeMap<i64, Vec<&Row>> = BTreeMap::new();
    for r in rows
        .iter()
        .filter(|r| r.debit && r.n.rail == Rail::Ach && r.category == INVESTMENTS)
    {
        by.entry(r.amount).or_default().push(r);
    }
    let mut out: Vec<SipOut> = by
        .into_iter()
        .filter_map(|(amount, rs)| {
            let last = rs.iter().copied().max_by_key(|r| r.on)?;
            let months: BTreeSet<i32> = rs.iter().map(|r| month_index(r.on)).collect();
            // Mandates of the same amount: the usual number of debits in a month.
            let mut per_month: BTreeMap<i32, usize> = BTreeMap::new();
            for r in &rs {
                *per_month.entry(month_index(r.on)).or_default() += 1;
            }
            let mandates = mode_day(per_month.values().map(|n| (*n).min(31) as u8)).unwrap_or(1).max(1) as usize;
            let one = SipOut {
                label: format!("SIP {}", whole_rupees(amount)),
                amount_paise: amount,
                usual_day: mode_day(rs.iter().map(|r| r.on.day())).unwrap_or(1),
                months_seen: months.len() as i64,
                last_on: last.on.to_string(),
            };
            Some(std::iter::repeat_n(one, mandates))
        })
        .flatten()
        .collect();
    out.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise));
    out
}

fn autopay(rows: &[Row]) -> Vec<AutopayOut> {
    let mut out: Vec<AutopayOut> = by_payee(rows.iter().filter(|r| r.debit && r.n.autopay))
        .into_iter()
        .filter_map(|(name, rs)| {
            let last = rs.iter().copied().max_by_key(|r| r.on)?;
            Some(AutopayOut {
                name,
                amount_paise: last.amount,
                last_on: last.on.to_string(),
                count: rs.len() as i64,
            })
        })
        .collect();
    out.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise).then_with(|| a.name.cmp(&b.name)));
    out
}

fn own_transfers(rows: &[Row], first: i32) -> Option<OwnTransfersOut> {
    let own: Vec<&Row> = rows
        .iter()
        .filter(|r| {
            r.category == TRANSFER
                && month_index(r.on) >= first
                && matches!(r.n.rail, Rail::Upi | Rail::Imps | Rail::Neft)
        })
        .collect();
    if own.is_empty() {
        return None;
    }
    let out_paise = own.iter().filter(|r| r.debit).map(|r| r.amount).sum();
    let inflow_paise = own.iter().filter(|r| !r.debit).map(|r| r.amount).sum();
    Some(OwnTransfersOut { count: own.len() as i64, out_paise, inflow_paise })
}

fn frequent_small(rows: &[Row], first: i32) -> Vec<FrequentOut> {
    let mut out: Vec<FrequentOut> = by_payee(
        rows.iter()
            .filter(|r| r.is_spend() && month_index(r.on) >= first && r.amount <= FREQUENT_MAX_PAISE),
    )
    .into_iter()
    .filter(|(_, rs)| rs.len() >= FREQUENT_MIN_COUNT)
    .map(|(name, rs)| FrequentOut {
        name,
        count: rs.len() as i64,
        paise: rs.iter().map(|r| r.amount).sum(),
    })
    .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| b.paise.cmp(&a.paise)));
    out
}

fn weekday_spend(rows: &[Row], recent_from: Date) -> [i64; 7] {
    let mut out = [0i64; 7];
    for r in rows.iter().filter(|r| r.is_spend() && r.on >= recent_from) {
        out[r.on.weekday().number_days_from_monday() as usize] += r.amount;
    }
    out
}

/// Whole rupees, for planning sentences: "₹5,000", never "₹5,000.00".
fn whole_rupees(paise: i64) -> String {
    format_inr(paise.saturating_add(50).div_euclid(100).saturating_mul(100))
}

fn ordinal(n: u8) -> String {
    let suffix = match (n % 100, n % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// "2026-09" -> "September 2026".
fn month_name_of(ym: &str) -> String {
    let Some((y, m)) = ym.split_once('-') else { return ym.to_string() };
    match m.parse::<usize>() {
        Ok(m @ 1..=12) => format!("{} {y}", MONTH_NAMES[m - 1]),
        _ => ym.to_string(),
    }
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Plain sentences with real numbers, most useful first. At most `NOTE_LIMIT`.
/// The pay window in a few words, for the salary card. Same reading as [`salary_days_note`].
fn salary_window(days: &[u8]) -> Option<String> {
    let (&lo, &hi) = (days.first()?, days.last()?);
    if lo == hi {
        return Some(format!("Around the {}", ordinal(lo)));
    }
    let late: Vec<u8> = days.iter().copied().filter(|d| *d >= 25).collect();
    let early: Vec<u8> = days.iter().copied().filter(|d| *d <= 5).collect();
    let others = days.iter().filter(|d| **d > 5 && **d < 25).count();
    if !late.is_empty() && late.len() + early.len() > others {
        let from = late.first().copied().unwrap_or(lo);
        let to = early.last().or(late.last()).copied().unwrap_or(hi);
        return Some(format!("Month-end ({}–{})", ordinal(from), ordinal(to)));
    }
    Some(format!("{}–{}", ordinal(lo), ordinal(hi)))
}

/// "Salary usually lands around month-end (29th–2nd); once on the 15th." Days 25-31 and 1-5 form one
/// window that wraps the month boundary; a plain min..max would read "between the 2nd and the 31st".
fn salary_days_note(days: &[u8]) -> Option<String> {
    let (&lo, &hi) = (days.first()?, days.last()?);
    if lo == hi {
        return Some(format!("Salary arrives around the {}.", ordinal(lo)));
    }
    let late: Vec<u8> = days.iter().copied().filter(|d| *d >= 25).collect();
    let early: Vec<u8> = days.iter().copied().filter(|d| *d <= 5).collect();
    let others: Vec<u8> = days.iter().copied().filter(|d| *d > 5 && *d < 25).collect();
    if !late.is_empty() && late.len() + early.len() > others.len() {
        let from = late.first().copied().unwrap_or(lo);
        let to = early.last().or(late.last()).copied().unwrap_or(hi);
        let mut text = format!("Salary usually lands around month-end ({}–{})", ordinal(from), ordinal(to));
        if !others.is_empty() {
            let list: Vec<String> = others.iter().map(|d| ordinal(*d)).collect();
            text.push_str(&format!("; other times the {}", list.join(", ")));
        }
        text.push('.');
        return Some(text);
    }
    Some(format!("Salary arrived between the {} and the {}.", ordinal(lo), ordinal(hi)))
}

fn notes(
    salary: &Option<SalaryOut>,
    savings_bps: Option<i64>,
    categories: &[CategoryOut],
    small: &SmallTicketOut,
    sips: &[SipOut],
) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(s) = salary {
        if let Some(text) = salary_days_note(&s.typical_days) {
            out.push(text);
        }
        match s.missing_months.as_slice() {
            [] => {}
            [one] => out.push(format!("{} had no salary.", month_name_of(one))),
            many => out.push(format!("{} months in your statements had no salary.", many.len())),
        }
    }
    if let Some(bps) = savings_bps {
        out.push(if bps < 0 {
            "You spent more than you earned over the last three full months.".to_string()
        } else {
            format!("You kept {}% of your income over the last three full months.", bps / 100)
        });
    }
    if small.per_month_paise > 0 {
        out.push(format!(
            "{} a month goes on payments under {}.",
            whole_rupees(small.per_month_paise),
            whole_rupees(small.threshold_paise)
        ));
    }
    if let Some(day) = mode_day(sips.iter().map(|s| s.usual_day)) {
        out.push(match sips.len() {
            1 => format!("1 SIP debits around the {}.", ordinal(day)),
            n => format!("{n} SIPs debit around the {}.", ordinal(day)),
        });
    }
    if let Some(top) = categories.first().filter(|c| c.share_bps > 0) {
        out.push(format!(
            "{} is the biggest spend: {}% of spending in the last 90 days.",
            capitalise(&top.category),
            top.share_bps / 100
        ));
    }
    out.truncate(NOTE_LIMIT);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::format_description;

    fn d(s: &str) -> Date {
        Date::parse(s, format_description!("[year]-[month]-[day]")).unwrap()
    }

    fn today() -> Date {
        d("2026-11-09")
    }

    fn txn(on: &str, debit: bool, amount: i64, category: &str, raw: &str) -> Txn {
        Txn { on: d(on), debit, amount, merchant: None, category: category.into(), raw_text: Some(raw.into()) }
    }

    const SALARY: &str = "SALARYFORTHEMONTHOFOCT2026";
    const UPI_SHOP: &str = "UPI-ASHA STORES-Q123456789@YBL-YESB0YBLUPI-612345678902-UPI";
    const AUTOPAY: &str = "UPI-STREAMCO-STREAMCO@YBL-YESB0YBLUPI-612345678905-SUBSCRIPTION DEBIT";
    const SIP: &str = "ACH D- BD-MF UTILITIES LUMP-TJAC12345678";
    const IMPS_SELF: &str = "IMPS-612345678908-RAVI KUMAR-KKBK-XXXXXXX1234-SELF";

    fn month<'a>(out: &'a AnalyticsOut, ym: &str) -> &'a MonthOut {
        out.months.iter().find(|m| m.month == ym).expect("month present")
    }

    #[test]
    fn empty_data_gives_the_zeroed_shape() {
        let out = build(&[], None, today());
        assert_eq!(out.months.len(), 12);
        assert!(out.months.iter().all(|m| m.spend_paise == 0 && m.income_paise == 0 && m.net_paise == 0));
        assert_eq!(out.months.first().map(|m| m.month.as_str()), Some("2025-12"));
        assert_eq!(out.months.last().map(|m| m.month.as_str()), Some("2026-11"));
        assert!(out.coverage.is_none());
        assert!(out.salary.is_none());
        assert!(out.savings_rate_bps.is_none());
        assert!(out.own_transfers.is_none());
        assert!(out.categories.is_empty() && out.top_payees.is_empty() && out.sips.is_empty());
        assert_eq!(out.weekday_spend_paise, [0; 7]);
        assert!(out.notes.is_empty());
    }

    #[test]
    fn months_split_income_spend_invested_transfers_and_refunds() {
        let txns = vec![
            txn("2026-10-01", false, 100_000, "income", SALARY),
            txn("2026-10-05", true, 20_000, "food", UPI_SHOP),
            txn("2026-10-06", true, 10_000, "investments", SIP),
            txn("2026-10-07", true, 5_000, "transfer", IMPS_SELF),
            txn("2026-10-08", false, 1_500, "refund", UPI_SHOP),
        ];
        let out = build(&txns, None, today());
        let oct = month(&out, "2026-10");
        assert_eq!(oct.income_paise, 100_000);
        assert_eq!(oct.spend_paise, 20_000);
        assert_eq!(oct.invested_paise, 10_000);
        assert_eq!(oct.transfers_paise, 5_000);
        assert_eq!(oct.refunds_paise, 1_500);
        assert_eq!(oct.net_paise, 100_000 + 1_500 - 20_000 - 10_000);
    }

    #[test]
    fn savings_rate_uses_the_last_three_complete_months_only() {
        let mut txns = Vec::new();
        for m in ["08", "09", "10"] {
            txns.push(txn(&format!("2026-{m}-01"), false, 100_000, "income", SALARY));
            txns.push(txn(&format!("2026-{m}-10"), true, 60_000, "food", UPI_SHOP));
        }
        // November is the current month, so it does not count; neither does a big August debit
        // outside the window.
        txns.push(txn("2026-11-02", false, 999_999, "income", SALARY));
        txns.push(txn("2026-11-03", true, 999_999, "food", UPI_SHOP));
        let out = build(&txns, None, today());
        assert_eq!(out.savings_rate_bps, Some(4_000));
    }

    #[test]
    fn categories_cover_the_last_ninety_days_with_shares() {
        let txns = vec![
            txn("2026-11-01", true, 30_000, "food", UPI_SHOP),
            txn("2026-10-20", true, 10_000, "groceries", UPI_SHOP),
            txn("2026-06-01", true, 60_000, "food", UPI_SHOP), // outside 90 days
            txn("2026-10-21", true, 40_000, "transfer", IMPS_SELF), // not spending
        ];
        let out = build(&txns, None, today());
        let names: Vec<(&str, i64, i64)> =
            out.categories.iter().map(|c| (c.category.as_str(), c.paise, c.share_bps)).collect();
        assert_eq!(names, vec![("food", 30_000, 7_500), ("groceries", 10_000, 2_500)]);
    }

    #[test]
    fn salary_reports_days_average_last_missing_months_and_next_expected() {
        let txns = vec![
            txn("2026-08-01", false, 100_000, "income", SALARY),
            txn("2026-10-29", false, 100_000, "income", SALARY),
        ];
        let cov = CoverageIn { first: d("2026-07-15"), last: d("2026-11-08"), imports: 2 };
        let out = build(&txns, Some(cov), today());
        let s = out.salary.expect("salary");
        assert_eq!(s.typical_days, vec![1, 29]);
        assert_eq!(s.avg_paise, 100_000);
        assert_eq!((s.last_paise, s.last_on.as_str()), (100_000, "2026-10-29"));
        // July starts after the 10th and November ends before it, so only September is missing.
        assert_eq!(s.missing_months, vec!["2026-09".to_string()]);
        // Days 1 and 29 tie at one each; the earlier day wins, and the 1st has passed this month.
        assert_eq!(s.next_expected, "2026-12-01");
        assert_eq!(out.coverage.map(|c| c.first_date).as_deref(), Some("2026-07-15"));
    }

    #[test]
    fn next_expected_clamps_to_the_month_length() {
        let txns = vec![
            txn("2026-08-31", false, 100_000, "income", SALARY),
            txn("2026-10-31", false, 100_000, "income", SALARY),
        ];
        let out = build(&txns, None, today());
        assert_eq!(out.salary.map(|s| s.next_expected).as_deref(), Some("2026-11-30"));
    }

    #[test]
    fn salary_falls_back_to_the_recurring_payee_without_a_salary_narration() {
        let employer = "UPI-ACME EMPLOYER-ACME@OKAXIS-SBIN0001234-612345678999-PAYMENT";
        let txns = vec![
            txn("2026-09-01", false, 90_000, "income", employer),
            txn("2026-10-01", false, 95_000, "income", employer),
            txn("2026-10-12", false, 3_000, "income", UPI_SHOP),
        ];
        let out = build(&txns, None, today());
        let s = out.salary.expect("fallback salary");
        assert_eq!(s.last_paise, 95_000);
        assert_eq!(s.typical_days, vec![1]);
    }

    #[test]
    fn sips_group_by_amount_with_the_usual_debit_day() {
        let txns = vec![
            txn("2026-08-10", true, 500_000, "investments", SIP),
            txn("2026-09-10", true, 500_000, "investments", SIP),
            txn("2026-10-11", true, 500_000, "investments", SIP),
            txn("2026-10-05", true, 200_000, "investments", SIP),
        ];
        let out = build(&txns, None, today());
        assert_eq!(out.sips.len(), 2);
        let big = &out.sips[0];
        assert_eq!(big.label, "SIP ₹5,000");
        assert_eq!((big.usual_day, big.months_seen, big.last_on.as_str()), (10, 3, "2026-10-11"));
        assert_eq!(out.sips[1].amount_paise, 200_000);
    }

    #[test]
    fn autopay_groups_by_payee_and_keeps_the_latest_amount() {
        let txns = vec![
            txn("2026-09-03", true, 49_900, "fees", AUTOPAY),
            txn("2026-10-03", true, 59_900, "fees", AUTOPAY),
        ];
        let out = build(&txns, None, today());
        assert_eq!(out.autopay.len(), 1);
        let a = &out.autopay[0];
        assert_eq!((a.name.as_str(), a.amount_paise, a.count, a.last_on.as_str()), ("STREAMCO", 59_900, 2, "2026-10-03"));
    }

    #[test]
    fn own_transfers_count_both_directions() {
        let txns = vec![
            txn("2026-10-02", true, 5_000, "transfer", IMPS_SELF),
            txn("2026-10-05", false, 2_000, "transfer", UPI_SHOP),
        ];
        let out = build(&txns, None, today());
        let o = out.own_transfers.expect("own transfers");
        assert_eq!((o.count, o.out_paise, o.inflow_paise), (2, 5_000, 2_000));
    }

    #[test]
    fn small_tickets_are_a_monthly_average_over_ninety_days() {
        let mut txns: Vec<Txn> = (1..=9).map(|i| txn(&format!("2026-10-{i:02}"), true, 15_000, "food", UPI_SHOP)).collect();
        txns.push(txn("2026-10-20", true, 50_000, "food", UPI_SHOP)); // not small
        let out = build(&txns, None, today());
        assert_eq!(out.small_ticket.per_month_count, 3);
        assert_eq!(out.small_ticket.per_month_paise, 45_000);
        assert_eq!(out.small_ticket.threshold_paise, 20_000);
    }

    #[test]
    fn frequent_small_needs_eight_payments_to_one_payee() {
        let mut txns: Vec<Txn> = (1..=8).map(|i| txn(&format!("2026-10-{i:02}"), true, 10_000, "food", UPI_SHOP)).collect();
        txns.extend((1..=7).map(|i| txn(&format!("2026-09-{i:02}"), true, 10_000, "food", IMPS_SELF)));
        let out = build(&txns, None, today());
        assert_eq!(out.frequent_small.len(), 1);
        assert_eq!(out.frequent_small[0].count, 8);
        assert_eq!(out.frequent_small[0].paise, 80_000);
    }

    #[test]
    fn top_payees_are_classified_as_merchant_shop_or_person() {
        let corner = "UPI-CORNER TEA-CORNER@YBL-YESB0YBLUPI-612345678777-UPI";
        let ravi = "UPI-RAVI KUMAR-RAVIK123@OKAXIS-SBIN0001234-612345678907-PAID VIA CRED";
        let acme = "UPI-ACME FOODS PVT LTD-ACMEFOODS@YBL-YESB0YBLUPI-612345678888-UPI";
        let mut txns = vec![
            txn("2026-10-01", true, 100_000, "food", acme),
            txn("2026-10-02", true, 30_000, "food", ravi),
        ];
        for i in 3..=5 {
            txns.push(txn(&format!("2026-10-{i:02}"), true, 20_000, "food", corner));
        }
        let out = build(&txns, None, today());
        let kind = |name: &str| out.top_payees.iter().find(|p| p.name == name).map(|p| p.kind);
        assert_eq!(kind("ACME FOODS PVT LTD"), Some("merchant"));
        assert_eq!(kind("CORNER TEA"), Some("shop"));
        assert_eq!(kind("RAVI KUMAR"), Some("person"));
        assert_eq!(out.top_payees[0].name, "ACME FOODS PVT LTD");
    }

    #[test]
    fn weekday_spend_counts_monday_first() {
        // 2026-11-02 is a Monday and 2026-11-04 a Wednesday.
        let txns = vec![
            txn("2026-11-02", true, 1_000, "food", UPI_SHOP),
            txn("2026-11-04", true, 2_000, "food", UPI_SHOP),
        ];
        let out = build(&txns, None, today());
        assert_eq!(out.weekday_spend_paise, [1_000, 0, 2_000, 0, 0, 0, 0]);
    }

    #[test]
    fn notes_are_capped_and_use_the_real_numbers() {
        let mut txns = vec![
            txn("2026-08-01", false, 100_000, "income", SALARY),
            txn("2026-10-01", false, 100_000, "income", SALARY),
            txn("2026-10-02", true, 15_000, "food", UPI_SHOP),
            txn("2026-10-03", true, 15_000, "food", UPI_SHOP),
            txn("2026-10-06", true, 5_000, "groceries", UPI_SHOP),
        ];
        for m in ["08", "09", "10"] {
            txns.push(txn(&format!("2026-{m}-10"), true, 40_000, "investments", SIP));
        }
        let cov = CoverageIn { first: d("2026-07-15"), last: d("2026-11-08"), imports: 1 };
        let out = build(&txns, Some(cov), today());
        assert!(out.notes.len() <= NOTE_LIMIT);
        assert!(out.notes.contains(&"Salary arrives around the 1st.".to_string()));
        assert!(out.notes.contains(&"September 2026 had no salary.".to_string()));
        assert!(out.notes.iter().any(|n| n.starts_with("₹") && n.contains("under ₹200")));
        assert!(out.notes.iter().any(|n| n.contains("1 SIP debits around the 10th.")));
    }

    #[test]
    fn ignores_transactions_after_today() {
        let txns = vec![txn("2026-11-20", true, 99_000, "food", UPI_SHOP)];
        let out = build(&txns, None, today());
        assert_eq!(month(&out, "2026-11").spend_paise, 0);
        assert!(out.categories.is_empty());
    }

    #[test]
    fn salary_note_reads_a_month_end_window() {
        assert_eq!(
            salary_days_note(&[2, 15, 29, 30, 31]).as_deref(),
            Some("Salary usually lands around month-end (29th–2nd); other times the 15th.")
        );
        assert_eq!(salary_days_note(&[1, 30]).as_deref(), Some("Salary usually lands around month-end (30th–1st)."));
        assert_eq!(salary_days_note(&[7]).as_deref(), Some("Salary arrives around the 7th."));
        assert_eq!(salary_days_note(&[7, 12]).as_deref(), Some("Salary arrived between the 7th and the 12th."));
        assert_eq!(salary_days_note(&[]), None);
    }


    #[test]
    fn truncated_business_names_are_merchants() {
        assert!(is_business("ACTIVE CAR ACCESSORI"));
        assert!(is_business("PASSPORT SEVA PROJEC"));
        assert!(is_business("SAINATH AUTO GARAGE"));
        assert!(!is_business("RAVI KUMAR"));
        assert!(!is_business("ASHA DE"), "short truncations of a person's name aren't businesses");
        assert_eq!(salary_window(&[2, 15, 29, 30, 31]).as_deref(), Some("Month-end (29th–2nd)"));
        assert_eq!(salary_window(&[7]).as_deref(), Some("Around the 7th"));
    }

}
