//! Diet and health: pure rules and the quarterly report section. Phase 5.
//! TODO(diet builder): fill `DietIn`, `habits` and `build_report`.

use serde::{Deserialize, Serialize};
use time::Date;

use crate::planner::habits::Habit;

/// What the diet and health habit rules read. Filled by `handlers::diet::habits_input`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DietIn {}

/// The diet and health habits (areas `food` and `health`). Never `high` impact.
pub fn habits(_input: &DietIn, _today: Date) -> Vec<Habit> {
    Vec::new()
}

/// One weekly diet habit over a check-in period.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DietHabitRate {
    pub label: String,
    pub target_days: i64,
    /// Average days hit as a share of the target, over logged weeks; None with no logged week.
    pub hit_bps: Option<i64>,
    pub weeks_on_target: i64,
    pub weeks_logged: i64,
}

/// The diet section of the quarter report. Its own wins and watch, at most 2 each.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DietReport {
    pub eating_out_paise: i64,
    pub groceries_paise: i64,
    pub prev_eating_out_paise: Option<i64>,
    pub prev_groceries_paise: Option<i64>,
    /// Eating out and delivery as a share of all food spend; None with no food spend.
    pub eating_out_share_bps: Option<i64>,
    /// Delivery orders per week over the period, in hundredths (350 = 3.5 a week).
    pub delivery_per_week_x100: i64,
    pub weeks_logged: i64,
    pub weeks_in_period: i64,
    pub habits: Vec<DietHabitRate>,
    /// Latest weight in the period, kg x10.
    pub weight_x10: Option<i64>,
    /// Change from the latest weight before the period, kg x10; None without both.
    pub weight_change_x10: Option<i64>,
    /// "YYYY-MM-DD"
    pub checkup_due_on: Option<String>,
    pub wins: Vec<String>,
    pub watch: Vec<String>,
}
