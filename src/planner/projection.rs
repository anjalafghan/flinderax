//! Multi-month projection and what-if scenarios. Pure, like the rest of the planner.
//!
//! The simulation walks day by day from today: salary lands, fixed payments go out, the monthly
//! top-up moves from the spending account to the reserve, the spending allowance is used, card
//! bills fall due from the reserve, EMIs run out, and FDs/RDs grow (quarterly compounding, the
//! Indian convention) and mature according to their renewal setting.
//!
//! A scenario adds hypothetical items (an RD, an FD, a one-off purchase) to the same simulation, so
//! "baseline vs with this change" is one function applied twice.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use super::cycle::{add_months, clamped};
use super::{commitment_dates, compute, month_index, Bucket, PlanInput, Role};

pub const MAX_MONTHS: i64 = 60;

/// What happens when an FD matures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Renewal {
    /// Principal + interest are paid out to an account.
    #[default]
    Payout,
    /// Principal renews; the interest earned is paid out.
    RenewPrincipal,
    /// Principal + interest renew together (compounding continues).
    RenewAll,
}

impl Renewal {
    pub fn parse(s: &str) -> Self {
        match s {
            "renew_principal" => Self::RenewPrincipal,
            "renew_all" => Self::RenewAll,
            _ => Self::Payout,
        }
    }
}

/// Extra facts about a locked account (an FD or RD).
#[derive(Debug, Clone)]
pub struct DepositIn {
    pub account_id: String,
    pub is_rd: bool,
    pub rate_bps: i64,
    pub maturity: Option<Date>,
    pub renewal: Renewal,
    pub payout_to: Option<String>,
}

/// A card EMI still running.
#[derive(Debug, Clone)]
pub struct EmiIn {
    pub monthly: i64,
    pub months_left: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    /// Monthly instalments into a new recurring deposit.
    Rd,
    /// A lump sum into a new fixed deposit.
    Fd,
    /// A one-off spend (the wedding, a gadget).
    OneOff,
}

/// One hypothetical change. Everything is a variable the UI can edit live.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioItem {
    pub name: String,
    pub kind: ItemKind,
    /// Rd: per month. Fd / OneOff: the lump sum.
    pub amount_paise: i64,
    /// First instalment / deposit / spend (`YYYY-MM-DD`).
    pub start: Date,
    /// Rd / Fd: how long it runs.
    pub tenure_months: Option<i64>,
    pub rate_bps: Option<i64>,
    /// Account the money leaves (default: the spending account).
    pub funded_from: Option<String>,
    /// Where a maturing deposit pays out (default: the reserve account).
    pub payout_to: Option<String>,
    pub renewal: Option<Renewal>,
    /// A goal for this item: reach `target_paise` by `target_date`.
    pub target_paise: Option<i64>,
    pub target_date: Option<Date>,
}

#[derive(Debug, Clone)]
pub struct ProjectionInput {
    pub plan: PlanInput,
    pub deposits: Vec<DepositIn>,
    pub emis: Vec<EmiIn>,
    /// How many months ahead (1..=MAX_MONTHS).
    pub months: i64,
    /// How much of the spending allowance you actually spend each month, in percent.
    pub spend_pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Point {
    /// `YYYY-MM`, the month this point closes.
    pub month: String,
    pub spending_paise: i64,
    pub reserve_paise: i64,
    pub fds_paise: i64,
    pub rds_paise: i64,
    /// SIPs and other savings that leave the accounts (contributions only, no returns assumed).
    pub invested_paise: i64,
    pub total_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stress {
    pub month: String,
    /// `spending_short` | `reserve_below_floor` | `fd_needed`
    pub kind: &'static str,
    pub amount_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Series {
    pub points: Vec<Point>,
    pub stress: Vec<Stress>,
}

/// How one new monthly cost gets absorbed, in a given order of what gives way first.
#[derive(Debug, Clone, Serialize)]
pub struct Absorption {
    pub cost_paise: i64,
    pub from_rest_savings_paise: i64,
    pub from_sip_paise: i64,
    pub from_spending_paise: i64,
    /// Whatever is left over has to come from an FD.
    pub from_fd_paise: i64,
    pub rest_savings_after_paise: i64,
    pub sip_after_paise: i64,
    pub spending_after_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Threshold {
    pub label: String,
    /// Monthly cost up to which only this stage is affected.
    pub up_to_paise: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemImpact {
    pub name: String,
    /// The monthly cost (Rd instalment); 0 for lumps.
    pub monthly_paise: i64,
    pub savings_first: Option<Absorption>,
    pub spending_first: Option<Absorption>,
    pub savings_first_thresholds: Vec<Threshold>,
    /// What a deposit is worth when it matures / ends (0 for one-offs).
    pub maturity_value_paise: i64,
    pub goal: Option<GoalCheck>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoalCheck {
    pub target_paise: i64,
    pub target_date: Date,
    pub projected_paise: i64,
    /// Positive = ahead of target, negative = short.
    pub gap_paise: i64,
    /// Monthly amount that would just reach the target (for an RD).
    pub required_monthly_paise: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Projection {
    pub baseline: Series,
    pub scenario: Option<Series>,
    pub impacts: Vec<ItemImpact>,
}

// ---------- interest ----------

/// Monthly growth factor for an annual rate compounded quarterly.
pub fn monthly_factor(rate_bps: i64) -> f64 {
    (1.0 + rate_bps as f64 / 10_000.0 / 4.0).powf(1.0 / 3.0)
}

/// An RD's value after `months` instalments of `monthly` paise (deposited at the start of each month).
pub fn rd_maturity(monthly: i64, rate_bps: i64, months: i64) -> i64 {
    let f = monthly_factor(rate_bps);
    let mut bal = 0.0;
    for _ in 0..months {
        bal = (bal + monthly as f64) * f;
    }
    bal.round() as i64
}

/// The monthly instalment that grows to `target` in `months`.
pub fn rd_required_monthly(target: i64, rate_bps: i64, months: i64) -> i64 {
    let per_rupee = rd_maturity(1_000_000, rate_bps, months) as f64 / 1_000_000.0;
    if per_rupee <= 0.0 { 0 } else { (target as f64 / per_rupee).ceil() as i64 }
}

/// A lump sum after `months` of quarterly compounding.
#[cfg(test)]
pub fn fd_value(principal: i64, rate_bps: i64, months: i64) -> i64 {
    (principal as f64 * monthly_factor(rate_bps).powi(months as i32)).round() as i64
}

// ---------- who absorbs a new monthly cost ----------

#[derive(Clone, Copy)]
enum Layer {
    Rest,
    Sip,
    Spending,
}

fn absorb(rest: i64, sip: i64, spending: i64, cost: i64, order: [Layer; 3]) -> Absorption {
    let (mut r, mut p, mut s) = (rest, sip, spending);
    let mut left = cost;
    let (mut fr, mut fp, mut fs) = (0, 0, 0);
    for layer in order {
        let (pool, taken) = match layer {
            Layer::Rest => (&mut r, &mut fr),
            Layer::Sip => (&mut p, &mut fp),
            Layer::Spending => (&mut s, &mut fs),
        };
        let t = left.min(*pool);
        *pool -= t;
        *taken += t;
        left -= t;
    }
    Absorption {
        cost_paise: cost,
        from_rest_savings_paise: fr,
        from_sip_paise: fp,
        from_spending_paise: fs,
        from_fd_paise: left,
        rest_savings_after_paise: r,
        sip_after_paise: p,
        spending_after_paise: s,
    }
}

// ---------- the simulation ----------

struct Dep {
    id: String,
    is_rd: bool,
    rate_bps: i64,
    maturity: Option<Date>,
    renewal: Renewal,
    payout_to: Option<String>,
    principal: i64,
}

struct Sim {
    bal: HashMap<String, i64>,
    deps: Vec<Dep>,
    invested: i64,
    spending_ids: Vec<String>,
    reserve_ids: Vec<String>,
    first_spend: String,
    first_reserve: Option<String>,
}

impl Sim {
    fn add(&mut self, id: &str, delta: i64) {
        *self.bal.entry(id.to_string()).or_default() += delta;
    }

    fn sum(&self, ids: &[String]) -> i64 {
        ids.iter().map(|i| self.bal.get(i).copied().unwrap_or(0)).sum()
    }

    fn point(&self, month: String) -> Point {
        let dep_sum = |rd: bool| {
            self.deps.iter().filter(|d| d.is_rd == rd).map(|d| self.bal.get(&d.id).copied().unwrap_or(0)).sum::<i64>()
        };
        let spending = self.sum(&self.spending_ids);
        let reserve = self.sum(&self.reserve_ids);
        let (fds, rds) = (dep_sum(false), dep_sum(true));
        let dep_ids: Vec<&String> = self.deps.iter().map(|d| &d.id).collect();
        let other: i64 = self
            .bal
            .iter()
            .filter(|(k, _)| !self.spending_ids.contains(k) && !self.reserve_ids.contains(k) && !dep_ids.contains(k))
            .map(|(_, v)| *v)
            .sum();
        Point {
            month,
            spending_paise: spending,
            reserve_paise: reserve,
            fds_paise: fds,
            rds_paise: rds,
            invested_paise: self.invested,
            total_paise: spending + reserve + fds + rds + other + self.invested,
        }
    }
}

fn month_key(d: Date) -> String {
    format!("{:04}-{:02}", d.year(), d.month() as u8)
}

/// `start` shifted forward by `months`, same day (clamped).
fn plus_months(start: Date, months: i64) -> Date {
    let (y, m) = add_months(start.year(), start.month(), months.clamp(-MAX_TENURE, MAX_TENURE) as i32);
    clamped(y, m, start.day())
}

/// Longest tenure the simulation will honour: the shared input bound. A larger value would make
/// one flow per month for centuries, so it is capped here as well as rejected at the API.
const MAX_TENURE: i64 = crate::validate::MAX_MONTHS;

/// An item's tenure in months: 12 when unset, at least 1, at most `MAX_TENURE`.
fn tenure_of(it: &ScenarioItem) -> i64 {
    it.tenure_months.unwrap_or(12).clamp(1, MAX_TENURE)
}

/// Runs the simulation with the plan's own commitments plus `items`.
fn simulate(inp: &ProjectionInput, items: &[ScenarioItem]) -> (Series, HashMap<String, i64>) {
    let plan = &inp.plan;
    let today = plan.today;
    let months = inp.months.clamp(1, MAX_MONTHS);
    let end = plus_months(today, months);
    let summary = compute(plan);
    let budget = &summary.budget;
    let payday1 = summary.until;
    let floor = plan.settings.savings_floor.unwrap_or(0);

    let spending_ids: Vec<String> =
        plan.accounts.iter().filter(|a| !a.locked && a.role == Role::Spending).map(|a| a.id.clone()).collect();
    let reserve_ids: Vec<String> =
        plan.accounts.iter().filter(|a| !a.locked && a.role == Role::Reserve).map(|a| a.id.clone()).collect();
    let spending_ids = if spending_ids.is_empty() {
        plan.accounts.iter().filter(|a| !a.locked && a.role == Role::Other).map(|a| a.id.clone()).collect()
    } else {
        spending_ids
    };
    let have_reserve = !reserve_ids.is_empty();
    let first_spend = spending_ids.first().cloned().unwrap_or_default();
    let first_reserve = reserve_ids.first().cloned();

    let mut sim = Sim {
        bal: plan.accounts.iter().map(|a| (a.id.clone(), a.balance)).collect(),
        deps: inp
            .deposits
            .iter()
            .map(|d| Dep {
                id: d.account_id.clone(),
                is_rd: d.is_rd,
                rate_bps: d.rate_bps,
                maturity: d.maturity,
                renewal: d.renewal,
                payout_to: d.payout_to.clone(),
                principal: plan.accounts.iter().find(|a| a.id == d.account_id).map_or(0, |a| a.balance),
            })
            .collect(),
        invested: 0,
        spending_ids,
        reserve_ids,
        first_spend,
        first_reserve,
    };

    // Scenario items become accounts + dated flows.
    struct Flow {
        date: Date,
        from: String,
        to: Option<String>,
        amount: i64,
    }
    let mut flows: Vec<Flow> = Vec::new();
    let mut item_accounts: HashMap<String, i64> = HashMap::new(); // name -> peak balance, filled later
    let mut item_ids: Vec<(String, String)> = Vec::new();
    for (n, it) in items.iter().enumerate() {
        let from = it.funded_from.clone().unwrap_or_else(|| sim.first_spend.clone());
        let payout = it.payout_to.clone().or_else(|| sim.first_reserve.clone()).or_else(|| Some(sim.first_spend.clone()));
        match it.kind {
            ItemKind::OneOff => flows.push(Flow { date: it.start, from, to: None, amount: it.amount_paise }),
            ItemKind::Fd | ItemKind::Rd => {
                let id = format!("__scenario_{n}");
                let tenure = tenure_of(it);
                sim.bal.insert(id.clone(), 0);
                sim.deps.push(Dep {
                    id: id.clone(),
                    is_rd: it.kind == ItemKind::Rd,
                    rate_bps: it.rate_bps.unwrap_or(0),
                    maturity: Some(plus_months(it.start, tenure)),
                    renewal: it.renewal.unwrap_or_default(),
                    payout_to: payout,
                    principal: 0,
                });
                if it.kind == ItemKind::Fd {
                    flows.push(Flow { date: it.start, from, to: Some(id.clone()), amount: it.amount_paise });
                } else {
                    for k in 0..tenure {
                        flows.push(Flow { date: plus_months(it.start, k), from: from.clone(), to: Some(id.clone()), amount: it.amount_paise });
                    }
                }
                item_ids.push((it.name.clone(), id));
            }
        }
    }

    // Existing EMIs: the first instalment is already in the next card bill.
    let emi_day = plan.cards.iter().find_map(|c| c.next_due_date).map_or(13, |d| d.day());
    let first_bill = plan.cards.iter().find_map(|c| c.next_due_date).unwrap_or(today);

    let mut points = vec![sim.point("start".to_string())];
    let mut stress: Vec<Stress> = Vec::new();
    // lowest balances seen this month: a mid-month dip matters even if the month ends fine
    let (mut min_spend, mut min_reserve) = (i64::MAX, i64::MAX);
    let mut d = today;
    while d <= end {
        let next = d + Duration::days(1);
        let month_end = next.month() != d.month();

        // salary
        if d > today {
            for inc in &plan.incomes {
                if clamped(d.year(), d.month(), inc.day) == d {
                    let to = inc.account_id.clone().unwrap_or_else(|| sim.first_spend.clone());
                    sim.add(&to, inc.amount);
                }
            }
        }
        // fixed payments
        for c in &plan.commitments {
            if commitment_dates(c, d, next).is_empty() {
                continue;
            }
            let from = match &c.account_id {
                Some(a) if sim.bal.contains_key(a) => a.clone(),
                _ if have_reserve && c.bucket == Bucket::Needs => sim.first_reserve.clone().unwrap_or_default(),
                _ => sim.first_spend.clone(),
            };
            sim.add(&from, -c.amount);
            match (&c.to_account_id, c.bucket) {
                (Some(to), _) => {
                    sim.add(to, c.amount);
                    if let Some(dep) = sim.deps.iter_mut().find(|x| &x.id == to) {
                        dep.principal += c.amount;
                    }
                }
                (None, Bucket::Savings) => sim.invested += c.amount,
                _ => {}
            }
        }
        // monthly top-up to the reserve (this period's was handled through the ledger)
        if have_reserve && d >= payday1 && clamped(d.year(), d.month(), plan.settings.allocation_day) == d {
            let (from, to) = (sim.first_spend.clone(), sim.first_reserve.clone().unwrap_or_default());
            sim.add(&from, -budget.allocation_paise);
            sim.add(&to, budget.allocation_paise);
        }
        // spending
        if d > today && d.day() == 15 {
            let spend = budget.spending_paise * inp.spend_pct / 100;
            let from = sim.first_spend.clone();
            sim.add(&from, -spend);
        }
        // existing card bills, paid from the reserve
        for c in &plan.cards {
            let reserve = sim.first_reserve.clone().unwrap_or_else(|| sim.first_spend.clone());
            if c.due_now > 0 && d == c.due_date.map_or(today, |x| x.max(today)) {
                sim.add(&reserve, -c.due_now);
            }
            if c.next_bill_est > 0 && Some(d) == c.next_due_date {
                sim.add(&reserve, -c.next_bill_est);
            }
        }
        // remaining EMI instalments
        if d.day() == emi_day.min(28) && d > first_bill {
            let k = (month_index(d) - month_index(first_bill)) as i64; // 1.. instalments after the first bill
            let reserve = sim.first_reserve.clone().unwrap_or_else(|| sim.first_spend.clone());
            for e in &inp.emis {
                if k >= 1 && k < e.months_left {
                    sim.add(&reserve, -e.monthly);
                }
            }
        }
        // scenario flows
        for f in flows.iter().filter(|f| f.date == d) {
            sim.add(&f.from, -f.amount);
            if let Some(to) = &f.to {
                sim.add(to, f.amount);
                if let Some(dep) = sim.deps.iter_mut().find(|x| &x.id == to) {
                    dep.principal += f.amount;
                }
            }
        }
        // deposits maturing today
        let maturing: Vec<usize> =
            sim.deps.iter().enumerate().filter(|(_, x)| x.maturity == Some(d)).map(|(i, _)| i).collect();
        for i in maturing {
            let (id, renewal, principal, payout) = {
                let x = &sim.deps[i];
                (x.id.clone(), x.renewal, x.principal, x.payout_to.clone())
            };
            let balance = sim.bal.get(&id).copied().unwrap_or(0);
            let to = payout.unwrap_or_else(|| sim.first_reserve.clone().unwrap_or_else(|| sim.first_spend.clone()));
            match renewal {
                Renewal::Payout => {
                    // peak value is recorded before this by the caller via item_accounts
                    item_accounts.entry(id.clone()).and_modify(|v| *v = (*v).max(balance)).or_insert(balance);
                    sim.bal.insert(id.clone(), 0);
                    sim.add(&to, balance);
                    sim.deps[i].maturity = None;
                }
                Renewal::RenewPrincipal => {
                    let interest = (balance - principal).max(0);
                    item_accounts.entry(id.clone()).and_modify(|v| *v = (*v).max(balance)).or_insert(balance);
                    sim.bal.insert(id.clone(), balance - interest);
                    sim.add(&to, interest);
                    sim.deps[i].maturity = sim.deps[i].maturity.map(|m| plus_months(m, 12));
                }
                Renewal::RenewAll => {
                    item_accounts.entry(id.clone()).and_modify(|v| *v = (*v).max(balance)).or_insert(balance);
                    sim.deps[i].principal = balance;
                    sim.deps[i].maturity = sim.deps[i].maturity.map(|m| plus_months(m, 12));
                }
            }
        }

        min_spend = min_spend.min(sim.sum(&sim.spending_ids));
        min_reserve = min_reserve.min(sim.sum(&sim.reserve_ids));

        // month end: interest, then record
        if month_end {
            for dep in &sim.deps {
                let b = sim.bal.get(&dep.id).copied().unwrap_or(0);
                if b > 0 && dep.rate_bps > 0 {
                    let grown = (b as f64 * monthly_factor(dep.rate_bps)).round() as i64;
                    sim.bal.insert(dep.id.clone(), grown);
                }
            }
            for dep in &sim.deps {
                let b = sim.bal.get(&dep.id).copied().unwrap_or(0);
                let e = item_accounts.entry(dep.id.clone()).or_insert(0);
                *e = (*e).max(b);
            }
            let key = month_key(d);
            let p = sim.point(key.clone());
            if min_spend < 0 {
                stress.push(Stress { month: key.clone(), kind: "spending_short", amount_paise: -min_spend });
            }
            if have_reserve && min_reserve < 0 {
                stress.push(Stress { month: key.clone(), kind: "fd_needed", amount_paise: -min_reserve });
            } else if have_reserve && min_reserve < floor {
                stress.push(Stress { month: key.clone(), kind: "reserve_below_floor", amount_paise: floor - min_reserve });
            }
            (min_spend, min_reserve) = (i64::MAX, i64::MAX);
            points.push(p);
        }
        d = next;
    }

    // peak value reached by each scenario deposit, keyed by item name
    let mut by_name: HashMap<String, i64> = HashMap::new();
    for (name, id) in item_ids {
        by_name.insert(name, item_accounts.get(&id).copied().unwrap_or(0));
    }
    (Series { points, stress }, by_name)
}

/// Baseline projection, plus the same projection with `items` applied and what each item does.
pub fn project(inp: &ProjectionInput, items: &[ScenarioItem]) -> Projection {
    // Run long enough for every item to mature, so its final value and goal check are real.
    let today = inp.plan.today;
    let needed = items
        .iter()
        .filter(|it| it.kind != ItemKind::OneOff)
        .map(|it| {
            let end = plus_months(it.start, tenure_of(it));
            i64::from(month_index(end) - month_index(today)) + 1
        })
        .max()
        .unwrap_or(0);
    let mut inp = inp.clone();
    inp.months = inp.months.max(needed).min(MAX_MONTHS);
    let inp = &inp;
    let (baseline, _) = simulate(inp, &[]);
    if items.is_empty() {
        return Projection { baseline, scenario: None, impacts: Vec::new() };
    }
    let (scenario, peaks) = simulate(inp, items);
    let budget = compute(&inp.plan).budget;

    let impacts = items
        .iter()
        .map(|it| {
            let tenure = tenure_of(it);
            let monthly =if it.kind == ItemKind::Rd { it.amount_paise } else { 0 };
            let (rest, sip, spend) = (budget.rest_savings_paise, budget.sip_paise, budget.spending_paise);
            let sf = (monthly > 0).then(|| absorb(rest, sip, spend, monthly, [Layer::Rest, Layer::Sip, Layer::Spending]));
            let sp = (monthly > 0).then(|| absorb(rest, sip, spend, monthly, [Layer::Spending, Layer::Rest, Layer::Sip]));
            let thresholds = if monthly > 0 {
                vec![
                    Threshold { label: "Only slows your monthly savings top-up".into(), up_to_paise: rest },
                    Threshold { label: "Starts cutting your SIP".into(), up_to_paise: rest + sip },
                    Threshold { label: "Starts eating your spending allowance".into(), up_to_paise: rest + sip + spend },
                    Threshold { label: "Beyond this an FD would have to be touched".into(), up_to_paise: rest + sip + spend },
                ]
            } else {
                Vec::new()
            };
            let maturity = peaks.get(&it.name).copied().unwrap_or(0);
            let goal = match (it.target_paise, it.target_date) {
                (Some(target), Some(target_date)) => {
                    let projected = match it.kind {
                        ItemKind::OneOff => 0,
                        _ => maturity,
                    };
                    let required = (it.kind == ItemKind::Rd).then(|| rd_required_monthly(target, it.rate_bps.unwrap_or(0), tenure));
                    Some(GoalCheck {
                        target_paise: target,
                        target_date,
                        projected_paise: projected,
                        gap_paise: projected - target,
                        required_monthly_paise: required,
                    })
                }
                _ => None,
            };
            ItemImpact {
                name: it.name.clone(),
                monthly_paise: monthly,
                savings_first: sf,
                spending_first: sp,
                savings_first_thresholds: thresholds,
                maturity_value_paise: maturity,
                goal,
            }
        })
        .collect();

    Projection { baseline, scenario: Some(scenario), impacts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::*;
    use time::macros::date;

    const HDFC: &str = "hdfc";
    const KOTAK: &str = "kotak";
    const FD1: &str = "fd1";

    fn commitment(label: &str, rupees: i64, day: u8, acct: &str, bucket: Bucket) -> CommitmentIn {
        CommitmentIn {
            label: label.into(),
            amount: rupees * 100,
            day,
            account_id: Some(acct.into()),
            active: true,
            bucket,
            freq: Freq::Monthly,
            due_month: None,
            to_account_id: None,
            start_month: None,
            end_month: None,
        }
    }

    fn input() -> ProjectionInput {
        let today = date!(2026 - 10 - 05);
        let acct = |id: &str, name: &str, bal: i64, locked: bool, role: Role| AccountIn {
            id: id.into(),
            name: name.into(),
            balance: bal,
            locked,
            role,
            updated: Some(today),
        };
        ProjectionInput {
            plan: PlanInput {
                today,
                accounts: vec![
                    acct(HDFC, "HDFC", 4_301_699, false, Role::Spending),
                    acct(KOTAK, "Kotak", 6_914_400, false, Role::Reserve),
                    acct(FD1, "iPhone FD", 8_738_500, true, Role::Other),
                ],
                incomes: vec![IncomeIn { label: "Salary".into(), amount: 7_549_900, day: 30, account_id: Some(HDFC.into()) }],
                commitments: vec![
                    commitment("House", 4_000, 1, KOTAK, Bucket::Needs),
                    commitment("SIP", 30_000, 10, HDFC, Bucket::Savings),
                ],
                cards: vec![CardIn {
                    card_id: "c1".into(),
                    name: "Main".into(),
                    due_now: 0,
                    due_date: Some(date!(2026 - 10 - 13)),
                    next_bill_est: 2_306_700,
                    next_due_date: Some(date!(2026 - 11 - 13)),
                    last_updated: Some(today),
                    pay_from: Some(KOTAK.into()),
                    due_covered: 0,
                    next_covered: 2_306_700,
                    spent_this_period: Some(0),
                    tracked_since: Some(today),
                }],
                settings: SettingsIn { spending: Some(1_887_500), spending_bps: None, allocation_day: 10, savings_floor: Some(0), recovery_months: 3 },
                allocation: AllocationState { moved: 0, skipped: 2_662_400 },
            },
            deposits: vec![DepositIn {
                account_id: FD1.into(),
                is_rd: false,
                rate_bps: 710,
                maturity: Some(date!(2026 - 11 - 22)),
                renewal: Renewal::Payout,
                payout_to: Some(KOTAK.into()),
            }],
            emis: vec![EmiIn { monthly: 381_500, months_left: 4 }],
            months: 6,
            spend_pct: 100,
        }
    }

    fn at<'a>(s: &'a Series, month: &str) -> &'a Point {
        s.points.iter().find(|p| p.month == month).unwrap_or_else(|| panic!("no point for {month}"))
    }

    #[test]
    fn interest_compounds_quarterly() {
        // 1,00,000 at 7.1% for a year: (1 + 0.071/4)^4
        let v = fd_value(10_000_000, 710, 12);
        assert!((v - 10_729_100).abs() < 500, "got {v}");
        assert_eq!(fd_value(10_000_000, 0, 12), 10_000_000);
    }

    #[test]
    fn rd_maturity_and_required_monthly_round_trip() {
        let m = rd_maturity(1_000_000, 700, 12); // ₹10,000 a month for a year at 7%
        assert!((12_400_000..12_550_000).contains(&m), "got {m}");
        let need = rd_required_monthly(m, 700, 12);
        assert!((need - 1_000_000).abs() <= 2, "got {need}");
        assert_eq!(rd_maturity(1_000_000, 700, 0), 0);
    }

    #[test]
    fn baseline_walks_the_months_forward() {
        let p = project(&input(), &[]).baseline;
        assert_eq!(p.points.len(), 7); // start + 6 month ends
        // Oct: 43,016.99 - SIP 30,000 - spending 18,875 on the 15th + salary on the 30th
        assert_eq!(at(&p, "2026-10").spending_paise, 4_301_699 - 3_000_000 - 1_887_500 + 7_549_900);
        // the Oct 30 salary came in; Kotak lost nothing until the 1st (house) and the 13th (bill)
        assert_eq!(at(&p, "2026-10").reserve_paise, 6_914_400);
        // Nov: house, the card bill and 1 EMI instalment leave Kotak; the 10th top-up arrives
        let nov = at(&p, "2026-11");
        // The iPhone FD's own balance (locked, role Other) is not in Kotak; only its payout is.
        assert_eq!(nov.reserve_paise, 6_914_400 - 400_000 - 2_306_700 + 2_662_400 + fd_payout(&p));
    }

    // The iPhone FD (₹87,385 at 7.1%, "payout") matures on 22 Nov into Kotak with a month's interest
    // already added at 31 Oct. Recompute that value the same way the engine does.
    fn fd_payout(_p: &Series) -> i64 {
        (8_738_500_f64 * monthly_factor(710)).round() as i64
    }

    #[test]
    fn deposits_pay_out_renew_or_compound() {
        let mut i = input();
        let payout = project(&i, &[]).baseline;
        assert_eq!(at(&payout, "2026-12").fds_paise, 0); // paid out to Kotak on 22 Nov

        i.deposits[0].renewal = Renewal::RenewAll;
        let compound = project(&i, &[]).baseline;
        assert!(at(&compound, "2026-12").fds_paise > 8_738_500);
        assert!(at(&compound, "2026-12").reserve_paise < at(&payout, "2026-12").reserve_paise);

        i.deposits[0].renewal = Renewal::RenewPrincipal; // interest pays out each year, principal stays
        let principal = project(&i, &[]).baseline;
        assert!(at(&principal, "2026-12").fds_paise >= 8_738_500);
        // payout hands Kotak the whole balance; renew_principal only the interest
        assert_eq!(at(&principal, "2026-12").reserve_paise, at(&payout, "2026-12").reserve_paise - 8_738_500);
    }

    #[test]
    fn spending_short_months_are_flagged() {
        let p = project(&input(), &[]).baseline;
        // spending the full allowance on 15 Oct while HDFC only has ₹13,017 after the SIP: the month
        // ends fine (salary lands on the 30th) but dips below zero in between
        let oct = p.stress.iter().find(|s| s.kind == "spending_short" && s.month == "2026-10").unwrap();
        assert_eq!(oct.amount_paise, 1_887_500 + 3_000_000 - 4_301_699);
        assert!(at(&p, "2026-10").spending_paise > 0);
    }

    #[test]
    fn an_rd_moves_money_into_a_deposit_and_tightens_the_months() {
        let rd = ScenarioItem {
            name: "Wedding RD".into(),
            kind: ItemKind::Rd,
            amount_paise: 1_000_000,
            start: date!(2026 - 11 - 10),
            tenure_months: Some(4),
            rate_bps: Some(700),
            funded_from: None,
            payout_to: None,
            renewal: None,
            target_paise: Some(4_200_000),
            target_date: Some(date!(2027 - 03 - 10)),
        };
        let out = project(&input(), &[rd]);
        let sc = out.scenario.unwrap();
        assert!(at(&sc, "2026-12").rds_paise > 2_000_000);
        assert!(at(&sc, "2026-12").spending_paise < at(&out.baseline, "2026-12").spending_paise);
        let imp = &out.impacts[0];
        assert_eq!(imp.monthly_paise, 1_000_000);
        // 10,000 a month fits inside the 19,624 of rest-of-savings: only the top-up shrinks
        let sf = imp.savings_first.as_ref().unwrap();
        assert_eq!((sf.from_rest_savings_paise, sf.from_sip_paise, sf.from_spending_paise, sf.from_fd_paise), (1_000_000, 0, 0, 0));
        // if spends give way first it comes out of the 18,875 allowance
        let sp = imp.spending_first.as_ref().unwrap();
        assert_eq!((sp.from_spending_paise, sp.from_rest_savings_paise), (1_000_000, 0));
        assert!(imp.maturity_value_paise > 4_000_000);
        let g = imp.goal.as_ref().unwrap();
        assert_eq!(g.projected_paise, imp.maturity_value_paise);
        assert!(g.required_monthly_paise.unwrap() > 0);
    }

    #[test]
    fn horizon_stretches_to_cover_a_deposit_that_matures_later() {
        let mut i = input();
        i.months = 6;
        let rd = ScenarioItem {
            name: "RD".into(),
            kind: ItemKind::Rd,
            amount_paise: 1_000_000,
            start: date!(2026 - 11 - 10),
            tenure_months: Some(12),
            rate_bps: Some(700),
            funded_from: None,
            payout_to: None,
            renewal: None,
            target_paise: Some(12_400_000),
            target_date: Some(date!(2027 - 11 - 10)),
        };
        let out = project(&i, &[rd]);
        assert!(out.scenario.as_ref().unwrap().points.len() > 12);
        let imp = &out.impacts[0];
        // twelve instalments at 7%: matches the closed-form RD value
        assert!((imp.maturity_value_paise - rd_maturity(1_000_000, 700, 12)).abs() < 5_000, "{}", imp.maturity_value_paise);
        assert!(imp.goal.as_ref().unwrap().gap_paise.abs() < 200_000);
    }

    #[test]
    fn absorption_thresholds_in_order() {
        // rest 19,624, SIP 30,000, spending 18,875
        let a = absorb(1_962_400, 3_000_000, 1_887_500, 2_500_000, [Layer::Rest, Layer::Sip, Layer::Spending]);
        assert_eq!((a.from_rest_savings_paise, a.from_sip_paise, a.from_spending_paise, a.from_fd_paise), (1_962_400, 537_600, 0, 0));
        let b = absorb(1_962_400, 3_000_000, 1_887_500, 2_500_000, [Layer::Spending, Layer::Rest, Layer::Sip]);
        assert_eq!((b.from_spending_paise, b.from_rest_savings_paise, b.from_sip_paise), (1_887_500, 612_500, 0));
        let c = absorb(100, 100, 100, 1_000, [Layer::Rest, Layer::Sip, Layer::Spending]);
        assert_eq!(c.from_fd_paise, 700);
    }

    #[test]
    fn a_lump_fd_and_a_one_off_behave() {
        let fd = ScenarioItem {
            name: "FD".into(),
            kind: ItemKind::Fd,
            amount_paise: 3_000_000,
            start: date!(2026 - 11 - 01),
            tenure_months: Some(12),
            rate_bps: Some(710),
            funded_from: Some(KOTAK.into()),
            payout_to: None,
            renewal: Some(Renewal::RenewAll),
            target_paise: None,
            target_date: None,
        };
        let oneoff = ScenarioItem {
            name: "Gadget".into(),
            kind: ItemKind::OneOff,
            amount_paise: 500_000,
            start: date!(2026 - 12 - 05),
            tenure_months: None,
            rate_bps: None,
            funded_from: None,
            payout_to: None,
            renewal: None,
            target_paise: None,
            target_date: None,
        };
        let out = project(&input(), &[fd, oneoff]);
        let sc = out.scenario.unwrap();
        assert!(at(&sc, "2026-12").fds_paise > at(&out.baseline, "2026-12").fds_paise + 3_000_000);
        // total wealth falls only by the one-off spend (plus a little FD interest gained)
        let diff = at(&sc, "2026-12").total_paise - at(&out.baseline, "2026-12").total_paise;
        assert!((-500_000..-400_000).contains(&diff) || diff > -500_000, "diff {diff}");
        assert_eq!(out.impacts[1].monthly_paise, 0);
    }

    #[test]
    fn huge_tenure_does_not_panic_or_build_a_flow_per_century() {
        let item = |kind: ItemKind, tenure: i64, start: Date| ScenarioItem {
            name: "Huge".into(),
            kind,
            amount_paise: 1_000_000,
            start,
            tenure_months: Some(tenure),
            rate_bps: Some(700),
            funded_from: None,
            payout_to: None,
            renewal: Some(Renewal::RenewAll),
            target_paise: Some(i64::MAX),
            target_date: Some(date!(9999 - 12 - 31)),
        };
        for tenure in [i64::MAX, i64::MIN, 10_000_000_000, 1_000_000] {
            let out = project(&input(), &[item(ItemKind::Rd, tenure, date!(2026 - 11 - 10)), item(ItemKind::Fd, tenure, date!(2026 - 11 - 10))]);
            assert!(out.scenario.is_some());
            assert_eq!(out.baseline.points.len(), out.scenario.as_ref().unwrap().points.len());
        }
        // Starting right at the top of the representable calendar must not panic either.
        let late = project(&input(), &[item(ItemKind::Rd, 1200, date!(9999 - 12 - 01))]);
        assert!(late.scenario.is_some());
    }
}
