//! "What to do, what it changes, why": a plain month plan from a salary, the fixed costs, the SIP,
//! a strict share of salary for spending, and any shared spending. Pure: no DB, no clock.
//!
//! Money is integer paise and the spending share is basis points. The leftover after fixed costs,
//! the SIP and the spending cap goes to the emergency fund until it covers `months` of what you
//! really spend (fixed costs plus the spending cap, not the SIP, which is saving).

use serde::{Deserialize, Serialize};

use super::format_inr;
use crate::validate::MAX_AMOUNT_PAISE;

pub const DEFAULT_COVER_MONTHS: i64 = 6;
pub const MAX_LINES: usize = 20;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Line {
    pub label: String,
    pub amount_paise: i64,
}

/// Something inside the spending share that someone else helps pay for.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Shared {
    pub label: String,
    pub total_paise: i64,
    pub partner_paise: i64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Input {
    pub salary_paise: i64,
    pub fixed: Vec<Line>,
    pub sip_paise: i64,
    pub spend_bps: i64,
    #[serde(default)]
    pub shared: Vec<Shared>,
    #[serde(default)]
    pub emergency_saved_paise: i64,
    #[serde(default)]
    pub cover_months: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Step {
    pub what: String,
    pub impact: String,
    pub why: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Guidance {
    pub spend_cap_paise: i64,
    pub shared_mine_paise: i64,
    pub free_to_spend_paise: i64,
    pub fixed_paise: i64,
    pub surplus_paise: i64,
    pub monthly_cost_paise: i64,
    pub emergency_target_paise: i64,
    /// None when nothing is left over to build it with.
    pub months_to_emergency: Option<i64>,
    pub steps: Vec<Step>,
    pub warnings: Vec<String>,
}

pub fn validate(i: &Input) -> Result<(), String> {
    let ok = |v: i64| (0..=MAX_AMOUNT_PAISE).contains(&v);
    if i.salary_paise <= 0 || !ok(i.salary_paise) {
        return Err("Salary must be more than zero.".into());
    }
    if !(0..=10_000).contains(&i.spend_bps) {
        return Err("Spending share must be between 0% and 100%.".into());
    }
    if i.fixed.len() > MAX_LINES || i.shared.len() > MAX_LINES {
        return Err("Too many lines.".into());
    }
    if !ok(i.sip_paise) || !ok(i.emergency_saved_paise) || i.fixed.iter().any(|l| !ok(l.amount_paise)) {
        return Err("Amounts can't be negative or too large.".into());
    }
    for s in &i.shared {
        if !ok(s.total_paise) || s.partner_paise < 0 || s.partner_paise > s.total_paise {
            return Err(format!("{}: the partner's part must be between zero and the total.", s.label));
        }
    }
    if i.cover_months.is_some_and(|m| !(1..=24).contains(&m)) {
        return Err("Cover months must be between 1 and 24.".into());
    }
    Ok(())
}

fn ceil_div(a: i64, b: i64) -> i64 {
    (a + b - 1) / b
}

pub fn plan(i: &Input) -> Guidance {
    let spend_cap = (i.salary_paise as i128 * i.spend_bps as i128 / 10_000) as i64;
    let fixed: i64 = i.fixed.iter().map(|l| l.amount_paise).sum();
    let mine: i64 = i.shared.iter().map(|s| s.total_paise - s.partner_paise).sum();
    let free = spend_cap - mine;
    let surplus = i.salary_paise - fixed - i.sip_paise - spend_cap;
    let monthly_cost = fixed + spend_cap;
    let months = i.cover_months.unwrap_or(DEFAULT_COVER_MONTHS);
    let target = monthly_cost * months;
    let gap = (target - i.emergency_saved_paise).max(0);
    let to_goal = if gap == 0 {
        Some(0)
    } else if surplus > 0 {
        Some(ceil_div(gap, surplus))
    } else {
        None
    };

    let mut warnings = Vec::new();
    if mine > spend_cap {
        warnings.push(format!(
            "Your share of the shared spending ({}) is more than your whole spending limit ({}). Something has to give.",
            format_inr(mine),
            format_inr(spend_cap)
        ));
    }
    if surplus < 0 {
        warnings.push(format!("Fixed costs, the SIP and the spending limit add up to {} more than your salary.", format_inr(-surplus)));
    }

    let mut steps = Vec::new();
    steps.push(Step {
        what: format!("Spend no more than {} a month, in total, on everything that is not fixed.", format_inr(spend_cap)),
        impact: if mine > 0 {
            format!("{} of it is your share of the shared spending, which leaves {} for everything else.", format_inr(mine), format_inr(free.max(0)))
        } else {
            "That is your whole spending budget for the month.".to_string()
        },
        why: "A hard limit is what makes the saving below happen. Without it the leftover gets spent before it is saved.".into(),
    });
    if i.sip_paise > 0 {
        steps.push(Step {
            what: format!("Keep the SIP at {} a month, on salary day.", format_inr(i.sip_paise)),
            impact: format!("It is {}% of your salary going to long-term growth before you can spend it.", i.sip_paise * 100 / i.salary_paise),
            why: "Money invested early has the longest time to grow, so this is the one saving you should not pause.".into(),
        });
    }
    match (to_goal, surplus) {
        (Some(0), _) => steps.push(Step {
            what: "Your emergency fund is already full. Send the leftover to the SIP or a goal.".into(),
            impact: format!("{} a month is free to grow instead of sitting idle.", format_inr(surplus.max(0))),
            why: "Cash above what an emergency needs earns less than the same money invested.".into(),
        }),
        (Some(n), s) => steps.push(Step {
            what: format!("Move {} a month into an emergency fund, on salary day.", format_inr(s)),
            impact: format!(
                "In {} it holds {}, which is {} months of what you really spend ({} a month).",
                months_text(n),
                format_inr(target),
                months,
                format_inr(monthly_cost)
            ),
            why: "A job loss or a hospital bill would otherwise land on credit or force you to sell investments at a bad time. Because your spending is capped, the fund is sized to that and stays small.".into(),
        }),
        (None, _) => steps.push(Step {
            what: "Nothing is left over for an emergency fund yet.".into(),
            impact: "Until something is, one bad month falls straight onto credit.".into(),
            why: "Lower the spending share, a fixed cost or the SIP a little until the leftover is above zero.".into(),
        }),
    }
    Guidance {
        spend_cap_paise: spend_cap,
        shared_mine_paise: mine,
        free_to_spend_paise: free,
        fixed_paise: fixed,
        surplus_paise: surplus,
        monthly_cost_paise: monthly_cost,
        emergency_target_paise: target,
        months_to_emergency: to_goal,
        steps,
        warnings,
    }
}

fn months_text(n: i64) -> String {
    if n >= 24 && n % 12 == 0 {
        format!("{} years", n / 12)
    } else if n > 12 {
        format!("{} months (about {:.1} years)", n, n as f64 / 12.0)
    } else {
        format!("{n} month{}", if n == 1 { "" } else { "s" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rs(r: i64) -> i64 {
        r * 100
    }

    fn input() -> Input {
        Input {
            salary_paise: rs(74_000),
            fixed: vec![
                Line { label: "Transport".into(), amount_paise: rs(2_000) },
                Line { label: "House".into(), amount_paise: rs(4_000) },
                Line { label: "Others".into(), amount_paise: rs(5_000) },
                Line { label: "Donation".into(), amount_paise: rs(1_000) },
            ],
            sip_paise: rs(25_000),
            spend_bps: 3_000,
            shared: vec![Shared { label: "Together".into(), total_paise: rs(18_000), partner_paise: rs(6_000) }],
            emergency_saved_paise: 0,
            cover_months: None,
        }
    }

    #[test]
    fn the_worked_example() {
        let g = plan(&input());
        assert_eq!(g.spend_cap_paise, rs(22_200));
        assert_eq!(g.shared_mine_paise, rs(12_000));
        assert_eq!(g.free_to_spend_paise, rs(10_200));
        assert_eq!(g.surplus_paise, rs(14_800));
        assert_eq!(g.monthly_cost_paise, rs(34_200));
        assert_eq!(g.emergency_target_paise, rs(205_200));
        assert_eq!(g.months_to_emergency, Some(14));
        assert_eq!(g.steps.len(), 3);
        assert!(g.warnings.is_empty());
    }

    #[test]
    fn savings_already_made_shorten_the_wait_and_a_full_fund_frees_the_money() {
        let g = plan(&Input { emergency_saved_paise: rs(100_000), ..input() });
        assert_eq!(g.months_to_emergency, Some(8));
        let full = plan(&Input { emergency_saved_paise: rs(300_000), ..input() });
        assert_eq!(full.months_to_emergency, Some(0));
    }

    #[test]
    fn overspending_the_plan_is_called_out() {
        let g = plan(&Input { sip_paise: rs(50_000), ..input() });
        assert!(g.surplus_paise < 0);
        assert_eq!(g.months_to_emergency, None);
        assert_eq!(g.warnings.len(), 1);
        let big = plan(&Input { shared: vec![Shared { label: "x".into(), total_paise: rs(30_000), partner_paise: 0 }], ..input() });
        assert_eq!(big.warnings.len(), 1);
    }

    #[test]
    fn bad_input_is_refused() {
        assert!(validate(&input()).is_ok());
        assert!(validate(&Input { salary_paise: 0, ..input() }).is_err());
        assert!(validate(&Input { spend_bps: 10_001, ..input() }).is_err());
        let bad = Shared { label: "x".into(), total_paise: 100, partner_paise: 200 };
        assert!(validate(&Input { shared: vec![bad], ..input() }).is_err());
        assert!(validate(&Input { cover_months: Some(0), ..input() }).is_err());
    }
}
