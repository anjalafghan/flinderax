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
    /// A hospital stay in `year`: the bill (in today's money) less what health cover pays.
    Hospital {
        year: i64,
        bill_paise: i64,
    },
    /// The user's pay stops for good from `year` (illness or death). Term cover, if any, pays out.
    LoseEarner {
        year: i64,
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
    /// Money set aside for the first child's education, if there is a plan for it.
    pub earmark: Option<Earmark>,
}

/// A pot kept apart for the first child's education. It starts from `corpus_paise` (taken out of
/// savings), grows at `return_bps`, takes `monthly_paise` a month up to the last fee, and pays each
/// fee in `payouts` first. What it cannot cover comes from savings. Where an earmark is given it
/// stands in for the default education estimate of the first child.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Earmark {
    pub corpus_paise: i64,
    pub monthly_paise: i64,
    pub return_bps: i64,
    /// (months from this month, fee then in paise), any order.
    pub payouts: Vec<(i64, i64)>,
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
        Choice::Hospital { year, bill_paise } => {
            if !year_ok(*year) {
                return Err("The year must be this year or later.".into());
            }
            positive_amount(*bill_paise)?;
        }
        Choice::LoseEarner { year } => {
            if !year_ok(*year) {
                return Err("The year must be this year or later.".into());
            }
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
/// A rough effective tax on investment gains, as a share of each month's growth. An estimate.
pub const GAINS_TAX_BPS: i64 = 1_000;
/// The cautious and hopeful lines shift every return by this much either way.
pub const BAND_SHIFT_BPS: i64 = 200;

/// Ways to bend the line without changing the household's facts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tweaks {
    /// Added to every investment return (negative: cautious, positive: hopeful).
    pub return_shift_bps: i64,
    /// Share of each month's gain that is lost to tax. Losses are not taxed back.
    pub gains_tax_bps: i64,
}

/// One year of the band: net worth in today's money on three assumptions about returns.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct BandRow {
    pub year: i64,
    pub age: i64,
    pub cautious_today_paise: i64,
    pub expected_today_paise: i64,
    pub hopeful_today_paise: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Band {
    pub shift_bps: i64,
    pub gains_tax_bps: i64,
    pub rows: Vec<BandRow>,
}

/// The same household on cautious, expected and hopeful returns, all after estimated tax on gains.
pub fn band(input: &LineInput, today: Date) -> Band {
    let run = |shift: i64| simulate_with(input, today, &Tweaks { return_shift_bps: shift, gains_tax_bps: GAINS_TAX_BPS });
    let (low, mid, high) = (run(-BAND_SHIFT_BPS), run(0), run(BAND_SHIFT_BPS));
    let rows = mid
        .rows
        .iter()
        .zip(low.rows.iter().zip(high.rows.iter()))
        .map(|(m, (l, h))| BandRow {
            year: m.year,
            age: m.age,
            cautious_today_paise: l.net_worth_today_paise,
            expected_today_paise: m.net_worth_today_paise,
            hopeful_today_paise: h.net_worth_today_paise,
        })
        .collect();
    Band { shift_bps: BAND_SHIFT_BPS, gains_tax_bps: GAINS_TAX_BPS, rows }
}

pub fn simulate(input: &LineInput, today: Date) -> Line {
    simulate_with(input, today, &Tweaks::default())
}

/// A cost given in today's money, grown at `bps` a year to month `month`.
fn grow(today_cost: f64, bps: i64, month: i64) -> f64 {
    today_cost * (1.0 + rate(bps)).powf(month as f64 / 12.0)
}

/// What the profile and the chosen changes add up to. Built once, then only read month by month.
struct Plan {
    ty: i64,
    age0: i64,
    retire_age: i64,
    end_age: i64,
    /// Months of the first calendar year already gone: rows are calendar years.
    offset: i64,
    total_months: i64,
    infl_bps: i64,
    equity_bps: i64,
    my_raise: f64,
    partner_raise: f64,
    /// Birth years, the planned child first.
    children: Vec<i64>,
    jobs: Vec<Job>,
    breaks: Vec<(i64, i64)>,
    homes: Vec<Home>,
    lumps: Vec<Lump>,
    /// (first month, last month) of the monthly cost of each child still under 18.
    child_spans: Vec<(i64, i64)>,
    save_more: f64,
    earner_lost: Option<i64>,
    medical_keep: f64,
}

impl Plan {
    fn new(input: &LineInput, today: Date) -> Self {
        let p = &input.profile;
        let age0 = p.my_age.clamp(MIN_AGE, 100);
        let end_age = p.live_to_age.clamp(age0, MAX_AGE);
        let offset = today.month() as i64 - 1;
        let mut plan = Plan {
            ty: today.year() as i64,
            age0,
            retire_age: p.retirement_age.clamp(age0, 100),
            end_age,
            offset,
            total_months: (end_age - age0 + 1) * MONTHS_PER_YEAR - offset,
            // Your own guesses, held to a sane range.
            infl_bps: p.inflation_bps.map_or(GENERAL_INFLATION_BPS, |b| b.clamp(0, 2_000)),
            equity_bps: p.equity_return_bps.map_or(EQUITY_RETURN_BPS, |b| b.clamp(0, 2_500)),
            my_raise: rate(p.raise_bps.clamp(0, 3_000)),
            partner_raise: rate(p.partner_raise_bps.clamp(0, 3_000)),
            children: p.child_birth_year.into_iter().collect(),
            jobs: Vec::new(),
            breaks: Vec::new(),
            homes: Vec::new(),
            lumps: Vec::new(),
            child_spans: Vec::new(),
            save_more: 0.0,
            earner_lost: None,
            medical_keep: 1.0,
        };
        for choice in input.choices.iter().take(MAX_CHOICES) {
            plan.apply(choice, p, today);
        }
        plan.jobs.sort_by_key(|j| j.from);
        plan.add_wedding(p, today);
        plan.add_children(input, today);
        plan
    }

    fn lump(&mut self, month: i64, amount: f64, label: impl Into<String>) {
        self.lumps.push(Lump { month, amount, label: label.into() });
    }

    /// A year the user picked, held to the next 80 years.
    fn year_from(&self, year: i64) -> i64 {
        year.clamp(self.ty, self.ty + 80)
    }

    fn apply(&mut self, choice: &Choice, p: &LifeProfile, today: Date) {
        let ty = self.ty;
        match choice {
            Choice::JobSwitch { month, take_home_paise, gap_months, raise_bps } => {
                let Some(from) = month_index(month, today) else { return };
                self.jobs.push(Job {
                    from,
                    gap: (*gap_months).clamp(0, 36),
                    pay: (*take_home_paise).clamp(0, MAX_AMOUNT_PAISE) as f64,
                    raise: raise_bps.map_or(self.my_raise, |b| rate(b.clamp(0, 3_000))),
                });
            }
            Choice::House { year, price_paise, down_bps, loan_bps, tenure_years, rent_saved_paise } => {
                let from = months_from(today, self.year_from(*year), 1);
                let price = grow((*price_paise).clamp(0, MAX_AMOUNT_PAISE) as f64, self.infl_bps, from);
                let down = price * rate((*down_bps).clamp(0, 10_000));
                let months = (*tenure_years).clamp(1, 40) * MONTHS_PER_YEAR;
                let monthly_rate = rate((*loan_bps).clamp(0, 3_000)) / 12.0;
                let loan = price - down;
                self.lump(from, down, "Home down payment");
                self.homes.push(Home {
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
                match self.children.first_mut() {
                    Some(planned) if !*another => *planned = year,
                    _ => self.children.push(year),
                }
            }
            Choice::RetireAt { age } => self.retire_age = (*age).clamp(self.age0, 100),
            Choice::BigSpend { name, year, amount_paise, every_years } => {
                let label = if name.trim().is_empty() { "Big spend" } else { name.trim() };
                let step = every_years.map(|n| n.clamp(1, 50) * MONTHS_PER_YEAR);
                let mut month = months_from(today, self.year_from(*year), 1);
                while month < self.total_months {
                    let amount = grow((*amount_paise).clamp(0, MAX_AMOUNT_PAISE) as f64, self.infl_bps, month);
                    self.lump(month, amount, label);
                    match step {
                        Some(s) => month += s,
                        None => break,
                    }
                }
            }
            Choice::Break { start, months } => {
                let Some(from) = month_index(start, today) else { return };
                self.breaks.push((from, from + (*months).clamp(1, 60)));
            }
            Choice::Hospital { year, bill_paise } => {
                let from = months_from(today, self.year_from(*year), 6);
                let cover = p.health_cover_paise.clamp(0, MAX_AMOUNT_PAISE);
                let net = ((*bill_paise).clamp(0, MAX_AMOUNT_PAISE) - cover).max(0) as f64;
                // Covered in full: nothing to pay, but the year still notes it.
                let amount = if net > 0.0 { grow(net, MEDICAL_INFLATION_BPS, from) } else { 0.0 };
                self.lump(from, amount, "Hospital bill");
            }
            Choice::LoseEarner { year } => {
                let from = months_from(today, self.year_from(*year), 6);
                self.earner_lost = Some(self.earner_lost.map_or(from, |e| e.min(from)));
                let cover = p.term_cover_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
                if cover > 0.0 {
                    // A payout is money coming in, which is a negative cost in that month.
                    self.lump(from, -cover, "Term cover pays out");
                }
            }
            Choice::SaveMore { monthly_paise } => self.save_more += (*monthly_paise).clamp(0, MAX_AMOUNT_PAISE) as f64,
            Choice::StayActive { medical_cut_bps } => {
                self.medical_keep = 1.0 - rate(medical_cut_bps.unwrap_or(DEFAULT_MEDICAL_CUT_BPS).clamp(0, 9_000));
            }
        }
    }

    fn add_wedding(&mut self, p: &LifeProfile, today: Date) {
        let Some(when) = p.wedding_date.as_deref().and_then(parse_date) else { return };
        let month = months_from(today, when.year() as i64, when.month() as i64);
        let budget = p.wedding_budget_paise.unwrap_or(DEFAULT_WEDDING_BUDGET).clamp(0, MAX_AMOUNT_PAISE) as f64;
        self.lump(month, grow(budget, WEDDING_INFLATION_BPS, month), "Wedding");
    }

    fn add_children(&mut self, input: &LineInput, today: Date) {
        let (ty, p) = (self.ty, &input.profile);
        for (n, birth_year) in self.children.clone().into_iter().enumerate() {
            let name = if n == 0 { "Child".to_string() } else { format!("Child {}", n + 1) };
            if birth_year >= ty {
                let month = months_from(today, birth_year, 1);
                self.lump(month, grow(CHILD_BIRTH_TODAY as f64, MEDICAL_INFLATION_BPS, month), format!("{name} is born"));
            }
            if birth_year + CHILD_YEARS >= ty {
                let month = months_from(today, birth_year + CHILD_YEARS, 6);
                let cost = p.child_education_today_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
                // With an education plan, its own fees (paid from the earmark) replace this estimate.
                if !(n == 0 && input.earmark.is_some()) {
                    self.lump(month, grow(cost, EDUCATION_INFLATION_BPS, month), format!("{name}'s education"));
                }
                self.child_spans.push((months_from(today, birth_year, 1), month));
            }
        }
    }

    /// The user's own pay in month `m`: the last job that has started wins; a gap or a break pays nothing.
    fn my_pay(&self, base: f64, m: i64, years_in: i32) -> f64 {
        let mut mine = base * (1.0 + self.my_raise).powi(years_in);
        if let Some(job) = self.jobs.iter().rev().find(|j| j.from <= m) {
            let since = m - job.from - job.gap;
            mine = if since < 0 { 0.0 } else { job.pay * (1.0 + job.raise).powi((since / MONTHS_PER_YEAR) as i32) };
        }
        let on_break = self.breaks.iter().any(|(from, to)| (*from..*to).contains(&m));
        if on_break || self.earner_lost.is_some_and(|from| m >= from) {
            return 0.0;
        }
        mine
    }

    /// Later-life medical cost this month; nothing before `MEDICAL_FROM_AGE`.
    fn medical(&self, m: i64, age: i64) -> f64 {
        if age < MEDICAL_FROM_AGE {
            return 0.0;
        }
        grow(LATER_MEDICAL_TODAY as f64 / 12.0, MEDICAL_INFLATION_BPS, m) * self.medical_keep
    }
}

/// The month-by-month run over a [`Plan`]: savings, the education pot, home loans, and the rows so far.
struct Sim<'a> {
    plan: &'a Plan,
    tweaks: &'a Tweaks,
    earmark: Option<&'a Earmark>,
    expenses: f64,
    my_base: f64,
    partner_base: f64,
    swr: f64,
    retire_month: i64,
    /// Savings can't go below where they started: what they could not pay is short.
    floor: f64,
    last_fee: i64,
    wealth: f64,
    pot: f64,
    loans: Vec<f64>,
    rows: Vec<Row>,
    marks: Marks,
    income_year: f64,
    spend_year: f64,
    short_year: f64,
    events: Vec<String>,
}

impl<'a> Sim<'a> {
    fn new(plan: &'a Plan, input: &'a LineInput, tweaks: &'a Tweaks) -> Self {
        let p = &input.profile;
        let mut wealth = input.wealth_paise.clamp(-MAX_AMOUNT_PAISE, MAX_AMOUNT_PAISE) as f64;
        // The earmark is taken out of savings (no more than there is) and kept as its own pot.
        let earmark = input.earmark.as_ref();
        let pot = earmark.map_or(0.0, |e| (e.corpus_paise.clamp(0, MAX_AMOUNT_PAISE) as f64).min(wealth.max(0.0)));
        wealth -= pot;
        Sim {
            plan,
            tweaks,
            earmark,
            expenses: input.expenses_paise.clamp(0, MAX_AMOUNT_PAISE) as f64,
            my_base: p.my_take_home_paise.clamp(0, MAX_AMOUNT_PAISE) as f64,
            partner_base: p.partner_take_home_paise.clamp(0, MAX_AMOUNT_PAISE) as f64,
            swr: rate(SWR_BPS),
            // Retirement starts in January of the year the user turns `retire_age`.
            retire_month: ((plan.retire_age - plan.age0) * MONTHS_PER_YEAR - plan.offset).max(0),
            // Starting in debt is allowed; going further below it is "short".
            floor: wealth.min(0.0),
            last_fee: earmark.and_then(|e| e.payouts.iter().map(|(m, _)| *m).max()).unwrap_or(0),
            wealth,
            pot,
            loans: plan.homes.iter().map(|h| h.loan).collect(),
            rows: Vec::new(),
            marks: Marks { retire_age: plan.retire_age, end_age: plan.end_age, ..Default::default() },
            income_year: 0.0,
            spend_year: 0.0,
            short_year: 0.0,
            events: Vec::new(),
        }
    }

    fn step(&mut self, m: i64) {
        let plan = self.plan;
        let year_index = (plan.offset + m) / MONTHS_PER_YEAR;
        let age = plan.age0 + year_index;
        let working = m < self.retire_month;

        let income = if working { self.income(m, year_index) } else { 0.0 };
        let mut spend = self.living_cost(m, age, working);
        self.pay_home_loans(m, &mut spend);
        let mut lump_labels = self.pay_lumps(m, &mut spend);
        self.run_earmark(m, &mut spend, &mut lump_labels);
        self.note_milestones(m);

        self.grow_savings(m);
        self.wealth += income - spend;
        let short = self.cover_shortfall(age, year_index, lump_labels);
        self.income_year += income;
        self.spend_year += spend;
        self.short_year += short;

        if (plan.offset + m) % MONTHS_PER_YEAR == MONTHS_PER_YEAR - 1 {
            self.close_year(m, age, year_index);
        }
    }

    fn income(&self, m: i64, year_index: i64) -> f64 {
        let years_in = year_index as i32;
        self.plan.my_pay(self.my_base, m, years_in) + self.partner_base * (1.0 + self.plan.partner_raise).powi(years_in)
    }

    /// The monthly living cost at month `m`, without one-offs, loans or children: what retirement must pay.
    fn regular_spend(&self, m: i64, age: i64) -> f64 {
        grow(self.expenses, self.plan.infl_bps, m) + self.plan.medical(m, age)
    }

    /// Living costs, less what the user chose to save while working, plus children's monthly cost.
    fn living_cost(&self, m: i64, age: i64, working: bool) -> f64 {
        let plan = self.plan;
        let mut spend = grow(self.expenses, plan.infl_bps, m);
        if working {
            spend = (spend - grow(plan.save_more, plan.infl_bps, m)).max(0.0);
        }
        spend += plan.medical(m, age);
        for (from, to) in &plan.child_spans {
            if (*from..*to).contains(&m) {
                spend += grow(CHILD_MONTHLY_TODAY as f64, plan.infl_bps, m);
            }
        }
        spend
    }

    /// Rent saved and the EMI of each home bought by month `m`.
    fn pay_home_loans(&mut self, m: i64, spend: &mut f64) {
        let plan = self.plan;
        for (home, loan) in plan.homes.iter().zip(self.loans.iter_mut()) {
            if m < home.from {
                continue;
            }
            *spend = (*spend - grow(home.rent_saved, plan.infl_bps, m)).max(0.0);
            if m < home.from + home.months && *loan > 0.0 {
                let interest = *loan * home.monthly_rate;
                *loan = (*loan + interest - home.emi).max(0.0);
                *spend += home.emi;
            }
            if m == home.from {
                self.events.push("Buy a home".into());
            }
        }
    }

    /// One-off costs falling in month `m`; returns their labels.
    fn pay_lumps(&mut self, m: i64, spend: &mut f64) -> Vec<&'a str> {
        let plan = self.plan;
        let mut labels = Vec::new();
        for lump in plan.lumps.iter().filter(|l| l.month == m) {
            *spend += lump.amount;
            labels.push(lump.label.as_str());
            if lump.label != "Home down payment" {
                self.note_event(&lump.label);
            }
        }
        labels
    }

    /// The education pot: growth first, then the month's top-up, then any fee, the order the plan's maths uses.
    fn run_earmark(&mut self, m: i64, spend: &mut f64, lump_labels: &mut Vec<&'a str>) {
        let Some(e) = self.earmark else { return };
        if m > 0 {
            self.pot *= 1.0 + rate(e.return_bps.clamp(0, 3_000)) / 12.0;
        }
        if m >= 1 && m <= self.last_fee {
            let top_up = e.monthly_paise.clamp(0, MAX_AMOUNT_PAISE) as f64;
            *spend += top_up;
            self.pot += top_up;
        }
        for (_, fee) in e.payouts.iter().filter(|(pm, _)| *pm == m) {
            let fee = (*fee).clamp(0, MAX_AMOUNT_PAISE) as f64;
            let from_pot = self.pot.min(fee);
            self.pot -= from_pot;
            *spend += fee - from_pot;
            if fee - from_pot > 0.0 {
                lump_labels.push("Child's education");
            }
            self.note_event("Child's education");
        }
    }

    /// Record a year's event once.
    fn note_event(&mut self, label: &str) {
        if !self.events.iter().any(|e| e == label) {
            self.events.push(label.to_string());
        }
    }

    fn note_milestones(&mut self, m: i64) {
        let plan = self.plan;
        if m == self.retire_month {
            self.events.push(format!("Retire at {}", plan.retire_age));
        }
        if let Some(job) = plan.jobs.iter().find(|j| j.from == m) {
            self.events.push(if job.pay > 0.0 { "New job".into() } else { "Leave job".into() });
        }
        if plan.breaks.iter().any(|(from, _)| *from == m) {
            self.events.push("Break from work".into());
        }
    }

    /// Equity until 7 years before retirement, then hybrid, then half hybrid and half debt.
    fn return_bps(&self, m: i64) -> i64 {
        let months_to_retire = self.retire_month - m;
        if months_to_retire > EQUITY_MIN_MONTHS {
            self.plan.equity_bps
        } else if months_to_retire > 0 {
            HYBRID_RETURN_BPS
        } else {
            RETIRED_RETURN_BPS
        }
    }

    /// What is there grows, less tax on a gain; a loss is not taxed back.
    fn grow_savings(&mut self, m: i64) {
        if self.wealth > 0.0 {
            let gain = self.wealth * rate(self.return_bps(m) + self.tweaks.return_shift_bps) / 12.0;
            let tax = rate(self.tweaks.gains_tax_bps.clamp(0, 9_000));
            self.wealth += if gain > 0.0 { gain * (1.0 - tax) } else { gain };
        }
    }

    /// Savings stop at the floor. Returns what they could not pay and marks what went unfunded.
    fn cover_shortfall(&mut self, age: i64, year_index: i64, lump_labels: Vec<&str>) -> f64 {
        if self.wealth >= self.floor {
            return 0.0;
        }
        let short = self.floor - self.wealth;
        self.wealth = self.floor;
        self.marks.runs_out_at_age.get_or_insert(age);
        let year = self.plan.ty + year_index;
        for label in lump_labels {
            let text = format!("{label} in {year}");
            if !self.marks.unfunded.contains(&text) {
                self.marks.unfunded.push(text);
            }
        }
        short
    }

    /// December: set the year's row and update the marks.
    fn close_year(&mut self, m: i64, age: i64, year_index: i64) {
        let plan = self.plan;
        let home_value: f64 = plan.homes.iter().filter(|h| h.from <= m).map(|h| grow(h.price, plan.infl_bps, m - h.from)).sum();
        let owed: f64 = plan.homes.iter().zip(&self.loans).filter(|(h, _)| h.from <= m).map(|(_, l)| *l).sum();
        let net = self.wealth + self.pot + home_value - owed;
        let deflate = (1.0 + rate(plan.infl_bps)).powf((m + 1) as f64 / 12.0);
        let net_today = to_paise(net / deflate);
        // Free: savings alone would pay a year of today's costs for good, and nothing is short.
        let yearly_need = self.regular_spend(m, age) * 12.0;
        if self.marks.free_at_age.is_none() && self.short_year == 0.0 && yearly_need > 0.0 && self.wealth * self.swr >= yearly_need {
            self.marks.free_at_age = Some(age);
        }
        if age == 60 {
            self.marks.at_60_today_paise = Some(net_today);
        }
        self.marks.at_end_today_paise = net_today;
        self.rows.push(Row {
            year: plan.ty + year_index,
            age,
            income_paise: to_paise(self.income_year),
            spend_paise: to_paise(self.spend_year),
            short_paise: to_paise(self.short_year),
            net_worth_paise: to_paise(net),
            net_worth_today_paise: net_today,
            events: std::mem::take(&mut self.events),
        });
        (self.income_year, self.spend_year, self.short_year) = (0.0, 0.0, 0.0);
    }
}

/// The assumptions the line rests on, stated for the reader. Only those that applied are listed.
fn notes(plan: &Plan, tweaks: &Tweaks, has_earmark: bool) -> Vec<String> {
    let mut notes = vec![
        "Every rupee not spent is invested: in equity until 7 years before retirement, then hybrid, then half hybrid and half debt.".to_string(),
        "Both incomes stop when you retire.".to_string(),
        format!(
            "Estimate: medical costs of {} a year in today's money from age {MEDICAL_FROM_AGE}, rising with medical inflation.",
            inr(LATER_MEDICAL_TODAY)
        ),
    ];
    if tweaks.gains_tax_bps > 0 {
        notes.push(format!(
            "Estimate: about {}% of each year's investment gains goes to tax. Your real rate depends on what you hold.",
            tweaks.gains_tax_bps / 100
        ));
    }
    if !plan.children.is_empty() {
        notes.push(format!("Estimate: raising a child costs {} a month in today's money until 18, plus education.", inr(CHILD_MONTHLY_TODAY)));
    }
    if has_earmark {
        notes.push("The education plan's fees replace the default estimate. Its top-up goes into a separate pot that pays them first, and what it cannot cover comes from savings.".to_string());
    }
    if !plan.homes.is_empty() {
        notes.push("A home is counted in net worth and keeps its value in today's money.".to_string());
    }
    if plan.medical_keep < 1.0 {
        notes.push("Estimate: staying active trims later-life medical costs. The size of the saving is a rule of thumb, not a promise.".to_string());
    }
    notes
}

pub fn simulate_with(input: &LineInput, today: Date, tweaks: &Tweaks) -> Line {
    let plan = Plan::new(input, today);
    let mut sim = Sim::new(&plan, input, tweaks);
    for m in 0..plan.total_months {
        sim.step(m);
    }
    Line { notes: notes(&plan, tweaks, input.earmark.is_some()), rows: sim.rows, marks: sim.marks }
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
            earmark: None,
        }
    }

    fn line(choices: Vec<Choice>) -> Line {
        simulate(&input(choices), today())
    }

    fn end_today(l: &Line) -> i64 {
        l.marks.at_end_today_paise
    }

    fn covered(health: i64, term: i64, choices: Vec<Choice>) -> Line {
        let mut i = input(choices);
        i.profile.health_cover_paise = health;
        i.profile.term_cover_paise = term;
        i.profile.partner_take_home_paise = 12_000_000; // someone is still earning when the other is lost
        simulate(&i, today())
    }

    #[test]
    fn a_hospital_bill_costs_what_health_cover_does_not() {
        let bill = Choice::Hospital { year: 2030, bill_paise: 50_000_000 };
        let none = end_today(&covered(0, 0, vec![]));
        let uncovered = end_today(&covered(0, 0, vec![bill.clone()]));
        let part = end_today(&covered(20_000_000, 0, vec![bill.clone()]));
        let all = end_today(&covered(60_000_000, 0, vec![bill]));
        assert!(uncovered < part && part < all, "{uncovered} {part} {all}");
        assert_eq!(all, none, "fully covered, it costs nothing");
        let l = covered(0, 0, vec![Choice::Hospital { year: 2030, bill_paise: 50_000_000 }]);
        assert!(l.rows.iter().any(|r| r.year == 2030 && r.events.iter().any(|e| e == "Hospital bill")));
    }

    #[test]
    fn losing_an_earner_stops_their_pay_and_term_cover_softens_it() {
        let lose = || Choice::LoseEarner { year: 2030 };
        let base = end_today(&covered(0, 0, vec![]));
        let bare = end_today(&covered(0, 0, vec![lose()]));
        let insured = end_today(&covered(0, 500_000_000, vec![lose()]));
        assert!(bare < base, "less income");
        assert!(insured > bare, "term cover pays out");
        let l = covered(0, 500_000_000, vec![lose()]);
        assert!(l.rows.iter().any(|r| r.year == 2030 && r.events.iter().any(|e| e.contains("Term cover"))));
        assert!(l.rows.last().unwrap().income_paise > 0 || l.marks.retire_age > 0);
    }

    #[test]
    fn shocks_are_checked() {
        assert!(validate(&Choice::Hospital { year: 2030, bill_paise: 1_000_000 }, today()).is_ok());
        assert!(validate(&Choice::Hospital { year: 2001, bill_paise: 1_000_000 }, today()).is_err());
        assert!(validate(&Choice::Hospital { year: 2030, bill_paise: 0 }, today()).is_err());
        assert!(validate(&Choice::LoseEarner { year: 2030 }, today()).is_ok());
        assert!(validate(&Choice::LoseEarner { year: 1999 }, today()).is_err());
        let json = serde_json::to_string(&Choice::LoseEarner { year: 2031 }).unwrap();
        assert_eq!(json, r#"{"kind":"lose_earner","year":2031}"#);
    }

    #[test]
    fn your_own_return_and_inflation_move_the_line() {
        let base = input(vec![]);
        let with = |f: &dyn Fn(&mut LifeProfile)| {
            let mut i = input(vec![]);
            f(&mut i.profile);
            end_today(&simulate(&i, today()))
        };
        let plain = end_today(&simulate(&base, today()));
        assert_eq!(with(&|p| { p.equity_return_bps = Some(EQUITY_RETURN_BPS); p.inflation_bps = Some(GENERAL_INFLATION_BPS); }), plain, "the defaults, spelled out");
        assert!(with(&|p| p.equity_return_bps = Some(900)) < plain, "lower returns, less money");
        assert!(with(&|p| p.equity_return_bps = Some(1400)) > plain);
        assert!(with(&|p| p.inflation_bps = Some(900)) < plain, "higher inflation, less in today's money");
        // Wild values are held to a sane range rather than breaking the line.
        let _ = with(&|p| { p.equity_return_bps = Some(i64::MAX); p.inflation_bps = Some(-5); });
    }

    #[test]
    fn no_tweaks_is_the_plain_line() {
        let i = input(vec![]);
        assert_eq!(simulate_with(&i, today(), &Tweaks::default()), simulate(&i, today()));
    }

    #[test]
    fn tax_on_gains_lowers_the_line_and_says_so() {
        let i = input(vec![]);
        let free = simulate(&i, today());
        let taxed = simulate_with(&i, today(), &Tweaks { gains_tax_bps: GAINS_TAX_BPS, ..Default::default() });
        assert!(end_today(&taxed) < end_today(&free), "{} vs {}", end_today(&taxed), end_today(&free));
        assert!(taxed.notes.iter().any(|n| n.contains("tax") && n.starts_with("Estimate")), "{:?}", taxed.notes);
        assert!(!free.notes.iter().any(|n| n.to_lowercase().contains("gains")), "an untaxed line does not claim to be taxed");
    }

    #[test]
    fn a_loss_is_not_taxed_back() {
        let i = input(vec![]);
        let cautious = Tweaks { return_shift_bps: -2_000, ..Default::default() };
        let with_tax = Tweaks { return_shift_bps: -2_000, gains_tax_bps: 5_000 };
        // Returns this low are negative in the retired years: tax must not make a loss smaller.
        assert!(end_today(&simulate_with(&i, today(), &with_tax)) <= end_today(&simulate_with(&i, today(), &cautious)));
    }

    #[test]
    fn the_band_runs_cautious_to_hopeful_around_the_expected_line() {
        let b = band(&input(vec![]), today());
        assert_eq!((b.shift_bps, b.rows.len()), (BAND_SHIFT_BPS, 62));
        for r in &b.rows {
            assert!(r.cautious_today_paise <= r.expected_today_paise && r.expected_today_paise <= r.hopeful_today_paise, "{r:?}");
        }
        let last = b.rows.last().unwrap();
        assert!(last.hopeful_today_paise > last.cautious_today_paise);
        assert_eq!(last.expected_today_paise, end_today(&simulate_with(&input(vec![]), today(), &Tweaks { gains_tax_bps: GAINS_TAX_BPS, ..Default::default() })));
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

    fn year_of(l: &Line, event: &str) -> Option<i64> {
        l.rows.iter().find(|r| r.events.iter().any(|e| e == event)).map(|r| r.year)
    }

    fn with_child_and_earmark(earmark: Option<Earmark>) -> Line {
        let mut i = input(vec![Choice::Child { birth_year: 2026, another: false }]);
        i.earmark = earmark;
        simulate(&i, today())
    }

    #[test]
    fn an_earmark_replaces_the_default_education_estimate_and_pays_the_fee_from_its_pot() {
        let fee_month = months_from(today(), 2044, 6);
        let none = with_child_and_earmark(None);
        assert_eq!(year_of(&none, "Child's education"), Some(2044));
        // A pot that already holds the fee and needs no top-up: the fee costs savings nothing at
        // the time, so the line only differs by what the pot earned against what savings earned.
        let covered = with_child_and_earmark(Some(Earmark { corpus_paise: 50_000_000, monthly_paise: 0, return_bps: 1_150, payouts: vec![(fee_month, 40_000_000)] }));
        assert_eq!(year_of(&covered, "Child's education"), Some(2044));
        let fee_year = covered.rows.iter().find(|r| r.year == 2044).unwrap();
        let plain_year = none.rows.iter().find(|r| r.year == 2044).unwrap();
        assert!(fee_year.spend_paise < plain_year.spend_paise, "the fee came out of the pot, not the month's money");
        assert!(covered.notes.iter().any(|n| n.contains("education plan")));
    }

    #[test]
    fn what_the_pot_cannot_cover_comes_from_savings_and_the_top_up_is_spent_each_month() {
        let fee_month = months_from(today(), 2036, 6);
        let fee = 30_000_000;
        let thin = with_child_and_earmark(Some(Earmark { corpus_paise: 0, monthly_paise: 0, return_bps: 1_150, payouts: vec![(fee_month, fee)] }));
        let topped = with_child_and_earmark(Some(Earmark { corpus_paise: 0, monthly_paise: 250_000, return_bps: 1_150, payouts: vec![(fee_month, fee)] }));
        let spend = |l: &Line, y: i64| l.rows.iter().find(|r| r.year == y).unwrap().spend_paise;
        assert!(spend(&thin, 2036) >= fee, "an empty pot leaves the whole fee to savings");
        assert!(spend(&topped, 2030) > spend(&thin, 2030), "the top-up is a monthly cost");
        assert!(spend(&topped, 2036) < spend(&thin, 2036) - 100_000_000 / 100, "and the pot takes the fee");
        // The top-up stops after the last fee.
        assert_eq!(spend(&topped, 2040), spend(&thin, 2040));
    }

    #[test]
    fn a_top_up_that_matches_the_plan_changes_the_line_a_little_not_a_lot() {
        use crate::planner::education::{earmark, Plan};
        let plan = Plan::starter(2026);
        let base = {
            let mut i = input(vec![Choice::Child { birth_year: 2026, another: false }]);
            i.profile.my_take_home_paise = 30_000_000;
            i
        };
        let mut with = base.clone();
        with.earmark = Some(earmark(&plan, today()));
        let (b, w) = (simulate(&base, today()), simulate(&with, today()));
        let c = compare(&b, &w);
        assert!(!c.lines.is_empty());
        // Saving into a pot that earns a return and is spent on fees is not wildly different from
        // paying the fees later out of savings that earn a return.
        assert!(c.at_end_change_paise.abs() < b.marks.at_end_today_paise.abs() / 2 + 1, "{c:?}");
    }

    #[test]
    fn an_earmark_larger_than_savings_only_takes_what_there_is() {
        let mut small = input(vec![]);
        small.wealth_paise = 1_000_000;
        let mut earmarked = small.clone();
        earmarked.earmark = Some(Earmark { corpus_paise: 900_000_000, monthly_paise: 0, return_bps: EQUITY_RETURN_BPS, payouts: vec![] });
        let (a, b) = (simulate(&small, today()), simulate(&earmarked, today()));
        let (x, y) = (a.rows[0].net_worth_paise, b.rows[0].net_worth_paise);
        assert!((x - y).abs() < x.abs() / 100, "moving savings into a pot that earns the same loses and creates almost nothing: {x} vs {y}");
    }

    #[test]
    fn choices_read_from_tagged_json() {
        let c: Choice = serde_json::from_str(r#"{"kind":"job_switch","month":"2027-04","take_home_paise":100}"#).unwrap();
        assert_eq!(c, Choice::JobSwitch { month: "2027-04".into(), take_home_paise: 100, gap_months: 0, raise_bps: None });
        let c: Choice = serde_json::from_str(r#"{"kind":"stay_active"}"#).unwrap();
        assert_eq!(c, Choice::StayActive { medical_cut_bps: None });
    }
}

/// Characterization tests: the whole simulation, pinned. A refactor of `simulate_with` must leave
/// this file byte-for-byte alone. Regenerate on purpose with `UPDATE_GOLDEN=1 cargo test golden`.
#[cfg(test)]
mod golden {
    use super::*;
    use time::macros::date;

    const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/lifeline_golden.json");

    fn profile() -> LifeProfile {
        LifeProfile {
            my_age: 31,
            partner_age: Some(29),
            my_take_home_paise: 18_000_000,
            partner_take_home_paise: 12_000_000,
            wedding_date: Some("2028-02-01".into()),
            wedding_budget_paise: Some(2_500_000_00),
            child_birth_year: Some(2030),
            child_education_today_paise: 3_000_000_00,
            retirement_age: 58,
            term_cover_paise: 10_000_000_00,
            health_cover_paise: 5_000_000_00,
            raise_bps: 700,
            partner_raise_bps: 500,
            live_to_age: 88,
            equity_return_bps: Some(1_100),
            inflation_bps: Some(550),
            ..Default::default()
        }
    }

    fn base(profile: LifeProfile, wealth: i64, choices: Vec<Choice>) -> LineInput {
        LineInput { profile, expenses_paise: 9_000_000, wealth_paise: wealth, choices, earmark: None }
    }

    fn every_choice() -> Vec<Choice> {
        vec![
            Choice::JobSwitch { month: "2027-06".into(), take_home_paise: 25_000_000, gap_months: 3, raise_bps: Some(900) },
            Choice::JobSwitch { month: "2033-01".into(), take_home_paise: 0, gap_months: 0, raise_bps: None },
            Choice::House { year: 2031, price_paise: 1_200_000_000, down_bps: 2_000, loan_bps: 850, tenure_years: 20, rent_saved_paise: 3_500_000 },
            Choice::Child { birth_year: 2032, another: true },
            Choice::RetireAt { age: 55 },
            Choice::BigSpend { name: " Car ".into(), year: 2029, amount_paise: 90_000_000, every_years: Some(8) },
            Choice::BigSpend { name: String::new(), year: 2040, amount_paise: 40_000_000, every_years: None },
            Choice::Break { start: "2035-03".into(), months: 8 },
            Choice::Hospital { year: 2034, bill_paise: 900_000_000 },
            Choice::LoseEarner { year: 2045 },
            Choice::SaveMore { monthly_paise: 1_000_000 },
            Choice::StayActive { medical_cut_bps: Some(4_000) },
        ]
    }

    fn scenarios() -> Vec<(&'static str, LineInput, Date, Tweaks)> {
        let plain = Tweaks::default();
        let mut earmarked = base(profile(), 600_000_000, vec![Choice::Child { birth_year: 2030, another: false }]);
        earmarked.earmark = Some(Earmark {
            corpus_paise: 20_000_000,
            monthly_paise: 1_500_000,
            return_bps: 900,
            payouts: vec![(150, 40_000_000), (160, 60_000_000), (12, 5_000_000)],
        });
        let broke = LifeProfile { my_age: 40, retirement_age: 45, live_to_age: 95, ..LifeProfile::default() };
        vec![
            ("baseline", base(profile(), 800_000_000, vec![]), date!(2026 - 11 - 10), plain),
            ("every_choice", base(profile(), 800_000_000, every_choice()), date!(2026 - 11 - 10), plain),
            ("every_choice_in_january", base(profile(), 800_000_000, every_choice()), date!(2027 - 01 - 01), plain),
            ("earmark", earmarked, date!(2026 - 12 - 31), plain),
            ("tweaked", base(profile(), 800_000_000, every_choice()), date!(2026 - 06 - 15), Tweaks { return_shift_bps: -250, gains_tax_bps: 1_250 }),
            ("in_debt_and_runs_out", base(broke, -50_000_000, vec![Choice::House { year: 2027, price_paise: 900_000_000, down_bps: 1_000, loan_bps: 900, tenure_years: 15, rent_saved_paise: 0 }]), date!(2026 - 03 - 03), plain),
            ("wild_inputs", base(LifeProfile { my_age: 5, retirement_age: 3, live_to_age: 400, ..profile() }, i64::MAX, vec![Choice::Break { start: "nonsense".into(), months: -4 }, Choice::SaveMore { monthly_paise: i64::MAX }]), date!(2026 - 11 - 10), plain),
        ]
    }

    fn snapshot() -> String {
        let all: Vec<_> = scenarios()
            .into_iter()
            .map(|(name, input, today, tweaks)| {
                serde_json::json!({
                    "name": name,
                    "line": simulate_with(&input, today, &tweaks),
                    "band": band(&input, today),
                })
            })
            .collect();
        serde_json::to_string_pretty(&all).unwrap() + "\n"
    }

    #[test]
    fn simulate_and_band_match_the_pinned_output() {
        let now = snapshot();
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            std::fs::write(FILE, &now).unwrap();
            return;
        }
        let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
        assert!(now == pinned, "life line output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
    }
}
