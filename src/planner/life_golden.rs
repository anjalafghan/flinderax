//! Characterization tests for [`plan`]: the whole LifePlan, pinned across profiles that reach each
//! branch. A refactor must leave `life_golden.json` byte-for-byte alone. Regenerate on purpose with
//! `UPDATE_GOLDEN=1 cargo test life_golden`.

use super::life::*;
use time::macros::date;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/planner/life_golden.json");

fn full() -> LifeProfile {
    LifeProfile {
        my_age: 31,
        partner_age: Some(29),
        my_take_home_paise: 18_000_000,
        partner_take_home_paise: 12_000_000,
        wedding_date: Some("2028-02-01".into()),
        wedding_budget_paise: Some(250_000_000),
        wedding_saved_paise: 40_000_000,
        child_birth_year: Some(2030),
        child_education_today_paise: 300_000_000,
        child_saved_paise: 5_000_000,
        retirement_age: 58,
        retirement_saved_paise: 100_000_000,
        emergency_saved_paise: 30_000_000,
        term_cover_paise: 1_000_000_000,
        health_cover_paise: 500_000_000,
        loans_paise: 200_000_000,
        ..Default::default()
    }
}

/// Take-home that leaves a surplus exactly equal to the emergency fund's monthly step-up.
fn exact_fit(today: time::Date) -> LifeProfile {
    let solo = LifeProfile { partner_take_home_paise: 0, wedding_date: None, child_birth_year: None, ..full() };
    let step = plan(&solo, 9_000_000, today).goals[0].step_up_sip_paise;
    LifeProfile { my_take_home_paise: 9_000_000 + step, ..solo }
}

fn scenarios() -> Vec<(&'static str, LifeProfile, i64, time::Date)> {
    let today = date!(2026 - 11 - 10);
    let p = |f: &dyn Fn(&mut LifeProfile)| {
        let mut x = full();
        f(&mut x);
        x
    };
    vec![
        ("default_profile", LifeProfile::default(), 0, today),
        ("full", full(), 9_000_000, today),
        ("full_in_january_leap_day", full(), 9_000_000, date!(2028 - 02 - 29)),
        ("wedding_in_the_past_and_default_budget", p(&|x| { x.wedding_date = Some("2020-06-01".into()); x.wedding_budget_paise = None; }), 9_000_000, today),
        ("unreadable_wedding_date", p(&|x| x.wedding_date = Some("soon".into())), 9_000_000, today),
        ("child_already_born_no_education", p(&|x| { x.child_birth_year = Some(2015); x.child_education_today_paise = 0; }), 9_000_000, today),
        ("child_far_future", p(&|x| x.child_birth_year = Some(9999)), 9_000_000, today),
        ("no_child_young_so_base_health", p(&|x| { x.child_birth_year = None; x.partner_age = None; x.my_age = 28; }), 9_000_000, today),
        ("no_child_older_so_wider_health", p(&|x| { x.child_birth_year = None; x.my_age = 47; }), 9_000_000, today),
        ("partner_older_wider_health", p(&|x| { x.child_birth_year = None; x.partner_age = Some(52); }), 9_000_000, today),
        ("solo_earner_tight_surplus", p(&|x| x.partner_take_home_paise = 0), 17_000_000, today),
        ("covered_everything", p(&|x| { x.term_cover_paise = i64::MAX; x.health_cover_paise = i64::MAX; x.emergency_saved_paise = i64::MAX; x.retirement_saved_paise = i64::MAX; x.wedding_saved_paise = i64::MAX; x.child_saved_paise = i64::MAX; }), 9_000_000, today),
        ("surplus_exactly_fits_first_goal", exact_fit(today), 9_000_000, today),
        ("no_income_at_all", p(&|x| { x.my_take_home_paise = 0; x.partner_take_home_paise = 0; }), 9_000_000, today),
        ("retire_before_now", p(&|x| x.retirement_age = 20), 9_000_000, today),
        ("extreme_ages_and_amounts", p(&|x| { x.my_age = 500; x.retirement_age = -9; x.my_take_home_paise = i64::MAX; x.partner_take_home_paise = i64::MAX; x.loans_paise = i64::MAX; }), i64::MAX, today),
    ]
}

fn snapshot() -> String {
    let all: Vec<_> = scenarios().into_iter().map(|(name, profile, expenses, today)| serde_json::json!({ "name": name, "plan": plan(&profile, expenses, today) })).collect();
    serde_json::to_string_pretty(&all).unwrap() + "\n"
}

#[test]
fn plan_matches_the_pinned_output() {
    let now = snapshot();
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(FILE, &now).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(FILE).expect("run once with UPDATE_GOLDEN=1 to create the snapshot");
    assert!(now == pinned, "life plan output changed; if on purpose, rerun with UPDATE_GOLDEN=1");
}
