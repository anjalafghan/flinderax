//! Safe-to-Spend engine. Pure: no DB, no clock. Everything comes in through `PlanInput`
//! and money is INTEGER paise (`i64`).

pub mod cycle;

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;
use time::{Date, Duration, Month};

use cycle::{add_months, clamped, day_in_month_offset};

pub const STALE_AFTER_DAYS: i64 = 7;
/// `per_week` at or above this is "green" (₹2,000).
pub const GREEN_PER_WEEK_PAISE: i64 = 200_000;
/// An income landing on or after this day pays for the *next* month ("30 Oct" -> November money).
const PAYS_NEXT_MONTH_FROM_DAY: u8 = 20;
const URGENT_WITHIN_DAYS: i64 = 3;

// ---------- input ----------

#[derive(Debug, Clone)]
pub struct AccountIn {
    pub id: String,
    pub name: String,
    pub balance: i64,
    pub locked: bool,
    pub updated: Option<Date>,
}

#[derive(Debug, Clone)]
pub struct CommitmentIn {
    pub label: String,
    pub amount: i64,
    pub day: u8,
    pub account_id: Option<String>,
    pub active: bool,
    /// Month index (`year * 12 + month0`), see [`month_index`].
    pub start_month: Option<i32>,
    pub end_month: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct IncomeIn {
    pub label: String,
    pub amount: i64,
    pub day: u8,
    pub account_id: Option<String>,
}

/// One card's breakdown (see `/api/card/breakdown`).
#[derive(Debug, Clone)]
pub struct CardIn {
    pub name: String,
    pub due_now: i64,
    pub due_date: Option<Date>,
    pub next_bill_est: i64,
    pub next_due_date: Option<Date>,
    pub last_updated: Option<Date>,
}

#[derive(Debug, Clone)]
pub struct SettingsIn {
    pub cushion: i64,
    pub next_period_living: i64,
}

#[derive(Debug, Clone)]
pub struct PlanInput {
    pub today: Date,
    pub accounts: Vec<AccountIn>,
    pub incomes: Vec<IncomeIn>,
    pub commitments: Vec<CommitmentIn>,
    pub cards: Vec<CardIn>,
    pub settings: SettingsIn,
}

pub fn month_index(d: Date) -> i32 {
    d.year() * 12 + (d.month() as i32 - 1)
}

/// Parses `'2026-12'` into a month index.
pub fn parse_year_month(s: &str) -> Option<i32> {
    let (y, m) = s.split_once('-')?;
    let (y, m): (i32, i32) = (y.parse().ok()?, m.parse().ok()?);
    (1..=12).contains(&m).then_some(y * 12 + (m - 1))
}

// ---------- output ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Serialize)]
pub struct NextIncome {
    pub label: String,
    pub arrives: Date,
}

#[derive(Debug, Clone, Serialize)]
pub struct LowestPoint {
    pub amount_paise: i64,
    pub date: Date,
}

#[derive(Debug, Clone, Serialize)]
pub struct Upcoming {
    pub date: Date,
    pub label: String,
    pub amount_paise: i64,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Action {
    pub text: String,
    /// `urgent` | `warn` | `info`
    pub severity: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stale {
    pub what: String,
    pub days_old: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub safe_to_spend_paise: i64,
    pub per_week_paise: i64,
    pub status: Status,
    pub until: Date,
    pub period_label: Option<String>,
    pub next_income: Option<NextIncome>,
    pub lowest_point: LowestPoint,
    pub upcoming: Vec<Upcoming>,
    pub actions: Vec<Action>,
    pub stale: Vec<Stale>,
}

// ---------- card breakdown ----------

/// What the bank app shows for a card, plus its EMIs and not-yet-posted charges.
/// `unbilled_spent` / `unbilled_credit` are deliberately absent: they are reference only, which
/// is why duplicate charges and refunds can't skew the totals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardNumbers {
    pub total_due: i64,
    pub outstanding: i64,
    pub emi_monthly: i64,
    pub emi_remaining: i64,
    pub pending_charges: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CardAmounts {
    pub due_now_paise: i64,
    pub next_bill_est_paise: i64,
    pub loan_not_due_paise: i64,
}

pub fn card_amounts(n: CardNumbers) -> CardAmounts {
    CardAmounts {
        due_now_paise: n.total_due,
        next_bill_est_paise: (n.outstanding - n.total_due - n.emi_remaining
            + n.emi_monthly
            + n.pending_charges)
            .max(0),
        loan_not_due_paise: (n.emi_remaining - n.emi_monthly).max(0),
    }
}

/// An EMI's monthly amount and remaining balance as of `now_idx` (a [`month_index`]), given
/// that `months_left` / `remaining` were true in month `as_of_idx`. The balance is assumed to
/// fall linearly. Returns `(monthly, remaining, months_left)`; a finished EMI is all zeros so it
/// stops counting toward the next bill.
pub fn emi_effective(
    monthly: i64,
    remaining: i64,
    months_left: i64,
    as_of_idx: i32,
    now_idx: i32,
) -> (i64, i64, i64) {
    let elapsed = i64::from((now_idx - as_of_idx).max(0));
    let left = (months_left - elapsed).max(0);
    if left == 0 || months_left == 0 {
        return (0, 0, 0);
    }
    (monthly, remaining * left / months_left, left)
}

// ---------- helpers ----------

/// "November money" for an income arriving on `arrives`.
pub fn money_label(arrives: Date) -> String {
    let (_, m) = if arrives.day() >= PAYS_NEXT_MONTH_FROM_DAY {
        add_months(arrives.year(), arrives.month(), 1)
    } else {
        (arrives.year(), arrives.month())
    };
    format!("{} money", month_name(m))
}

fn month_name(m: Month) -> &'static str {
    [
        "January", "February", "March", "April", "May", "June", "July", "August", "September",
        "October", "November", "December",
    ][m as usize - 1]
}

pub fn short_date(d: Date) -> String {
    format!("{} {}", d.day(), &month_name(d.month())[..3])
}

/// ₹1,23,456 (Indian grouping); paise shown only when non-zero.
pub fn format_inr(paise: i64) -> String {
    let sign = if paise < 0 { "-" } else { "" };
    let abs = paise.unsigned_abs();
    let (rupees, rem) = (abs / 100, abs % 100);
    let digits = rupees.to_string();
    let grouped = if digits.len() <= 3 {
        digits
    } else {
        let (head, tail) = digits.split_at(digits.len() - 3);
        let mut parts: Vec<String> = Vec::new();
        let hb = head.as_bytes();
        let mut i = hb.len();
        while i > 0 {
            let start = i.saturating_sub(2);
            parts.push(head[start..i].to_string());
            i = start;
        }
        parts.reverse();
        format!("{},{}", parts.join(","), tail)
    };
    if rem == 0 {
        format!("{sign}₹{grouped}")
    } else {
        format!("{sign}₹{grouped}.{rem:02}")
    }
}

/// Every date in `[from, to)` on which a "day of month" recurrence falls (clamped per month).
fn occurrences(day: u8, from: Date, to: Date) -> Vec<Date> {
    let mut out = Vec::new();
    let mut anchor = from;
    for _ in 0..=((to - from).whole_days() / 28 + 2) {
        let d = clamped(anchor.year(), anchor.month(), day);
        if d >= from && d < to {
            out.push(d);
        }
        anchor = day_in_month_offset(anchor, 1, 1);
    }
    out.sort();
    out.dedup();
    out
}

fn next_after(day: u8, after: Date) -> Date {
    let d = clamped(after.year(), after.month(), day);
    if d > after { d } else { day_in_month_offset(after, 1, day) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Income,
    Commitment,
    CardDue,
    CardBill,
    Living,
}

struct Event {
    date: Date,
    delta: i64,
    label: String,
    kind: Kind,
    account_id: Option<String>,
}

// ---------- engine ----------

pub fn compute(input: &PlanInput) -> Summary {
    let today = input.today;

    // 1. Paydays. Income dated today is assumed to be in the balances already.
    let next_pay: Option<(Date, &IncomeIn)> = input
        .incomes
        .iter()
        .map(|i| (next_after(i.day, today), i))
        .min_by_key(|(d, _)| *d);
    let (payday1, payday2) = match next_pay {
        Some((p1, _)) => {
            let p2 = input
                .incomes
                .iter()
                .map(|i| next_after(i.day, p1))
                .min()
                .expect("at least one income");
            (p1, p2)
        }
        None => (today + Duration::days(30), today + Duration::days(60)),
    };

    // 2. Opening balance: everything not locked.
    let start: i64 = input.accounts.iter().filter(|a| !a.locked).map(|a| a.balance).sum();

    // 3. Dated events inside [today, payday2).
    let mut events: Vec<Event> = Vec::new();
    for inc in &input.incomes {
        for d in occurrences(inc.day, today + Duration::days(1), payday2) {
            events.push(Event {
                date: d,
                delta: inc.amount,
                label: inc.label.clone(),
                kind: Kind::Income,
                account_id: inc.account_id.clone(),
            });
        }
    }
    for c in input.commitments.iter().filter(|c| c.active) {
        for d in occurrences(c.day, today, payday2) {
            let mi = month_index(d);
            if c.start_month.is_some_and(|s| mi < s) || c.end_month.is_some_and(|e| mi > e) {
                continue;
            }
            events.push(Event {
                date: d,
                delta: -c.amount,
                label: c.label.clone(),
                kind: Kind::Commitment,
                account_id: c.account_id.clone(),
            });
        }
    }
    for card in &input.cards {
        if card.due_now > 0 {
            // Past-due or undated bills are treated as payable today.
            let d = card.due_date.map_or(today, |d| d.max(today));
            events.push(Event {
                date: d,
                delta: -card.due_now,
                label: format!("{} bill", card.name),
                kind: Kind::CardDue,
                account_id: None,
            });
        }
        if card.next_bill_est > 0 {
            if let Some(d) = card.next_due_date {
                if d >= today && d < payday2 {
                    events.push(Event {
                        date: d,
                        delta: -card.next_bill_est,
                        label: format!("{} next bill (est.)", card.name),
                        kind: Kind::CardBill,
                        account_id: None,
                    });
                }
            }
        }
    }
    // Living costs for the next period, spread evenly over [payday1, payday2).
    let span = (payday2 - payday1).whole_days().max(1);
    let (base, extra) = (input.settings.next_period_living / span, input.settings.next_period_living % span);
    for i in 0..span {
        events.push(Event {
            date: payday1 + Duration::days(i),
            delta: -(base + i64::from(i < extra)),
            label: "Living costs".into(),
            kind: Kind::Living,
            account_id: None,
        });
    }

    // 4. Day-by-day simulation.
    let mut by_day: BTreeMap<Date, i64> = BTreeMap::new();
    for e in &events {
        *by_day.entry(e.date).or_default() += e.delta;
    }
    let mut balance = start;
    let mut lowest = LowestPoint { amount_paise: start, date: today };
    for (date, delta) in &by_day {
        balance += delta;
        if balance < lowest.amount_paise {
            lowest = LowestPoint { amount_paise: balance, date: *date };
        }
    }

    // 5-6. Safe to spend, weekly figure, status.
    let safe = (lowest.amount_paise - input.settings.cushion).max(0);
    let days_to_payday = (payday1 - today).whole_days().max(1);
    let weeks = ((days_to_payday + 6) / 7).max(1);
    let per_week = safe / weeks;
    let status = if per_week >= GREEN_PER_WEEK_PAISE {
        Status::Green
    } else if safe > 0 {
        Status::Yellow
    } else {
        Status::Red
    };

    // Labels.
    let (next_income, period_label) = match next_pay {
        Some((p1, src)) => (
            Some(NextIncome { label: money_label(p1), arrives: p1 }),
            Some(money_label(day_in_month_offset(p1, -1, src.day))),
        ),
        None => (None, None),
    };

    let mut upcoming: Vec<Upcoming> = events
        .iter()
        .filter(|e| matches!(e.kind, Kind::Commitment | Kind::CardDue | Kind::CardBill))
        .map(|e| Upcoming {
            date: e.date,
            label: e.label.clone(),
            amount_paise: -e.delta,
            kind: match e.kind {
                Kind::Commitment => "commitment",
                Kind::CardDue => "card_due",
                _ => "card_bill",
            },
        })
        .collect();
    upcoming.sort_by_key(|u| u.date);

    // 7. Actions + stale data.
    let mut actions = transfer_actions(input, &events);
    let mut stale = Vec::new();
    for a in input.accounts.iter().filter(|a| !a.locked) {
        if let Some(u) = a.updated {
            let age = (today - u).whole_days();
            if age > STALE_AFTER_DAYS {
                stale.push(Stale { what: format!("{} balance", a.name), days_old: age });
            }
        }
    }
    for c in &input.cards {
        if let Some(u) = c.last_updated {
            let age = (today - u).whole_days();
            if age > STALE_AFTER_DAYS {
                stale.push(Stale { what: format!("{} snapshot", c.name), days_old: age });
            }
        }
    }
    for s in &stale {
        actions.push(Action {
            text: format!("Update {} (last updated {} days ago)", s.what, s.days_old),
            severity: "info",
        });
    }

    Summary {
        safe_to_spend_paise: safe,
        per_week_paise: per_week,
        status,
        until: payday1,
        period_label,
        next_income,
        lowest_point: lowest,
        upcoming,
        actions,
        stale,
    }
}

/// Projects each unlocked account on its own (only the incomes/commitments tied to it) and, when
/// it would go below zero before a payment, proposes moving the shortfall from the account
/// with the most spare money.
fn transfer_actions(input: &PlanInput, events: &[Event]) -> Vec<Action> {
    let accounts: Vec<&AccountIn> = input.accounts.iter().filter(|a| !a.locked).collect();

    // Per-account timeline of (date, delta, label), credits before debits within a day.
    let mut timelines: HashMap<&str, Vec<(Date, i64, &str)>> = HashMap::new();
    for e in events {
        if let Some(id) = e.account_id.as_deref() {
            timelines.entry(id).or_default().push((e.date, e.delta, &e.label));
        }
    }
    for t in timelines.values_mut() {
        t.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    }

    // Own-balance at the end of `date`, and the lowest own balance from `date` onward.
    let spare_at = |acc: &AccountIn, date: Date| -> i64 {
        let tl = timelines.get(acc.id.as_str());
        let mut bal = acc.balance;
        let mut lowest = i64::MAX;
        if let Some(tl) = tl {
            for (d, delta, _) in tl {
                bal += delta;
                if *d >= date {
                    lowest = lowest.min(bal);
                }
            }
        }
        let at_date = acc.balance + tl.map_or(0, |t| t.iter().filter(|(d, ..)| *d <= date).map(|(_, x, _)| x).sum::<i64>());
        lowest.min(at_date)
    };

    let mut moved_out: HashMap<&str, i64> = HashMap::new();
    let mut moved_in: HashMap<&str, i64> = HashMap::new();
    let mut found: Vec<(Date, Action)> = Vec::new();

    let mut ordered: Vec<(&AccountIn, Date)> = Vec::new();
    for acc in &accounts {
        if let Some(tl) = timelines.get(acc.id.as_str()) {
            ordered.push((acc, tl.first().map_or(input.today, |t| t.0)));
        }
    }
    ordered.sort_by_key(|(_, d)| *d);

    for (acc, _) in ordered {
        let tl = &timelines[acc.id.as_str()];
        let mut bal = acc.balance;
        for (date, delta, label) in tl {
            bal += delta;
            let funded = *moved_in.get(acc.id.as_str()).unwrap_or(&0);
            let short = -(bal + funded);
            if *delta >= 0 || short <= 0 {
                continue;
            }
            let urgency = if (*date - input.today).whole_days() <= URGENT_WITHIN_DAYS { "urgent" } else { "warn" };
            let source = accounts
                .iter()
                .filter(|s| s.id != acc.id)
                .map(|s| (*s, spare_at(s, *date) - moved_out.get(s.id.as_str()).copied().unwrap_or(0)))
                .max_by_key(|(_, spare)| *spare);
            let mut remaining = short;
            if let Some((src, spare)) = source {
                let amount = remaining.min(spare.max(0));
                if amount > 0 {
                    *moved_out.entry(src.id.as_str()).or_default() += amount;
                    *moved_in.entry(acc.id.as_str()).or_default() += amount;
                    remaining -= amount;
                    found.push((
                        *date,
                        Action {
                            text: format!(
                                "Move {} {} → {} before {} ({})",
                                format_inr(amount),
                                src.name,
                                acc.name,
                                short_date(*date),
                                label
                            ),
                            severity: urgency,
                        },
                    ));
                }
            }
            if remaining > 0 {
                found.push((
                    *date,
                    Action {
                        text: format!(
                            "{} is short by {} for {} on {} and no other account has spare money",
                            acc.name,
                            format_inr(remaining),
                            label,
                            short_date(*date)
                        ),
                        severity: "urgent",
                    },
                ));
                *moved_in.entry(acc.id.as_str()).or_default() += remaining; // don't repeat for later events
            }
        }
    }
    found.sort_by_key(|(d, _)| *d);
    found.into_iter().map(|(_, a)| a).collect()
}

#[cfg(test)]
mod tests;
