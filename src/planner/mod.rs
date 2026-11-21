//! Safe-to-Spend engine. Pure: no DB, no clock. Everything comes in through `PlanInput`
//! and money is INTEGER paise (`i64`).
//!
//! The money model is two pots plus a ledger:
//!
//! * **Spending account (HDFC):** salary lands here, the SIP debits here, and what is left is the
//!   spending pot. Every card swipe should be matched by moving the same amount to the reserve.
//! * **Reserve account (Kotak):** savings, card bills and needs are paid from here.
//! * **Ledger (`moves`):** each "I moved it" / "already covered" / "skip this month" click.
//!   A bill is only "to move" for the part not yet covered.
//!
//! Safe to spend covers the CURRENT PAY PERIOD only (today up to the next payday); the next salary
//! is never used.

pub mod afford;
pub mod analytics;
pub mod autopost;
pub mod bills;
pub mod calibration;
pub mod checkin;
pub mod coming;
pub mod couple_goals;
pub mod wedding;
pub mod cycle;
pub mod diet;
pub mod forecast;
pub mod goals;
pub mod habits;
pub mod history;
pub mod insights;
pub mod joy;
pub mod life;
pub mod education;
pub mod guidance;
pub mod lifeline;
pub mod merchant;
pub mod merchant_seed;
pub mod people;
pub mod projection;
pub mod recurring;
pub mod reminders;
pub mod review;
pub mod reward_fund;
pub mod rewards;
pub mod streaks;
pub mod activity;
pub mod tax;
pub mod week_digest;
pub mod work;
pub mod year_ahead;

use serde::Serialize;
use time::{Date, Duration, Month};

use cycle::{add_months, clamped, day_in_month_offset};

pub const STALE_AFTER_DAYS: i64 = 7;
/// `per_week` at or above this is "green" (₹2,000).
pub const GREEN_PER_WEEK_PAISE: i64 = 200_000;
/// An income landing on or after this day pays for the *next* month ("30 Oct" -> November money).
const PAYS_NEXT_MONTH_FROM_DAY: u8 = 20;
const URGENT_WITHIN_DAYS: i64 = 3;
/// Spending share of income when the user has not set an amount.
const DEFAULT_SPENDING_PCT: i64 = 20;

// ---------- input ----------

/// Which share of income a fixed payment is drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Bucket {
    Needs,
    Savings,
    Wants,
}

impl Bucket {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "needs" => Some(Self::Needs),
            "savings" => Some(Self::Savings),
            "wants" => Some(Self::Wants),
            _ => None,
        }
    }
}

/// Default bucket for a category label (the user can override it per payment).
pub fn default_bucket(category: &str) -> Bucket {
    match category {
        "sip" | "rd" | "savings" | "investment" => Bucket::Savings,
        "subscription" | "donation" | "entertainment" => Bucket::Wants,
        _ => Bucket::Needs,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Spending,
    Reserve,
    Other,
}

impl Role {
    pub fn parse(s: &str) -> Self {
        match s {
            "spending" => Self::Spending,
            "reserve" => Self::Reserve,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freq {
    Monthly,
    Yearly,
    /// Every `months` months (2-11), starting at month index `first` and never before it: a
    /// perfume every four months, a service every six. Like a yearly bill it is set aside evenly
    /// (a `months`th a month) and falls due only in its own months.
    Every { months: u8, first: i32 },
}

impl Freq {
    /// From the stored columns. A cadence that does not make sense (a yearly bill with a gap, a
    /// gap without a readable first month) falls back to the plainer reading rather than failing:
    /// monthly over-plans, which is the safe side.
    pub fn from_columns(frequency: &str, every_months: i64, first_month: Option<&str>) -> Freq {
        if frequency == "yearly" {
            return Freq::Yearly;
        }
        match (u8::try_from(every_months), first_month.and_then(parse_year_month)) {
            (Ok(months @ 2..=11), Some(first)) => Freq::Every { months, first },
            _ => Freq::Monthly,
        }
    }

    /// Whether a payment of this kind falls in `d`'s month.
    fn due_in(self, d: Date, due_month: Option<u8>) -> bool {
        match self {
            Freq::Monthly => true,
            Freq::Yearly => due_month.is_some_and(|m| m == d.month() as u8),
            Freq::Every { months, first } => {
                let mi = month_index(d);
                mi >= first && (mi - first) % i32::from(months) == 0
            }
        }
    }

    /// Months between one payment and the next, for set-aside maths: 12 for a yearly bill.
    pub fn period_months(self) -> i64 {
        match self {
            Freq::Monthly => 1,
            Freq::Yearly => 12,
            Freq::Every { months, .. } => i64::from(months),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountIn {
    pub id: String,
    pub name: String,
    pub balance: i64,
    pub locked: bool,
    pub role: Role,
    pub updated: Option<Date>,
}

#[derive(Debug, Clone)]
pub struct CommitmentIn {
    pub label: String,
    pub amount: i64,
    /// Day of the month it is paid.
    pub day: u8,
    pub account_id: Option<String>,
    pub active: bool,
    pub bucket: Bucket,
    pub freq: Freq,
    /// 1-12, for yearly bills. `None` on a yearly bill = month unknown: it is set aside monthly
    /// but no due-date event is scheduled.
    pub due_month: Option<u8>,
    /// Where the money lands when this is a transfer to savings (an RD, an FD top-up...).
    pub to_account_id: Option<String>,
    /// Month index (`year * 12 + month0`), see [`month_index`].
    pub start_month: Option<i32>,
    pub end_month: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct IncomeIn {
    #[allow(dead_code)] // shown by callers building inputs; the engine only needs amounts and days
    pub label: String,
    pub amount: i64,
    pub day: u8,
    pub account_id: Option<String>,
}

/// One card, with its bills, ledger coverage and (when known) spending this period.
#[derive(Debug, Clone)]
pub struct CardIn {
    pub card_id: String,
    pub name: String,
    pub due_now: i64,
    pub due_date: Option<Date>,
    pub next_bill_est: i64,
    pub next_due_date: Option<Date>,
    pub last_updated: Option<Date>,
    /// Account that pays this card's bill (usually the reserve).
    pub pay_from: Option<String>,
    /// Already moved to the reserve (or marked as already there) for the current / next bill.
    pub due_covered: i64,
    pub next_covered: i64,
    /// Spent on this card since the baseline snapshot; `None` when it can't be inferred.
    pub spent_this_period: Option<i64>,
    /// The date that baseline was taken. Spending before it is assumed to follow the pace of the
    /// allowance (we weren't watching yet).
    pub tracked_since: Option<Date>,
}

#[derive(Debug, Clone)]
pub struct SettingsIn {
    /// Monthly spending allowance as an amount. Wins over `spending_bps`.
    pub spending: Option<i64>,
    /// Monthly spending allowance as a share of income, in basis points (2500 = 25%).
    /// With neither set, the allowance is 20% of income.
    pub spending_bps: Option<i64>,
    /// Day of the month the savings/needs money should have reached the reserve.
    pub allocation_day: u8,
    /// Savings in the reserve that card bills must not eat; `None` = no floor.
    pub savings_floor: Option<i64>,
    /// Pay periods over which an overspend / shortfall is paid back.
    pub recovery_months: i64,
}

/// What the ledger says about this period's savings top-up (needs + rest of savings -> reserve).
#[derive(Debug, Clone, Copy, Default)]
pub struct AllocationState {
    pub moved: i64,
    pub skipped: i64,
}

/// A date from Dates that is planned for later and still unpaid: its estimated cost is held back
/// from Safe to Spend until it is kept.
#[derive(Debug, Clone)]
pub struct PlannedDateIn {
    pub date: Date,
    pub label: String,
    /// Estimated cost in paise, less what the jar already covers.
    pub amount: i64,
}

#[derive(Debug, Clone)]
pub struct PlanInput {
    pub today: Date,
    pub accounts: Vec<AccountIn>,
    pub incomes: Vec<IncomeIn>,
    pub commitments: Vec<CommitmentIn>,
    pub cards: Vec<CardIn>,
    pub settings: SettingsIn,
    pub allocation: AllocationState,
    pub planned_dates: Vec<PlannedDateIn>,
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

/// One line of the "how this is calculated" walk-through.
#[derive(Debug, Clone, Serialize)]
pub struct Line {
    pub label: String,
    /// Signed: money in is positive, money set aside or going out is negative.
    pub amount_paise: i64,
    pub note: Option<String>,
    /// A subtotal / result row rather than an item.
    pub total: bool,
}

/// A button the user clicks after doing something in their bank.
#[derive(Debug, Clone, Serialize)]
pub struct LedgerAction {
    /// `card_bill` | `allocation`
    pub kind: &'static str,
    pub key: String,
    /// How much to move now.
    pub amount_paise: i64,
    pub from_account_id: Option<String>,
    pub to_account_id: Option<String>,
    /// The part of the plan that cannot be moved right now (offered as "skip this month").
    pub skippable_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Action {
    pub text: String,
    /// `urgent` | `warn` | `info`
    pub severity: &'static str,
    pub ledger: Option<LedgerAction>,
}

/// The monthly budget in amounts: needs = listed bills, spending = allowance, savings = the rest.
#[derive(Debug, Clone, Serialize)]
pub struct Budget {
    pub income_paise: i64,
    pub needs_paise: i64,
    pub spending_paise: i64,
    pub savings_paise: i64,
    pub sip_paise: i64,
    /// Savings beyond the SIP: moves to the reserve every month.
    pub rest_savings_paise: i64,
    /// What should reach the reserve each month: needs money + the rest of savings.
    pub allocation_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpendingPot {
    pub account_name: Option<String>,
    pub balance_paise: i64,
    /// Fixed payments leaving this account before payday (the SIP...).
    pub committed_paise: i64,
    /// Money still to be moved to the reserve (card bills not yet covered + savings top-up).
    pub to_move_paise: i64,
    pub free_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReservePot {
    pub account_name: Option<String>,
    pub balance_paise: i64,
    /// Card bills (covered or not) plus needs payable from here before payday.
    pub required_paise: i64,
    pub free_savings_paise: i64,
    pub floor_paise: i64,
    pub below_floor: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LockedPot {
    pub total_paise: i64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pots {
    pub spending: SpendingPot,
    pub reserve: ReservePot,
    pub locked: LockedPot,
}

/// Spending so far this period against the spending allowance.
#[derive(Debug, Clone, Serialize)]
pub struct Spending {
    /// False when card snapshots can't tell us, so `spent_paise` is unknown.
    pub tracked: bool,
    pub spent_paise: i64,
    /// Allowance assumed used before tracking started, at an even pace.
    pub before_tracking_paise: i64,
    pub allowance_paise: i64,
    pub left_paise: i64,
    pub overspend_paise: i64,
}

/// How to pay back a shortfall in small monthly chunks instead of breaking an FD.
#[derive(Debug, Clone, Serialize)]
pub struct Recovery {
    pub owed_paise: i64,
    pub months: i64,
    pub per_month_paise: i64,
}

/// What the next pay period already has to cover, shown as a heads-up only.
#[derive(Debug, Clone, Serialize)]
pub struct NextPeriod {
    pub label: String,
    pub arrives: Date,
    pub income_paise: i64,
    pub committed_paise: i64,
    pub left_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Upcoming {
    pub date: Date,
    pub label: String,
    pub amount_paise: i64,
    pub kind: &'static str,
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
    /// The next payday: the safe number covers today up to this date.
    pub until: Date,
    pub days_left: i64,
    /// Start of the current pay period (the key for this period's allocation in the ledger).
    pub period_start: Date,
    pub period_label: Option<String>,
    pub next_income: Option<NextIncome>,
    /// How far the spending account is under water (0 when there is room).
    pub short_by_paise: i64,
    pub breakdown: Vec<Line>,
    pub budget: Budget,
    pub pots: Pots,
    pub spending: Spending,
    pub recovery: Option<Recovery>,
    pub next_period: Option<NextPeriod>,
    pub upcoming: Vec<Upcoming>,
    pub actions: Vec<Action>,
    pub stale: Vec<Stale>,
}

// ---------- card breakdown ----------

/// What the bank app shows for a card, plus EMIs and not-yet-posted charges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CardNumbers {
    pub total_due: i64,
    pub outstanding: i64,
    /// Bank-reported "unbilled spent - unbilled credit", when entered. Preferred for the next
    /// bill because EMI conversions make `outstanding` include loan principal that is not billed.
    pub unbilled_net: Option<i64>,
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
    match n.unbilled_net {
        Some(unbilled) => {
            let unbilled = unbilled.max(0);
            CardAmounts {
                due_now_paise: n.total_due,
                next_bill_est_paise: unbilled + n.pending_charges,
                loan_not_due_paise: (n.outstanding - n.total_due - unbilled).max(0),
            }
        }
        None => CardAmounts {
            due_now_paise: n.total_due,
            next_bill_est_paise: (n.outstanding - n.total_due - n.emi_remaining
                + n.emi_monthly
                + n.pending_charges)
                .max(0),
            loan_not_due_paise: (n.emi_remaining - n.emi_monthly).max(0),
        },
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
pub(crate) fn occurrences(day: u8, from: Date, to: Date) -> Vec<Date> {
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

/// The current pay period: `start` (last payday) .. `payday1` (next payday), then `payday2`.
#[derive(Debug, Clone, Copy)]
pub struct PayPeriod {
    pub start: Date,
    pub payday1: Date,
    pub payday2: Date,
}

/// Income dated today is assumed to be in the balances already; income on the payday itself
/// belongs to the NEXT period.
pub fn pay_period(today: Date, incomes: &[IncomeIn]) -> PayPeriod {
    let next = incomes.iter().map(|i| (next_after(i.day, today), i.day)).min_by_key(|(d, _)| *d);
    let payday1 = next.map_or(today + Duration::days(30), |(d, _)| d);
    let payday2 = incomes
        .iter()
        .map(|i| next_after(i.day, payday1))
        .min()
        .unwrap_or(payday1 + Duration::days(30));
    let start = next.map_or(today, |(p1, day)| day_in_month_offset(p1, -1, day));
    PayPeriod { start, payday1, payday2 }
}

/// Dates in `[from, to)` on which `c` is paid, honouring its frequency and first/last month.
pub fn commitment_dates(c: &CommitmentIn, from: Date, to: Date) -> Vec<Date> {
    if !c.active {
        return Vec::new();
    }
    occurrences(c.day, from, to)
        .into_iter()
        .filter(|d| {
            let mi = month_index(*d);
            let in_range = !(c.start_month.is_some_and(|s| mi < s) || c.end_month.is_some_and(|e| mi > e));
            in_range && c.freq.due_in(*d, c.due_month)
        })
        .collect()
}

/// What `c` costs per month: yearly bills are set aside at a twelfth a month.
pub fn monthly_equivalent(c: &CommitmentIn, month_idx: i32) -> i64 {
    let in_range = !(c.start_month.is_some_and(|s| month_idx < s) || c.end_month.is_some_and(|e| month_idx > e));
    if !c.active || !in_range {
        return 0;
    }
    c.amount / c.freq.period_months()
}

/// Everything due from today up to `payday2`, soonest first.
fn upcoming_list(input: &PlanInput, payday2: Date) -> Vec<Upcoming> {
    let today = input.today;
    let mut upcoming: Vec<Upcoming> = Vec::new();
    for c in &input.commitments {
        for d in commitment_dates(c, today, payday2) {
            upcoming.push(Upcoming { date: d, label: c.label.clone(), amount_paise: c.amount, kind: "commitment" });
        }
    }
    for d in input.planned_dates.iter().filter(|d| d.date >= today && d.date < payday2 && d.amount > 0) {
        upcoming.push(Upcoming { date: d.date, label: d.label.clone(), amount_paise: d.amount, kind: "planned_date" });
    }
    for card in &input.cards {
        if card.due_now > 0 {
            upcoming.push(Upcoming {
                date: card.due_date.map_or(today, |d| d.max(today)),
                label: format!("{} bill", card.name),
                amount_paise: card.due_now,
                kind: "card_due",
            });
        }
        if let (true, Some(d)) = (card.next_bill_est > 0, card.next_due_date)
            && d >= today && d < payday2 {
                upcoming.push(Upcoming {
                    date: d,
                    label: format!("{} next bill (est.)", card.name),
                    amount_paise: card.next_bill_est,
                    kind: "card_bill",
                });
            }
    }
    upcoming.sort_by_key(|u| u.date);
    upcoming
}

/// Balances and card snapshots that have not been updated for a while.
fn stale_items(input: &PlanInput) -> Vec<Stale> {
    let today = input.today;
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
    stale
}

/// The current pay period in days: how much is left of it, for prorating a monthly figure.
struct Period {
    start: Date,
    payday1: Date,
    payday2: Date,
    days_left: i64,
    days_total: i64,
}

impl Period {
    fn of(today: Date, incomes: &[IncomeIn]) -> Self {
        let p = pay_period(today, incomes);
        Period {
            start: p.start,
            payday1: p.payday1,
            payday2: p.payday2,
            days_total: (p.payday1 - p.start).whole_days().max(1),
            days_left: (p.payday1 - today).whole_days().max(1),
        }
    }

    fn prorate(&self, monthly: i64) -> i64 {
        monthly * self.days_left / self.days_total
    }
}

/// The monthly budget in amounts, and the part of it already listed as wants.
fn budget_of(input: &PlanInput) -> (Budget, i64) {
    let this_month = month_index(input.today);
    let sum_bucket = |b: Bucket| -> i64 {
        input.commitments.iter().filter(|c| c.bucket == b).map(|c| monthly_equivalent(c, this_month)).sum()
    };
    let income: i64 = input.incomes.iter().map(|i| i.amount).sum();
    let (needs, sip, wants_listed) = (sum_bucket(Bucket::Needs), sum_bucket(Bucket::Savings), sum_bucket(Bucket::Wants));
    let st = &input.settings;
    let allowance = st
        .spending
        .or_else(|| st.spending_bps.map(|bps| income * bps / 10_000))
        .unwrap_or(income * DEFAULT_SPENDING_PCT / 100);
    let savings_total = (income - needs - allowance).max(0);
    let rest_savings = (savings_total - sip).max(0);
    let budget = Budget {
        income_paise: income,
        needs_paise: needs,
        spending_paise: allowance,
        savings_paise: savings_total,
        sip_paise: sip,
        rest_savings_paise: rest_savings,
        allocation_paise: needs + rest_savings,
    };
    (budget, wants_listed)
}

/// The accounts that matter, by role. Accounts without a role: everything unlocked is "spending",
/// with no reserve checks.
struct Accounts<'a> {
    spend: Vec<&'a AccountIn>,
    reserve: Vec<&'a AccountIn>,
    locked_total: i64,
    locked_count: usize,
}

impl<'a> Accounts<'a> {
    fn of(accounts: &'a [AccountIn]) -> Self {
        let unlocked = || accounts.iter().filter(|a| !a.locked);
        let reserve = unlocked().filter(|a| a.role == Role::Reserve).collect();
        let mut spend: Vec<&AccountIn> = unlocked().filter(|a| a.role == Role::Spending).collect();
        if spend.is_empty() {
            spend = unlocked().filter(|a| a.role == Role::Other).collect();
        }
        Accounts {
            spend,
            reserve,
            locked_total: accounts.iter().filter(|a| a.locked).map(|a| a.balance).sum(),
            locked_count: accounts.iter().filter(|a| a.locked).count(),
        }
    }

    fn have_reserve(&self) -> bool {
        !self.reserve.is_empty()
    }

    fn spend_balance(&self) -> i64 {
        self.spend.iter().map(|a| a.balance).sum()
    }

    fn reserve_balance(&self) -> i64 {
        self.reserve.iter().map(|a| a.balance).sum()
    }

    /// Whether a fixed payment leaves the reserve: by its account, else needs when there is a reserve.
    fn paid_from_reserve(&self, c: &CommitmentIn) -> bool {
        let among = |set: &[&AccountIn]| c.account_id.as_ref().is_some_and(|id| set.iter().any(|a| &a.id == id));
        if among(&self.reserve) {
            true
        } else if among(&self.spend) {
            false
        } else {
            self.have_reserve() && c.bucket == Bucket::Needs
        }
    }
}

/// Fixed payments dated before payday, and who pays them.
struct Outflows {
    from_spending: i64,
    from_reserve: i64,
    wants_in_period: i64,
}

impl Outflows {
    fn of(input: &PlanInput, accts: &Accounts, period: &Period) -> Self {
        let mut out = Outflows { from_spending: 0, from_reserve: 0, wants_in_period: 0 };
        for c in &input.commitments {
            let due = commitment_dates(c, input.today, period.payday1).len() as i64 * c.amount;
            if accts.paid_from_reserve(c) {
                out.from_reserve += due;
            } else {
                out.from_spending += due;
            }
            if c.bucket == Bucket::Wants {
                out.wants_in_period += due;
            }
        }
        out
    }
}

/// Card bills, and the part the ledger does not yet cover.
struct CardBills {
    total: i64,
    uncovered: i64,
}

impl CardBills {
    fn of(cards: &[CardIn]) -> Self {
        CardBills {
            total: cards.iter().map(|c| c.due_now.max(0) + c.next_bill_est.max(0)).sum(),
            uncovered: cards
                .iter()
                .map(|c| (c.due_now - c.due_covered).max(0) + (c.next_bill_est - c.next_covered).max(0))
                .sum(),
        }
    }

    /// With no reserve account there is nothing to move to: every bill is a charge on the pool.
    fn charge(&self, have_reserve: bool) -> i64 {
        if have_reserve { self.uncovered } else { self.total }
    }
}

/// The spending allowance: actual spend when known, else an even pace over the period.
fn spending_of(input: &PlanInput, period: &Period, allowance: i64, wants_listed: i64, wants_in_period: i64) -> Spending {
    let cards = &input.cards;
    let tracked = !cards.is_empty() && cards.iter().all(|c| c.spent_this_period.is_some());
    let spent: i64 = cards.iter().filter_map(|c| c.spent_this_period).sum();
    let before_tracking = match (tracked, cards.iter().filter_map(|c| c.tracked_since).min()) {
        (true, Some(b)) => allowance * (b - period.start).whole_days().clamp(0, period.days_total) / period.days_total,
        _ => 0,
    };
    let left = if tracked {
        (allowance - wants_listed - before_tracking - spent).max(0)
    } else {
        (period.prorate(allowance) - wants_in_period).max(0)
    };
    let overspend = if tracked { (spent + before_tracking + wants_listed - allowance).max(0) } else { 0 };
    Spending {
        tracked,
        spent_paise: spent,
        before_tracking_paise: before_tracking,
        allowance_paise: allowance,
        left_paise: left,
        overspend_paise: overspend,
    }
}

/// Cash in the spending account. The savings top-up (needs money + the rest of savings -> reserve)
/// only ever comes from cash BEYOND the spending pot you still have: it never eats it.
struct Cash {
    free_before_top_up: i64,
    alloc_remaining: i64,
    alloc_to_move: i64,
    alloc_unfunded: i64,
    free: i64,
    safe_cash: i64,
    dates_held_applied: i64,
    safe: i64,
}

impl Cash {
    fn of(input: &PlanInput, period: &Period, accts: &Accounts, out: &Outflows, card_charge: i64, spending_left: i64, allocation_plan: i64) -> Self {
        let free_before_top_up = accts.spend_balance() - out.from_spending - card_charge;
        let alloc_remaining = if accts.have_reserve() {
            (allocation_plan - input.allocation.moved - input.allocation.skipped).max(0)
        } else {
            0
        };
        let beyond_pot = (free_before_top_up - free_before_top_up.max(0).min(spending_left)).max(0);
        let alloc_to_move = alloc_remaining.min(beyond_pot);
        let free = free_before_top_up - alloc_to_move;
        let safe_cash = free.max(0);
        // Planned dates before payday are spending that has not happened yet: hold their estimates back.
        let dates_held: i64 = planned_before(input, period.payday1).map(|d| d.amount.max(0)).fold(0i64, |a, b| a.saturating_add(b));
        let safe_before_dates = safe_cash.min(spending_left);
        let dates_held_applied = dates_held.min(safe_before_dates);
        Cash {
            free_before_top_up,
            alloc_remaining,
            alloc_to_move,
            alloc_unfunded: alloc_remaining - alloc_to_move,
            free,
            safe_cash,
            dates_held_applied,
            safe: safe_before_dates - dates_held_applied,
        }
    }
}

fn planned_before(input: &PlanInput, until: Date) -> impl Iterator<Item = &PlannedDateIn> {
    let today = input.today;
    input.planned_dates.iter().filter(move |d| d.date >= today && d.date < until)
}

fn status_of(per_week: i64, safe: i64) -> Status {
    if per_week >= GREEN_PER_WEEK_PAISE {
        Status::Green
    } else if safe > 0 {
        Status::Yellow
    } else {
        Status::Red
    }
}

/// What is missing this month: cash that can't cover the SIP/bills, plus savings that can't be
/// funded or were skipped. Paying it back over a few months keeps the FDs untouched.
fn recovery_of(input: &PlanInput, cash: &Cash) -> (i64, Option<Recovery>) {
    let short_by = (-cash.free_before_top_up).max(0);
    let owed = short_by + cash.alloc_unfunded + input.allocation.skipped;
    let months = input.settings.recovery_months.max(1);
    let recovery = (owed > 0).then(|| Recovery { owed_paise: owed, months, per_month_paise: (owed + months - 1) / months });
    (short_by, recovery)
}

fn reserve_pot(input: &PlanInput, accts: &Accounts, bills: &CardBills, out: &Outflows) -> ReservePot {
    let floor = input.settings.savings_floor.unwrap_or(0);
    let required = if accts.have_reserve() { bills.total + out.from_reserve } else { 0 };
    let balance = accts.reserve_balance();
    let free_savings = balance - required;
    ReservePot {
        account_name: accts.reserve.first().map(|a| a.name.clone()),
        balance_paise: balance,
        required_paise: required,
        free_savings_paise: free_savings,
        floor_paise: floor,
        below_floor: accts.have_reserve() && free_savings < floor,
    }
}

/// Labels, heads-up for next period, from the next payday.
fn next_period_of(input: &PlanInput, period: &Period) -> (Option<NextIncome>, Option<String>, Option<NextPeriod>) {
    let next_pay = input.incomes.iter().map(|i| (next_after(i.day, input.today), i)).min_by_key(|(d, _)| *d);
    let Some((p1, src)) = next_pay else { return (None, None, None) };
    let income_in: i64 = input.incomes.iter().map(|i| occurrences(i.day, p1, period.payday2).len() as i64 * i.amount).sum();
    let committed: i64 = input.commitments.iter().map(|c| commitment_dates(c, p1, period.payday2).len() as i64 * c.amount).sum();
    (
        Some(NextIncome { label: money_label(p1), arrives: p1 }),
        Some(money_label(day_in_month_offset(p1, -1, src.day))),
        Some(NextPeriod { label: money_label(p1), arrives: p1, income_paise: income_in, committed_paise: committed, left_paise: income_in - committed }),
    )
}

/// Everything worked out so far, for the parts that explain it: the walk-through and the actions.
struct Facts<'a> {
    input: &'a PlanInput,
    period: &'a Period,
    accts: &'a Accounts<'a>,
    out: &'a Outflows,
    bills: &'a CardBills,
    card_charge: i64,
    spending: &'a Spending,
    wants_listed: i64,
    cash: &'a Cash,
    reserve: &'a ReservePot,
    /// Names for the sentences: the spending account and the reserve.
    sn: String,
    rn: String,
}

fn line(label: impl Into<String>, amount_paise: i64, note: Option<String>, total: bool) -> Line {
    Line { label: label.into(), amount_paise, note, total }
}

/// The "how this is calculated" walk-through, from the spending balance down to Safe to Spend.
fn breakdown(f: &Facts) -> Vec<Line> {
    let (sn, rn, cash) = (&f.sn, &f.rn, f.cash);
    let have_reserve = f.accts.have_reserve();
    let mut rows = vec![line(format!("{sn} balance"), f.accts.spend_balance(), None, false)];
    if f.out.from_spending > 0 {
        rows.push(line(format!("Fixed payments from it before {}", short_date(f.period.payday1)), -f.out.from_spending, None, false));
    }
    if f.card_charge > 0 {
        rows.push(line(
            if have_reserve { format!("Card bills still to move to {rn}") } else { "Card bills already spent".into() },
            -f.card_charge,
            Some(if have_reserve {
                format!("{} of {} is already covered", format_inr(f.bills.total - f.bills.uncovered), format_inr(f.bills.total))
            } else {
                "Counted now, even if due after payday".into()
            }),
            false,
        ));
    }
    if cash.alloc_to_move > 0 {
        rows.push(line(
            format!("Savings top-up to move to {rn}"),
            -cash.alloc_to_move,
            Some("Only from cash beyond your spending pot".into()),
            false,
        ));
    }
    rows.push(line(format!("Free in {sn}"), cash.free, None, true));
    if cash.alloc_unfunded > 0 {
        rows.push(line(
            "Savings top-up you can't fund this month",
            0,
            Some(format!("{} of the plan; see \"Paying it back\"", format_inr(cash.alloc_unfunded))),
            false,
        ));
    }
    if cash.safe_cash > cash.safe {
        let s = f.spending;
        rows.push(line(
            "Held back to fit your spending allowance",
            -(cash.safe_cash - cash.safe),
            Some(if s.tracked {
                format!(
                    "{} of {} is already used this period",
                    format_inr(s.spent_paise + s.before_tracking_paise + f.wants_listed),
                    format_inr(s.allowance_paise)
                )
            } else {
                "Spare cash beyond your spending allowance is better saved".into()
            }),
            false,
        ));
    }
    if cash.dates_held_applied > 0 {
        let n = planned_before(f.input, f.period.payday1).filter(|d| d.amount > 0).count();
        rows.push(line(
            "Held back for planned dates",
            -cash.dates_held_applied,
            Some(format!("{n} planned before {}; released when you keep them", short_date(f.period.payday1))),
            false,
        ));
    }
    rows.push(line("Safe to spend", cash.safe, None, true));
    rows
}

fn urgency(today: Date, date: Option<Date>) -> &'static str {
    match date {
        Some(d) if (d - today).whole_days() <= URGENT_WITHIN_DAYS => "urgent",
        _ => "warn",
    }
}

/// A button per card bill the ledger has not covered yet: move the open amount to the card's payer.
fn card_bill_actions(f: &Facts, actions: &mut Vec<Action>) {
    let (today, sn, rn) = (f.input.today, &f.sn, &f.rn);
    for c in &f.input.cards {
        let bills = [
            (c.due_now, c.due_covered, c.due_date, "current bill"),
            (c.next_bill_est, c.next_covered, c.next_due_date, "next bill"),
        ];
        for (amount, covered, due, what) in bills {
            let open = (amount - covered).max(0);
            if open == 0 {
                continue;
            }
            let by = due.map_or(String::new(), |d| format!(", pay by {}", short_date(d)));
            actions.push(Action {
                text: format!("Move {} {} → {} for {}'s {}{}", format_inr(open), sn, rn, c.name, what, by),
                severity: urgency(today, due),
                ledger: Some(LedgerAction {
                    kind: "card_bill",
                    key: format!("{}:{}", c.card_id, due.map_or("none".to_string(), |d| d.to_string())),
                    amount_paise: open,
                    from_account_id: f.accts.spend.first().map(|a| a.id.clone()),
                    to_account_id: c.pay_from.clone().or_else(|| f.accts.reserve.first().map(|a| a.id.clone())),
                    skippable_paise: 0,
                }),
            });
        }
    }
}

/// The savings top-up to the reserve: what to move, by when, and what can be skipped.
fn allocation_action(f: &Facts) -> Action {
    let (sn, rn, cash) = (&f.sn, &f.rn, f.cash);
    let start = f.period.start;
    let day = f.input.settings.allocation_day;
    let alloc_date = clamped(start.year(), start.month(), day);
    let alloc_date = if alloc_date <= start { day_in_month_offset(start, 1, day) } else { alloc_date };
    let text = if cash.alloc_to_move == 0 {
        format!(
            "Savings top-up of {} {} → {} is due by {}, but nothing is free in {}. Skip it this month, or move money in first",
            format_inr(cash.alloc_remaining), sn, rn, short_date(alloc_date), sn
        )
    } else if cash.alloc_to_move < cash.alloc_remaining {
        format!(
            "By {}: move {} {} → {} (needs + savings). The plan is {}, but only {} is free this month",
            short_date(alloc_date), format_inr(cash.alloc_to_move), sn, rn, format_inr(cash.alloc_remaining), format_inr(cash.alloc_to_move)
        )
    } else {
        format!("By {}: move {} {} → {} (needs + savings)", short_date(alloc_date), format_inr(cash.alloc_to_move), sn, rn)
    };
    Action {
        text,
        severity: urgency(f.input.today, Some(alloc_date)),
        ledger: Some(LedgerAction {
            kind: "allocation",
            key: start.to_string(),
            amount_paise: cash.alloc_to_move,
            from_account_id: f.accts.spend.first().map(|a| a.id.clone()),
            to_account_id: f.accts.reserve.first().map(|a| a.id.clone()),
            skippable_paise: cash.alloc_remaining - cash.alloc_to_move,
        }),
    }
}

/// Ledger buttons first, then warnings, then reminders to update stale figures.
fn actions_of(f: &Facts, stale: &[Stale]) -> Vec<Action> {
    let mut actions = Vec::new();
    if f.accts.have_reserve() {
        let (sn, rn, spend_bal, leaving) = (&f.sn, &f.rn, f.accts.spend_balance(), f.out.from_spending);
        card_bill_actions(f, &mut actions);
        if f.cash.alloc_remaining > 0 {
            actions.push(allocation_action(f));
        }
        if spend_bal < leaving {
            actions.push(Action {
                text: format!(
                    "{sn} holds {} but {} leaves it before {}: move {} {rn} → {sn}",
                    format_inr(spend_bal), format_inr(leaving), short_date(f.period.payday1), format_inr(leaving - spend_bal)
                ),
                severity: "urgent",
                ledger: None,
            });
        }
        if f.reserve.below_floor {
            actions.push(Action {
                text: format!("{rn} free savings are {}, below your {} floor", format_inr(f.reserve.free_savings_paise), format_inr(f.reserve.floor_paise)),
                severity: "warn",
                ledger: None,
            });
        }
    }
    for s in stale {
        actions.push(Action { text: format!("Update {} (last updated {} days ago)", s.what, s.days_old), severity: "info", ledger: None });
    }
    actions
}

pub fn compute(input: &PlanInput) -> Summary {
    let period = Period::of(input.today, &input.incomes);
    let (budget, wants_listed) = budget_of(input);
    let accts = Accounts::of(&input.accounts);
    let out = Outflows::of(input, &accts, &period);
    let bills = CardBills::of(&input.cards);
    let card_charge = bills.charge(accts.have_reserve());
    let spending = spending_of(input, &period, budget.spending_paise, wants_listed, out.wants_in_period);
    let cash = Cash::of(input, &period, &accts, &out, card_charge, spending.left_paise, budget.allocation_paise);

    let weeks = ((period.days_left + 6) / 7).max(1);
    let per_week = cash.safe / weeks;
    let (short_by, recovery) = recovery_of(input, &cash);
    let reserve = reserve_pot(input, &accts, &bills, &out);
    let (next_income, period_label, next_period) = next_period_of(input, &period);
    let stale = stale_items(input);

    let spend_name = accts.spend.first().map(|a| a.name.clone());
    let facts = Facts {
        input,
        period: &period,
        accts: &accts,
        out: &out,
        bills: &bills,
        card_charge,
        spending: &spending,
        wants_listed,
        cash: &cash,
        reserve: &reserve,
        sn: spend_name.clone().unwrap_or_else(|| "spending account".into()),
        rn: reserve.account_name.clone().unwrap_or_else(|| "reserve".into()),
    };
    let breakdown = breakdown(&facts);
    let actions = actions_of(&facts, &stale);

    Summary {
        safe_to_spend_paise: cash.safe,
        per_week_paise: per_week,
        status: status_of(per_week, cash.safe),
        until: period.payday1,
        days_left: period.days_left,
        period_start: period.start,
        period_label,
        next_income,
        short_by_paise: short_by,
        breakdown,
        budget,
        pots: Pots {
            spending: SpendingPot {
                account_name: spend_name,
                balance_paise: accts.spend_balance(),
                committed_paise: out.from_spending,
                to_move_paise: card_charge + cash.alloc_to_move,
                free_paise: cash.free,
            },
            reserve,
            locked: LockedPot { total_paise: accts.locked_total, count: accts.locked_count },
        },
        spending,
        recovery,
        next_period,
        upcoming: upcoming_list(input, period.payday2),
        actions,
        stale,
    }
}

#[cfg(test)]
mod compute_golden;
#[cfg(test)]
mod projection_golden;
#[cfg(test)]
mod life_golden;
#[cfg(test)]
mod afford_golden;
#[cfg(test)]
mod insights_golden;
#[cfg(test)]
mod tests;
