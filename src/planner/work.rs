//! Work: the job-search habits. Pure: no DB, no clock. The handler fills `WorkIn`.
//! Two rules: a lead's next step is due, and an open search where nothing has moved for two weeks.

use serde::{Deserialize, Serialize};
use time::Date;

use crate::planner::habits::{Area, Cadence, Habit, Impact, Link, Status};

/// A next step shows up this many days before its date.
const STEP_SOON_DAYS: i64 = 7;
const MAX_STEP_HABITS: usize = 2;
/// An open search with nothing moved for this long is stalled; twice this long is overdue.
const STALLED_DAYS: i64 = 14;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Offer,
    Interview,
    Applied,
    Idea,
    Closed,
}

impl Stage {
    pub fn parse(s: &str) -> Option<Stage> {
        Some(match s {
            "idea" => Stage::Idea,
            "applied" => Stage::Applied,
            "interview" => Stage::Interview,
            "offer" => Stage::Offer,
            "closed" => Stage::Closed,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Stage::Idea => "idea",
            Stage::Applied => "applied",
            Stage::Interview => "interview",
            Stage::Offer => "offer",
            Stage::Closed => "closed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LeadIn {
    pub id: String,
    pub company: String,
    pub stage: Stage,
    pub next_step: String,
    pub next_on: Option<Date>,
    pub updated_on: Date,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorkIn {
    pub leads: Vec<LeadIn>,
}

fn link() -> Option<Link> {
    Some(Link { label: "Open Work".into(), to: "/work".into() })
}

/// The work habits (area `work`). Never `high` impact.
pub fn habits(input: &WorkIn, today: Date) -> Vec<Habit> {
    let mut out = Vec::new();
    let open: Vec<&LeadIn> = input.leads.iter().filter(|l| l.stage != Stage::Closed).collect();

    let mut due: Vec<(Date, &LeadIn)> = open
        .iter()
        .filter_map(|l| {
            let on = l.next_on?;
            ((on - today).whole_days() <= STEP_SOON_DAYS).then_some((on, *l))
        })
        .collect();
    due.sort_by(|a, b| (a.0, &a.1.company, &a.1.id).cmp(&(b.0, &b.1.company, &b.1.id)));
    for (on, lead) in due.into_iter().take(MAX_STEP_HABITS) {
        let step = lead.next_step.trim();
        out.push(Habit {
            key: format!("work_step:{}", lead.id),
            area: Area::Work,
            title: if step.is_empty() { format!("Next step with {}", lead.company) } else { format!("{step} ({})", lead.company) },
            why: match (on - today).whole_days() {
                d if d < 0 => format!("This was due {} days ago.", -d),
                0 => "This is due today.".to_string(),
                d => format!("This is due in {d} days."),
            },
            action: link(),
            cadence: Cadence::Once,
            due_on: Some(on.to_string()),
            status: if on < today { Status::Overdue } else { Status::DueSoon },
            impact: Impact::Medium,
        });
    }

    // A search is open and nothing has moved or is planned: say so once a week.
    let planned = open.iter().any(|l| l.next_on.is_some_and(|on| on >= today));
    if let Some(last) = open.iter().map(|l| l.updated_on).max().filter(|_| !planned) {
        let days = (today - last).whole_days();
        if days >= STALLED_DAYS {
            out.push(Habit {
                key: "work_stalled".into(),
                area: Area::Work,
                title: "Move your job search forward".into(),
                why: format!("Nothing has moved in {days} days. Apply to one role or set a next step."),
                action: link(),
                cadence: Cadence::Weekly,
                due_on: None,
                status: if days >= STALLED_DAYS * 2 { Status::Overdue } else { Status::DueSoon },
                impact: Impact::Low,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn lead(id: &str, stage: Stage, next_on: Option<Date>, updated_on: Date) -> LeadIn {
        LeadIn { id: id.into(), company: format!("Company {id}"), stage, next_step: String::new(), next_on, updated_on }
    }

    #[test]
    fn a_step_due_within_a_week_becomes_a_habit_and_a_late_one_is_overdue() {
        let today = date!(2026 - 11 - 10);
        let mut soon = lead("a", Stage::Interview, Some(date!(2026 - 11 - 13)), today);
        soon.next_step = "Send the take-home".into();
        let input = WorkIn {
            leads: vec![
                soon,
                lead("b", Stage::Applied, Some(date!(2026 - 11 - 01)), today),
                lead("c", Stage::Applied, Some(date!(2026 - 12 - 20)), today),
                lead("d", Stage::Closed, Some(date!(2026 - 11 - 11)), today),
            ],
        };
        let got = habits(&input, today);
        let keys: Vec<&str> = got.iter().map(|h| h.key.as_str()).collect();
        assert_eq!(keys, vec!["work_step:b", "work_step:a"]);
        assert_eq!(got[0].status, Status::Overdue);
        assert_eq!(got[0].title, "Next step with Company b");
        assert_eq!(got[1].title, "Send the take-home (Company a)");
        assert_eq!(got[1].why, "This is due in 3 days.");
        assert!(got.iter().all(|h| h.area == Area::Work && h.impact != Impact::High));
    }

    #[test]
    fn an_open_search_with_nothing_moving_is_stalled() {
        let today = date!(2026 - 11 - 10);
        let quiet = WorkIn { leads: vec![lead("a", Stage::Applied, None, date!(2026 - 10 - 20))] };
        let got = habits(&quiet, today);
        assert_eq!(got.len(), 1);
        assert_eq!((got[0].key.as_str(), got[0].status), ("work_stalled", Status::DueSoon));
        assert!(got[0].why.contains("21 days"));

        let long = WorkIn { leads: vec![lead("a", Stage::Idea, None, date!(2026 - 09 - 01))] };
        assert_eq!(habits(&long, today)[0].status, Status::Overdue);
    }

    #[test]
    fn no_search_a_recent_move_or_a_planned_step_is_not_stalled() {
        let today = date!(2026 - 11 - 10);
        assert!(habits(&WorkIn::default(), today).is_empty());
        let closed = WorkIn { leads: vec![lead("a", Stage::Closed, None, date!(2026 - 01 - 01))] };
        assert!(habits(&closed, today).is_empty());
        let recent = WorkIn { leads: vec![lead("a", Stage::Applied, None, date!(2026 - 11 - 01))] };
        assert!(habits(&recent, today).is_empty());
        let planned = WorkIn { leads: vec![lead("a", Stage::Applied, Some(date!(2026 - 12 - 15)), date!(2026 - 09 - 01))] };
        assert!(habits(&planned, today).is_empty());
    }
}
