//! Relationships: pure rules, upcoming dates and the quarterly report section. Phase 5.
//! TODO(people builder): fill `PeopleIn`, `habits` and the rest.

use serde::{Deserialize, Serialize};
use time::Date;

use crate::planner::habits::Habit;

/// What the people habit rules read. Filled by `handlers::people::habits_input`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PeopleIn {}

/// The people habits (area `people`). Never `high` impact.
pub fn habits(_input: &PeopleIn, _today: Date) -> Vec<Habit> {
    Vec::new()
}

/// The people section of the quarter report. No names. Its own wins and watch, at most 2 each.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PeopleReport {
    /// Catch-ups logged in the period.
    pub catch_ups: i64,
    /// People past their catch-up date today.
    pub overdue_people: i64,
    pub gifts_given_paise: i64,
    /// Gift budgets of occasions that fell in the period.
    pub gifts_budgeted_paise: i64,
    /// Gift budgets (dates and festival seasons) in the 91 days after the period.
    pub next_quarter_gifts_paise: i64,
    pub next_quarter_occasions: i64,
    /// Costs of visits logged in the period.
    pub visit_cost_paise: i64,
    /// Spend in the `people` category (money sent to individuals) in the period.
    pub sent_to_people_paise: i64,
    pub prev_sent_to_people_paise: Option<i64>,
    pub wins: Vec<String>,
    pub watch: Vec<String>,
}
