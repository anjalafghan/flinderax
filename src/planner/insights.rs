//! Insights from the user's own transactions: failed payments owed back, bank charges, double
//! charges, unusual months, a money-health score and a spend-to-payday range. Pure: no DB, no
//! clock. The caller passes `today`, the liquid balance and the next payday in. Money is paise.
//!
//! Failed payments are owed back within the RBI turnaround time (P2P T+1, P2M T+5). An unknown
//! rail uses the shorter T+1 so that a claim is never missed. Late payments accrue ₹100 a day.

use std::collections::{BTreeMap, BTreeSet};

use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Serialize;
use time::{Date, Duration};

use crate::alerts::normalize;
use crate::planner::{cycle::day_in_month_offset, format_inr};

/// Money in advice text: whole rupees (planning figures, not ledger entries).
fn inr(paise: i64) -> String {
    format_inr(paise.saturating_add(50).div_euclid(100).saturating_mul(100))
}

/// A transaction as the insights need it.
#[derive(Debug, Clone, PartialEq)]
pub struct Txn {
    pub txn_id: String,
    pub on: Date,
    pub debit: bool,
    pub amount: i64,
    pub merchant: Option<String>,
    /// Category label; `transfer` marks card bill payments and other known transfers.
    pub category: String,
    pub raw_text: Option<String>,
}

/// Seed for the spend-to-payday bootstrap, so the same data gives the same range.
pub const SEED: u64 = 0x5EED_2026;

const TRANSFER: &str = "transfer";
const P2P_TAT_DAYS: i64 = 1;
const P2M_TAT_DAYS: i64 = 5;
const COMPENSATION_PER_DAY_PAISE: i64 = 10_000;
const REVERSAL_WINDOW_DAYS: i64 = 30;
const CHARGES_WINDOW_DAYS: i64 = 365;
/// Debits at or above ₹25,000 are not "daily discretionary" spend.
const BIG_TICKET_PAISE: i64 = 2_500_000;
const SPEND_WINDOW_DAYS: i64 = 90;
const SIMULATIONS: usize = 500;
const TARGET_SAVINGS_BPS: i64 = 2_000;
const TARGET_EMERGENCY_TENTHS: i64 = 30;
const CV_ALERT_BPS: i64 = 2_500;
const CHARGES_ALERT_PAISE: i64 = 200_000;
const MERCHANT_WORDS: &[&str] = &[
    "store", "pvt", "ltd", "merchant", "bharatpe", "razorpay", "payu", "swiggy", "zomato", "amazon",
    "flipkart", "bigbasket", "blinkit", "zepto", "netflix", "spotify", "mart", "restaurant", "cafe",
];
const FAILED_WORDS: &[&str] = &["failed", "declined", "unsuccessful", "reversal pending", "will be reversed"];
const REVERSAL_WORDS: &[&str] = &["reversal", "refund", "reversed"];
/// Fee needle (lower case) -> label. Checked in order; the first match wins.
const FEE_LABELS: &[(&str, &str)] = &[
    ("annual fee", "Annual fee"),
    ("late payment fee", "Late payment fee"),
    ("finance charge", "Finance charge"),
    ("interest", "Interest"),
    ("sms charge", "SMS charges"),
    ("atm fee", "ATM fee"),
    ("forex markup", "Forex markup"),
    ("processing fee", "Processing fee"),
];

fn haystack(t: &Txn) -> String {
    format!("{} {}", t.merchant.as_deref().unwrap_or(""), t.raw_text.as_deref().unwrap_or("")).to_lowercase()
}

fn has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(*n))
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty())
}

fn has_merchant_word(text: &str) -> bool {
    words(text).any(|w| MERCHANT_WORDS.contains(&w))
}

fn month_index(d: Date) -> i32 {
    d.year() * 12 + i32::from(d.month() as u8) - 1
}

fn month_label(idx: i32) -> String {
    format!("{:04}-{:02}", idx.div_euclid(12), idx.rem_euclid(12) + 1)
}

fn median(mut values: Vec<i64>) -> i64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let n = values.len();
    if n % 2 == 1 { values[n / 2] } else { (values[n / 2 - 1] + values[n / 2]) / 2 }
}

/// Median absolute deviation from `centre`.
fn mad(values: &[i64], centre: i64) -> i64 {
    median(values.iter().map(|v| (v - centre).abs()).collect())
}

// ---------- rail and refunds ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rail {
    P2p,
    P2m,
    Unknown,
}

impl Rail {
    fn tat_days(self) -> i64 {
        match self {
            Rail::P2m => P2M_TAT_DAYS,
            Rail::P2p | Rail::Unknown => P2P_TAT_DAYS,
        }
    }
}

fn is_person_handle(handle: &str) -> bool {
    handle.starts_with("ok") || matches!(handle, "ybl" | "ibl" | "axl" | "paytm")
}

/// The payment rail from lower-case merchant/raw text. A VPA with a person-like handle whose name
/// has no merchant words is P2P; merchant words otherwise mean P2M.
pub fn rail_of(text: &str) -> Rail {
    let lower = text.to_lowercase();
    let person = lower
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')' | '"' | '\'' | '<' | '>'))
        .map(|tok| tok.trim_end_matches('.'))
        .filter_map(|tok| tok.split_once('@'))
        .any(|(name, handle)| is_person_handle(handle) && !has_merchant_word(name));
    if person {
        Rail::P2p
    } else if has_merchant_word(&lower) {
        Rail::P2m
    } else {
        Rail::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RefundStatus {
    Waiting,
    Overdue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Refund {
    pub txn_id: String,
    pub occurred_on: String,
    pub amount_paise: i64,
    pub merchant: Option<String>,
    pub rail: Rail,
    pub due_by: String,
    pub days_overdue: i64,
    pub compensation_paise: i64,
    pub status: RefundStatus,
}

/// Failed debits not yet reversed. A credit of the same amount that mentions a reversal or refund,
/// dated within 30 days after the debit, clears one debit (each credit clears at most one).
pub fn refunds(txns: &[Txn], today: Date) -> Vec<Refund> {
    let reversals: Vec<&Txn> = txns
        .iter()
        .filter(|t| !t.debit && has_any(&haystack(t), REVERSAL_WORDS))
        .collect();
    let mut failed: Vec<&Txn> = txns
        .iter()
        .filter(|t| t.debit && has_any(&haystack(t), FAILED_WORDS))
        .collect();
    failed.sort_by_key(|t| t.on);

    let mut used: BTreeSet<usize> = BTreeSet::new();
    let mut out = Vec::new();
    for d in failed {
        let rail = rail_of(&haystack(d));
        let due = d.on + Duration::days(rail.tat_days());
        let cleared = reversals
            .iter()
            .enumerate()
            .find(|(i, r)| {
                !used.contains(i)
                    && r.amount == d.amount
                    && r.on >= d.on
                    && (r.on - d.on).whole_days() <= REVERSAL_WINDOW_DAYS
            })
            .map(|(i, _)| i);
        if let Some(i) = cleared {
            used.insert(i);
            continue;
        }
        let overdue = (today - due).whole_days().max(0);
        out.push(Refund {
            txn_id: d.txn_id.clone(),
            occurred_on: d.on.to_string(),
            amount_paise: d.amount,
            merchant: d.merchant.clone(),
            rail,
            due_by: due.to_string(),
            days_overdue: overdue,
            compensation_paise: overdue * COMPENSATION_PER_DAY_PAISE,
            status: if overdue > 0 { RefundStatus::Overdue } else { RefundStatus::Waiting },
        });
    }
    out
}

// ---------- charges ----------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChargeItem {
    pub label: String,
    pub amount_paise: i64,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Charges {
    pub last_12m_paise: i64,
    pub items: Vec<ChargeItem>,
}

/// The label of a fee debit. A GST line that names a fee (e.g. "GST on annual fee") groups with
/// that fee; any other GST-on-fee line is "GST on fees".
pub fn charge_label(text: &str) -> Option<&'static str> {
    if let Some(&(_, label)) = FEE_LABELS.iter().find(|(needle, _)| text.contains(*needle)) {
        return Some(label);
    }
    (text.contains("gst") && text.contains("fee")).then_some("GST on fees")
}

/// Fee debits in the last 12 months, grouped by label, largest first.
pub fn charges(txns: &[Txn], today: Date) -> Charges {
    let since = today - Duration::days(CHARGES_WINDOW_DAYS);
    let mut by_label: BTreeMap<&'static str, (i64, usize)> = BTreeMap::new();
    for t in txns.iter().filter(|t| t.debit && t.on >= since && t.on <= today) {
        if let Some(label) = charge_label(&haystack(t)) {
            let e = by_label.entry(label).or_insert((0, 0));
            e.0 = e.0.saturating_add(t.amount);
            e.1 += 1;
        }
    }
    let mut items: Vec<ChargeItem> = by_label
        .into_iter()
        .map(|(label, (amount_paise, count))| ChargeItem { label: label.to_string(), amount_paise, count })
        .collect();
    items.sort_by(|a, b| b.amount_paise.cmp(&a.amount_paise).then_with(|| a.label.cmp(&b.label)));
    let last_12m_paise = items.iter().fold(0_i64, |acc, i| acc.saturating_add(i.amount_paise));
    Charges { last_12m_paise, items }
}

// ---------- duplicates ----------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Duplicate {
    pub merchant: String,
    /// The amount of ONE charge.
    pub amount_paise: i64,
    pub occurred_on: String,
    /// How many identical charges were seen that day.
    pub count: usize,
    pub txn_ids: Vec<String>,
}

/// Debits with the same merchant and amount on the same day, seen at least twice. Transfers excluded.
pub fn duplicates(txns: &[Txn]) -> Vec<Duplicate> {
    let mut groups: BTreeMap<(String, i64, Date), Vec<&Txn>> = BTreeMap::new();
    for t in txns.iter().filter(|t| t.debit && t.category != TRANSFER) {
        let Some(merchant) = t.merchant.as_deref() else { continue };
        let key = normalize(merchant);
        if key.is_empty() {
            continue;
        }
        groups.entry((key, t.amount, t.on)).or_default().push(t);
    }
    groups
        .into_values()
        .filter(|g| g.len() >= 2)
        .map(|g| Duplicate {
            merchant: g[0].merchant.clone().unwrap_or_default(),
            amount_paise: g[0].amount,
            occurred_on: g[0].on.to_string(),
            count: g.len(),
            txn_ids: g.iter().map(|t| t.txn_id.clone()).collect(),
        })
        .collect()
}

// ---------- unusual months ----------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Unusual {
    pub category: String,
    /// YYYY-MM
    pub month: String,
    pub amount_paise: i64,
    /// Median of the previous 6 months.
    pub typical_paise: i64,
}

/// A completed month whose debit total in a category is above median + 3 x MAD of the previous 6
/// calendar months (missing months count as zero), and by at least a quarter and ₹1,000. Needs at
/// least 4 non-zero months in that window.
/// The month in progress is skipped because it is not complete.
/// Smallest rupee margin over the usual month that counts as unusual (₹1,000).
const UNUSUAL_MIN_MARGIN_PAISE: i64 = 100_000;

pub fn unusual(txns: &[Txn], today: Date) -> Vec<Unusual> {
    let current = month_index(today);
    let mut by_cat: BTreeMap<&str, BTreeMap<i32, i64>> = BTreeMap::new();
    for t in txns.iter().filter(|t| t.debit && t.category != TRANSFER) {
        let m = month_index(t.on);
        if m >= current {
            continue;
        }
        let e = by_cat.entry(t.category.as_str()).or_default().entry(m).or_insert(0);
        *e = e.saturating_add(t.amount);
    }
    let mut out = Vec::new();
    for (category, months) in &by_cat {
        for (&m, &amount) in months {
            let window: Vec<i64> = (m - 6..m).map(|p| months.get(&p).copied().unwrap_or(0)).collect();
            if window.iter().filter(|v| **v > 0).count() < 4 {
                continue;
            }
            let typical = median(window.clone());
            let spread = mad(&window, typical);
            // A floor on the margin: when spending barely varies (MAD ≈ 0) a few rupees over the
            // usual would otherwise be "unusual". Needs 3×MAD, a quarter over usual, and ₹1,000.
            let margin = spread.saturating_mul(3).max(typical / 4).max(UNUSUAL_MIN_MARGIN_PAISE);
            if amount > typical.saturating_add(margin) {
                out.push(Unusual {
                    category: category.to_string(),
                    month: month_label(m),
                    amount_paise: amount,
                    typical_paise: typical,
                });
            }
        }
    }
    out.sort_by(|a, b| b.month.cmp(&a.month).then_with(|| a.category.cmp(&b.category)));
    out
}

// ---------- health ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Impact {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Action {
    pub text: String,
    pub impact: Impact,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Health {
    /// 0..=100.
    pub score: i64,
    pub savings_rate_bps: Option<i64>,
    pub emergency_months_tenths: Option<i64>,
    pub income_cv_bps: Option<i64>,
    /// Ranked by impact, at most 4.
    pub actions: Vec<Action>,
}

/// (credits, debits) per completed calendar month, keyed by month index.
fn monthly(txns: &[Txn], today: Date) -> BTreeMap<i32, (i64, i64)> {
    let current = month_index(today);
    let mut map: BTreeMap<i32, (i64, i64)> = BTreeMap::new();
    for t in txns.iter().filter(|t| month_index(t.on) < current) {
        let e = map.entry(month_index(t.on)).or_insert((0, 0));
        if t.debit {
            e.1 = e.1.saturating_add(t.amount);
        } else {
            e.0 = e.0.saturating_add(t.amount);
        }
    }
    map
}

/// Population coefficient of variation in basis points. None with fewer than 2 values.
fn cv_bps(values: &[i64]) -> Option<i64> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().map(|v| *v as f64).sum::<f64>() / n;
    if mean <= 0.0 {
        return None;
    }
    let var = values.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / n;
    Some((var.sqrt() / mean * 10_000.0).round() as i64)
}

/// Money health. Score starts at 100 and loses points for:
/// - savings rate below 20% (up to 30, scaled by the shortfall),
/// - emergency cover below 3 months (up to 25, scaled),
/// - income CV above 25% (10),
/// - bank charges above ₹2,000 in 12 months (10).
///
/// Savings rate and emergency cover use the last 3 completed months. Income CV uses the last 6.
/// Missing data gives no penalty and no number.
pub fn health(txns: &[Txn], liquid_paise: i64, open_refunds: &[Refund], today: Date) -> Health {
    let months = monthly(txns, today);
    let current = month_index(today);
    let (credits, debits) = (current - 3..current)
        .filter_map(|m| months.get(&m).copied())
        .fold((0_i64, 0_i64), |(c, d), (mc, md)| (c.saturating_add(mc), d.saturating_add(md)));
    let avg_credits = credits / 3;
    let avg_debits = debits / 3;
    let avg_surplus = (credits - debits) / 3;

    let savings_rate_bps = (credits > 0).then(|| (credits - debits).saturating_mul(10_000) / credits);
    let emergency = (avg_debits > 0).then(|| liquid_paise.max(0).saturating_mul(10) / avg_debits);
    let incomes: Vec<i64> = (current - 6..current)
        .filter_map(|m| months.get(&m).map(|v| v.0))
        .filter(|c| *c > 0)
        .collect();
    let income_cv = cv_bps(&incomes);
    let charges_out = charges(txns, today);

    let mut score = 100.0_f64;
    if let Some(rate) = savings_rate_bps.filter(|r| *r < TARGET_SAVINGS_BPS) {
        score -= ((TARGET_SAVINGS_BPS - rate) as f64 * 30.0 / TARGET_SAVINGS_BPS as f64).min(30.0);
    }
    if let Some(t) = emergency.filter(|t| *t < TARGET_EMERGENCY_TENTHS) {
        score -= ((TARGET_EMERGENCY_TENTHS - t) as f64 * 25.0 / TARGET_EMERGENCY_TENTHS as f64).min(25.0);
    }
    if income_cv.is_some_and(|cv| cv > CV_ALERT_BPS) {
        score -= 10.0;
    }
    if charges_out.last_12m_paise > CHARGES_ALERT_PAISE {
        score -= 10.0;
    }
    let score = score.round().clamp(0.0, 100.0) as i64;

    let mut actions: Vec<Action> = Vec::new();
    if !open_refunds.is_empty() {
        let total = open_refunds.iter().fold(0_i64, |acc, r| acc.saturating_add(r.amount_paise));
        let overdue = open_refunds.iter().filter(|r| r.status == RefundStatus::Overdue).count();
        actions.push(Action {
            text: match overdue {
                0 => format!("Watch {} of failed payments that haven't been reversed yet.", inr(total)),
                1 => format!("Chase {} of failed payments that haven't been reversed; one is past its refund deadline.", inr(total)),
                n => format!("Chase {} of failed payments that haven't been reversed; {n} are past their refund deadline.", inr(total)),
            },
            impact: if overdue > 0 { Impact::High } else { Impact::Medium },
        });
    }
    if let Some(tenths) = emergency.filter(|t| *t < TARGET_EMERGENCY_TENTHS) {
        let needed = (3 * avg_debits - liquid_paise.max(0)).max(0);
        if needed > 0 {
            actions.push(Action {
                text: format!("Keep {} more in liquid balances to reach 3 months of spending.", inr(needed)),
                impact: if tenths < 10 { Impact::High } else { Impact::Medium },
            });
        }
    }
    if let Some(rate) = savings_rate_bps.filter(|r| *r < TARGET_SAVINGS_BPS) {
        let gap = avg_credits * TARGET_SAVINGS_BPS / 10_000 - avg_surplus;
        if gap > 0 {
            actions.push(Action {
                text: format!("Save {} more a month to keep 20% of your income.", inr(gap)),
                impact: if rate < 1_000 { Impact::High } else { Impact::Medium },
            });
        }
    }
    if charges_out.last_12m_paise > CHARGES_ALERT_PAISE {
        let top = charges_out.items.first().map(|i| i.label.clone()).unwrap_or_default();
        actions.push(Action {
            text: format!(
                "Bank charges cost {} in the last 12 months, mostly {}. Ask the bank to waive or switch off the ones you do not need.",
                inr(charges_out.last_12m_paise),
                top
            ),
            impact: Impact::Medium,
        });
    }
    if let Some(cv) = income_cv.filter(|cv| *cv > CV_ALERT_BPS) {
        actions.push(Action {
            text: format!(
                "Your monthly income swings by about {}%. Plan spending on your lowest month and keep the emergency fund liquid.",
                cv / 100
            ),
            impact: Impact::Medium,
        });
    }
    if actions.is_empty() {
        actions.push(Action { text: "Nothing urgent: keep saving steadily.".to_string(), impact: Impact::Low });
    }
    actions.sort_by_key(|a| a.impact);
    actions.truncate(4);

    Health {
        score,
        savings_rate_bps,
        emergency_months_tenths: emergency,
        income_cv_bps: income_cv,
        actions,
    }
}

// ---------- spend to payday ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SpendToPayday {
    pub days: i64,
    pub p10_paise: i64,
    pub p50_paise: i64,
    pub p90_paise: i64,
}

fn end_of_month(d: Date) -> Date {
    day_in_month_offset(d, 1, 1) - Duration::days(1)
}

/// Bootstrap of the discretionary spend until the next payday. Daily totals from the last 90 days
/// (excluding transfers and debits of ₹25,000 or more) are resampled `days` times per simulation.
/// `next_payday` None (or not after today) means the end of the month. None when there is no
/// discretionary debit in the window.
pub fn spend_to_payday(txns: &[Txn], today: Date, next_payday: Option<Date>, seed: u64) -> Option<SpendToPayday> {
    let payday = match next_payday.filter(|p| *p > today) {
        Some(p) => p,
        None => {
            let eom = end_of_month(today);
            if eom > today { eom } else { end_of_month(today + Duration::days(1)) }
        }
    };
    let days = (payday - today).whole_days().max(1);

    let start = today - Duration::days(SPEND_WINDOW_DAYS);
    let mut daily = vec![0_i64; SPEND_WINDOW_DAYS as usize];
    let mut seen = false;
    for t in txns.iter().filter(|t| {
        t.debit && t.category != TRANSFER && t.amount < BIG_TICKET_PAISE && t.on >= start && t.on < today
    }) {
        let idx = (t.on - start).whole_days() as usize;
        daily[idx] = daily[idx].saturating_add(t.amount);
        seen = true;
    }
    if !seen {
        return None;
    }

    let mut rng = StdRng::seed_from_u64(seed);
    let len = daily.len();
    let mut sums = Vec::with_capacity(SIMULATIONS);
    for _ in 0..SIMULATIONS {
        let mut total = 0_i64;
        for _ in 0..days {
            total = total.saturating_add(daily[rng.gen_range(0..len)]);
        }
        sums.push(total);
    }
    sums.sort_unstable();
    let pct = |p: usize| sums[(sums.len() - 1) * p / 100];
    Some(SpendToPayday { days, p10_paise: pct(10), p50_paise: pct(50), p90_paise: pct(90) })
}

// ---------- everything ----------

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InsightsOut {
    pub refunds: Vec<Refund>,
    pub charges: Charges,
    pub duplicates: Vec<Duplicate>,
    pub unusual: Vec<Unusual>,
    pub health: Health,
    pub spend_to_payday: Option<SpendToPayday>,
}

/// Runs every insight. `next_payday` None means end of month.
pub fn build(txns: &[Txn], liquid_paise: i64, next_payday: Option<Date>, today: Date) -> InsightsOut {
    let refunds = refunds(txns, today);
    InsightsOut {
        charges: charges(txns, today),
        duplicates: duplicates(txns),
        unusual: unusual(txns, today),
        health: health(txns, liquid_paise, &refunds, today),
        spend_to_payday: spend_to_payday(txns, today, next_payday, SEED),
        refunds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn d(y: i32, m: u8, day: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), day).unwrap()
    }

    fn today() -> Date {
        d(2026, 11, 9)
    }

    fn txn(id: &str, on: Date, debit: bool, amount: i64, merchant: Option<&str>, category: &str, raw: Option<&str>) -> Txn {
        Txn {
            txn_id: id.into(),
            on,
            debit,
            amount,
            merchant: merchant.map(Into::into),
            category: category.into(),
            raw_text: raw.map(Into::into),
        }
    }

    #[test]
    fn failed_p2p_debit_is_due_next_day_and_accrues_compensation() {
        let txns = [txn("t1", d(2026, 11, 1), true, 145_000, Some("rahul.k@okaxis"), "uncategorized", Some("UPI payment to rahul.k@okaxis failed"))];
        let r = refunds(&txns, today());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].rail, Rail::P2p);
        assert_eq!(r[0].due_by, "2026-11-02");
        assert_eq!(r[0].days_overdue, 7);
        assert_eq!(r[0].compensation_paise, 70_000);
        assert_eq!(r[0].status, RefundStatus::Overdue);
        // On the due day itself it is still waiting, with nothing owed.
        let r = refunds(&txns, d(2026, 11, 2));
        assert_eq!((r[0].status, r[0].days_overdue, r[0].compensation_paise), (RefundStatus::Waiting, 0, 0));
    }

    #[test]
    fn failed_p2m_debit_is_due_in_five_days() {
        let txns = [txn("t2", d(2026, 11, 1), true, 80_000, Some("SWIGGY"), "food", Some("Payment to SWIGGY failed"))];
        let r = refunds(&txns, d(2026, 11, 4));
        assert_eq!(r[0].rail, Rail::P2m);
        assert_eq!(r[0].due_by, "2026-11-06");
        assert_eq!((r[0].status, r[0].days_overdue), (RefundStatus::Waiting, 0));
        let r = refunds(&txns, today());
        assert_eq!((r[0].days_overdue, r[0].compensation_paise), (3, 30_000));
    }

    #[test]
    fn reversal_credit_clears_the_refund_but_other_amounts_do_not() {
        let failed = txn("t1", d(2026, 11, 1), true, 14_500, Some("rahul.k@okaxis"), "uncategorized", Some("UPI failed"));
        let other_amount = txn("c0", d(2026, 11, 3), false, 9_900, None, "uncategorized", Some("UPI reversal"));
        let reversal = txn("c1", d(2026, 11, 3), false, 14_500, None, "uncategorized", Some("UPI reversal of failed txn"));
        assert_eq!(refunds(&[failed.clone(), other_amount.clone()], today()).len(), 1);
        assert!(refunds(&[failed, other_amount, reversal], today()).is_empty());
    }

    #[test]
    fn fees_group_by_label_with_their_gst_and_ignore_old_ones() {
        let txns = [
            txn("f1", d(2026, 10, 20), true, 50_000, Some("HDFC Bank"), "uncategorized", Some("Annual fee debited for card")),
            txn("f2", d(2026, 10, 20), true, 9_000, Some("HDFC Bank"), "uncategorized", Some("GST on annual fee")),
            txn("f3", d(2025, 9, 1), true, 40_000, Some("HDFC Bank"), "uncategorized", Some("Annual fee debited")),
        ];
        let c = charges(&txns, today());
        assert_eq!(c.last_12m_paise, 59_000);
        assert_eq!(
            c.items.iter().map(|i| (i.label.as_str(), i.amount_paise, i.count)).collect::<Vec<_>>(),
            vec![("Annual fee", 59_000, 2)]
        );
    }

    #[test]
    fn duplicate_same_day_same_amount_is_flagged_once() {
        let txns = [
            txn("s1", d(2026, 11, 2), true, 25_000, Some("Swiggy"), "food", None),
            txn("s2", d(2026, 11, 2), true, 25_000, Some("SWIGGY"), "food", None),
            txn("s3", d(2026, 11, 3), true, 25_000, Some("Swiggy"), "food", None),
            txn("x1", d(2026, 11, 4), true, 10_000, Some("Rent"), "transfer", None),
            txn("x2", d(2026, 11, 4), true, 10_000, Some("Rent"), "transfer", None),
        ];
        let dups = duplicates(&txns);
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0].count, 2);
        assert_eq!(dups[0].amount_paise, 25_000, "amount is one charge, not the total");
        assert_eq!(dups[0].txn_ids, vec!["s1", "s2"]);
    }

    #[test]
    fn unusual_month_is_flagged_and_a_normal_one_is_not() {
        let mut txns = Vec::new();
        // Food: Apr..Oct 2026 in hundreds of rupees; October spikes (₹15,000 vs a usual ₹10,000).
        let food = [100, 104, 96, 98, 102, 100, 150];
        for (i, k) in food.iter().enumerate() {
            txns.push(txn(&format!("f{i}"), d(2026, 4 + i as u8, 15), true, k * 10_000, Some("Cafe X"), "food", None));
        }
        // Groceries: only 3 months of data plus a spike, so not enough history.
        for (i, k) in [1, 1, 1, 50].iter().enumerate() {
            txns.push(txn(&format!("g{i}"), d(2026, 7 + i as u8, 10), true, k * 1_000, Some("Mart"), "groceries", None));
        }
        let u = unusual(&txns, today());
        assert_eq!(u.len(), 1, "{u:?}");
        assert_eq!((u[0].category.as_str(), u[0].month.as_str()), ("food", "2026-10"));
        assert_eq!((u[0].amount_paise, u[0].typical_paise), (1_500_000, 1_000_000));
    }

    #[test]
    fn health_with_no_data_does_not_panic() {
        let h = health(&[], 0, &[], today());
        assert_eq!(h.score, 100);
        assert_eq!((h.savings_rate_bps, h.emergency_months_tenths, h.income_cv_bps), (None, None, None));
        assert_eq!(h.actions.len(), 1);
        assert_eq!(h.actions[0].impact, Impact::Low);
    }

    #[test]
    fn savings_rate_and_score_follow_the_documented_maths() {
        let mut txns = Vec::new();
        for m in [8u8, 9, 10] {
            txns.push(txn("c", d(2026, m, 5), false, 5_000_000, Some("Employer"), "salary", None));
            txns.push(txn("d", d(2026, m, 10), true, 4_000_000, Some("Rent"), "bills", None));
        }
        // The current month is not a full month, so it must not count.
        txns.push(txn("now", d(2026, 11, 2), false, 9_000_000, Some("Employer"), "salary", None));
        let h = health(&txns, 0, &[], today());
        // (15M - 12M) / 15M = 20%: exactly on target, no savings penalty.
        assert_eq!(h.savings_rate_bps, Some(2_000));
        // No emergency money against 4M a month of spending: 0 tenths, a 25 point penalty.
        assert_eq!(h.emergency_months_tenths, Some(0));
        assert_eq!(h.score, 75);
        assert!(h.actions.iter().any(|a| a.text.starts_with("Keep")));
    }

    #[test]
    fn bootstrap_is_deterministic_and_ordered() {
        let txns: Vec<Txn> = (1..=60)
            .map(|k| txn(&format!("b{k}"), today() - Duration::days(k), true, 1_000 * k, Some("Grocer"), "groceries", None))
            .collect();
        let payday = d(2026, 11, 20);
        let a = spend_to_payday(&txns, today(), Some(payday), SEED).unwrap();
        let b = spend_to_payday(&txns, today(), Some(payday), SEED).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.days, 11);
        assert!(a.p10_paise <= a.p50_paise && a.p50_paise <= a.p90_paise);
        assert!(a.p10_paise > 0);
        // With no payday given, the window runs to the end of the month.
        assert_eq!(spend_to_payday(&txns, today(), None, SEED).unwrap().days, 21);
    }

    #[test]
    fn no_transactions_gives_no_spend_estimate() {
        assert_eq!(spend_to_payday(&[], today(), None, 1), None);
        assert_eq!(build(&[], 0, None, today()).spend_to_payday, None);
    }

    #[test]
    fn rails_are_classified_from_the_payee() {
        assert_eq!(rail_of("upi payment to rahul.k@okaxis"), Rail::P2p);
        assert_eq!(rail_of("paid to swiggy@icici"), Rail::P2m);
        assert_eq!(rail_of("atm withdrawal"), Rail::Unknown);
    }

    #[test]
    fn median_and_mad_are_robust_to_one_outlier() {
        let v = vec![100, 101, 99, 100, 10_000];
        let m = median(v.clone());
        assert_eq!(m, 100);
        assert_eq!(mad(&v, m), 1);
    }

    #[test]
    fn a_steady_category_is_not_flagged_for_a_few_rupees_more() {
        let today = time::macros::date!(2026 - 11 - 09);
        let mut txns = Vec::new();
        // Six months at exactly ₹2,000 (MAD 0), then October at ₹2,300: not unusual.
        for (m, amount) in [(4u8, 200_000), (5, 200_000), (6, 200_000), (7, 200_000), (8, 200_000), (9, 200_000), (10, 230_000)] {
            txns.push(Txn {
                on: Date::from_calendar_date(2026, time::Month::try_from(m).unwrap(), 10).unwrap(),
                debit: true,
                amount,
                merchant: Some("Airtel".into()),
                category: "phone".into(),
                raw_text: None,
                txn_id: format!("t{m}"),
            });
        }
        assert!(unusual(&txns, today).is_empty());
    }

}
