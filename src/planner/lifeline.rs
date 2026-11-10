//! Life line: the household's money, month by month, from today to the plan-to age (90 by
//! default), and what a choice (a job switch, a house, a child, retiring earlier) does to it.
//! Pure: no DB, no clock. `today` comes in as a parameter.
//! Money is integer paise at the edges and f64 paise inside; rates are basis points.
//!
//! What it assumes, and says so in [`Line::notes`]:
//! * every rupee not spent is invested;
//! * both incomes stop when the user retires;
//! * a home keeps its value in today's money;
//! * later-life medical costs and the cost of raising a child are estimates.

use serde::{Deserialize, Serialize};
use time::Date;

use super::life::{
    inr, parse_date, LifeProfile, CHILD_BIRTH_TODAY, DEBT_RETURN_BPS, DEFAULT_WEDDING_BUDGET, EDUCATION_INFLATION_BPS,
    EQUITY_RETURN_BPS, GENERAL_INFLATION_BPS, HYBRID_RETURN_BPS, MEDICAL_INFLATION_BPS, SWR_BPS, WEDDING_INFLATION_BPS,
};
use crate::validate::{non_negative_amount, positive_amount, MAX_AMOUNT_PAISE};

const MONTHS_PER_YEAR: i64 = 12;
/// More than this many months from retirement the money is in equity; closer in, hybrid.
const EQUITY_MIN_MONTHS: i64 = 84;
/// In retirement: half hybrid, half debt.
pub const RETIRED_RETURN_BPS: i64 = (HYBRID_RETURN_BPS + DEBT_RETURN_BPS) / 2;
/// Estimate: out-of-pocket medical costs from `MEDICAL_FROM_AGE`, ₹1,00,000 a year in today's money.
pub const LATER_MEDICAL_TODAY: i64 = 10_000_000;
pub const MEDICAL_FROM_AGE: i64 = 60;
/// Estimate: raising a child, ₹15,000 a month in today's money, from birth to 18.
pub const CHILD_MONTHLY_TODAY: i64 = 1_500_000;
const CHILD_YEARS: i64 = 18;
/// Estimate: how much staying active trims later-life medical costs when no figure is given.
pub const DEFAULT_MEDICAL_CUT_BPS: i64 = 3_000;
const MIN_AGE: i64 = 18;
const MAX_AGE: i64 = 110;
pub const MAX_CHOICES: usize = 20;

/// One thing the user could do. Amounts are in today's money unless a field says otherwise.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Choice {
    /// A new job from `month` ("YYYY-MM"): `gap_months` without pay, then the new take-home
    /// (the amount on the offer, per month), rising by `raise_bps` a year.
    JobSwitch {
        month: String,
        take_home_paise: i64,
        #[serde(default)]
        gap_months: i64,
        #[serde(default)]
        raise_bps: Option<i64>,
    },
    /// A home bought in `year`: down payment from savings, an EMI for the tenure, rent saved.
    House {
        year: i64,
        price_paise: i64,
        down_bps: i64,
        loan_bps: i64,
        tenure_years: i64,
        #[serde(default)]
        rent_saved_paise: i64,
    },
    /// A child born in `birth_year`. `another` adds one; otherwise it moves the planned child.
    Child {
        birth_year: i64,
        #[serde(default)]
        another: bool,
    },
    RetireAt {
        age: i64,
    },
    /// A one-off cost in `year`, repeated every `every_years` if set (a car every 8 years).
    BigSpend {
        #[serde(default)]
        name: String,
        year: i64,
        amount_paise: i64,
        #[serde(default)]
        every_years: Option<i64>,
    },
    /// Months without the user's pay from `start` ("YYYY-MM"): a sabbatical or leave.
    Break {
        start: String,
        months: i64,
    },
    /// Spend this much less each month while working; it is invested instead.
    SaveMore {
        monthly_paise: i64,
    },
    /// Estimate: staying active trims later-life medical costs by `medical_cut_bps`.
    StayActive {
        #[serde(default)]
        medical_cut_bps: Option<i64>,
    },
}

/// Everything the line is drawn from.
#[derive(Clone, Debug, PartialEq)]
pub struct LineInput {
    pub profile: LifeProfile,
    /// Monthly household spend today.
    pub expenses_paise: i64,
    /// What is owned today, less loans. May be negative.
    pub wealth_paise: i64,
    pub choices: Vec<Choice>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Row {
    pub year: i64,
    pub age: i64,
    pub income_paise: i64,
    pub spend_paise: i64,
    /// What the year needed and savings could not pay.
    pub short_paise: i64,
    pub net_worth_paise: i64,
    /// The same net worth in today's money.
    pub net_worth_today_paise: i64,
    pub events: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Default)]
pub struct Marks {
    /// First age at which savings alone would pay the bills for good.
    pub free_at_age: Option<i64>,
    /// First age in a year the money did not cover. `None`: it lasts to `end_age`.
    pub runs_out_at_age: Option<i64>,
    pub retire_age: i64,
    pub end_age: i64,
    /// Net worth at 60 in today's money, when the line passes 60.
    pub at_60_today_paise: Option<i64>,
    pub at_end_today_paise: i64,
    /// Planned costs that savings could not fully pay, e.g. "Wedding in 2028".
    pub unfunded: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Line {
    pub rows: Vec<Row>,
    pub marks: Marks,
    pub notes: Vec<String>,
}

/// What a set of choices changes, in plain sentences, with the two figures behind them.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Consequences {
    pub lines: Vec<String>,
    pub at_60_change_paise: Option<i64>,
    pub at_end_change_paise: i64,
    /// True when the choices leave the user better off at the end and money lasts as long.
    pub better: bool,
}

fn rate(bps: i64) -> f64 {
    bps as f64 / 10_000.0
}

/// Rounds to paise inside `±MAX_AMOUNT_PAISE`. NaN becomes 0.
fn to_paise(x: f64) -> i64 {
    if x.is_nan() {
        0
    } else {
        x.round().clamp(-(MAX_AMOUNT_PAISE as f64), MAX_AMOUNT_PAISE as f64) as i64
    }
}

/// Months from `today`'s month to a "YYYY-MM" (or "YYYY-MM-DD") month. Past months give 0.
fn month_index(s: &str, today: Date) -> Option<i64> {
    let s = s.trim();
    let date = parse_date(s).or_else(|| parse_date(&format!("{s}-01")))?;
    Some(months_from(today, date.year() as i64, date.month() as i64))
}

fn months_from(today: Date, year: i64, month: i64) -> i64 {
    ((year - today.year() as i64) * MONTHS_PER_YEAR + (month - today.month() as i64)).max(0)
}

/// Checks one choice. The text is shown to the user.
pub fn validate(choice: &Choice, today: Date) -> Result<(), String> {
    let ty = today.year() as i64;
    let year_ok = |y: i64| (ty..=ty + 80).contains(&y);
    let bps_ok = |b: i64, hi: i64| (0..=hi).contains(&b);
    match choice {
        Choice::JobSwitch { month, take_home_paise, gap_months, raise_bps } => {
            month_index(month, today).ok_or("The month must be YYYY-MM.")?;
            non_negative_amount(*take_home_paise)?;
            if !(0..=36).contains(gap_months) {
                return Err("The gap must be between 0 and 36 months.".into());
            }
            if raise_bps.is_some_and(|b| !bps_ok(b, 3_000)) {
                return Err("The yearly raise must be between 0% and 30%.".into());
            }
        }
        Choice::House { year, price_paise, down_bps, loan_bps, tenure_years, rent_saved_paise } => {
            if !year_ok(*year) {
                return Err("The year to buy must be this year or later.".into());
            }
            positive_amount(*price_paise)?;
            non_negative_amount(*rent_saved_paise)?;
            if !bps_ok(*down_bps, 10_000) {
                return Err("The down payment must be between 0% and 100%.".into());
            }
            if !bps_ok(*loan_bps, 3_000) {
                return Err("The loan rate must be between 0% and 30%.".into());
            }
            if !(1..=40).contains(tenure_years) {
                return Err("The loan must run between 1 and 40 years.".into());
            }
        }
        Choice::Child { birth_year, .. } => {
            if !(ty - 17..=ty + 25).contains(birth_year) {
                return Err(format!("The birth year must be between {} and {}.", ty - 17, ty + 25));
            }
        }
        Choice::RetireAt { age } => {
            if !(30..=80).contains(age) {
                return Err("The retirement age must be between 30 and 80.".into());
            }
        }
        Choice::BigSpend { name, year, amount_paise, every_years } => {
            if !year_ok(*year) {
                return Err("The year must be this year or later.".into());
            }
            positive_amount(*amount_paise)?;
            if every_years.is_some_and(|n| !(1..=50).contains(&n)) {
                return Err("Repeat every 1 to 50 years.".into());
            }
            if name.chars().count() > 40 {
                return Err("The name is too long.".into());
            }
        }
        Choice::Break { start, months } => {
            month_index(start, today).ok_or("The start must be YYYY-MM.")?;
            if !(1..=60).contains(months) {
                return Err("A break must last between 1 and 60 months.".into());
            }
        }
        Choice::SaveMore { monthly_paise } => {
            positive_amount(*monthly_paise)?;
        }
        Choice::StayActive { medical_cut_bps } => {
            if medical_cut_bps.is_some_and(|b| !bps_ok(b, 9_000)) {
                return Err("The saving on medical costs must be between 0% and 90%.".into());
            }
        }
    }
    Ok(())
}

/// A one-off cost in a month, in that month's money.
struct Lump {
    month: i64,
    amount: f64,
    label: String,
}

struct Job {
    from: i64,
    gap: i64,
    pay: f64,
    raise: f64,
}

struct Home {
    from: i64,
    price: f64,
    loan: f64,
    emi: f64,
    monthly_rate: f64,
    months: i64,
    rent_saved: f64,
}

/// The fixed monthly payment that clears `loan` in `months` at `monthly_rate`.
fn emi(loan: f64, monthly_rate: f64, months: i64) -> f64 {
    if loan <= 0.0 || months <= 0 {
        return 0.0;
    }
    if monthly_rate == 0.0 {
        return loan / months as f64;
    }
    let growth = (1.0 + monthly_rate).powi(months as i32);
    loan * monthly_rate * growth / (growth - 1.0)
}

/// Runs the line for one input.
pub fn simulate(input: &LineInput, today: Date) -> Line {
    let p = &input.profile;
    let ty = today.year() as i64;
    let age0 = p.my_age.clamp(MIN_AGE, 100);
    let mut retire_age = p.retirement_age.clamp(age0, 100);
    let end_age = p.live_to_age.clamp(age0, MAX_AGE);
    // Rows are calendar years, so the first one is what is left of this year.
    let offset = today.month() as i64 - 1;
    let total_months = (end_age - age0 + 1) * MONTHS_PER_YEAR - offset;
    let inflation = rate(GENERAL_INFLATION_BPS);
    let grow = |today_cost: f64, bps: i64, month: i64| today_cost * (1.0 + rate(bps)).powf(month as f64 / 12.0);

    let mut children: Vec<i64> = p.child_birth_year.into_iter().collect();
    let mut jobs: Vec<Job> = Vec::new();
    let mut breaks: Vec<(i64, i64)> = Vec::new();
    let mut homes: Vec<Home> = Vec::new();
    let mut lumps: Vec<Lump> = Vec::new();
    let mut save_more = 0.0;
    let mut medical_keep = 1.0;
    let my_raise = rate(p.raise_bps.clamp(0, 3_000));
    let partner_raise = rate(p.partner_raise_bps.clamp(0, 3_000));

    for choice in input.choices.iter().take(MAX_CHOICES) {
        match choice {
            Choice::JobSwitch { month, take_home_paise, gap_months, raise_bps } => {
                let Some(from) = month_index(month, today) else { continue };
                jobs.push(Job {
                    from,
                    gap: (*gap_months).clamp(0, 36),
                    pay: (*take_home_paise).clamp(0, MAX_AMOUNT_PAISE) as f64,
                    raise: raise_bps.map_or(my_raise, |b| rate(b.clamp(0, 3_000))),
                });
            }
            Choice::House { year, price_paise, down_bps, loan_bps, tenure_years, rent_saved_paise } => {
                let from = months_from(today, (*year).clamp(ty, ty + 80), 1);
                let price = grow((*price_paise).clamp(0, MAX_AMOUNT_PAISE) as f64, GENERAL_INFLATION_BPS, from);
                let down = price * rate((*down_bps).clamp(0, 10_000));
                let months = (*tenure_years).clamp(1, 40) * MONTHS_PER_YEAR;
                let monthly_rate = rate((*loan_bps).clamp(0, 3_000)) / 12.0;
                let loan = price - down;
                lumps.push(Lump { month: from, amount: down, label: "Home down payment".into() });
                homes.push(Home {
                    from,
                    price,
                    loan,
                    emi: emi(loan, monthly_rate, months),
                    monthly_rate,
                    months,
                    rent_saved: (*rent_saved_paise).clamp(0, MAX_AMOUNT_PAISE) as f64,
                });
            }
            Choice::Child { birth_year, another } => {
                let year = (*birth_year).clamp(ty - 17, ty + 25);
                if *another || children.is_empty() {
                    children.push(year);
                } else {
                    children[0] = year;
                }
            }
            Choice::RetireAt { age } => retire_age = (*age).clamp(age0, 100),
            Choice::BigSpend { name, year, amount_paise, every_years } => {
                let label = if name.trim().is_empty() { "Big spend".to_string() } else { name.trim().to_string() };
                let step = every_years.map(|n| n.clamp(1, 50) * MONTHS_PER_YEAR);
                let mut month = months_from(today, (*year).clamp(ty, ty + 80), 1);
                while month < total_months {
                    let amount = grow((*amount_paise).clamp(0, MAX_AMOUNT_PAISE) as f64, GENERAL_INFLATION_BPS, month);
                    lumps.push(Lump { month, amount, label: label.clone() });
                    match step {
                        Some(s) => month += s,
                        None => break,
                    }
                }
            }
            Choice::Break { start, months } => {
                let Some(from) = month_index(start, today) else { continue };
                breaks.push((from, from + (*months).clamp(1, 60)));
            }
            Choice::SaveMore { monthly_paise } => save_more += (*monthly_paise).clamp(0, MAX_AMOUNT_PAISE) as f64,
            Choice::StayActive { medical_cut_bps } => {
                medical_keep = 1.0 - rate(medical_cut_bps.unwrap_or(DEFAULT_MEDICAL_CUT_BPS).clamp(0, 9_000));
            }
        }
    }
    jobs.sort_by_key(|j| j.from);

    if let Some(when) = p.wedding_date.as_deref().and_then(parse_date) {
        let month = months_from(today, when.year() as i64, when.month() as i64);
        let budget = p.wedding_budget_paise.unwrap_or(DEFAULT_WEDDING_BUDGET).clamp(0, MAX_AMOUNT_PAISE) as f64;
        lumps.push(Lump { month, amount: grow(budget, WEDDING_INFLATION_BPS, month), label: "Wedding".into() });
    }
    // (first month, last month) of the monthly cost of each child still under 18.
    let mut child_spans: Vec<(i64, i64)> = Vec::new();
    for (n, birth_year) in children.iter().enumerate() {
        let name = if n == 0 { "Child".to_string() } else { format!("Child {}", n + 1) };
        if *birth_year >= ty {
            let month = months_from(today, *birth_year, 1);
            let amount = grow(CHILD_BIRTH_TODAY as f64, MEDICAL_INFLATION_BPS, month);
            lumps.push(Lump { month, amount, label: format!("{name} is born") });
        }
        if birth_year + CHILD_YEARS >= ty {
            let month = months_from(today, birth_year + CHILD_YEARS, 6);
            let cost = p.child_education_today_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
            lumps.push(Lump {
                month,
                amount: grow(cost, EDUCATION_INFLATION_BPS, month),
                label: format!("{name}'s education"),
            });
            child_spans.push((months_from(today, *birth_year, 1), month));
        }
    }

    // Retirement starts in January of the year the user turns `retire_age`.
    let retire_month = ((retire_age - age0) * MONTHS_PER_YEAR - offset).max(0);
    let my_base = p.my_take_home_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
    let partner_base = p.partner_take_home_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
    let expenses = input.expenses_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
    let swr = rate(SWR_BPS);

    let mut wealth = input.wealth_paise.clamp(-MAX_AMOUNT_PAISE, MAX_AMOUNT_PAISE) as f64;
    // Starting in debt is allowed; going further below it is "short".
    let floor = wealth.min(0.0);
    let mut loans: Vec<f64> = homes.iter().map(|h| h.loan).collect();
    let mut rows: Vec<Row> = Vec::new();
    let mut marks = Marks { retire_age, end_age, ..Default::default() };
    let (mut income_year, mut spend_year, mut short_year) = (0.0, 0.0, 0.0);
    let mut events: Vec<String> = Vec::new();

    for m in 0..total_months {
        let year_index = (offset + m) / MONTHS_PER_YEAR;
        let age = age0 + year_index;
        let working = m < retire_month;
        let years_in = year_index as i32;

        // Pay: the last job that has started wins; a gap or a break pays nothing.
        let mut mine = my_base * (1.0 + my_raise).powi(years_in);
        if let Some(job) = jobs.iter().rev().find(|j| j.from <= m) {
            let since = m - job.from - job.gap;
            mine = if since < 0 { 0.0 } else { job.pay * (1.0 + job.raise).powi((since / MONTHS_PER_YEAR) as i32) };
        }
        if breaks.iter().any(|(from, to)| (*from..*to).contains(&m)) {
            mine = 0.0;
        }
        let income = if working { mine + partner_base * (1.0 + partner_raise).powi(years_in) } else { 0.0 };

        let mut spend = grow(expenses, GENERAL_INFLATION_BPS, m);
        if working {
            spend = (spend - grow(save_more, GENERAL_INFLATION_BPS, m)).max(0.0);
        }
        if age >= MEDICAL_FROM_AGE {
            spend += grow(LATER_MEDICAL_TODAY as f64 / 12.0, MEDICAL_INFLATION_BPS, m) * medical_keep;
        }
        for (from, to) in &child_spans {
            if (*from..*to).contains(&m) {
                spend += grow(CHILD_MONTHLY_TODAY as f64, GENERAL_INFLATION_BPS, m);
            }
        }
        for (home, loan) in homes.iter().zip(loans.iter_mut()) {
            if m < home.from {
                continue;
            }
            spend = (spend - grow(home.rent_saved, GENERAL_INFLATION_BPS, m)).max(0.0);
            if m < home.from + home.months && *loan > 0.0 {
                let interest = *loan * home.monthly_rate;
                *loan = (*loan + interest - home.emi).max(0.0);
                spend += home.emi;
            }
            if m == home.from {
                events.push("Buy a home".into());
            }
        }
        let mut lump_labels: Vec<&str> = Vec::new();
        for lump in lumps.iter().filter(|l| l.month == m) {
            spend += lump.amount;
            lump_labels.push(&lump.label);
            if !events.iter().any(|e| e == &lump.label) && lump.label != "Home down payment" {
                events.push(lump.label.clone());
            }
        }
        if m == retire_month {
            events.push(format!("Retire at {retire_age}"));
        }
        if let Some(job) = jobs.iter().find(|j| j.from == m) {
            events.push(if job.pay > 0.0 { "New job".into() } else { "Leave job".into() });
        }
        if breaks.iter().any(|(from, _)| *from == m) {
            events.push("Break from work".into());
        }

        // Grow what is there, then add the month's pay and take its costs.
        let months_to_retire = retire_month - m;
        let return_bps = if months_to_retire > EQUITY_MIN_MONTHS {
            EQUITY_RETURN_BPS
        } else if months_to_retire > 0 {
            HYBRID_RETURN_BPS
        } else {
            RETIRED_RETURN_BPS
        };
        if wealth > 0.0 {
            wealth *= 1.0 + rate(return_bps) / 12.0;
        }
        wealth += income - spend;
        let mut short = 0.0;
        if wealth < floor {
            // Savings can't go below where they started: what they could not pay is short.
            short = floor - wealth;
            wealth = floor;
            marks.runs_out_at_age.get_or_insert(age);
            let year = ty + year_index;
            for label in lump_labels {
                let text = format!("{label} in {year}");
                if !marks.unfunded.contains(&text) {
                    marks.unfunded.push(text);
                }
            }
        }
        income_year += income;
        spend_year += spend;
        short_year += short;

        if (offset + m) % MONTHS_PER_YEAR == MONTHS_PER_YEAR - 1 {
            let i = year_index;
            let home_value: f64 = homes.iter().filter(|h| h.from <= m).map(|h| grow(h.price, GENERAL_INFLATION_BPS, m - h.from)).sum();
            let owed: f64 = homes.iter().zip(&loans).filter(|(h, _)| h.from <= m).map(|(_, l)| *l).sum();
            let net = wealth + home_value - owed;
            let deflate = (1.0 + inflation).powf((m + 1) as f64 / 12.0);
            let net_today = to_paise(net / deflate);
            // Free: savings alone would pay a year of today's costs for good, and nothing is short.
            let yearly_need = spend_regular(expenses, m, age, medical_keep, &grow) * 12.0;
            if marks.free_at_age.is_none() && short_year == 0.0 && yearly_need > 0.0 && wealth * swr >= yearly_need {
                marks.free_at_age = Some(age);
            }
            if age == 60 {
                marks.at_60_today_paise = Some(net_today);
            }
            marks.at_end_today_paise = net_today;
            rows.push(Row {
                year: ty + i,
                age,
                income_paise: to_paise(income_year),
                spend_paise: to_paise(spend_year),
                short_paise: to_paise(short_year),
                net_worth_paise: to_paise(net),
                net_worth_today_paise: net_today,
                events: std::mem::take(&mut events),
            });
            (income_year, spend_year, short_year) = (0.0, 0.0, 0.0);
        }
    }

    let mut notes = vec![
        "Every rupee not spent is invested: in equity until 7 years before retirement, then hybrid, then half hybrid and half debt.".to_string(),
        "Both incomes stop when you retire.".to_string(),
        format!(
            "Estimate: medical costs of {} a year in today's money from age {MEDICAL_FROM_AGE}, rising with medical inflation.",
            inr(LATER_MEDICAL_TODAY)
        ),
    ];
    if !children.is_empty() {
        notes.push(format!(
            "Estimate: raising a child costs {} a month in today's money until 18, plus education.",
            inr(CHILD_MONTHLY_TODAY)
        ));
    }
    if !homes.is_empty() {
        notes.push("A home is counted in net worth and keeps its value in today's money.".to_string());
    }
    if medical_keep < 1.0 {
        notes.push("Estimate: staying active trims later-life medical costs. The size of the saving is a rule of thumb, not a promise.".to_string());
    }
    Line { rows, marks, notes }
}

/// The monthly living cost at month `m`, without one-offs, loans or children: what retirement must pay.
fn spend_regular(expenses: f64, m: i64, age: i64, medical_keep: f64, grow: &impl Fn(f64, i64, i64) -> f64) -> f64 {
    let mut spend = grow(expenses, GENERAL_INFLATION_BPS, m);
    if age >= MEDICAL_FROM_AGE {
        spend += grow(LATER_MEDICAL_TODAY as f64 / 12.0, MEDICAL_INFLATION_BPS, m) * medical_keep;
    }
    spend
}

/// The life line's marks as a check-in keeps them, with one sentence on how they moved.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LifelineNote {
    pub free_at_age: Option<i64>,
    pub runs_out_at_age: Option<i64>,
    pub end_age: i64,
    pub line: String,
}

/// Today's marks set against the note kept with the last check-in, if there was one.
pub fn note(now: &Marks, before: Option<&LifelineNote>) -> LifelineNote {
    let free = |m: Option<i64>, end: i64| m.map_or(format!("not before {end}"), |a| format!("at {a}"));
    let today = match (now.free_at_age, now.runs_out_at_age) {
        (_, Some(out)) => format!("On today's numbers, money runs short at {out}."),
        (Some(at), None) => format!("On today's numbers, work becomes optional at {at} and money lasts past {}.", now.end_age),
        (None, None) => format!("On today's numbers, money lasts past {}, but work never becomes optional.", now.end_age),
    };
    let moved = before.and_then(|b| {
        if b.runs_out_at_age != now.runs_out_at_age {
            Some(match (b.runs_out_at_age, now.runs_out_at_age) {
                (Some(x), None) => format!("Last check-in it ran short at {x}."),
                (None, Some(_)) => format!("Last check-in it lasted past {}.", b.end_age),
                (Some(x), Some(_)) => format!("Last check-in that was {x}."),
                (None, None) => return None,
            })
        } else if b.free_at_age != now.free_at_age && now.runs_out_at_age.is_none() {
            Some(format!("Last check-in work became optional {}.", free(b.free_at_age, b.end_age)))
        } else {
            None
        }
    });
    LifelineNote {
        free_at_age: now.free_at_age,
        runs_out_at_age: now.runs_out_at_age,
        end_age: now.end_age,
        line: match moved {
            Some(m) => format!("{today} {m}"),
            None => today,
        },
    }
}

/// A long-run amount in words: "₹5.7 crore", "₹57 lakh". To the rupee would be false precision.
fn rough(paise: i64) -> String {
    let rupees = paise.abs() as f64 / 100.0;
    let one = |x: f64| {
        let t = format!("{x:.1}");
        t.strip_suffix(".0").map(str::to_string).unwrap_or(t)
    };
    if rupees >= 1e7 {
        format!("₹{} crore", one(rupees / 1e7))
    } else if rupees >= 1e5 {
        format!("₹{} lakh", one(rupees / 1e5))
    } else {
        inr(paise.abs())
    }
}

fn years(n: i64) -> String {
    if n == 1 {
        "1 year".into()
    } else {
        format!("{n} years")
    }
}

/// What `with` changes against `base`, in plain sentences.
pub fn compare(base: &Line, with: &Line) -> Consequences {
    let (b, w) = (&base.marks, &with.marks);
    let mut lines = Vec::new();

    match (b.free_at_age, w.free_at_age) {
        (Some(x), Some(y)) if y < x => lines.push(format!("Work becomes optional {} earlier, at {y}.", years(x - y))),
        (Some(x), Some(y)) if y > x => lines.push(format!("Work becomes optional {} later, at {y}.", years(y - x))),
        (Some(x), None) => lines.push(format!("Work no longer becomes optional (it did at {x}).")),
        (None, Some(y)) => lines.push(format!("Work becomes optional at {y}. It never did before.")),
        _ => {}
    }
    let end = w.end_age;
    match (b.runs_out_at_age, w.runs_out_at_age) {
        (None, Some(y)) => lines.push(format!("Money runs short at {y}. It lasted to {end} before.")),
        (Some(x), None) => lines.push(format!("Money now lasts to {end}. It ran short at {x} before.")),
        (Some(x), Some(y)) if y > x => lines.push(format!("Money lasts {} longer, to {y}.", years(y - x))),
        (Some(x), Some(y)) if y < x => lines.push(format!("Money runs short {} sooner, at {y}.", years(x - y))),
        _ => {}
    }
    let change = |delta: i64, at: &str| {
        let word = if delta > 0 { "more" } else { "less" };
        format!("{} {word} at {at}, in today's money.", rough(delta))
    };
    let at_60 = match (b.at_60_today_paise, w.at_60_today_paise) {
        (Some(x), Some(y)) => Some(y.saturating_sub(x)),
        _ => None,
    };
    // Under ₹1,000 either way is rounding, not a consequence.
    const NOTICEABLE: i64 = 100_000;
    if let Some(d) = at_60.filter(|d| d.abs() >= NOTICEABLE) {
        lines.push(change(d, "60"));
    }
    let at_end = w.at_end_today_paise.saturating_sub(b.at_end_today_paise);
    if at_end.abs() >= NOTICEABLE {
        lines.push(change(at_end, &end.to_string()));
    }
    for item in w.unfunded.iter().filter(|u| !b.unfunded.contains(u)) {
        lines.push(format!("{item} can no longer be paid in full from savings."));
    }
    for item in b.unfunded.iter().filter(|u| !w.unfunded.contains(u)) {
        lines.push(format!("{item} can now be paid in full."));
    }
    if lines.is_empty() {
        lines.push("No change you would notice.".to_string());
    }
    let lasts = |m: &Marks| m.runs_out_at_age.unwrap_or(i64::MAX);
    Consequences {
        lines,
        at_60_change_paise: at_60,
        at_end_change_paise: at_end,
        better: lasts(w) >= lasts(b) && at_end >= 0 && (lasts(w) > lasts(b) || at_end > 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn today() -> Date {
        date!(2026 - 11 - 10)
    }

    /// 29, ₹1.5L take-home, ₹70k spend, ₹10L saved, retire at 60, plan to 90.
    fn input(choices: Vec<Choice>) -> LineInput {
        LineInput {
            profile: LifeProfile { my_age: 29, my_take_home_paise: 15_000_000, ..Default::default() },
            expenses_paise: 7_000_000,
            wealth_paise: 100_000_000,
            choices,
        }
    }

    fn line(choices: Vec<Choice>) -> Line {
        simulate(&input(choices), today())
    }

    #[test]
    fn the_line_runs_from_today_to_ninety() {
        let l = line(vec![]);
        assert_eq!(l.rows.len(), 62);
        assert_eq!((l.rows[0].age, l.rows[0].year), (29, 2026));
        assert_eq!(l.rows.last().unwrap().age, 90);
        assert_eq!(l.marks.end_age, 90);
        assert!(l.marks.at_60_today_paise.is_some());
        assert!(l.rows.iter().any(|r| r.events.iter().any(|e| e == "Retire at 60")));
    }

    #[test]
    fn no_choices_is_the_baseline_and_changes_nothing() {
        let c = compare(&line(vec![]), &line(vec![]));
        assert_eq!(c.lines, vec!["No change you would notice."]);
        assert_eq!(c.at_end_change_paise, 0);
        assert!(!c.better);
    }

    #[test]
    fn income_stops_at_retirement_and_savings_never_go_below_zero() {
        let l = line(vec![Choice::RetireAt { age: 40 }]);
        assert!(l.rows.iter().filter(|r| r.age >= 40).all(|r| r.income_paise == 0));
        assert!(l.rows.iter().all(|r| r.net_worth_paise >= 0));
        // Retiring at 40 on this income can't last: the mark and the short years must say so.
        let out = l.marks.runs_out_at_age.expect("runs out");
        assert!(out > 40 && out < 90, "ran out at {out}");
        assert!(l.rows.iter().filter(|r| r.age >= out).all(|r| r.short_paise > 0));
        assert!(l.rows.iter().filter(|r| r.age < out).all(|r| r.short_paise == 0));
    }

    #[test]
    fn a_better_paid_job_never_makes_work_optional_later() {
        let base = line(vec![]);
        let switch = line(vec![Choice::JobSwitch {
            month: "2027-04".into(),
            take_home_paise: 22_000_000,
            gap_months: 1,
            raise_bps: None,
        }]);
        let (b, s) = (base.marks.free_at_age.expect("free"), switch.marks.free_at_age.expect("free"));
        assert!(s <= b, "switch {s} vs base {b}");
        assert!(switch.marks.at_end_today_paise > base.marks.at_end_today_paise);
        let c = compare(&base, &switch);
        assert!(c.better);
        assert!(c.lines.iter().any(|l| l.contains("more at 90")), "{:?}", c.lines);
        assert!(switch.rows.iter().any(|r| r.events.iter().any(|e| e == "New job")));
    }

    #[test]
    fn a_gap_and_a_break_pay_nothing() {
        let gap = line(vec![Choice::JobSwitch { month: "2027-01".into(), take_home_paise: 15_000_000, gap_months: 12, raise_bps: None }]);
        assert_eq!(gap.rows[0].income_paise, 15_000_000 * 2, "November and December of this year");
        assert_eq!(gap.rows[1].income_paise, 0);
        assert!(gap.rows[2].income_paise > 0);
        let rest = line(vec![Choice::Break { start: "2027-01".into(), months: 12 }]);
        assert_eq!(rest.rows[1].income_paise, 0);
        assert!(rest.rows[1].events.iter().any(|e| e == "Break from work"));
        assert!(!compare(&line(vec![]), &rest).better);
    }

    #[test]
    fn a_home_dips_savings_but_is_counted_in_net_worth() {
        let home = Choice::House {
            year: 2030,
            price_paise: 800_000_000,
            down_bps: 2_000,
            loan_bps: 850,
            tenure_years: 20,
            rent_saved_paise: 2_500_000,
        };
        let (base, with) = (line(vec![]), line(vec![home]));
        let at = |l: &Line, year: i64| l.rows.iter().find(|r| r.year == year).unwrap().clone();
        assert!(at(&with, 2030).spend_paise > at(&base, 2030).spend_paise);
        assert!(at(&with, 2030).events.iter().any(|e| e == "Buy a home"));
        // The home is an asset: net worth does not fall by the whole down payment.
        let fall = at(&base, 2030).net_worth_paise - at(&with, 2030).net_worth_paise;
        assert!(fall < 160_000_000, "fell {fall}");
        // After the loan ends, the EMI stops.
        assert!(at(&with, 2052).spend_paise < at(&with, 2049).spend_paise);
    }

    #[test]
    fn a_child_costs_money_and_moving_the_year_moves_the_events() {
        let base = line(vec![]);
        let child = line(vec![Choice::Child { birth_year: 2029, another: false }]);
        assert!(child.marks.at_end_today_paise < base.marks.at_end_today_paise);
        let year_of = |l: &Line, text: &str| l.rows.iter().find(|r| r.events.iter().any(|e| e == text)).map(|r| r.year);
        assert_eq!(year_of(&child, "Child is born"), Some(2029));
        assert_eq!(year_of(&child, "Child's education"), Some(2047));
        assert!(child.notes.iter().any(|n| n.contains("raising a child")));

        let mut planned = input(vec![Choice::Child { birth_year: 2031, another: false }]);
        planned.profile.child_birth_year = Some(2029);
        let moved = simulate(&planned, today());
        assert_eq!(year_of(&moved, "Child is born"), Some(2031));
        planned.choices = vec![Choice::Child { birth_year: 2031, another: true }];
        let two = simulate(&planned, today());
        assert_eq!(year_of(&two, "Child 2 is born"), Some(2031));
        assert_eq!(year_of(&two, "Child is born"), Some(2029));
    }

    #[test]
    fn a_wedding_savings_cannot_pay_is_listed_as_unfunded() {
        let mut i = input(vec![]);
        i.wealth_paise = 0;
        i.profile.wedding_date = Some("2027-02-01".into());
        i.profile.wedding_budget_paise = Some(250_000_000);
        let l = simulate(&i, today());
        assert_eq!(l.marks.unfunded, vec!["Wedding in 2027"]);
        assert_eq!(l.marks.runs_out_at_age, Some(30));
        let ok = simulate(&LineInput { wealth_paise: 400_000_000, ..i }, today());
        assert!(ok.marks.unfunded.is_empty());
        let c = compare(&l, &ok);
        assert!(c.lines.iter().any(|x| x == "Wedding in 2027 can now be paid in full."), "{:?}", c.lines);
    }

    #[test]
    fn saving_more_and_staying_active_both_help() {
        let base = line(vec![]);
        let saver = line(vec![Choice::SaveMore { monthly_paise: 1_000_000 }]);
        assert!(saver.marks.free_at_age.unwrap() <= base.marks.free_at_age.unwrap());
        assert!(saver.marks.at_60_today_paise > base.marks.at_60_today_paise);
        let active = line(vec![Choice::StayActive { medical_cut_bps: None }]);
        let at_59 = |l: &Line| l.rows.iter().find(|r| r.age == 59).unwrap().net_worth_paise;
        assert_eq!(at_59(&active), at_59(&base), "medical costs start at 60");
        assert!(active.marks.at_end_today_paise > base.marks.at_end_today_paise);
        assert!(active.notes.iter().any(|n| n.contains("rule of thumb")));
    }

    #[test]
    fn a_repeating_spend_lands_every_n_years() {
        let l = line(vec![Choice::BigSpend { name: "Car".into(), year: 2030, amount_paise: 120_000_000, every_years: Some(10) }]);
        let years: Vec<i64> = l.rows.iter().filter(|r| r.events.iter().any(|e| e == "Car")).map(|r| r.year).collect();
        assert_eq!(years, vec![2030, 2040, 2050, 2060, 2070, 2080]);
    }

    #[test]
    fn wild_inputs_are_clamped_and_do_not_panic() {
        let mut i = input(vec![
            Choice::JobSwitch { month: "9999-12".into(), take_home_paise: i64::MAX, gap_months: i64::MAX, raise_bps: Some(i64::MAX) },
            Choice::House { year: i64::MAX, price_paise: i64::MAX, down_bps: -5, loan_bps: i64::MAX, tenure_years: i64::MAX, rent_saved_paise: i64::MAX },
            Choice::Child { birth_year: i64::MIN, another: true },
            Choice::RetireAt { age: i64::MAX },
            Choice::BigSpend { name: String::new(), year: i64::MIN, amount_paise: i64::MAX, every_years: Some(0) },
            Choice::Break { start: "nonsense".into(), months: i64::MAX },
            Choice::SaveMore { monthly_paise: i64::MAX },
            Choice::StayActive { medical_cut_bps: Some(i64::MAX) },
        ]);
        i.profile.my_age = i64::MIN;
        i.profile.live_to_age = i64::MAX;
        i.profile.raise_bps = i64::MAX;
        i.wealth_paise = i64::MIN;
        i.expenses_paise = i64::MAX;
        let l = simulate(&i, today());
        assert_eq!(l.rows.len(), (MAX_AGE - MIN_AGE + 1) as usize);
        assert!(l.rows.iter().all(|r| r.net_worth_paise.abs() <= MAX_AMOUNT_PAISE && r.spend_paise <= MAX_AMOUNT_PAISE));
    }

    #[test]
    fn validation_names_what_is_wrong() {
        let t = today();
        assert!(validate(&Choice::RetireAt { age: 55 }, t).is_ok());
        assert!(validate(&Choice::RetireAt { age: 12 }, t).is_err());
        assert!(validate(&Choice::JobSwitch { month: "2027-04".into(), take_home_paise: 1, gap_months: 0, raise_bps: None }, t).is_ok());
        assert!(validate(&Choice::JobSwitch { month: "April".into(), take_home_paise: 1, gap_months: 0, raise_bps: None }, t).is_err());
        assert!(validate(&Choice::BigSpend { name: "x".into(), year: 2020, amount_paise: 5, every_years: None }, t).is_err());
        assert!(validate(&Choice::SaveMore { monthly_paise: 0 }, t).is_err());
        assert!(validate(&Choice::House { year: 2030, price_paise: 1, down_bps: 2_000, loan_bps: 850, tenure_years: 0, rent_saved_paise: 0 }, t).is_err());
    }

    #[test]
    fn a_check_in_note_says_where_the_line_is_and_how_it_moved() {
        let marks = |free, out| Marks { free_at_age: free, runs_out_at_age: out, end_age: 90, ..Default::default() };
        let first = note(&marks(Some(58), None), None);
        assert_eq!(first.line, "On today's numbers, work becomes optional at 58 and money lasts past 90.");
        let better = note(&marks(Some(57), None), Some(&first));
        assert_eq!(better.line, "On today's numbers, work becomes optional at 57 and money lasts past 90. Last check-in work became optional at 58.");
        assert_eq!(note(&marks(Some(57), None), Some(&better)).line, "On today's numbers, work becomes optional at 57 and money lasts past 90.");
        let worse = note(&marks(None, Some(81)), Some(&better));
        assert_eq!(worse.line, "On today's numbers, money runs short at 81. Last check-in it lasted past 90.");
        assert_eq!(note(&marks(None, None), Some(&worse)).line, "On today's numbers, money lasts past 90, but work never becomes optional. Last check-in it ran short at 81.");
    }

    #[test]
    fn long_run_amounts_are_rounded_to_lakh_and_crore() {
        assert_eq!(rough(5_74_91_921_00), "₹5.7 crore");
        assert_eq!(rough(-10_00_00_000_00), "₹10 crore");
        assert_eq!(rough(57_40_000_00), "₹57.4 lakh");
        assert_eq!(rough(45_000_00), "₹45,000");
    }

    #[test]
    fn choices_read_from_tagged_json() {
        let c: Choice = serde_json::from_str(r#"{"kind":"job_switch","month":"2027-04","take_home_paise":100}"#).unwrap();
        assert_eq!(c, Choice::JobSwitch { month: "2027-04".into(), take_home_paise: 100, gap_months: 0, raise_bps: None });
        let c: Choice = serde_json::from_str(r#"{"kind":"stay_active"}"#).unwrap();
        assert_eq!(c, Choice::StayActive { medical_cut_bps: None });
    }
}
