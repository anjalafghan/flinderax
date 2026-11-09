//! Habits: the rules that say which data upload is due and which money habit to change, from the
//! user's own numbers. Pure: no DB, no clock. The handler fills `HabitsInput` and passes `today`.
//! Money is paise; every figure in the text is whole rupees.

use serde::Serialize;
use time::{Date, Duration, Month};

use crate::planner::{format_inr, short_date};

/// Most habits returned, after sorting.
const MAX_HABITS: usize = 12;
/// A dated habit is `due_soon` within this many days.
const DUE_SOON_DAYS: i64 = 14;
/// Advance tax and return dates are listed this many days ahead.
const TAX_WINDOW_DAYS: i64 = 30;
/// A card's statement is due this many days after its statement day.
const CARD_DUE_AFTER_DAYS: i64 = 3;
/// Bank statements are wanted every 90 days.
const BANK_CADENCE_DAYS: i64 = 90;
/// Uncategorised share of 90-day spend at or above this percentage raises the categorise habit.
const CATEGORISE_SHARE_PCT: i64 = 20;
/// Own-account transfers out in 90 days at or above this count raise the automate habit.
const TRANSFERS_MIN_COUNT: i64 = 6;
/// A salary spread wider than this many days raises the buffer habit.
const BUFFER_SPREAD_DAYS: u8 = 10;
const EMERGENCY_TARGET_MONTHS: i64 = 6;
/// Emergency cover below this many tenths of a month (3 months) is high impact.
const EMERGENCY_LOW_TENTHS: i64 = 30;
/// Each April, SIPs step up by this percentage.
const SIP_STEP_UP_PCT: i64 = 10;
/// A monthly SIP total at or above this (₹10,000) is medium impact.
const SIP_MEDIUM_PAISE: i64 = 1_000_000;
/// Payments under this amount are "small" (₹200); matches the analytics threshold.
const SMALL_PAYMENT_PAISE: i64 = 20_000;
/// Small payments at or above this a month (₹2,000) raise the habit.
const SMALL_PAYMENTS_MIN_PAISE: i64 = 200_000;
/// Weeks in a month, in hundredths (4.33).
const WEEKS_PER_MONTH_HUNDREDTHS: i64 = 433;
/// Food this month is over pace once it reaches this percentage of the usual pace.
const FOOD_OVER_PACE_PCT: i64 = 115;
/// Advance tax instalments and the income tax return, as (month, day, is_advance_tax).
const TAX_DATES: [(Month, u8, bool); 5] = [
    (Month::June, 15, true),
    (Month::September, 15, true),
    (Month::December, 15, true),
    (Month::March, 15, true),
    (Month::July, 31, false),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Area {
    Data,
    Money,
    Food,
    Goals,
    Protection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Cadence {
    Once,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

/// Declared in urgency order: the sort relies on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Overdue,
    DueSoon,
    Ok,
}

/// Declared in priority order: the sort relies on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Impact {
    High,
    Medium,
    Low,
}

/// An in-app route the habit links to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Link {
    pub label: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Habit {
    pub key: String,
    pub area: Area,
    pub title: String,
    pub why: String,
    pub action: Option<Link>,
    pub cadence: Cadence,
    pub due_on: Option<String>,
    pub status: Status,
    pub impact: Impact,
}

/// A bank account and the last day its statements cover, if any statement was ever imported.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BankIn {
    pub account_id: String,
    pub name: String,
    pub last_covered: Option<Date>,
}

/// A card and its statement day (1..=31), if the user set one.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CardIn {
    pub card_id: String,
    pub name: String,
    pub statement_day: Option<u8>,
}

/// Spending over the last 90 days, plus money held and transfers, all in paise.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpendIn {
    pub spend_90d_paise: i64,
    pub uncategorised_90d_paise: i64,
    /// Any spend at all in the analytics window.
    pub any_spend: bool,
    /// Average spend of the last three completed months (0 when there is none).
    pub avg_monthly_spend_paise: i64,
    /// Balances of accounts that are not locked.
    pub liquid_paise: i64,
    /// Liquid cover in tenths of a month, from the insights health. None when unknown.
    pub emergency_months_tenths: Option<i64>,
    /// Own-account transfers out over the last 90 days.
    pub own_transfers_out_count: i64,
    pub own_transfers_out_paise: i64,
    /// Payments under ₹200, per month.
    pub small_per_month_count: i64,
    pub small_per_month_paise: i64,
}

/// Salary as seen in the statements.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SalaryIn {
    /// Months in the statement span with no salary, as "YYYY-MM", oldest first.
    pub missing_months: Vec<String>,
    /// Distinct days of the month salary arrived on.
    pub days: Vec<u8>,
}

/// Food and groceries spend. `any_tagged` is true when any food or grocery spend is categorised.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FoodIn {
    pub any_tagged: bool,
    pub this_month_paise: i64,
    /// Average of the previous three months.
    pub prior_avg_paise: i64,
    /// Today's day of the month, and the number of days in this month.
    pub day: u8,
    pub days_in_month: u8,
}

/// The saved life plan, reduced to what the habits need.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LifeIn {
    pub fits: bool,
    pub shortfall_paise: i64,
    /// The goal that takes the most a month, e.g. "Wedding".
    pub top_goal: Option<String>,
    pub term_gap_paise: i64,
    pub health_gap_paise: i64,
}

/// Everything the rules read. The handler fills it from the DB; tests build it directly.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HabitsInput {
    pub banks: Vec<BankIn>,
    pub cards: Vec<CardIn>,
    pub spend: SpendIn,
    /// None when the statements show no salary.
    pub salary: Option<SalaryIn>,
    /// One amount per SIP mandate, in paise.
    pub sip_paise: Vec<i64>,
    pub food: FoodIn,
    /// None when no life profile is saved.
    pub life: Option<LifeIn>,
}

/// The habits to act on now, sorted: overdue, then due soon, then ok; within each by impact,
/// then due date. Ok habits with a once cadence are dropped, and the list is capped at 12.
pub fn build(input: &HabitsInput, today: Date) -> Vec<Habit> {
    let mut out = Vec::new();
    bank_statements(input, today, &mut out);
    card_statements(input, today, &mut out);
    categorise(&input.spend, &mut out);
    automate_transfers(&input.spend, &mut out);
    salary_buffer(input, &mut out);
    emergency_fund(&input.spend, &mut out);
    sip_step_up(input, today, &mut out);
    small_payments(&input.spend, &mut out);
    food_budget(input, &mut out);
    life_habits(input, &mut out);
    tax_dates(today, &mut out);

    out.retain(|h| !(h.status == Status::Ok && h.cadence == Cadence::Once));
    out.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then(a.impact.cmp(&b.impact))
            .then(a.due_on.is_none().cmp(&b.due_on.is_none()))
            .then(a.due_on.cmp(&b.due_on))
            .then(a.key.cmp(&b.key))
    });
    out.truncate(MAX_HABITS);
    out
}

fn link(label: &str, to: &str) -> Option<Link> {
    Some(Link {
        label: label.to_string(),
        to: to.to_string(),
    })
}

/// Status of a dated habit: overdue once the date has passed, due soon within two weeks.
fn due_status(due: Date, today: Date) -> Status {
    let days = (due - today).whole_days();
    if days < 0 {
        Status::Overdue
    } else if days <= DUE_SOON_DAYS {
        Status::DueSoon
    } else {
        Status::Ok
    }
}

/// Money in text: whole rupees, Indian grouping.
fn inr(paise: i64) -> String {
    format_inr(paise.saturating_add(50).div_euclid(100).saturating_mul(100))
}

/// "2026-10" becomes "October 2026". Anything else is returned as it came.
fn month_label(ym: &str) -> String {
    const NAMES: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let mut parts = ym.split('-');
    let year = parts.next();
    let month = parts
        .next()
        .and_then(|m| m.parse::<usize>().ok())
        .and_then(|m| m.checked_sub(1));
    match (year, month.and_then(|i| NAMES.get(i))) {
        (Some(y), Some(name)) => format!("{name} {y}"),
        _ => ym.to_string(),
    }
}

/// The date `day` of the month, clamped to the month's length.
fn on_day(year: i32, month: Month, day: u8) -> Date {
    let day = day.clamp(1, month.length(year));
    // The arguments are clamped above, so this only falls back on an impossible year.
    Date::from_calendar_date(year, month, day).unwrap_or(Date::MIN)
}

fn next_month(date: Date) -> (i32, Month) {
    if date.month() == Month::December {
        (date.year() + 1, Month::January)
    } else {
        (
            date.year(),
            Month::try_from(date.month() as u8 + 1).unwrap_or(Month::January),
        )
    }
}

/// The next statement date: this month's statement day if still ahead, else next month's. With
/// no statement day, the 5th of next month.
fn next_statement(day: Option<u8>, today: Date) -> Date {
    let (y, m) = next_month(today);
    match day {
        Some(d) => {
            let this = on_day(today.year(), today.month(), d);
            if this >= today {
                this
            } else {
                on_day(y, m, d)
            }
        }
        None => on_day(y, m, 5),
    }
}

fn upload() -> Option<Link> {
    link("Upload", "/spending")
}

fn bank_statements(input: &HabitsInput, today: Date, out: &mut Vec<Habit>) {
    for b in &input.banks {
        let key = format!("bank_statement:{}", b.account_id);
        let title = format!("Upload your {} statement", b.name);
        let habit = match b.last_covered {
            None => Habit {
                key,
                area: Area::Data,
                title,
                why: "No statement from this account yet. One upload every 3 months keeps every number in joy real."
                    .to_string(),
                action: upload(),
                cadence: Cadence::Once,
                due_on: None,
                status: Status::Overdue,
                impact: Impact::High,
            },
            Some(last) => {
                let due = last + Duration::days(BANK_CADENCE_DAYS);
                Habit {
                    key,
                    area: Area::Data,
                    title,
                    why: format!(
                        "Covers up to {}. One upload every 3 months keeps every number in joy real.",
                        short_date(last)
                    ),
                    action: upload(),
                    cadence: Cadence::Quarterly,
                    due_on: Some(due.to_string()),
                    status: due_status(due, today),
                    impact: Impact::High,
                }
            }
        };
        out.push(habit);
    }
}

fn card_statements(input: &HabitsInput, today: Date, out: &mut Vec<Habit>) {
    for c in &input.cards {
        let due = next_statement(c.statement_day, today) + Duration::days(CARD_DUE_AFTER_DAYS);
        out.push(Habit {
            key: format!("card_statement:{}", c.card_id),
            area: Area::Data,
            title: format!("Upload your {} statement", c.name),
            why: "Card spends stay invisible until the statement is in; paying the full bill keeps the interest-free period."
                .to_string(),
            action: upload(),
            cadence: Cadence::Monthly,
            due_on: Some(due.to_string()),
            status: due_status(due, today),
            impact: Impact::High,
        });
    }
}

fn categorise(s: &SpendIn, out: &mut Vec<Habit>) {
    let (total, uncat) = (s.spend_90d_paise, s.uncategorised_90d_paise);
    if total <= 0 || uncat.saturating_mul(100) < total.saturating_mul(CATEGORISE_SHARE_PCT) {
        return;
    }
    // Rounded percentage: (2 * uncat * 100 + total) / (2 * total).
    let pct = uncat.saturating_mul(200).saturating_add(total) / total.saturating_mul(2);
    out.push(Habit {
        key: "categorise".to_string(),
        area: Area::Data,
        title: "Categorise your spending".to_string(),
        why: format!(
            "{} of {} ({}%) has no category, so budgets can't be checked.",
            inr(uncat),
            inr(total),
            pct
        ),
        action: link("Categorise", "/spending/sort"),
        cadence: Cadence::Once,
        due_on: None,
        status: Status::Overdue,
        impact: Impact::High,
    });
}

fn automate_transfers(s: &SpendIn, out: &mut Vec<Habit>) {
    if s.own_transfers_out_count < TRANSFERS_MIN_COUNT {
        return;
    }
    out.push(Habit {
        key: "automate_transfers".to_string(),
        area: Area::Money,
        title: "Automate your monthly transfer".to_string(),
        why: format!(
            "{} manual transfers to your own accounts in 3 months (avg {} a month). One standing instruction on payday does it for you.",
            s.own_transfers_out_count,
            inr(s.own_transfers_out_paise / 3)
        ),
        action: None,
        cadence: Cadence::Monthly,
        due_on: None,
        status: Status::DueSoon,
        impact: Impact::Medium,
    });
}

fn salary_buffer(input: &HabitsInput, out: &mut Vec<Habit>) {
    let (Some(salary), avg) = (&input.salary, input.spend.avg_monthly_spend_paise) else {
        return;
    };
    if avg <= 0 {
        return;
    }
    let spread = match (salary.days.iter().min(), salary.days.iter().max()) {
        (Some(lo), Some(hi)) => hi - lo,
        _ => 0,
    };
    let (why, status) = if let Some(month) = salary.missing_months.last() {
        (
            format!(
                "No salary landed in {} in your statements. Keep {} aside so a late payday doesn't stop you.",
                month_label(month),
                inr(avg)
            ),
            Status::Overdue,
        )
    } else if spread > BUFFER_SPREAD_DAYS {
        let lo = salary.days.iter().min().copied().unwrap_or_default();
        let hi = salary.days.iter().max().copied().unwrap_or_default();
        (
            format!(
                "Salary lands anywhere from day {lo} to day {hi} of the month. Keep {} aside so a late payday doesn't stop you.",
                inr(avg)
            ),
            Status::DueSoon,
        )
    } else {
        return;
    };
    out.push(Habit {
        key: "salary_buffer".to_string(),
        area: Area::Money,
        title: "Keep one month's spending as a buffer".to_string(),
        why,
        action: None,
        cadence: Cadence::Once,
        due_on: None,
        status,
        impact: Impact::High,
    });
}

fn emergency_fund(s: &SpendIn, out: &mut Vec<Habit>) {
    let avg = s.avg_monthly_spend_paise;
    if avg <= 0 {
        return;
    }
    let liquid = s.liquid_paise.max(0);
    let target = avg.saturating_mul(EMERGENCY_TARGET_MONTHS);
    if liquid >= target {
        return;
    }
    let tenths = s
        .emergency_months_tenths
        .unwrap_or(liquid.saturating_mul(10) / avg);
    let gap = target - liquid;
    let (status, impact) = if tenths < EMERGENCY_LOW_TENTHS {
        (Status::Overdue, Impact::High)
    } else {
        (Status::DueSoon, Impact::Medium)
    };
    out.push(Habit {
        key: "emergency_fund".to_string(),
        area: Area::Money,
        title: "Build a six-month emergency fund".to_string(),
        why: format!(
            "Liquid money covers {}.{} months; 6 months is {}. Moving {} each payday gets there in 12 months.",
            tenths / 10,
            tenths % 10,
            inr(target),
            inr(gap / 12)
        ),
        action: None,
        cadence: Cadence::Monthly,
        due_on: None,
        status,
        impact,
    });
}

/// The next 1 April, today included.
fn next_april(today: Date) -> Date {
    let this = on_day(today.year(), Month::April, 1);
    if this >= today {
        this
    } else {
        on_day(today.year() + 1, Month::April, 1)
    }
}

fn sip_step_up(input: &HabitsInput, today: Date, out: &mut Vec<Habit>) {
    let total: i64 = input
        .sip_paise
        .iter()
        .fold(0i64, |acc, p| acc.saturating_add(*p));
    if total <= 0 {
        return;
    }
    let count = input.sip_paise.len();
    let due = next_april(today);
    out.push(Habit {
        key: "sip_step_up".to_string(),
        area: Area::Money,
        title: "Step up your SIPs by 10% each April".to_string(),
        why: format!(
            "{} a month in {} SIP{}. A {}% step-up each April adds {} a month.",
            inr(total),
            count,
            if count == 1 { "" } else { "s" },
            SIP_STEP_UP_PCT,
            inr(total.saturating_mul(SIP_STEP_UP_PCT) / 100)
        ),
        action: None,
        cadence: Cadence::Yearly,
        due_on: Some(due.to_string()),
        status: due_status(due, today),
        impact: if total >= SIP_MEDIUM_PAISE {
            Impact::Medium
        } else {
            Impact::Low
        },
    });
}

fn small_payments(s: &SpendIn, out: &mut Vec<Habit>) {
    if s.small_per_month_paise < SMALL_PAYMENTS_MIN_PAISE {
        return;
    }
    out.push(Habit {
        key: "small_payments".to_string(),
        area: Area::Money,
        title: "Batch your small payments".to_string(),
        why: format!(
            "{} payments under {} a month come to {} a month. Batching them into one weekly spend keeps them visible.",
            s.small_per_month_count,
            inr(SMALL_PAYMENT_PAISE),
            inr(s.small_per_month_paise)
        ),
        action: None,
        cadence: Cadence::Monthly,
        due_on: None,
        status: Status::DueSoon,
        impact: Impact::Low,
    });
}

/// Below this average a month, food tagging is too sparse to plan from (₹1,000).
const MIN_FOOD_MONTH_PAISE: i64 = 100_000;

fn food_budget(input: &HabitsInput, out: &mut Vec<Habit>) {
    let f = &input.food;
    // Under ₹1,000 a month tagged as food is almost always untagged food, not a real food budget:
    // planning "meals within ₹9 a week" from it would be nonsense.
    let too_little = f.any_tagged && f.prior_avg_paise < MIN_FOOD_MONTH_PAISE;
    if !f.any_tagged || too_little {
        if input.spend.any_spend {
            let why = if too_little {
                format!(
                    "Only {} a month is tagged food or groceries, too little to plan a food budget from.",
                    inr(f.prior_avg_paise)
                )
            } else {
                "Nothing you spend is tagged food or groceries yet, so joy can't plan a food budget.".to_string()
            };
            out.push(Habit {
                key: "tag_food_payees".to_string(),
                area: Area::Food,
                title: "Tag your food and grocery payees".to_string(),
                why,
                action: link("Tag", "/spending"),
                cadence: Cadence::Once,
                due_on: None,
                status: Status::Overdue,
                impact: Impact::Medium,
            });
        }
        return;
    }
    if f.prior_avg_paise <= 0 {
        return;
    }
    let weekly = f
        .prior_avg_paise
        .saturating_mul(100)
        .saturating_add(WEEKS_PER_MONTH_HUNDREDTHS / 2)
        / WEEKS_PER_MONTH_HUNDREDTHS;
    let days = i64::from(f.days_in_month.max(1));
    let pace = f.prior_avg_paise.saturating_mul(i64::from(f.day)) / days;
    let over = pace > 0
        && f.this_month_paise.saturating_mul(100) >= pace.saturating_mul(FOOD_OVER_PACE_PCT);
    let why = if over {
        format!(
            "Food is {} this month, {} over your usual pace.",
            inr(f.this_month_paise),
            inr(f.this_month_paise - pace)
        )
    } else {
        format!(
            "Food is {} this month, within your usual pace.",
            inr(f.this_month_paise)
        )
    };
    out.push(Habit {
        key: "food_budget".to_string(),
        area: Area::Food,
        title: format!("Plan this week's meals within {}", inr(weekly)),
        why,
        action: None,
        cadence: Cadence::Weekly,
        due_on: None,
        status: if over { Status::DueSoon } else { Status::Ok },
        impact: Impact::Medium,
    });
}

fn life_habits(input: &HabitsInput, out: &mut Vec<Habit>) {
    let Some(life) = &input.life else {
        out.push(Habit {
            key: "life_plan_missing".to_string(),
            area: Area::Goals,
            title: "Add your wedding and family goals".to_string(),
            why: "Joy can't check your goals or insurance until it knows your wedding, child and retirement plans."
                .to_string(),
            action: link("Add goals", "/life"),
            cadence: Cadence::Once,
            due_on: None,
            status: Status::Overdue,
            impact: Impact::High,
        });
        return;
    };
    if !life.fits && life.shortfall_paise > 0 {
        let why = match &life.top_goal {
            Some(goal) => format!(
                "Your goals need {} a month more than you save; the {} is the biggest part.",
                inr(life.shortfall_paise),
                goal.to_lowercase()
            ),
            None => format!(
                "Your goals need {} a month more than you save.",
                inr(life.shortfall_paise)
            ),
        };
        out.push(Habit {
            key: "goals_shortfall".to_string(),
            area: Area::Goals,
            title: "Close the monthly gap to your goals".to_string(),
            why,
            action: link("Review goals", "/life"),
            cadence: Cadence::Monthly,
            due_on: None,
            status: Status::Overdue,
            impact: Impact::High,
        });
    }
    if life.term_gap_paise > 0 {
        out.push(Habit {
            key: "insurance:term".to_string(),
            area: Area::Protection,
            title: "Raise your term cover".to_string(),
            why: format!(
                "Your term cover is {} short of what your family needs.",
                inr(life.term_gap_paise)
            ),
            action: link("Review cover", "/life"),
            cadence: Cadence::Yearly,
            due_on: None,
            status: Status::Overdue,
            impact: Impact::High,
        });
    }
    if life.health_gap_paise > 0 {
        out.push(Habit {
            key: "insurance:health".to_string(),
            area: Area::Protection,
            title: "Raise your health cover".to_string(),
            why: format!(
                "Your health cover is {} short of what your family needs.",
                inr(life.health_gap_paise)
            ),
            action: link("Review cover", "/life"),
            cadence: Cadence::Yearly,
            due_on: None,
            status: Status::Overdue,
            impact: Impact::High,
        });
    }
}

/// Advance tax and return dates within the window, in the current and next year.
fn tax_dates(today: Date, out: &mut Vec<Habit>) {
    for year in [today.year(), today.year() + 1] {
        for (month, day, advance) in TAX_DATES {
            let due = on_day(year, month, day);
            let days = (due - today).whole_days();
            if !(0..=TAX_WINDOW_DAYS).contains(&days) {
                continue;
            }
            let (title, why) = if advance {
                (
                    format!("Pay advance tax by {}", short_date(due)),
                    format!("Advance tax instalment due {}.", short_date(due)),
                )
            } else {
                (
                    "File your income tax return".to_string(),
                    format!("Income tax return due {}.", short_date(due)),
                )
            };
            out.push(Habit {
                key: format!("tax:{due}"),
                area: Area::Money,
                title,
                why,
                action: None,
                cadence: Cadence::Once,
                due_on: Some(due.to_string()),
                status: Status::DueSoon,
                impact: Impact::Medium,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), day).unwrap()
    }

    fn today() -> Date {
        d(2026, 11, 9)
    }

    fn find<'a>(habits: &'a [Habit], key: &str) -> Option<&'a Habit> {
        habits.iter().find(|h| h.key == key)
    }

    fn bank(id: &str, name: &str, last: Option<Date>) -> BankIn {
        BankIn {
            account_id: id.into(),
            name: name.into(),
            last_covered: last,
        }
    }

    fn card(id: &str, day: Option<u8>) -> CardIn {
        CardIn {
            card_id: id.into(),
            name: format!("Card {id}"),
            statement_day: day,
        }
    }

    /// Life profile that fits its plan, so no life habit fires.
    fn life_fits() -> Option<LifeIn> {
        Some(LifeIn {
            fits: true,
            ..LifeIn::default()
        })
    }

    #[test]
    fn empty_input_only_asks_for_the_life_plan() {
        let habits = build(&HabitsInput::default(), today());
        let keys: Vec<&str> = habits.iter().map(|h| h.key.as_str()).collect();
        assert_eq!(keys, vec!["life_plan_missing"]);
        assert_eq!(habits[0].status, Status::Overdue);
        assert_eq!(habits[0].impact, Impact::High);
        assert_eq!(habits[0].cadence, Cadence::Once);
    }

    #[test]
    fn bank_statement_is_due_90_days_after_coverage_and_overdue_after() {
        // Covers to 1 Sep: due 30 Nov, 21 days away, so on track and kept as an ok habit.
        let input = HabitsInput {
            banks: vec![bank("acctA", "HDFC", Some(d(2026, 9, 1)))],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "bank_statement:acctA")
            .expect("bank habit")
            .clone();
        assert_eq!(h.due_on.as_deref(), Some("2026-11-30"));
        assert_eq!(
            (h.status, h.cadence, h.impact),
            (Status::Ok, Cadence::Quarterly, Impact::High)
        );
        assert_eq!(h.title, "Upload your HDFC statement");
        assert_eq!(
            h.why,
            "Covers up to 1 Sep. One upload every 3 months keeps every number in joy real."
        );

        // Covers to 20 Aug: due 18 Nov, 9 days away.
        let input = HabitsInput {
            banks: vec![bank("acctA", "HDFC", Some(d(2026, 8, 20)))],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "bank_statement:acctA")
            .expect("bank habit")
            .clone();
        assert_eq!(
            (h.due_on.as_deref(), h.status),
            (Some("2026-11-18"), Status::DueSoon)
        );

        // Covers to 1 Jul: due 29 Sep, so overdue.
        let input = HabitsInput {
            banks: vec![bank("acctA", "HDFC", Some(d(2026, 7, 1)))],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "bank_statement:acctA")
            .expect("bank habit")
            .clone();
        assert_eq!(
            (h.due_on.as_deref(), h.status),
            (Some("2026-09-29"), Status::Overdue)
        );
    }

    #[test]
    fn bank_account_never_uploaded_is_once_overdue_and_high() {
        let input = HabitsInput {
            banks: vec![bank("acctB", "Kotak", None)],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "bank_statement:acctB")
            .expect("never-uploaded habit")
            .clone();
        assert_eq!(
            (h.cadence, h.status, h.impact, h.due_on),
            (Cadence::Once, Status::Overdue, Impact::High, None)
        );
        assert!(h.why.starts_with("No statement from this account yet."));
    }

    #[test]
    fn card_statement_is_due_three_days_after_its_statement_day() {
        // Statement day 20 is still ahead of 9 Nov: statement 20 Nov, due 23 Nov, 14 days away.
        let input = HabitsInput {
            cards: vec![card("c1", Some(20))],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "card_statement:c1")
            .expect("card habit")
            .clone();
        assert_eq!(h.due_on.as_deref(), Some("2026-11-23"));
        assert_eq!(
            (h.cadence, h.status, h.impact),
            (Cadence::Monthly, Status::DueSoon, Impact::High)
        );

        // Statement day 3 has passed this month: statement 3 Dec, due 6 Dec.
        let input = HabitsInput {
            cards: vec![card("c1", Some(3))],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "card_statement:c1")
            .expect("card habit")
            .clone();
        assert_eq!(h.due_on.as_deref(), Some("2026-12-06"));
    }

    #[test]
    fn card_without_statement_day_uses_the_fifth_of_next_month() {
        let input = HabitsInput {
            cards: vec![card("c1", None)],
            ..Default::default()
        };
        let h = find(&build(&input, today()), "card_statement:c1")
            .expect("card habit")
            .clone();
        assert_eq!(h.due_on.as_deref(), Some("2026-12-08"));
        assert_eq!(h.status, Status::Ok);
    }

    #[test]
    fn no_cards_and_no_banks_means_no_statement_habits() {
        let habits = build(
            &HabitsInput {
                life: life_fits(),
                ..Default::default()
            },
            today(),
        );
        assert!(habits.iter().all(|h| h.area != Area::Data));
    }

    #[test]
    fn categorise_fires_at_twenty_percent_uncategorised() {
        let mut spend = SpendIn {
            spend_90d_paise: 1_000_000,
            uncategorised_90d_paise: 199_999,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: spend.clone(),
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&input, today()), "categorise").is_none());

        spend.uncategorised_90d_paise = 200_000;
        let input = HabitsInput {
            spend,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "categorise")
            .expect("categorise habit")
            .clone();
        assert_eq!(
            (h.cadence, h.status, h.impact),
            (Cadence::Once, Status::Overdue, Impact::High)
        );
        assert_eq!(h.action.map(|a| a.to), Some("/spending/sort".to_string()));
    }

    #[test]
    fn categorise_states_the_rupee_share() {
        let spend = SpendIn {
            spend_90d_paise: 21_800_000,
            uncategorised_90d_paise: 21_359_600,
            ..Default::default()
        };
        let input = HabitsInput {
            spend,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "categorise")
            .expect("categorise habit")
            .clone();
        assert_eq!(
            h.why,
            "₹2,13,596 of ₹2,18,000 (98%) has no category, so budgets can't be checked."
        );
    }

    #[test]
    fn automate_transfers_needs_six_transfers_out() {
        let few = SpendIn {
            own_transfers_out_count: 5,
            own_transfers_out_paise: 1_800_000,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: few,
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&input, today()), "automate_transfers").is_none());

        let many = SpendIn {
            own_transfers_out_count: 6,
            own_transfers_out_paise: 1_800_000,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: many,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "automate_transfers")
            .expect("automate habit")
            .clone();
        assert_eq!((h.cadence, h.impact), (Cadence::Monthly, Impact::Medium));
        assert_eq!(
            h.why,
            "6 manual transfers to your own accounts in 3 months (avg ₹6,000 a month). One standing instruction on payday does it for you."
        );
    }

    #[test]
    fn salary_buffer_names_the_missing_month_and_the_average_spend() {
        let input = HabitsInput {
            salary: Some(SalaryIn {
                missing_months: vec!["2026-10".into()],
                days: vec![1, 2],
            }),
            spend: SpendIn {
                avg_monthly_spend_paise: 4_500_000,
                ..Default::default()
            },
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "salary_buffer")
            .expect("buffer habit")
            .clone();
        assert_eq!((h.status, h.impact), (Status::Overdue, Impact::High));
        assert_eq!(
            h.why,
            "No salary landed in October 2026 in your statements. Keep ₹45,000 aside so a late payday doesn't stop you."
        );
    }

    #[test]
    fn salary_spread_over_ten_days_raises_the_buffer_and_a_tight_spread_does_not() {
        let spend = SpendIn {
            avg_monthly_spend_paise: 4_500_000,
            ..Default::default()
        };
        let wide = HabitsInput {
            salary: Some(SalaryIn {
                missing_months: vec![],
                days: vec![1, 15],
            }),
            spend: spend.clone(),
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&wide, today()), "salary_buffer")
            .expect("buffer habit")
            .clone();
        assert_eq!(h.status, Status::DueSoon);
        assert!(h.why.contains("day 1 to day 15"));

        let tight = HabitsInput {
            salary: Some(SalaryIn {
                missing_months: vec![],
                days: vec![1, 8],
            }),
            spend,
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&tight, today()), "salary_buffer").is_none());
    }

    #[test]
    fn emergency_fund_shows_cover_target_and_monthly_move() {
        // Avg spend ₹1,000 a month, ₹1,200 liquid: 1.2 months, target ₹6,000, gap ₹4,800 over 12 months.
        let spend = SpendIn {
            avg_monthly_spend_paise: 100_000,
            liquid_paise: 120_000,
            emergency_months_tenths: Some(12),
            ..Default::default()
        };
        let input = HabitsInput {
            spend,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "emergency_fund")
            .expect("emergency habit")
            .clone();
        assert_eq!(
            (h.cadence, h.status, h.impact),
            (Cadence::Monthly, Status::Overdue, Impact::High)
        );
        assert_eq!(
            h.why,
            "Liquid money covers 1.2 months; 6 months is ₹6,000. Moving ₹400 each payday gets there in 12 months."
        );
    }

    #[test]
    fn emergency_fund_is_silent_when_covered_or_without_spend() {
        let covered = SpendIn {
            avg_monthly_spend_paise: 100_000,
            liquid_paise: 600_000,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: covered,
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&input, today()), "emergency_fund").is_none());

        let no_spend = SpendIn {
            avg_monthly_spend_paise: 0,
            liquid_paise: 0,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: no_spend,
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&input, today()), "emergency_fund").is_none());
    }

    #[test]
    fn sip_step_up_adds_ten_percent_each_april() {
        let input = HabitsInput {
            sip_paise: vec![1_500_000, 1_000_000],
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "sip_step_up")
            .expect("sip habit")
            .clone();
        assert_eq!(h.due_on.as_deref(), Some("2027-04-01"));
        assert_eq!(
            (h.cadence, h.status, h.impact),
            (Cadence::Yearly, Status::Ok, Impact::Medium)
        );
        assert_eq!(
            h.why,
            "₹25,000 a month in 2 SIPs. A 10% step-up each April adds ₹2,500 a month."
        );

        let none = HabitsInput {
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&none, today()), "sip_step_up").is_none());
    }

    #[test]
    fn small_payments_fire_from_two_thousand_a_month() {
        let big = SpendIn {
            small_per_month_count: 9,
            small_per_month_paise: 250_000,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: big,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "small_payments")
            .expect("small payments habit")
            .clone();
        assert_eq!((h.cadence, h.impact), (Cadence::Monthly, Impact::Low));
        assert_eq!(
            h.why,
            "9 payments under ₹200 a month come to ₹2,500 a month. Batching them into one weekly spend keeps them visible."
        );

        let small = SpendIn {
            small_per_month_count: 5,
            small_per_month_paise: 150_000,
            ..Default::default()
        };
        let input = HabitsInput {
            spend: small,
            life: life_fits(),
            ..Default::default()
        };
        assert!(find(&build(&input, today()), "small_payments").is_none());
    }

    #[test]
    fn food_budget_gives_a_weekly_figure_and_flags_being_over_pace() {
        // Prior average ₹43,300 a month: weekly ₹10,000. Day 9 of 30 is a pace of ₹12,990.
        // Spend so far ₹16,000 is ₹3,010 over that pace.
        let food = FoodIn {
            any_tagged: true,
            this_month_paise: 1_600_000,
            prior_avg_paise: 4_330_000,
            day: 9,
            days_in_month: 30,
        };
        let input = HabitsInput {
            food,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "food_budget")
            .expect("food habit")
            .clone();
        assert_eq!(h.title, "Plan this week's meals within ₹10,000");
        assert_eq!(
            (h.cadence, h.status, h.area),
            (Cadence::Weekly, Status::DueSoon, Area::Food)
        );
        assert_eq!(
            h.why,
            "Food is ₹16,000 this month, ₹3,010 over your usual pace."
        );

        let on_pace = FoodIn {
            this_month_paise: 1_000_000,
            ..input.food.clone()
        };
        let input = HabitsInput {
            food: on_pace,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "food_budget")
            .expect("food habit")
            .clone();
        assert_eq!(h.status, Status::Ok);
        assert_eq!(h.why, "Food is ₹10,000 this month, within your usual pace.");
    }

    #[test]
    fn untagged_food_asks_for_tags_only_when_there_is_spend() {
        let spend = SpendIn {
            any_spend: true,
            ..Default::default()
        };
        let input = HabitsInput {
            spend,
            life: life_fits(),
            ..Default::default()
        };
        let h = find(&build(&input, today()), "tag_food_payees")
            .expect("tag habit")
            .clone();
        assert_eq!(
            (h.cadence, h.status, h.impact),
            (Cadence::Once, Status::Overdue, Impact::Medium)
        );
        assert!(find(
            &build(
                &HabitsInput {
                    life: life_fits(),
                    ..Default::default()
                },
                today()
            ),
            "tag_food_payees"
        )
        .is_none());
    }

    #[test]
    fn missing_life_plan_is_asked_for_and_fits_hide_the_goal_habits() {
        let missing = build(&HabitsInput::default(), today());
        assert!(find(&missing, "life_plan_missing").is_some());

        let fitting = build(
            &HabitsInput {
                life: life_fits(),
                ..Default::default()
            },
            today(),
        );
        assert!(find(&fitting, "life_plan_missing").is_none());
        assert!(find(&fitting, "goals_shortfall").is_none());
    }

    #[test]
    fn goals_shortfall_and_insurance_gaps_show_their_numbers() {
        let life = LifeIn {
            fits: false,
            shortfall_paise: 5_000_000,
            top_goal: Some("Wedding".into()),
            term_gap_paise: 10_000_000,
            health_gap_paise: 0,
        };
        let habits = build(
            &HabitsInput {
                life: Some(life),
                ..Default::default()
            },
            today(),
        );
        let goals = find(&habits, "goals_shortfall").expect("goals habit");
        assert_eq!(
            (goals.cadence, goals.impact),
            (Cadence::Monthly, Impact::High)
        );
        assert_eq!(
            goals.why,
            "Your goals need ₹50,000 a month more than you save; the wedding is the biggest part."
        );
        let term = find(&habits, "insurance:term").expect("term habit");
        assert_eq!(
            term.why,
            "Your term cover is ₹1,00,000 short of what your family needs."
        );
        assert!(find(&habits, "insurance:health").is_none());
    }

    #[test]
    fn advance_tax_and_return_dates_show_inside_thirty_days() {
        // 15 Dec is 14 days away from 1 Dec.
        let habits = build(
            &HabitsInput {
                life: life_fits(),
                ..Default::default()
            },
            d(2026, 12, 1),
        );
        let tax = find(&habits, "tax:2026-12-15").expect("advance tax habit");
        assert_eq!(
            (tax.status, tax.cadence, tax.impact),
            (Status::DueSoon, Cadence::Once, Impact::Medium)
        );
        assert_eq!(tax.title, "Pay advance tax by 15 Dec");
        assert_eq!(tax.why, "Advance tax instalment due 15 Dec.");

        // 31 Jul is 11 days from 20 Jul.
        let habits = build(
            &HabitsInput {
                life: life_fits(),
                ..Default::default()
            },
            d(2026, 7, 20),
        );
        let itr = find(&habits, "tax:2026-07-31").expect("return habit");
        assert_eq!(itr.title, "File your income tax return");
        assert_eq!(itr.why, "Income tax return due 31 Jul.");

        // 9 Nov: the next date, 15 Dec, is 36 days away.
        let habits = build(
            &HabitsInput {
                life: life_fits(),
                ..Default::default()
            },
            today(),
        );
        assert!(habits.iter().all(|h| !h.key.starts_with("tax:")));
    }

    #[test]
    fn overdue_then_due_soon_then_ok_then_by_impact_then_due_date() {
        let input = HabitsInput {
            banks: vec![
                bank("acctA", "HDFC", Some(d(2026, 8, 20))),
                bank("acctC", "Kotak", Some(d(2026, 9, 1))),
            ],
            cards: vec![card("cardX", Some(20))],
            food: FoodIn {
                any_tagged: true,
                this_month_paise: 1_600_000,
                prior_avg_paise: 4_330_000,
                day: 9,
                days_in_month: 30,
            },
            ..Default::default()
        };
        let keys: Vec<String> = build(&input, today()).into_iter().map(|h| h.key).collect();
        assert_eq!(
            keys,
            vec![
                "life_plan_missing",
                "bank_statement:acctA",
                "card_statement:cardX",
                "food_budget",
                "bank_statement:acctC",
            ]
        );
    }

    #[test]
    fn at_most_twelve_habits_are_returned() {
        let banks = (0..6)
            .map(|i| bank(&format!("b{i}"), &format!("Bank {i}"), None))
            .collect();
        let cards = (0..4).map(|i| card(&format!("c{i}"), Some(20))).collect();
        let spend = SpendIn {
            spend_90d_paise: 1_000_000,
            uncategorised_90d_paise: 900_000,
            own_transfers_out_count: 8,
            own_transfers_out_paise: 2_400_000,
            small_per_month_count: 10,
            small_per_month_paise: 300_000,
            ..Default::default()
        };
        let input = HabitsInput {
            banks,
            cards,
            spend,
            ..Default::default()
        };
        let habits = build(&input, today());
        assert_eq!(habits.len(), MAX_HABITS);
        // The eight overdue habits come first, then the four card statements due soon.
        assert!(habits[..8].iter().all(|h| h.status == Status::Overdue));
        assert!(habits[8..]
            .iter()
            .all(|h| h.area == Area::Data && h.status == Status::DueSoon));
    }

    #[test]
    fn habit_serialises_to_the_contract_shape() {
        let input = HabitsInput {
            food: FoodIn {
                any_tagged: true,
                this_month_paise: 1_600_000,
                prior_avg_paise: 4_330_000,
                day: 9,
                days_in_month: 30,
            },
            ..Default::default()
        };
        let habits = build(&input, today());
        let json = serde_json::to_value(find(&habits, "food_budget").expect("food habit")).unwrap();
        assert_eq!(json["area"], "food");
        assert_eq!(json["status"], "due_soon");
        assert_eq!(json["cadence"], "weekly");
        assert_eq!(json["impact"], "medium");
        assert!(json["action"].is_null());
        assert!(json["due_on"].is_null());
    }

    #[test]
    fn sparse_food_tags_ask_for_tagging_not_a_tiny_budget() {
        let input = HabitsInput {
            food: FoodIn { any_tagged: true, this_month_paise: 0, prior_avg_paise: 4_000, day: 9, days_in_month: 30 },
            spend: SpendIn { any_spend: true, ..Default::default() },
            ..Default::default()
        };
        let habits = build(&input, today());
        assert!(find(&habits, "food_budget").is_none());
        let tag = find(&habits, "tag_food_payees").expect("asks to tag food");
        assert!(tag.why.starts_with("Only ₹40 a month"), "{}", tag.why);
    }

}
