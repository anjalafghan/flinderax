//! Phase 2 quarterly check-in, pure: what is due and what is left, the quarter report and the
//! estimated balances between check-ins. No DB, no clock: the handler loads the rows and passes
//! `today`. Money is paise. Dates are `Date` here and 'YYYY-MM-DD' strings in the JSON.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use crate::alerts::is_spending;
use crate::planner::habits::Habit;
use crate::planner::{format_inr, short_date};

/// The next check-in is due this many days after the last one was done.
pub const CADENCE_DAYS: i64 = 91;
/// With no check-in yet, the first is due this many days after the earliest bank statement.
const FIRST_DUE_AFTER_DAYS: i64 = 90;
/// With no check-in yet, the first period looks back this many days.
const FIRST_PERIOD_DAYS: i64 = 90;
/// A statement whose coverage ends more than this many days ago needs a newer one.
const STATEMENT_STALE_DAYS: i64 = 35;
/// `due_soon` when the due date is within this many days, inclusive.
const DUE_SOON_DAYS: i64 = 14;
/// Uncategorised payees are fine while their total is under this percentage of the period's spend.
const UNCATEGORISED_PCT: i64 = 10;
/// Balances set this many days ago or more get an estimate.
const ESTIMATE_AFTER_DAYS: i64 = 7;
/// An estimate needs at least this many days between an account's first and last transaction.
const ESTIMATE_HISTORY_DAYS: i64 = 30;
/// The net flow behind an estimate is taken over this many days ending on the day the balance was set.
const FLOW_DAYS: i64 = 90;
/// Most spending categories in the report.
const MAX_CATEGORIES: usize = 8;
/// Most wins, and most watch lines.
const MAX_LINES: usize = 3;
/// Most habits in the report.
const MAX_HABITS: usize = 3;
/// A category swing counts when it moves by at least this many basis points (20%) ...
const SWING_BPS: i64 = 2_000;
/// ... and by at least this many paise (₹2,000).
const SWING_PAISE: i64 = 200_000;
/// Income that rose by this many basis points (10%) is a win.
const INCOME_RISE_BPS: i64 = 1_000;

/// A span of days, both ends included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub start: Date,
    pub end: Date,
}

impl Period {
    /// Days in the period, counting both ends. Zero or less when the period is inverted.
    pub fn days(&self) -> i64 {
        (self.end - self.start).whole_days() + 1
    }

    /// The same number of days immediately before this period.
    pub fn previous(&self) -> Period {
        let end = self.start - Duration::days(1);
        Period {
            start: end - Duration::days(self.days() - 1),
            end,
        }
    }

    pub fn span(&self) -> Span {
        Span {
            start: self.start.to_string(),
            end: self.end.to_string(),
        }
    }

    fn contains(&self, on: Date) -> bool {
        on >= self.start && on <= self.end
    }
}

/// A period as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: String,
    pub end: String,
}

/// The period a check-in covers: from the day after the last check-in's period, or the last 90 days.
pub fn period_after(today: Date, last_period_end: Option<Date>) -> Period {
    match last_period_end {
        Some(end) => Period {
            start: end + Duration::days(1),
            end: today,
        },
        None => Period {
            start: today - Duration::days(FIRST_PERIOD_DAYS),
            end: today,
        },
    }
}

/// The period the status covers, given the last check-in.
/// A check-in finished today keeps showing the period it covered, until tomorrow.
pub fn period_for(today: Date, last: Option<&LastCheckin>) -> Period {
    match last {
        Some(l) if l.done_on == today => Period { start: l.period_start.min(l.period_end), end: l.period_end },
        _ => period_after(today, last.map(|l| l.period_end)),
    }
}

/// A check-in already done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastCheckin {
    pub done_on: Date,
    pub period_start: Date,
    pub period_end: Date,
}

/// A money account, with the last day its statements cover and its balance as last set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountIn {
    pub account_id: String,
    pub name: String,
    /// 'bank', 'fd', 'rd' or 'cash'.
    pub kind: String,
    pub locked: bool,
    pub balance_paise: i64,
    pub balance_set_on: Option<Date>,
    pub covered_to: Option<Date>,
}

/// A card, with the last day its statements cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardIn {
    pub card_id: String,
    pub name: String,
    pub covered_to: Option<Date>,
}

/// One transaction, as the check-in reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxnIn {
    pub on: Date,
    pub debit: bool,
    pub amount_paise: i64,
    pub merchant: Option<String>,
    pub category: String,
    pub account_id: Option<String>,
}

/// Everything the status reads, loaded for one user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusIn {
    pub today: Date,
    pub last: Option<LastCheckin>,
    pub accounts: Vec<AccountIn>,
    pub cards: Vec<CardIn>,
    pub txns: Vec<TxnIn>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DueState {
    Overdue,
    DueSoon,
    Ok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Account,
    Card,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Upload {
    pub kind: Kind,
    pub id: String,
    pub name: String,
    pub covered_to: Option<String>,
    pub needed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    pub key: &'static str,
    pub title: &'static str,
    pub detail: String,
    pub done: bool,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Estimate {
    pub account_id: String,
    pub name: String,
    pub known_paise: i64,
    pub known_on: String,
    pub estimate_paise: i64,
    pub days_since: i64,
    pub basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckinStatus {
    pub last_done_on: Option<String>,
    /// A check-in was finished today: `period` is the one it covered.
    pub done_today: bool,
    pub next_due_on: String,
    pub status: DueState,
    pub period: Span,
    pub uploads: Vec<Upload>,
    pub steps: Vec<Step>,
    pub estimates: Vec<Estimate>,
}

/// Totals for one span of days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Totals {
    pub income_paise: i64,
    pub spend_paise: i64,
    pub invested_paise: i64,
    pub savings_rate_bps: Option<i64>,
}

/// One spending category, this span against the span before.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CategoryChange {
    pub category: String,
    pub paise: i64,
    pub prev_paise: i64,
    pub change_bps: Option<i64>,
}

/// What `report` returns: totals, previous totals, categories, wins, watch.
pub type ReportParts = (
    Totals,
    Option<Totals>,
    Vec<CategoryChange>,
    Vec<String>,
    Vec<String>,
);

/// The quarter report, as shown and as saved with a finished check-in.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuarterReport {
    pub period: Span,
    pub previous: Span,
    pub totals: Totals,
    pub prev: Option<Totals>,
    pub categories: Vec<CategoryChange>,
    pub wins: Vec<String>,
    pub watch: Vec<String>,
    pub habits: Vec<Habit>,
}

/// The status: what is due, the four steps, and the estimated balances.
pub fn status(input: &StatusIn) -> CheckinStatus {
    let today = input.today;
    let last = input.last.as_ref();
    let next = next_due(today, last, &input.accounts);
    let period = period_for(today, last);
    let uploads = upload_list(today, &input.accounts, &input.cards);
    let in_period: Vec<&TxnIn> = input.txns.iter().filter(|t| period.contains(t.on)).collect();
    let steps = vec![
        statements_step(&uploads),
        balances_step(&input.accounts, period),
        payees_step(&in_period),
        report_step(period),
    ];
    CheckinStatus {
        last_done_on: last.map(|l| l.done_on.to_string()),
        done_today: last.is_some_and(|l| l.done_on == today),
        next_due_on: next.to_string(),
        status: due_state(today, next),
        period: period.span(),
        uploads,
        steps,
        estimates: estimates(today, &input.accounts, &input.txns),
    }
}

/// When the next check-in is due: 91 days after the last one, or, with none yet, 90 days after the
/// earliest bank statement, or today when no bank statement exists.
pub fn next_due(today: Date, last: Option<&LastCheckin>, accounts: &[AccountIn]) -> Date {
    if let Some(l) = last {
        return l.done_on + Duration::days(CADENCE_DAYS);
    }
    accounts
        .iter()
        .filter(|a| a.kind == "bank")
        .filter_map(|a| a.covered_to)
        .min()
        .map_or(today, |d| d + Duration::days(FIRST_DUE_AFTER_DAYS))
}

pub fn due_state(today: Date, next: Date) -> DueState {
    if next < today {
        DueState::Overdue
    } else if (next - today).whole_days() <= DUE_SOON_DAYS {
        DueState::DueSoon
    } else {
        DueState::Ok
    }
}

/// Bank accounts that are not locked, then every card. A statement is needed when none covers
/// the account, or the newest one ends more than 35 days ago.
fn upload_list(today: Date, accounts: &[AccountIn], cards: &[CardIn]) -> Vec<Upload> {
    let stale_before = today - Duration::days(STATEMENT_STALE_DAYS);
    let mut out: Vec<Upload> = accounts
        .iter()
        .filter(|a| a.kind == "bank" && !a.locked)
        .map(|a| upload(Kind::Account, &a.account_id, &a.name, a.covered_to, stale_before))
        .collect();
    out.extend(
        cards
            .iter()
            .map(|c| upload(Kind::Card, &c.card_id, &c.name, c.covered_to, stale_before)),
    );
    out
}

fn upload(kind: Kind, id: &str, name: &str, covered: Option<Date>, stale_before: Date) -> Upload {
    Upload {
        kind,
        id: id.to_string(),
        name: name.to_string(),
        covered_to: covered.map(|d| d.to_string()),
        needed: covered.is_none_or(|d| d < stale_before),
    }
}

fn statements_step(uploads: &[Upload]) -> Step {
    let total = uploads.len();
    let needed = uploads.iter().filter(|u| u.needed).count();
    let detail = if total == 0 {
        "Add a bank account or a card to start.".to_string()
    } else if needed == 0 {
        "Everything is covered to within the last five weeks.".to_string()
    } else if needed == 1 {
        format!("1 of {total} needs a newer statement.")
    } else {
        format!("{needed} of {total} need a newer statement.")
    };
    Step {
        key: "statements",
        title: "Upload your statements",
        detail,
        done: needed == 0,
        count: needed as i64,
    }
}

/// Unlocked accounts whose balance was not set since the period began.
fn balances_step(accounts: &[AccountIn], period: Period) -> Step {
    let open: Vec<&AccountIn> = accounts.iter().filter(|a| !a.locked).collect();
    let stale = open
        .iter()
        .filter(|a| a.balance_set_on.is_none_or(|d| d < period.start))
        .count();
    let detail = if open.is_empty() {
        "Add an account to start.".to_string()
    } else if stale == 0 {
        "Every balance is confirmed for this quarter.".to_string()
    } else {
        let verb = if stale == 1 { "needs" } else { "need" };
        format!("{stale} of {} balances {verb} confirming.", open.len())
    };
    Step {
        key: "balances",
        title: "Confirm your balances",
        detail,
        done: stale == 0,
        count: stale as i64,
    }
}

/// Uncategorised debits in the period. Done when their total is under 10% of the period's spend.
fn payees_step(in_period: &[&TxnIn]) -> Step {
    let spend: i64 = in_period
        .iter()
        .filter(|t| t.debit && is_spending(&t.category))
        .map(|t| t.amount_paise)
        .sum();
    let open: Vec<&TxnIn> = in_period
        .iter()
        .copied()
        .filter(|t| t.debit && t.category == "uncategorized")
        .collect();
    let total: i64 = open.iter().map(|t| t.amount_paise).sum();
    let payees: BTreeSet<String> = open
        .iter()
        .map(|t| t.merchant.as_deref().unwrap_or("").trim().to_uppercase())
        .collect();
    let done = total == 0 || total * 100 < spend * UNCATEGORISED_PCT;
    let detail = if total == 0 {
        "Every payee this quarter has a category.".to_string()
    } else {
        let (noun, verb) = if payees.len() == 1 {
            ("payee", "has")
        } else {
            ("payees", "have")
        };
        format!(
            "{} across {} {noun} {verb} no category.",
            rupees(total),
            payees.len()
        )
    };
    Step {
        key: "payees",
        title: "Sort new payees",
        detail,
        done,
        count: if done { 0 } else { payees.len() as i64 },
    }
}

/// Always open: finishing the check-in completes it.
fn report_step(period: Period) -> Step {
    let detail = if period.days() > 0 {
        format!(
            "Read the {} days from {} to {}, then finish.",
            period.days(),
            short_date(period.start),
            short_date(period.end)
        )
    } else {
        "Nothing new since your last check-in. Read the quarter, then finish.".to_string()
    };
    Step {
        key: "report",
        title: "Read your quarter",
        detail,
        done: false,
        count: 1,
    }
}

/// Estimated balances for accounts whose balance was set at least a week ago and that have at
/// least 30 days of transactions. The estimate adds the usual net daily flow for each day since.
pub fn estimates(today: Date, accounts: &[AccountIn], txns: &[TxnIn]) -> Vec<Estimate> {
    let mut out: Vec<Estimate> = accounts
        .iter()
        .filter_map(|a| estimate(today, a, txns))
        .collect();
    out.sort_by(|x, y| x.name.cmp(&y.name).then_with(|| x.account_id.cmp(&y.account_id)));
    out
}

fn estimate(today: Date, a: &AccountIn, txns: &[TxnIn]) -> Option<Estimate> {
    if a.locked {
        return None;
    }
    let known_on = a.balance_set_on?;
    let days_since = (today - known_on).whole_days();
    if days_since < ESTIMATE_AFTER_DAYS {
        return None;
    }
    let mine: Vec<&TxnIn> = txns
        .iter()
        .filter(|t| t.account_id.as_deref() == Some(a.account_id.as_str()))
        .collect();
    let first = mine.iter().map(|t| t.on).min()?;
    let last = mine.iter().map(|t| t.on).max()?;
    if (last - first).whole_days() < ESTIMATE_HISTORY_DAYS {
        return None;
    }
    let window_start = known_on - Duration::days(FLOW_DAYS - 1);
    let net: i64 = mine
        .iter()
        .filter(|t| t.on >= window_start && t.on <= known_on)
        .map(|t| if t.debit { -t.amount_paise } else { t.amount_paise })
        .sum();
    let drift = i128::from(net) * i128::from(days_since) / i128::from(FLOW_DAYS);
    let estimate = i64::try_from((i128::from(a.balance_paise) + drift).max(0)).unwrap_or(i64::MAX);
    // Three months of net flow make the monthly figure in the basis sentence.
    let monthly = net / 3;
    let (way, amount) = if monthly >= 0 {
        ("in", monthly)
    } else {
        ("out", -monthly)
    };
    Some(Estimate {
        account_id: a.account_id.clone(),
        name: a.name.clone(),
        known_paise: a.balance_paise,
        known_on: known_on.to_string(),
        estimate_paise: estimate,
        days_since,
        basis: format!(
            "Usual net {} a month {way}, over {days_since} days since {}.",
            rupees(amount),
            short_date(known_on)
        ),
    })
}

/// Totals, previous totals, top categories, wins and watch lines for `period`, compared with
/// `previous`. `prev` is None when the previous span has no transactions at all.
pub fn report(txns: &[TxnIn], period: Period, previous: Period) -> ReportParts {
    let now: Vec<&TxnIn> = txns.iter().filter(|t| period.contains(t.on)).collect();
    let before: Vec<&TxnIn> = txns.iter().filter(|t| previous.contains(t.on)).collect();
    let totals = sum_totals(&now);
    let prev = (!before.is_empty()).then(|| sum_totals(&before));
    let categories = category_changes(&now, &before);
    let wins = win_lines(&totals, prev.as_ref(), &categories);
    let watch = watch_lines(&totals, prev.as_ref(), &categories);
    (totals, prev, categories, wins, watch)
}

/// The whole quarter report: the pure report plus the first three habits.
pub fn quarter_report(txns: &[TxnIn], period: Period, habits: Vec<Habit>) -> QuarterReport {
    let previous = period.previous();
    let (totals, prev, categories, wins, watch) = report(txns, period, previous);
    QuarterReport {
        period: period.span(),
        previous: previous.span(),
        totals,
        prev,
        categories,
        wins,
        watch,
        habits: habits.into_iter().take(MAX_HABITS).collect(),
    }
}

fn sum_totals(txns: &[&TxnIn]) -> Totals {
    let mut income = 0;
    let mut spend = 0;
    let mut invested = 0;
    for t in txns {
        if t.debit && is_spending(&t.category) {
            spend += t.amount_paise;
        }
        if t.debit && t.category == "investments" {
            invested += t.amount_paise;
        }
        if !t.debit && t.category == "income" {
            income += t.amount_paise;
        }
    }
    Totals {
        income_paise: income,
        spend_paise: spend,
        invested_paise: invested,
        savings_rate_bps: (income > 0).then(|| (income - spend) * 10_000 / income),
    }
}

fn spend_by_category(txns: &[&TxnIn]) -> BTreeMap<String, i64> {
    let mut map = BTreeMap::new();
    for t in txns.iter().filter(|t| t.debit && is_spending(&t.category)) {
        *map.entry(t.category.clone()).or_insert(0) += t.amount_paise;
    }
    map
}

fn category_changes(now: &[&TxnIn], before: &[&TxnIn]) -> Vec<CategoryChange> {
    let current = spend_by_category(now);
    let earlier = spend_by_category(before);
    let mut rows: Vec<CategoryChange> = current
        .into_iter()
        .map(|(category, paise)| {
            let prev_paise = earlier.get(&category).copied().unwrap_or(0);
            CategoryChange {
                change_bps: change_bps(paise, prev_paise),
                category,
                paise,
                prev_paise,
            }
        })
        .collect();
    rows.sort_by(|a, b| b.paise.cmp(&a.paise).then_with(|| a.category.cmp(&b.category)));
    rows.truncate(MAX_CATEGORIES);
    rows
}

/// Change in basis points; None when there was nothing before to compare with.
fn change_bps(now: i64, before: i64) -> Option<i64> {
    (before != 0).then(|| (now - before) * 10_000 / before)
}

fn win_lines(now: &Totals, prev: Option<&Totals>, categories: &[CategoryChange]) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(rate) = now.savings_rate_bps.filter(|r| *r > 0) {
        // A negative rate last span is not a base to be "up from".
        match prev.and_then(|p| p.savings_rate_bps).filter(|b| *b >= 0) {
            Some(before) if pct(rate) > pct(before) => lines.push(format!(
                "You kept {}% of your income, up from {}%.",
                pct(rate),
                pct(before)
            )),
            Some(_) => {}
            None => lines.push(format!("You kept {}% of your income.", pct(rate))),
        }
    }
    if now.invested_paise > 0 {
        lines.push(format!(
            "You put {} into investments.",
            rupees(now.invested_paise)
        ));
    }
    let income_rise = prev
        .filter(|p| p.income_paise > 0)
        .and_then(|p| change_bps(now.income_paise, p.income_paise))
        .filter(|bps| *bps >= INCOME_RISE_BPS);
    if let Some(bps) = income_rise {
        lines.push(format!(
            "Income rose {}% to {}.",
            pct(bps),
            rupees(now.income_paise)
        ));
    }
    let biggest_fall = categories
        .iter()
        .filter(|c| {
            c.category != "uncategorized"
                && c.change_bps.is_some_and(|b| b <= -SWING_BPS)
                && c.prev_paise - c.paise >= SWING_PAISE
        })
        .min_by_key(|c| c.paise - c.prev_paise);
    if let Some(c) = biggest_fall {
        lines.push(format!(
            "{} fell {}% to {}.",
            label(&c.category),
            pct(-c.change_bps.unwrap_or(0)),
            rupees(c.paise)
        ));
    }
    lines.truncate(MAX_LINES);
    lines
}

/// A savings rate that fell five points or more, then categories that rose sharply, biggest rise
/// first. Uncategorised spend is left out: the payees step already asks for it, and a rise there
/// says more about tagging than about spending.
/// A savings rate this many basis points lower than last time is worth a line.
const RATE_DROP_BPS: i64 = 500;

fn watch_lines(now: &Totals, prev: Option<&Totals>, categories: &[CategoryChange]) -> Vec<String> {
    let mut lines = Vec::new();
    if let (Some(rate), Some(before)) = (now.savings_rate_bps, prev.and_then(|p| p.savings_rate_bps))
        && before - rate >= RATE_DROP_BPS
    {
        lines.push(format!("You kept {}% of your income, down from {}%.", pct(rate), pct(before)));
    }
    let mut rises: Vec<(&CategoryChange, i64)> = categories
        .iter()
        .filter(|c| c.category != "uncategorized")
        .filter_map(|c| {
            let bps = c.change_bps?;
            let rise = c.paise - c.prev_paise;
            (bps >= SWING_BPS && rise >= SWING_PAISE).then_some((c, bps))
        })
        .collect();
    rises.sort_by_key(|(c, _)| Reverse(c.paise - c.prev_paise));
    lines.extend(rises.into_iter().map(|(c, bps)| {
        format!("{} rose {}% to {}.", label(&c.category), pct(bps), rupees(c.paise))
    }));
    lines.truncate(MAX_LINES);
    lines
}

/// Basis points as whole percent, rounded.
fn pct(bps: i64) -> i64 {
    (bps as f64 / 100.0).round() as i64
}

fn label(category: &str) -> String {
    let mut chars = category.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Paise as whole rupees with Indian grouping: ₹2,13,596. Halves round away from zero.
pub fn rupees(paise: i64) -> String {
    let whole = (paise.abs() + 50) / 100 * paise.signum();
    format_inr(whole * 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::habits::{Area, Cadence, Impact, Status};
    use time::macros::format_description;

    fn d(s: &str) -> Date {
        Date::parse(s, format_description!("[year]-[month]-[day]")).expect("test date")
    }

    /// Rupees to paise, so the tests read in the units people see.
    fn rs(r: i64) -> i64 {
        r * 100
    }

    fn txn(on: &str, debit: bool, paise: i64, category: &str, merchant: &str) -> TxnIn {
        TxnIn {
            on: d(on),
            debit,
            amount_paise: paise,
            merchant: Some(merchant.to_string()),
            category: category.to_string(),
            account_id: None,
        }
    }

    fn on_account(mut t: TxnIn, account: &str) -> TxnIn {
        t.account_id = Some(account.to_string());
        t
    }

    fn period(start: &str, end: &str) -> Period {
        Period {
            start: d(start),
            end: d(end),
        }
    }

    fn account(id: &str, name: &str, kind: &str, locked: bool) -> AccountIn {
        AccountIn {
            account_id: id.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            locked,
            balance_paise: 0,
            balance_set_on: None,
            covered_to: None,
        }
    }

    fn card(id: &str, name: &str, covered: Option<&str>) -> CardIn {
        CardIn {
            card_id: id.to_string(),
            name: name.to_string(),
            covered_to: covered.map(d),
        }
    }

    fn empty_status(today: &str) -> StatusIn {
        StatusIn {
            today: d(today),
            last: None,
            accounts: vec![],
            cards: vec![],
            txns: vec![],
        }
    }

    fn step<'a>(s: &'a CheckinStatus, key: &str) -> &'a Step {
        s.steps.iter().find(|x| x.key == key).expect("step")
    }

    // ---- The quarter report ----

    #[test]
    fn totals_count_spending_income_and_investing_by_the_rules() {
        let txns = vec![
            txn("2026-08-02", false, rs(100_000), "income", "Employer"),
            txn("2026-08-05", true, rs(20_000), "food", "Corner Tea"),
            txn("2026-08-06", true, rs(5_000), "transfer", "Self"),
            txn("2026-08-07", true, rs(10_000), "investments", "Fund House"),
            txn("2026-08-08", false, rs(2_000), "refund", "Shop"),
            txn("2026-07-30", true, rs(9_999), "food", "Before the window"),
        ];
        let (totals, prev, _, _, _) = report(&txns, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(totals.income_paise, rs(100_000));
        assert_eq!(totals.spend_paise, rs(20_000));
        assert_eq!(totals.invested_paise, rs(10_000));
        // (100000 - 20000) * 10000 / 100000 = 8000 bps: 80% kept.
        assert_eq!(totals.savings_rate_bps, Some(8_000));
        assert!(prev.is_some(), "the previous month has a transaction");
    }

    #[test]
    fn savings_rate_is_none_without_income_and_negative_when_overspent() {
        let no_income = vec![txn("2026-08-05", true, rs(500), "food", "Tea")];
        let (totals, _, _, _, _) = report(&no_income, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(totals.savings_rate_bps, None);

        let overspent = vec![
            txn("2026-08-02", false, rs(1_000), "income", "Employer"),
            txn("2026-08-05", true, rs(1_500), "food", "Tea"),
        ];
        let (totals, _, _, _, _) = report(&overspent, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(totals.savings_rate_bps, Some(-5_000));
    }

    #[test]
    fn previous_is_none_only_when_the_previous_span_is_empty() {
        let now_only = vec![txn("2026-08-05", true, rs(10), "food", "Tea")];
        let (_, prev, _, _, _) = report(&now_only, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(prev, None);

        let with_earlier = vec![txn("2026-07-05", false, 1, "income", "Employer")];
        let (_, prev, _, _, _) = report(&with_earlier, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert!(prev.is_some(), "a transaction with no spend still counts as a span with data");
    }

    #[test]
    fn previous_span_is_the_same_number_of_days_just_before() {
        let p = period("2026-08-01", "2026-08-30");
        assert_eq!(p.days(), 30);
        let prev = p.previous();
        assert_eq!(prev.end, d("2026-07-31"));
        assert_eq!(prev.start, d("2026-07-02"));
        assert_eq!(prev.days(), 30);
    }

    #[test]
    fn category_change_in_basis_points_and_none_without_a_base() {
        let txns = vec![
            txn("2026-08-05", true, rs(30_000), "food", "Tea"),
            txn("2026-07-05", true, rs(20_000), "food", "Tea"),
            txn("2026-08-06", true, rs(4_000), "fuel", "Pump"),
        ];
        let (_, _, categories, _, _) = report(&txns, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        let food = categories.iter().find(|c| c.category == "food").expect("food");
        assert_eq!((food.paise, food.prev_paise, food.change_bps), (rs(30_000), rs(20_000), Some(5_000)));
        let fuel = categories.iter().find(|c| c.category == "fuel").expect("fuel");
        assert_eq!(fuel.prev_paise, 0);
        assert_eq!(fuel.change_bps, None);
    }

    #[test]
    fn categories_are_the_top_eight_by_spend_and_skip_non_spending() {
        let mut txns: Vec<TxnIn> = (1..=10)
            .map(|i| txn("2026-08-05", true, rs(i * 1_000), &format!("cat{i:02}"), "Shop"))
            .collect();
        txns.push(txn("2026-08-06", true, rs(999_999), "transfer", "Self"));
        let (_, _, categories, _, _) = report(&txns, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(categories.len(), 8);
        assert_eq!(categories[0].category, "cat10");
        assert_eq!(categories[7].category, "cat03");
        assert!(categories.iter().all(|c| c.category != "transfer"));
    }

    #[test]
    fn savings_win_says_up_from_when_the_rate_rose_and_nothing_when_it_fell() {
        let now = vec![
            txn("2026-08-02", false, rs(100_000), "income", "Employer"),
            txn("2026-08-05", true, rs(20_000), "food", "Tea"),
        ];
        let before = vec![
            txn("2026-07-02", false, rs(100_000), "income", "Employer"),
            txn("2026-07-05", true, rs(50_000), "food", "Tea"),
        ];
        let mut all = now.clone();
        all.extend(before.clone());
        let (_, _, _, wins, _) = report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(wins[0], "You kept 80% of your income, up from 50%.");

        // This month keeps 40% and last month kept 50%: no savings win.
        let now_worse = vec![
            txn("2026-08-02", false, rs(100_000), "income", "Employer"),
            txn("2026-08-05", true, rs(60_000), "food", "Tea"),
        ];
        let mut all = now_worse;
        all.extend(before);
        let (_, _, _, wins, _) = report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert!(wins.iter().all(|w| !w.starts_with("You kept")), "{wins:?}");
    }

    #[test]
    fn savings_win_without_a_previous_span_is_plain() {
        let now = vec![
            txn("2026-08-02", false, rs(100_000), "income", "Employer"),
            txn("2026-08-05", true, rs(20_000), "food", "Tea"),
        ];
        let (_, _, _, wins, _) = report(&now, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(wins, vec!["You kept 80% of your income.".to_string()]);
    }

    #[test]
    fn wins_are_at_most_three() {
        let now = vec![
            txn("2026-08-02", false, rs(100_000), "income", "Employer"),
            txn("2026-08-03", true, rs(10_000), "investments", "Fund"),
            txn("2026-08-05", true, rs(20_000), "food", "Tea"),
        ];
        let before = vec![
            txn("2026-07-02", false, rs(50_000), "income", "Employer"),
            txn("2026-07-05", true, rs(80_000), "food", "Tea"),
        ];
        let mut all = now;
        all.extend(before);
        let (_, _, _, wins, _) = report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(wins.len(), 3);
    }

    #[test]
    fn a_big_fall_in_a_category_is_a_win() {
        let now = vec![txn("2026-08-05", true, rs(12_000), "food", "Tea")];
        let before = vec![txn("2026-07-05", true, rs(20_000), "food", "Tea")];
        let mut all = now;
        all.extend(before);
        let (_, _, _, wins, _) = report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(wins, vec!["Food fell 40% to ₹12,000.".to_string()]);
    }

    #[test]
    fn watch_needs_a_rise_of_twenty_percent_and_two_thousand_rupees() {
        let rise = |prev: i64, now: i64| {
            let all = vec![
                txn("2026-08-05", true, now, "food", "Tea"),
                txn("2026-07-05", true, prev, "food", "Tea"),
            ];
            report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31")).4
        };
        // 38% and ₹4,940 more: on the list.
        assert_eq!(
            rise(rs(13_000), 1_794_000),
            vec!["Food rose 38% to ₹17,940.".to_string()]
        );
        // 19% but only ₹1,900 more: not on the list.
        assert!(rise(rs(10_000), rs(11_900)).is_empty());
        // 22% but only ₹1,100 more: not on the list.
        assert!(rise(rs(5_000), rs(6_100)).is_empty());
        // Nothing before to compare with: not on the list.
        assert!(rise(0, rs(50_000)).is_empty());
    }

    #[test]
    fn watch_names_a_falling_savings_rate_and_skips_uncategorised() {
        let all = vec![
            txn("2026-08-01", false, rs(100_000), "income", "Acme Payroll"),
            txn("2026-08-05", true, rs(60_000), "uncategorized", "Someone"),
            txn("2026-08-06", true, rs(20_000), "food", "Tea"),
            txn("2026-07-01", false, rs(100_000), "income", "Acme Payroll"),
            txn("2026-07-05", true, rs(10_000), "uncategorized", "Someone"),
            txn("2026-07-06", true, rs(20_000), "food", "Tea"),
        ];
        let (_, _, _, _, watch) = report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(watch, vec!["You kept 20% of your income, down from 70%.".to_string()]);
    }

    #[test]
    fn watch_is_at_most_three_and_biggest_rise_first() {
        let mut all = Vec::new();
        for (i, now) in [(1, 90_000), (2, 70_000), (3, 60_000), (4, 50_000)] {
            all.push(txn("2026-08-05", true, rs(now), &format!("c{i}"), "Shop"));
            all.push(txn("2026-07-05", true, rs(now / 2), &format!("c{i}"), "Shop"));
        }
        let (_, _, _, _, watch) = report(&all, period("2026-08-01", "2026-08-31"), period("2026-07-01", "2026-07-31"));
        assert_eq!(watch.len(), 3);
        assert!(watch[0].starts_with("C1 rose 100%"), "{watch:?}");
    }

    #[test]
    fn rupees_use_indian_grouping_and_round_to_whole_rupees() {
        assert_eq!(rupees(21_359_600), "₹2,13,596");
        assert_eq!(rupees(49), "₹0");
        assert_eq!(rupees(50), "₹1");
        assert_eq!(rupees(-50), "-₹1");
        assert_eq!(rupees(-5_050), "-₹51");
        assert_eq!(rupees(0), "₹0");
    }

    #[test]
    fn quarter_report_keeps_three_habits_and_the_spans() {
        let habit = |key: &str| Habit {
            key: key.to_string(),
            area: Area::Money,
            title: "t".into(),
            why: "w".into(),
            action: None,
            cadence: Cadence::Once,
            due_on: None,
            status: Status::Ok,
            impact: Impact::Low,
        };
        let habits = vec![habit("a"), habit("b"), habit("c"), habit("d")];
        let q = quarter_report(&[], period("2026-08-01", "2026-08-30"), habits);
        assert_eq!(q.habits.len(), 3);
        assert_eq!(q.period, Span { start: "2026-08-01".into(), end: "2026-08-30".into() });
        assert_eq!(q.previous, Span { start: "2026-07-02".into(), end: "2026-07-31".into() });
        assert_eq!(q.prev, None);
    }

    // ---- The status ----

    #[test]
    fn with_no_data_the_check_in_is_due_today_and_covers_the_last_90_days() {
        let s = status(&empty_status("2026-11-09"));
        assert_eq!(s.last_done_on, None);
        assert_eq!(s.next_due_on, "2026-11-09");
        assert_eq!(s.status, DueState::DueSoon);
        assert_eq!(s.period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
        let keys: Vec<&str> = s.steps.iter().map(|x| x.key).collect();
        assert_eq!(keys, vec!["statements", "balances", "payees", "report"]);
        assert!(step(&s, "statements").done);
        assert!(step(&s, "balances").done);
        assert!(step(&s, "payees").done);
        assert!(!step(&s, "report").done);
        assert!(s.uploads.is_empty() && s.estimates.is_empty());
    }

    #[test]
    fn with_a_check_in_next_is_91_days_on_and_the_period_starts_after_it() {
        let mut input = empty_status("2026-11-09");
        input.last = Some(LastCheckin {
            done_on: d("2026-08-10"),
            period_start: d("2026-05-12"),
            period_end: d("2026-08-10"),
        });
        let s = status(&input);
        assert_eq!(s.last_done_on.as_deref(), Some("2026-08-10"));
        assert_eq!(s.next_due_on, "2026-11-09");
        assert_eq!(s.period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
    }

    #[test]
    fn next_due_comes_from_the_earliest_bank_statement_when_there_is_no_check_in() {
        let mut input = empty_status("2026-11-09");
        input.accounts = vec![
            AccountIn {
                covered_to: Some(d("2026-08-20")),
                ..account("a", "A", "bank", false)
            },
            AccountIn {
                covered_to: Some(d("2026-07-01")),
                ..account("b", "B", "bank", false)
            },
            // Cards do not set the first due date.
            AccountIn {
                covered_to: Some(d("2026-05-01")),
                ..account("c", "C", "cash", false)
            },
        ];
        input.cards = vec![card("x", "Card", Some("2026-06-01"))];
        assert_eq!(next_due(input.today, None, &input.accounts), d("2026-09-29"));
    }

    #[test]
    fn a_check_in_finished_today_keeps_its_own_period() {
        let mut input = empty_status("2026-11-09");
        input.last = Some(LastCheckin { done_on: d("2026-11-09"), period_start: d("2026-08-11"), period_end: d("2026-11-09") });
        let s = status(&input);
        assert!(s.done_today);
        assert_eq!(s.period, Span { start: "2026-08-11".into(), end: "2026-11-09".into() });
        assert_eq!(s.next_due_on, "2027-02-08");
        assert_eq!(s.status, DueState::Ok);
        // Tomorrow the next period starts the day after it.
        input.today = d("2026-11-10");
        let next = status(&input);
        assert!(!next.done_today);
        assert_eq!(next.period, Span { start: "2026-11-10".into(), end: "2026-11-10".into() });
    }

    #[test]
    fn due_states_use_the_day_boundaries() {
        let today = d("2026-11-09");
        assert_eq!(due_state(today, d("2026-11-08")), DueState::Overdue);
        assert_eq!(due_state(today, today), DueState::DueSoon);
        assert_eq!(due_state(today, d("2026-11-23")), DueState::DueSoon);
        assert_eq!(due_state(today, d("2026-11-24")), DueState::Ok);
    }

    #[test]
    fn a_check_in_done_long_ago_is_overdue() {
        let mut input = empty_status("2026-11-09");
        input.last = Some(LastCheckin {
            done_on: d("2026-06-01"),
            period_start: d("2026-03-04"),
            period_end: d("2026-06-01"),
        });
        let s = status(&input);
        assert_eq!(s.next_due_on, "2026-08-31");
        assert_eq!(s.status, DueState::Overdue);
    }

    #[test]
    fn uploads_are_needed_when_missing_or_older_than_35_days() {
        let mut input = empty_status("2026-11-09");
        // Stale cut-off is 2026-10-05. Strictly older than that is needed.
        input.accounts = vec![
            AccountIn {
                covered_to: Some(d("2026-08-01")),
                ..account("a", "A", "bank", false)
            },
            AccountIn {
                covered_to: Some(d("2026-10-05")),
                ..account("b", "B", "bank", false)
            },
            account("c", "C", "bank", false),
            account("l", "Locked", "bank", true),
            account("k", "Cash", "cash", false),
        ];
        input.cards = vec![card("x", "Card X", None), card("y", "Card Y", Some("2026-10-20"))];
        let s = status(&input);
        let rows: Vec<(&str, bool)> = s.uploads.iter().map(|u| (u.id.as_str(), u.needed)).collect();
        assert_eq!(rows, vec![("a", true), ("b", false), ("c", true), ("x", true), ("y", false)]);
        assert_eq!(s.uploads[0].covered_to.as_deref(), Some("2026-08-01"));
        assert_eq!(s.uploads[2].covered_to, None);
        let st = step(&s, "statements");
        assert_eq!((st.done, st.count), (false, 3));
        assert_eq!(st.detail, "3 of 5 need a newer statement.");
    }

    #[test]
    fn the_statements_step_is_done_when_nothing_is_needed() {
        let mut input = empty_status("2026-11-09");
        input.accounts = vec![AccountIn {
            covered_to: Some(d("2026-11-01")),
            ..account("a", "A", "bank", false)
        }];
        let s = status(&input);
        let st = step(&s, "statements");
        assert_eq!((st.done, st.count), (true, 0));
    }

    #[test]
    fn balances_step_counts_unlocked_accounts_not_confirmed_this_period() {
        let mut input = empty_status("2026-11-09");
        input.accounts = vec![
            AccountIn {
                balance_set_on: Some(d("2026-08-05")),
                ..account("a", "A", "bank", false)
            },
            AccountIn {
                balance_set_on: Some(d("2026-08-20")),
                ..account("b", "Wallet", "cash", false)
            },
            // Locked accounts are not counted.
            AccountIn {
                balance_set_on: None,
                ..account("l", "FD", "fd", true)
            },
        ];
        let s = status(&input);
        let st = step(&s, "balances");
        assert_eq!((st.done, st.count), (false, 1));
        assert_eq!(st.detail, "1 of 2 balances needs confirming.");
    }

    #[test]
    fn payees_step_is_done_under_ten_percent_of_spend() {
        // Spend 11,000 in the period. Uncategorised: 3,000 (27%) across two payees.
        let base = vec![
            txn("2026-09-01", true, rs(8_000), "food", "Tea"),
            txn("2026-09-02", true, rs(1_200), "uncategorized", "Shop"),
            txn("2026-09-03", true, rs(800), "uncategorized", "shop "),
            txn("2026-09-04", true, rs(1_000), "uncategorized", "Mart"),
        ];
        let mut input = empty_status("2026-11-09");
        input.txns = base.clone();
        let s = status(&input);
        let st = step(&s, "payees");
        assert!(!st.done);
        assert_eq!(st.count, 2);
        assert_eq!(st.detail, "₹3,000 across 2 payees have no category.");

        // Uncategorised 500 of 10,500 spend: under 10%, so done.
        input.txns = vec![
            txn("2026-09-01", true, rs(10_000), "food", "Tea"),
            txn("2026-09-02", true, rs(500), "uncategorized", "Shop"),
        ];
        let s = status(&input);
        let st = step(&s, "payees");
        assert!(st.done);
        assert_eq!(st.count, 0);
    }

    #[test]
    fn payees_step_is_done_when_nothing_is_uncategorised() {
        let mut input = empty_status("2026-11-09");
        input.txns = vec![txn("2026-09-01", true, rs(100), "food", "Tea")];
        let s = status(&input);
        let st = step(&s, "payees");
        assert_eq!((st.done, st.count), (true, 0));
        assert_eq!(st.detail, "Every payee this quarter has a category.");
    }

    #[test]
    fn the_report_step_is_open_and_names_the_days() {
        let s = status(&empty_status("2026-11-09"));
        let st = step(&s, "report");
        assert!(!st.done);
        assert_eq!(st.count, 1);
        assert_eq!(st.detail, "Read the 91 days from 11 Aug to 9 Nov, then finish.");
    }

    fn estimate_input(today: &str, accounts: Vec<AccountIn>, txns: Vec<TxnIn>) -> CheckinStatus {
        let mut input = empty_status(today);
        input.accounts = accounts;
        input.txns = txns;
        status(&input)
    }

    fn with_set_on(id: &str, name: &str, balance: i64, set_on: &str) -> AccountIn {
        AccountIn {
            balance_paise: balance,
            balance_set_on: Some(d(set_on)),
            ..account(id, name, "bank", false)
        }
    }

    #[test]
    fn estimate_adds_the_usual_net_flow_for_each_day_since_the_balance() {
        let txns = vec![
            on_account(txn("2026-07-05", false, rs(9_000), "income", "Employer"), "a"),
            on_account(txn("2026-07-20", true, rs(3_000), "food", "Tea"), "a"),
            on_account(txn("2026-09-20", true, rs(600), "food", "Tea"), "a"),
        ];
        let s = estimate_input("2026-11-09", vec![with_set_on("a", "Acme Bank", rs(1_000), "2026-10-01")], txns);
        assert_eq!(s.estimates.len(), 1);
        let e = &s.estimates[0];
        // Net 5,400 over 90 days: 5400 * 39 / 90 = 2,340 more on top of 1,000.
        assert_eq!(e.known_paise, rs(1_000));
        assert_eq!(e.known_on, "2026-10-01");
        assert_eq!(e.days_since, 39);
        assert_eq!(e.estimate_paise, rs(3_340));
        assert_eq!(e.basis, "Usual net ₹1,800 a month in, over 39 days since 1 Oct.");
    }

    #[test]
    fn estimates_need_seven_days_and_thirty_days_of_history() {
        let mut txns = vec![
            on_account(txn("2026-07-05", false, rs(9_000), "income", "Employer"), "a"),
            on_account(txn("2026-09-20", true, rs(600), "food", "Tea"), "a"),
            on_account(txn("2026-07-05", false, rs(9_000), "income", "Employer"), "b"),
            on_account(txn("2026-07-08", true, rs(600), "food", "Tea"), "b"),
        ];
        let accounts = vec![
            // Six days old: too recent.
            with_set_on("a", "Recent", rs(1_000), "2026-11-03"),
            // Seven days old, with 77 days of history: included.
            with_set_on("a2", "Seven", rs(1_000), "2026-11-02"),
            // Old enough, but only three days of transactions.
            with_set_on("b", "Thin", rs(1_000), "2026-10-01"),
        ];
        txns.push(on_account(txn("2026-07-05", false, rs(1), "income", "Employer"), "a2"));
        txns.push(on_account(txn("2026-09-20", true, rs(1), "food", "Tea"), "a2"));
        let s = estimate_input("2026-11-09", accounts, txns);
        let names: Vec<&str> = s.estimates.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["Seven"]);
    }

    #[test]
    fn estimates_are_clamped_at_zero_and_sorted_by_name() {
        let txns = vec![
            on_account(txn("2026-07-05", false, rs(5), "income", "Employer"), "c"),
            on_account(txn("2026-07-10", true, rs(9_000), "food", "Tea"), "c"),
            on_account(txn("2026-09-30", true, rs(50), "food", "Tea"), "c"),
            on_account(txn("2026-07-05", false, rs(5), "income", "Employer"), "z"),
            on_account(txn("2026-09-30", true, rs(5), "food", "Tea"), "z"),
            on_account(txn("2026-07-06", true, rs(1), "food", "Tea"), "z"),
        ];
        let accounts = vec![
            with_set_on("z", "Zeta", rs(100), "2026-10-01"),
            with_set_on("c", "Alpha", rs(10), "2026-10-01"),
        ];
        let s = estimate_input("2026-11-09", accounts, txns);
        let names: Vec<&str> = s.estimates.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Zeta"]);
        assert_eq!(s.estimates[0].estimate_paise, 0);
        assert!(s.estimates[0].basis.contains("out"), "{}", s.estimates[0].basis);
    }
}
