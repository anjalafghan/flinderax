//! The road ahead: one dated list of what is coming, drawn from the life plan (a wedding, a child,
//! retirement), the savings goals, the big one-off payments of the year view and the insurance
//! gaps, with what each needs a month from now. Where the profile leaves a big life event undated
//! it says so as a prompt instead of guessing a date.
//! Pure: no DB, no clock; `today` comes in. Money is integer paise.

use serde::Serialize;
use time::{Date, Month};

use super::life::LifePlan;
use super::year_ahead::{next_occurrence, Direction, Lumpy};

/// One-off payments further away than this are left to the year view.
const PAYMENT_HORIZON_MONTHS: i64 = 24;
/// An insurance gap is put this many months out: soon, but with room to compare policies.
const COVER_IN_MONTHS: i64 = 3;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Area {
    Money,
    Family,
    Health,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// From the life plan (`/life`).
    Life,
    /// A savings goal with a date.
    Goal,
    /// A dated one-off payment from the year view.
    Payment,
    /// Cover the life plan says is missing.
    Cover,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Milestone {
    /// "YYYY-MM-DD".
    pub on: String,
    pub title: String,
    pub area: Area,
    pub source: Source,
    /// What it costs on the day (future rupees for life goals). 0 when unknown.
    pub cost_paise: i64,
    pub saved_paise: i64,
    /// What it needs each month from now: the life plan's SIP, a goal's own figure, or the cost
    /// spread evenly over the months left.
    pub per_month_paise: i64,
    pub months_away: i64,
    /// The page where it is set or changed.
    pub link: &'static str,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Prompt {
    pub title: &'static str,
    pub body: &'static str,
    pub link: &'static str,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Road {
    pub milestones: Vec<Milestone>,
    pub prompts: Vec<Prompt>,
    /// The sum of every milestone's monthly need: what saving for all of them at once takes.
    pub pace_all_paise: i64,
    /// The single largest monthly need, if any.
    pub biggest: Option<Milestone>,
}

/// A savings goal as the road needs it.
#[derive(Clone, Debug, PartialEq)]
pub struct GoalIn {
    pub name: String,
    pub target_paise: i64,
    pub saved_paise: i64,
    pub target_date: Option<Date>,
    pub needs_monthly_paise: i64,
}

pub struct Inputs<'a> {
    pub today: Date,
    pub life: &'a LifePlan,
    pub wedding_dated: bool,
    pub child_dated: bool,
    pub goals: &'a [GoalIn],
    pub payments: &'a [Lumpy],
}

/// Calendar months from `today`'s month to `on`'s month (0 for this month, never negative).
pub fn months_between(today: Date, on: Date) -> i64 {
    let a = i64::from(today.year()) * 12 + i64::from(today.month() as u8);
    let b = i64::from(on.year()) * 12 + i64::from(on.month() as u8);
    (b - a).max(0)
}

fn spread(cost: i64, months: i64) -> i64 {
    if cost <= 0 {
        return 0;
    }
    let m = months.max(1);
    cost.saturating_add(m - 1) / m
}

fn add_months(d: Date, n: i64) -> Date {
    let total = i64::from(d.year()) * 12 + i64::from(d.month() as u8) - 1 + n;
    let year = i32::try_from(total.div_euclid(12)).unwrap_or(d.year());
    let month = Month::try_from(u8::try_from(total.rem_euclid(12) + 1).unwrap_or(1)).unwrap_or(Month::January);
    Date::from_calendar_date(year, month, 1).unwrap_or(d)
}

fn parse_day(s: &str) -> Option<Date> {
    crate::planner::dates::parse_ymd_prefix(s)
}

pub fn road(i: &Inputs) -> Road {
    let today = i.today;
    let mut out: Vec<Milestone> = Vec::new();

    for g in &i.life.goals {
        // The emergency fund is a savings goal of its own; retirement keeps its own page.
        if g.key == "emergency" {
            continue;
        }
        let Some(on) = g.when.as_deref().and_then(parse_day) else { continue };
        if on < today {
            continue;
        }
        let area = if g.key == "retirement" { Area::Money } else { Area::Family };
        out.push(Milestone {
            on: on.to_string(),
            title: g.label.to_string(),
            area,
            source: Source::Life,
            cost_paise: g.future_cost_paise,
            saved_paise: g.saved_paise,
            per_month_paise: g.monthly_sip_paise.max(0),
            months_away: months_between(today, on),
            link: "/life",
        });
    }

    for g in i.goals {
        let Some(on) = g.target_date else { continue };
        if on < today || g.saved_paise >= g.target_paise {
            continue;
        }
        out.push(Milestone {
            on: on.to_string(),
            title: g.name.clone(),
            area: Area::Money,
            source: Source::Goal,
            cost_paise: g.target_paise,
            saved_paise: g.saved_paise,
            per_month_paise: g.needs_monthly_paise.max(0),
            months_away: months_between(today, on),
            link: "/plan",
        });
    }

    for p in i.payments {
        if p.direction != Direction::Out {
            continue;
        }
        let Some(on) = next_occurrence(p, today) else { continue };
        let months = months_between(today, on);
        if months > PAYMENT_HORIZON_MONTHS {
            continue;
        }
        let area = match p.kind.as_str() {
            "wedding" => Area::Family,
            "premium" => Area::Health,
            _ => Area::Money,
        };
        out.push(Milestone {
            on: on.to_string(),
            title: p.label.clone(),
            area,
            source: Source::Payment,
            cost_paise: p.amount_paise,
            saved_paise: 0,
            per_month_paise: spread(p.amount_paise, months),
            months_away: months,
            link: "/year",
        });
    }

    let cover_on = add_months(today, COVER_IN_MONTHS);
    let ins = &i.life.insurance;
    if ins.term_gap_paise > 0 {
        out.push(Milestone {
            on: cover_on.to_string(),
            title: format!("Term cover of {}", super::life::inr(ins.term_needed_paise)),
            area: Area::Health,
            source: Source::Cover,
            cost_paise: 0,
            saved_paise: 0,
            per_month_paise: 0,
            months_away: COVER_IN_MONTHS,
            link: "/life",
        });
    }
    if ins.health_gap_paise > 0 {
        out.push(Milestone {
            on: cover_on.to_string(),
            title: format!("Health cover of {}", super::life::inr(ins.health_needed_paise)),
            area: Area::Health,
            source: Source::Cover,
            cost_paise: 0,
            saved_paise: 0,
            per_month_paise: 0,
            months_away: COVER_IN_MONTHS,
            link: "/life",
        });
    }

    out.sort_by(|a, b| a.on.cmp(&b.on).then_with(|| a.title.cmp(&b.title)));

    let mut prompts = Vec::new();
    if !i.wedding_dated {
        prompts.push(Prompt {
            title: "Give the wedding a date",
            body: "With a date and a budget, joy prices it in the rupees of that year and puts it on the road.",
            link: "/life",
        });
    }
    if !i.child_dated {
        prompts.push(Prompt {
            title: "Pick a year for a child",
            body: "A birth year lets joy plan the first year's costs and the education fund.",
            link: "/life",
        });
    }

    let pace_all_paise = out.iter().map(|m| m.per_month_paise).fold(0i64, i64::saturating_add);
    let biggest = out.iter().filter(|m| m.per_month_paise > 0).max_by_key(|m| m.per_month_paise).cloned();
    Road { milestones: out, prompts, pace_all_paise, biggest }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::life::{plan, LifeProfile};
    use time::macros::date;

    fn life(profile: &LifeProfile, today: Date) -> LifePlan {
        plan(profile, 4_000_000, today)
    }

    #[test]
    fn undated_life_events_become_prompts() {
        let today = date!(2026 - 11 - 25);
        let lp = life(&LifeProfile { my_take_home_paise: 7_500_000, ..LifeProfile::default() }, today);
        let r = road(&Inputs { today, life: &lp, wedding_dated: false, child_dated: false, goals: &[], payments: &[] });
        assert_eq!(r.prompts.len(), 2);
        assert!(r.milestones.iter().all(|m| m.title != "Wedding"));
    }

    #[test]
    fn a_dated_wedding_is_on_the_road_with_its_sip() {
        let today = date!(2026 - 11 - 25);
        let profile = LifeProfile {
            my_take_home_paise: 7_500_000,
            wedding_date: Some("2029-02-14".into()),
            wedding_budget_paise: Some(60_000_000),
            ..LifeProfile::default()
        };
        let lp = life(&profile, today);
        let r = road(&Inputs { today, life: &lp, wedding_dated: true, child_dated: false, goals: &[], payments: &[] });
        let w = r.milestones.iter().find(|m| m.source == Source::Life && m.area == Area::Family).expect("wedding");
        assert_eq!(w.months_away, 27);
        assert!(w.per_month_paise > 0);
        assert!(w.cost_paise > 60_000_000, "priced in the rupees of 2029");
        assert_eq!(r.prompts.len(), 1);
    }

    #[test]
    fn goals_and_payments_are_sorted_and_paced() {
        let today = date!(2026 - 11 - 25);
        let lp = life(&LifeProfile::default(), today);
        let goals = [
            GoalIn { name: "Emergency fund".into(), target_paise: 10_000_000, saved_paise: 0, target_date: Some(date!(2027 - 09 - 01)), needs_monthly_paise: 909_091 },
            GoalIn { name: "Done already".into(), target_paise: 100, saved_paise: 100, target_date: Some(date!(2027 - 01 - 01)), needs_monthly_paise: 0 },
            GoalIn { name: "No date".into(), target_paise: 100, saved_paise: 0, target_date: None, needs_monthly_paise: 0 },
        ];
        let payments = [
            Lumpy { on: date!(2027 - 03 - 10), label: "Car insurance".into(), amount_paise: 1_200_000, direction: Direction::Out, yearly: false, kind: "premium".into() },
            Lumpy { on: date!(2027 - 01 - 05), label: "Bonus".into(), amount_paise: 5_000_000, direction: Direction::In, yearly: false, kind: "other".into() },
            Lumpy { on: date!(2030 - 01 - 05), label: "Far away".into(), amount_paise: 100, direction: Direction::Out, yearly: false, kind: "other".into() },
        ];
        let r = road(&Inputs { today, life: &lp, wedding_dated: true, child_dated: true, goals: &goals, payments: &payments });
        let titles: Vec<&str> = r.milestones.iter().filter(|m| m.source != Source::Life && m.source != Source::Cover).map(|m| m.title.as_str()).collect();
        assert_eq!(titles, vec!["Car insurance", "Emergency fund"]);
        let car = &r.milestones.iter().find(|m| m.title == "Car insurance").unwrap();
        assert_eq!(car.months_away, 4);
        assert_eq!(car.per_month_paise, 300_000);
        assert_eq!(car.area, Area::Health);
        assert!(r.pace_all_paise >= 909_091 + 300_000);
        assert!(r.prompts.is_empty());
    }

    #[test]
    fn insurance_gaps_land_three_months_out() {
        let today = date!(2026 - 11 - 25);
        let lp = life(&LifeProfile { my_take_home_paise: 7_500_000, ..LifeProfile::default() }, today);
        let r = road(&Inputs { today, life: &lp, wedding_dated: true, child_dated: true, goals: &[], payments: &[] });
        let cover: Vec<_> = r.milestones.iter().filter(|m| m.source == Source::Cover).collect();
        assert!(!cover.is_empty());
        assert!(cover.iter().all(|m| m.on == "2027-02-01"));
    }

    #[test]
    fn months_between_counts_calendar_months() {
        assert_eq!(months_between(date!(2026 - 11 - 25), date!(2026 - 11 - 30)), 0);
        assert_eq!(months_between(date!(2026 - 11 - 25), date!(2027 - 01 - 01)), 2);
        assert_eq!(months_between(date!(2026 - 11 - 25), date!(2026 - 01 - 01)), 0);
        assert_eq!(add_months(date!(2026 - 11 - 25), 3), date!(2027 - 02 - 01));
        assert_eq!(spread(1_000, 3), 334);
        assert_eq!(spread(0, 3), 0);
    }
}
