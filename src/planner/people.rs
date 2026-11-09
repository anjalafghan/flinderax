//! Relationships: pure rules, upcoming dates and the quarterly report section. Phase 5.
//! Dates repeat every year; 29 Feb falls on 28 Feb in years that have no 29th.
//! Money is paise. No names appear in the report text.

use serde::{Deserialize, Serialize};
use time::{macros::format_description, util::is_leap_year, Date, Duration, Month};

use crate::planner::{
    checkin::{rupees, Period},
    habits::{Area, Cadence, Habit, Impact, Link, Status},
    short_date,
};

const ISO: &[time::format_description::FormatItem<'static>] = format_description!("[year]-[month]-[day]");

/// A person is `due_soon` this many days before the catch-up date.
pub const CATCH_UP_SOON_DAYS: i64 = 7;
/// Upcoming dates look this many days ahead.
pub const UPCOMING_DAYS: i64 = 60;
/// A gift is planned this many days before its reminder starts.
const GIFT_LEAD_DAYS: i64 = 14;
/// Planned gifts above this (Rs 10,000) in a year raise the gift fund habit.
const GIFT_FUND_MIN_PAISE: i64 = 1_000_000;
/// Catch-up habits shown at most.
const MAX_CATCH_UP_HABITS: usize = 2;
/// A report looks this many days past its end for the next quarter's gifts.
const NEXT_QUARTER_DAYS: i64 = 91;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Circle {
    Family,
    Close,
    Friend,
    Work,
    Other,
}

impl Circle {
    pub fn as_str(self) -> &'static str {
        match self {
            Circle::Family => "family",
            Circle::Close => "close",
            Circle::Friend => "friend",
            Circle::Work => "work",
            Circle::Other => "other",
        }
    }
    pub fn parse(s: &str) -> Option<Circle> {
        Some(match s {
            "family" => Circle::Family,
            "close" => Circle::Close,
            "friend" => Circle::Friend,
            "work" => Circle::Work,
            "other" => Circle::Other,
            _ => return None,
        })
    }
    /// Sort position: family first.
    pub fn rank(self) -> u8 {
        self as u8
    }
    /// How the circle reads in a calendar event title.
    pub fn phrase(self) -> &'static str {
        match self {
            Circle::Family => "family",
            Circle::Close => "close friend",
            Circle::Friend => "friend",
            Circle::Work => "work",
            Circle::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DateKind {
    Birthday,
    Anniversary,
    Remembrance,
    Other,
}

impl DateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DateKind::Birthday => "birthday",
            DateKind::Anniversary => "anniversary",
            DateKind::Remembrance => "remembrance",
            DateKind::Other => "other",
        }
    }
    pub fn parse(s: &str) -> Option<DateKind> {
        Some(match s {
            "birthday" => DateKind::Birthday,
            "anniversary" => DateKind::Anniversary,
            "remembrance" => DateKind::Remembrance,
            "other" => DateKind::Other,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GiftStatus {
    Idea,
    Bought,
    Given,
}

impl GiftStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            GiftStatus::Idea => "idea",
            GiftStatus::Bought => "bought",
            GiftStatus::Given => "given",
        }
    }
    pub fn parse(s: &str) -> Option<GiftStatus> {
        Some(match s {
            "idea" => GiftStatus::Idea,
            "bought" => GiftStatus::Bought,
            "given" => GiftStatus::Given,
            _ => return None,
        })
    }
    /// True for a gift that is already taken care of.
    pub fn settled(self) -> bool {
        self != GiftStatus::Idea
    }
}

/// The best gift status for a date and year, for the upcoming list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GiftState {
    None,
    Idea,
    Bought,
    Given,
}

impl GiftState {
    pub fn best(statuses: impl Iterator<Item = GiftStatus>) -> GiftState {
        statuses
            .map(|s| match s {
                GiftStatus::Idea => GiftState::Idea,
                GiftStatus::Bought => GiftState::Bought,
                GiftStatus::Given => GiftState::Given,
            })
            .max_by_key(|s| *s as u8)
            .unwrap_or(GiftState::None)
    }
}

/// A person as the rules see them.
#[derive(Debug, Clone, PartialEq)]
pub struct PersonIn {
    pub id: String,
    pub name: String,
    pub circle: Circle,
    pub every_days: Option<i64>,
    pub last_contact_on: Option<Date>,
    /// The day the person was added: the clock starts here until the first catch-up.
    pub created_on: Date,
}

impl PersonIn {
    /// Last catch-up (or the day added) plus the cadence. None without a cadence.
    pub fn next_due_on(&self) -> Option<Date> {
        let every = self.every_days?;
        self.last_contact_on.unwrap_or(self.created_on).checked_add(Duration::days(every))
    }

    /// Overdue once past, due soon within a week, else ok. None without a cadence.
    pub fn status(&self, today: Date) -> Option<Status> {
        let days = (self.next_due_on()? - today).whole_days();
        Some(if days < 0 {
            Status::Overdue
        } else if days <= CATCH_UP_SOON_DAYS {
            Status::DueSoon
        } else {
            Status::Ok
        })
    }
}

/// A repeating date with what the rules need about gifts.
#[derive(Debug, Clone, PartialEq)]
pub struct DateIn {
    pub date_id: String,
    pub person_id: String,
    pub name: String,
    pub circle: Circle,
    pub kind: DateKind,
    pub label: String,
    pub month: u8,
    pub day: u8,
    pub year: Option<i32>,
    pub remind_days: i64,
    pub gift_budget_paise: Option<i64>,
    /// Years for which a gift to this date is already bought or given.
    pub settled_years: Vec<i32>,
}

/// A festival season: a month and a yearly budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonIn {
    pub id: String,
    pub label: String,
    pub month: u8,
    pub budget_paise: i64,
}

/// What the people habit rules read. Filled by `handlers::people::habits_input`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PeopleIn {
    /// Not archived.
    pub people: Vec<PersonIn>,
    /// Dates of people who are not archived.
    pub dates: Vec<DateIn>,
    pub seasons: Vec<SeasonIn>,
    /// An active goal named "Gifts" exists.
    pub has_gifts_goal: bool,
}

// ---------- date maths ----------

/// The day in `year`; 29 Feb becomes 28 Feb when the year has no 29th.
pub fn date_in_year(year: i32, month: u8, day: u8) -> Option<Date> {
    let m = Month::try_from(month).ok()?;
    let day = if month == 2 && day == 29 && !is_leap_year(year) { 28 } else { day };
    Date::from_calendar_date(year, m, day).ok()
}

/// The next time (month, day) comes round on or after `today`.
pub fn next_occurrence(month: u8, day: u8, today: Date) -> Date {
    match date_in_year(today.year(), month, day) {
        Some(d) if d >= today => d,
        _ => date_in_year(today.year() + 1, month, day).unwrap_or(today),
    }
}

/// How old, or how many years, on that occurrence. None when the year is not known.
pub fn turns(year: Option<i32>, occurrence: Date) -> Option<i64> {
    year.map(|y| i64::from(occurrence.year() - y))
}

/// Whether (month, day) exists on some year: 29 Feb does, 30 Feb and 31 Apr do not.
pub fn possible_day(month: u8, day: u8) -> bool {
    Month::try_from(month).is_ok_and(|m| day >= 1 && day <= m.length(2000))
}

fn parse_iso(s: &str) -> Option<Date> {
    Date::parse(s.get(..10)?, ISO).ok()
}

/// Calendar text for the kind, used in titles.
fn kind_words(d: &DateIn) -> String {
    match d.kind {
        DateKind::Birthday => "birthday".into(),
        DateKind::Anniversary => "anniversary".into(),
        DateKind::Remembrance => "remembrance day".into(),
        DateKind::Other if !d.label.trim().is_empty() => d.label.trim().to_string(),
        DateKind::Other => "special day".into(),
    }
}

// ---------- upcoming ----------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Upcoming {
    pub date_id: String,
    pub person_id: String,
    pub name: String,
    pub kind: DateKind,
    pub label: String,
    pub on: String,
    pub days_away: i64,
    pub turns: Option<i64>,
    pub gift_budget_paise: Option<i64>,
    pub gift_status: GiftState,
}

/// Dates in the next `UPCOMING_DAYS` days, soonest first. `status` gives the best gift state for a
/// date id and year.
pub fn upcoming(dates: &[DateIn], today: Date, status: impl Fn(&str, i32) -> GiftState) -> Vec<Upcoming> {
    let mut out: Vec<Upcoming> = dates
        .iter()
        .filter_map(|d| {
            let on = next_occurrence(d.month, d.day, today);
            let days_away = (on - today).whole_days();
            (days_away <= UPCOMING_DAYS).then(|| Upcoming {
                date_id: d.date_id.clone(),
                person_id: d.person_id.clone(),
                name: d.name.clone(),
                kind: d.kind,
                label: d.label.clone(),
                on: on.to_string(),
                days_away,
                turns: turns(d.year, on),
                gift_budget_paise: d.gift_budget_paise,
                gift_status: status(&d.date_id, on.year()),
            })
        })
        .collect();
    out.sort_by(|a, b| (a.days_away, &a.name, &a.date_id).cmp(&(b.days_away, &b.name, &b.date_id)));
    out
}

// ---------- gift plan ----------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonthPlan {
    pub month: String,
    pub budget_paise: i64,
    pub occasions: i64,
}

/// Twelve calendar months starting with this one: each dated gift budget and festival season
/// counts once, in its month.
pub fn gift_months(dates: &[DateIn], seasons: &[SeasonIn], today: Date) -> Vec<MonthPlan> {
    let mut out = Vec::with_capacity(12);
    let (mut y, mut m) = (today.year(), today.month() as u8);
    for _ in 0..12 {
        let mut plan = MonthPlan { month: format!("{y:04}-{m:02}"), budget_paise: 0, occasions: 0 };
        for d in dates.iter().filter(|d| d.month == m) {
            if let Some(b) = d.gift_budget_paise {
                plan.budget_paise += b;
                plan.occasions += 1;
            }
        }
        for s in seasons.iter().filter(|s| s.month == m) {
            plan.budget_paise += s.budget_paise;
            plan.occasions += 1;
        }
        out.push(plan);
        if m == 12 {
            (y, m) = (y + 1, 1);
        } else {
            m += 1;
        }
    }
    out
}

/// Gift budgets and the number of occasions that fall on days in `from..=to`.
pub fn occasions_between(dates: &[DateIn], seasons: &[SeasonIn], from: Date, to: Date) -> (i64, i64) {
    let (mut paise, mut count) = (0, 0);
    for year in from.year()..=to.year() {
        for d in dates {
            let Some(b) = d.gift_budget_paise else { continue };
            if date_in_year(year, d.month, d.day).is_some_and(|on| on >= from && on <= to) {
                paise += b;
                count += 1;
            }
        }
        for s in seasons {
            if date_in_year(year, s.month, 1).is_some_and(|on| on >= from && on <= to) {
                paise += s.budget_paise;
                count += 1;
            }
        }
    }
    (paise, count)
}

/// Planned gifts over the next twelve months.
pub fn yearly_total(dates: &[DateIn], seasons: &[SeasonIn]) -> i64 {
    dates.iter().filter_map(|d| d.gift_budget_paise).sum::<i64>() + seasons.iter().map(|s| s.budget_paise).sum::<i64>()
}

/// A twelfth of a year's total, rounded.
pub fn monthly_share(total_paise: i64) -> i64 {
    (total_paise + 6) / 12
}

// ---------- habits ----------

fn due_status(due: Date, today: Date) -> Status {
    let days = (due - today).whole_days();
    if days < 0 {
        Status::Overdue
    } else if days <= 14 {
        Status::DueSoon
    } else {
        Status::Ok
    }
}

fn ago(last: Option<Date>, today: Date) -> String {
    let Some(last) = last else { return "No catch-up logged yet.".into() };
    let days = (today - last).whole_days().max(0);
    let plural = |n: i64, unit: &str| if n == 1 { format!("1 {unit}") } else { format!("{n} {unit}s") };
    let when = if days < 7 {
        plural(days, "day")
    } else if days < 60 {
        plural(days / 7, "week")
    } else {
        plural(days / 30, "month")
    };
    format!("Last caught up {when} ago.")
}

/// The people habits (area `people`). Never `high` impact.
pub fn habits(input: &PeopleIn, today: Date) -> Vec<Habit> {
    let mut out = Vec::new();

    // Catch-ups: the two most overdue people who are due or within a week of it.
    let mut due: Vec<(Date, &PersonIn)> = input
        .people
        .iter()
        .filter_map(|p| {
            let on = p.next_due_on()?;
            ((on - today).whole_days() <= CATCH_UP_SOON_DAYS).then_some((on, p))
        })
        .collect();
    due.sort_by(|a, b| (a.0, &a.1.name, &a.1.id).cmp(&(b.0, &b.1.name, &b.1.id)));
    for (on, p) in due.into_iter().take(MAX_CATCH_UP_HABITS) {
        out.push(Habit {
            key: format!("catch_up:{}", p.id),
            area: Area::People,
            title: format!("Catch up with {}", p.name),
            why: ago(p.last_contact_on, today),
            action: Some(Link { label: "Caught up".into(), to: format!("/people/{}", p.id) }),
            cadence: Cadence::Monthly,
            due_on: Some(on.to_string()),
            status: due_status(on, today),
            impact: if p.circle == Circle::Family { Impact::Medium } else { Impact::Low },
        });
    }

    // Gifts for dates coming up, until one is bought or given.
    for d in &input.dates {
        let Some(budget) = d.gift_budget_paise else { continue };
        let on = next_occurrence(d.month, d.day, today);
        if (on - today).whole_days() > d.remind_days + GIFT_LEAD_DAYS || d.settled_years.contains(&on.year()) {
            continue;
        }
        let due_on = on - Duration::days(d.remind_days);
        out.push(Habit {
            key: format!("gift:{}:{}", d.date_id, on.year()),
            area: Area::People,
            title: format!("Gift for {}'s {} on {}, about {}.", d.name, kind_words(d), short_date(on), rupees(budget)),
            why: "A little time to choose something they will like.".into(),
            action: Some(Link { label: "Plan gift".into(), to: format!("/people/{}", d.person_id) }),
            cadence: Cadence::Once,
            due_on: Some(due_on.to_string()),
            status: due_status(due_on, today),
            impact: Impact::Medium,
        });
    }

    // A fund for the year's gifts.
    let total = yearly_total(&input.dates, &input.seasons);
    if total > GIFT_FUND_MIN_PAISE && !input.has_gifts_goal {
        out.push(Habit {
            key: "gift_fund".into(),
            area: Area::People,
            title: format!("Set aside {} a month for gifts", rupees(monthly_share(total))),
            why: format!("About {} of gifts are planned over the year. A small monthly amount keeps them easy.", rupees(total)),
            action: Some(Link { label: "Gift plan".into(), to: "/people".into() }),
            cadence: Cadence::Once,
            due_on: None,
            status: Status::Ok,
            impact: Impact::Low,
        });
    }
    out
}

// ---------- calendar feed ----------

/// A calendar event for one occurrence of a date or a season.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayEvent {
    pub uid: String,
    pub on: Date,
    pub summary: String,
    pub description: String,
}

/// Events in the next `days` days: person dates on the day, and one per festival season on the 1st of
/// its month. Without `names`, titles say only the kind and circle ("Birthday (family)").
pub fn day_events(dates: &[DateIn], seasons: &[SeasonIn], today: Date, days: i64, names: bool) -> Vec<DayEvent> {
    let mut out = Vec::new();
    for d in dates {
        let on = next_occurrence(d.month, d.day, today);
        if (on - today).whole_days() > days {
            continue;
        }
        let (summary, description) = if names {
            let turns = turns(d.year, on).filter(|t| *t > 0).map(|t| format!(" Turns {t}.")).unwrap_or_default();
            (format!("{}'s {}", d.name, kind_words(d)), format!("Added in joy.{turns}"))
        } else {
            let kind = match d.kind {
                DateKind::Birthday => "Birthday",
                DateKind::Anniversary => "Anniversary",
                DateKind::Remembrance => "Remembrance",
                DateKind::Other => "Special day",
            };
            (format!("{kind} ({})", d.circle.phrase()), "Added in joy.".to_string())
        };
        out.push(DayEvent { uid: format!("joy-pdate-{}-{}@joy", d.date_id, on.year()), on, summary, description });
    }
    for s in seasons {
        let on = next_occurrence(s.month, 1, today);
        if (on - today).whole_days() > days {
            continue;
        }
        out.push(DayEvent {
            uid: format!("joy-gseason-{}-{}@joy", s.id, on.year()),
            on,
            summary: format!("Gift season: {}", s.label),
            description: format!("About {} set aside for gifts.", rupees(s.budget_paise)),
        });
    }
    out.sort_by(|a, b| (a.on, &a.uid).cmp(&(b.on, &b.uid)));
    out
}

// ---------- report ----------

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

/// Fills `wins` and `watch` (at most 2 each, no names) from the numbers.
pub fn notes(r: &mut PeopleReport) {
    let mut wins = Vec::new();
    let mut watch = Vec::new();
    if r.catch_ups > 0 {
        wins.push(if r.catch_ups == 1 {
            "You caught up once this quarter.".to_string()
        } else {
            format!("You caught up {} times this quarter.", r.catch_ups)
        });
    }
    if r.gifts_given_paise > 0 && r.gifts_budgeted_paise >= r.gifts_given_paise {
        wins.push(format!(
            "Gifts came to {}, within the {} planned.",
            rupees(r.gifts_given_paise),
            rupees(r.gifts_budgeted_paise)
        ));
    }
    if r.overdue_people > 0 {
        watch.push(if r.overdue_people == 1 {
            "1 person is past their catch-up date.".to_string()
        } else {
            format!("{} people are past their catch-up date.", r.overdue_people)
        });
    }
    if r.next_quarter_gifts_paise > 0 {
        watch.push(format!(
            "{} of gifts are due next quarter across {} {}.",
            rupees(r.next_quarter_gifts_paise),
            r.next_quarter_occasions,
            if r.next_quarter_occasions == 1 { "occasion" } else { "occasions" }
        ));
    }
    if r.gifts_budgeted_paise > 0 && r.gifts_given_paise > r.gifts_budgeted_paise {
        watch.push(format!(
            "Gifts came to {}, over the {} planned.",
            rupees(r.gifts_given_paise),
            rupees(r.gifts_budgeted_paise)
        ));
    }
    wins.truncate(2);
    watch.truncate(2);
    r.wins = wins;
    r.watch = watch;
}

/// The day range the next-quarter gifts cover: the 91 days after the period.
pub fn next_quarter(period: Period) -> (Date, Date) {
    (period.end + Duration::days(1), period.end + Duration::days(NEXT_QUARTER_DAYS))
}

/// Parse a stored day.
pub fn day_of(s: &str) -> Option<Date> {
    parse_iso(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn person(id: &str, circle: Circle, every: Option<i64>, last: Option<Date>) -> PersonIn {
        PersonIn {
            id: id.into(),
            name: format!("Person {id}"),
            circle,
            every_days: every,
            last_contact_on: last,
            created_on: date!(2026 - 01 - 01),
        }
    }

    fn pdate(id: &str, month: u8, day: u8, budget: Option<i64>) -> DateIn {
        DateIn {
            date_id: id.into(),
            person_id: "p1".into(),
            name: "Person A".into(),
            circle: Circle::Family,
            kind: DateKind::Birthday,
            label: String::new(),
            month,
            day,
            year: Some(1966),
            remind_days: 7,
            gift_budget_paise: budget,
            settled_years: vec![],
        }
    }

    #[test]
    fn next_occurrence_is_today_or_later() {
        let today = date!(2026 - 11 - 09);
        assert_eq!(next_occurrence(11, 9, today), today, "today counts");
        assert_eq!(next_occurrence(11, 10, today), date!(2026 - 11 - 10));
        assert_eq!(next_occurrence(11, 8, today), date!(2027 - 11 - 08));
    }

    #[test]
    fn december_wraps_into_january() {
        assert_eq!(next_occurrence(1, 5, date!(2026 - 12 - 20)), date!(2027 - 01 - 05));
        assert_eq!(next_occurrence(12, 31, date!(2026 - 12 - 31)), date!(2026 - 12 - 31));
        assert_eq!(next_occurrence(1, 1, date!(2026 - 12 - 31)), date!(2027 - 01 - 01));
    }

    #[test]
    fn leap_day_falls_on_the_28th_in_other_years() {
        assert_eq!(date_in_year(2027, 2, 29), Some(date!(2027 - 02 - 28)));
        assert_eq!(date_in_year(2028, 2, 29), Some(date!(2028 - 02 - 29)));
        assert_eq!(next_occurrence(2, 29, date!(2026 - 11 - 09)), date!(2027 - 02 - 28));
        assert_eq!(next_occurrence(2, 29, date!(2027 - 03 - 01)), date!(2028 - 02 - 29));
        assert_eq!(next_occurrence(2, 29, date!(2027 - 02 - 28)), date!(2027 - 02 - 28));
        assert_eq!(next_occurrence(2, 29, date!(2028 - 02 - 29)), date!(2028 - 02 - 29));
        assert_eq!(next_occurrence(2, 29, date!(2100 - 01 - 01)), date!(2100 - 02 - 28), "2100 is not a leap year");
    }

    #[test]
    fn impossible_days_are_refused() {
        assert!(possible_day(2, 29));
        assert!(!possible_day(2, 30));
        assert!(!possible_day(4, 31));
        assert!(possible_day(12, 31));
        assert!(!possible_day(13, 1));
        assert!(!possible_day(1, 0));
    }

    #[test]
    fn turns_counts_years_when_the_year_is_known() {
        assert_eq!(turns(Some(1966), date!(2026 - 12 - 14)), Some(60));
        assert_eq!(turns(None, date!(2026 - 12 - 14)), None);
        assert_eq!(turns(Some(2020), date!(2027 - 01 - 05)), Some(7));
    }

    #[test]
    fn catch_up_clock_starts_at_the_day_added() {
        let today = date!(2026 - 11 - 09);
        let never = person("1", Circle::Friend, Some(28), None);
        assert_eq!(never.next_due_on(), Some(date!(2026 - 01 - 29)));
        assert_eq!(never.status(today), Some(Status::Overdue));
        let recent = person("2", Circle::Friend, Some(28), Some(date!(2026 - 10 - 20)));
        assert_eq!(recent.next_due_on(), Some(date!(2026 - 11 - 17)));
        assert_eq!(recent.status(today), Some(Status::Ok), "8 days away is not within 7");
        let soon = person("3", Circle::Friend, Some(28), Some(date!(2026 - 10 - 13)));
        assert_eq!(soon.status(today), Some(Status::DueSoon));
        let fine = person("4", Circle::Friend, Some(28), Some(date!(2026 - 11 - 01)));
        assert_eq!(fine.status(today), Some(Status::Ok));
        assert_eq!(person("5", Circle::Friend, None, None).status(today), None);
        let due_today = person("6", Circle::Friend, Some(28), Some(date!(2026 - 10 - 12)));
        assert_eq!(due_today.status(today), Some(Status::DueSoon), "due today is not overdue yet");
    }

    #[test]
    fn upcoming_is_sixty_days_soonest_first_with_wrap() {
        let today = date!(2026 - 12 - 20);
        let dates = [pdate("far", 3, 1, None), pdate("jan", 1, 5, None), pdate("dec", 12, 25, None), pdate("edge", 2, 18, None)];
        let got = upcoming(&dates, today, |_, _| GiftState::None);
        let ids: Vec<&str> = got.iter().map(|u| u.date_id.as_str()).collect();
        assert_eq!(ids, vec!["dec", "jan", "edge"], "18 Feb is exactly 60 days away");
        assert_eq!(got[1].on, "2027-01-05");
        assert_eq!(got[1].days_away, 16);
        assert_eq!(got[1].turns, Some(61));
    }

    #[test]
    fn upcoming_reads_gift_status_for_that_occurrence_year() {
        let dates = [pdate("d", 1, 5, Some(100_000))];
        let got = upcoming(&dates, date!(2026 - 12 - 20), |id, year| {
            assert_eq!((id, year), ("d", 2027));
            GiftState::Bought
        });
        assert_eq!(got[0].gift_status, GiftState::Bought);
    }

    #[test]
    fn best_gift_state() {
        use GiftStatus::*;
        assert_eq!(GiftState::best([].into_iter()), GiftState::None);
        assert_eq!(GiftState::best([Idea, Given, Bought].into_iter()), GiftState::Given);
        assert_eq!(GiftState::best([Idea, Bought].into_iter()), GiftState::Bought);
        assert_eq!(GiftState::best([Idea].into_iter()), GiftState::Idea);
    }

    #[test]
    fn catch_up_habits_are_the_two_most_overdue() {
        let today = date!(2026 - 11 - 09);
        let input = PeopleIn {
            people: vec![
                person("a", Circle::Family, Some(28), Some(date!(2026 - 09 - 01))),
                person("b", Circle::Friend, Some(28), Some(date!(2026 - 08 - 01))),
                person("c", Circle::Work, Some(28), Some(date!(2026 - 10 - 01))),
                person("d", Circle::Friend, Some(28), Some(date!(2026 - 11 - 08))),
                person("e", Circle::Friend, None, None),
            ],
            ..Default::default()
        };
        let got = habits(&input, today);
        let keys: Vec<&str> = got.iter().map(|h| h.key.as_str()).collect();
        assert_eq!(keys, vec!["catch_up:b", "catch_up:a"]);
        assert_eq!(got[0].title, "Catch up with Person b");
        assert_eq!(got[0].impact, Impact::Low);
        assert_eq!(got[1].impact, Impact::Medium, "family is medium");
        assert_eq!(got[0].status, Status::Overdue);
        assert_eq!(got[0].cadence, Cadence::Monthly);
        assert_eq!(got[0].due_on.as_deref(), Some("2026-08-29"));
        assert_eq!(got[0].why, "Last caught up 3 months ago.");
        let link = got[0].action.as_ref().unwrap();
        assert_eq!((link.label.as_str(), link.to.as_str()), ("Caught up", "/people/b"));
        assert!(got.iter().all(|h| h.area == Area::People && h.impact != Impact::High));
    }

    #[test]
    fn catch_up_includes_someone_due_within_a_week_but_not_later() {
        let today = date!(2026 - 11 - 09);
        let mk = |last| PeopleIn { people: vec![person("a", Circle::Close, Some(28), Some(last))], ..Default::default() };
        assert_eq!(habits(&mk(date!(2026 - 10 - 16)), today).len(), 1, "due in 5 days");
        assert_eq!(habits(&mk(date!(2026 - 10 - 20)), today).len(), 0, "due in 8 days");
    }

    #[test]
    fn why_reads_naturally() {
        let today = date!(2026 - 11 - 09);
        assert_eq!(ago(None, today), "No catch-up logged yet.");
        assert_eq!(ago(Some(date!(2026 - 11 - 06)), today), "Last caught up 3 days ago.");
        assert_eq!(ago(Some(date!(2026 - 11 - 08)), today), "Last caught up 1 day ago.");
        assert_eq!(ago(Some(date!(2026 - 10 - 05)), today), "Last caught up 5 weeks ago.");
        assert_eq!(ago(Some(date!(2026 - 03 - 01)), today), "Last caught up 8 months ago.");
    }

    #[test]
    fn gift_habit_appears_inside_the_window_and_names_the_day() {
        let mut d = pdate("d1", 12, 14, Some(200_000));
        d.remind_days = 14;
        let input = PeopleIn { dates: vec![d], ..Default::default() };
        assert!(habits(&input, date!(2026 - 11 - 09)).is_empty(), "35 days away, window is 28");
        let got = habits(&input, date!(2026 - 11 - 16));
        assert_eq!(got.len(), 1, "28 days away is inside the window");
        let today = date!(2026 - 11 - 20);
        let got = habits(&input, today);
        let h = &got[0];
        assert_eq!(h.key, "gift:d1:2026");
        assert_eq!(h.title, "Gift for Person A's birthday on 14 Dec, about \u{20b9}2,000.");
        assert_eq!(h.due_on.as_deref(), Some("2026-11-30"));
        assert_eq!(h.status, Status::DueSoon);
        assert_eq!(h.cadence, Cadence::Once);
        assert_eq!(h.impact, Impact::Medium);
        assert_eq!(h.action.as_ref().unwrap().to, "/people/p1");
    }

    #[test]
    fn gift_habit_goes_once_bought_or_given_and_needs_a_budget() {
        let today = date!(2026 - 12 - 10);
        let mut d = pdate("d1", 12, 14, Some(200_000));
        d.settled_years = vec![2026];
        assert!(habits(&PeopleIn { dates: vec![d.clone()], ..Default::default() }, today).is_empty());
        d.settled_years = vec![2025];
        let got = habits(&PeopleIn { dates: vec![d.clone()], ..Default::default() }, today);
        assert_eq!(got.len(), 1, "last year's gift does not count");
        assert_eq!(got[0].status, Status::Overdue, "reminder day (7 Dec) has passed");
        d.gift_budget_paise = None;
        assert!(habits(&PeopleIn { dates: vec![d], ..Default::default() }, today).is_empty());
    }

    #[test]
    fn gift_habit_across_new_year_uses_the_next_year() {
        let d = pdate("d1", 1, 5, Some(100_000));
        let got = habits(&PeopleIn { dates: vec![d], ..Default::default() }, date!(2026 - 12 - 20));
        assert_eq!(got[0].key, "gift:d1:2027");
        assert_eq!(got[0].due_on.as_deref(), Some("2026-12-29"));
        assert!(got[0].title.contains("5 Jan"));
    }

    #[test]
    fn gift_habit_for_a_leap_day() {
        let d = pdate("d1", 2, 29, Some(100_000));
        let got = habits(&PeopleIn { dates: vec![d], ..Default::default() }, date!(2027 - 02 - 10));
        assert_eq!(got[0].key, "gift:d1:2027");
        assert!(got[0].title.contains("on 28 Feb"));
    }

    #[test]
    fn gift_fund_needs_ten_thousand_and_no_goal() {
        let today = date!(2026 - 11 - 09);
        let season = |m, b| SeasonIn { id: format!("s{m}"), label: "Diwali".into(), month: m, budget_paise: b };
        let mut input = PeopleIn {
            dates: vec![pdate("d1", 3, 1, Some(500_000))],
            seasons: vec![season(10, 500_001)],
            ..Default::default()
        };
        let got: Vec<_> = habits(&input, today).into_iter().filter(|h| h.key == "gift_fund").collect();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].title, "Set aside \u{20b9}833 a month for gifts");
        assert_eq!(got[0].due_on, None);
        assert_eq!(got[0].cadence, Cadence::Once);
        assert_eq!(got[0].impact, Impact::Low);
        assert_eq!(got[0].action.as_ref().unwrap().to, "/people");
        input.has_gifts_goal = true;
        assert!(habits(&input, today).iter().all(|h| h.key != "gift_fund"));
        input.has_gifts_goal = false;
        input.seasons = vec![season(10, 500_000)];
        assert!(habits(&input, today).iter().all(|h| h.key != "gift_fund"), "exactly 10,000 is not more");
    }

    #[test]
    fn the_year_plan_has_twelve_months_from_this_one() {
        let dates = [pdate("a", 11, 20, Some(100_000)), pdate("b", 1, 5, Some(200_000)), pdate("c", 1, 6, None)];
        let seasons = [SeasonIn { id: "s".into(), label: "Diwali".into(), month: 1, budget_paise: 300_000 }];
        let plan = gift_months(&dates, &seasons, date!(2026 - 11 - 09));
        assert_eq!(plan.len(), 12);
        assert_eq!(plan[0], MonthPlan { month: "2026-11".into(), budget_paise: 100_000, occasions: 1 });
        assert_eq!(plan[2].month, "2027-01");
        assert_eq!((plan[2].budget_paise, plan[2].occasions), (500_000, 2));
        assert_eq!(plan[11].month, "2027-10");
        assert_eq!(plan.iter().map(|m| m.budget_paise).sum::<i64>(), yearly_total(&dates, &seasons));
        assert_eq!(monthly_share(600_000), 50_000);
        assert_eq!(monthly_share(100_007), 8_334);
    }

    #[test]
    fn occasions_between_counts_dates_and_seasons_across_years() {
        let dates = [pdate("a", 1, 5, Some(100_000)), pdate("b", 12, 30, Some(50_000)), pdate("c", 1, 6, None)];
        let seasons = [SeasonIn { id: "s".into(), label: "Diwali".into(), month: 1, budget_paise: 300_000 }];
        let (paise, n) = occasions_between(&dates, &seasons, date!(2026 - 12 - 28), date!(2027 - 03 - 28));
        assert_eq!((paise, n), (450_000, 3));
        let (paise, n) = occasions_between(&dates, &seasons, date!(2027 - 01 - 02), date!(2027 - 01 - 04));
        assert_eq!((paise, n), (0, 0), "the 1st is before the range");
        let (paise, n) = occasions_between(&dates, &seasons, date!(2027 - 01 - 01), date!(2027 - 01 - 01));
        assert_eq!((paise, n), (300_000, 1), "a season is on the 1st");
    }

    #[test]
    fn leap_day_budget_counts_in_a_non_leap_february() {
        let dates = [pdate("a", 2, 29, Some(100_000))];
        let (paise, _) = occasions_between(&dates, &[], date!(2027 - 02 - 28), date!(2027 - 02 - 28));
        assert_eq!(paise, 100_000);
    }

    #[test]
    fn next_quarter_is_the_91_days_after() {
        let p = Period { start: date!(2026 - 07 - 01), end: date!(2026 - 09 - 30) };
        assert_eq!(next_quarter(p), (date!(2026 - 10 - 01), date!(2026 - 12 - 30)));
    }

    #[test]
    fn report_notes_have_no_names_and_cap_at_two() {
        let mut r = PeopleReport {
            catch_ups: 12,
            overdue_people: 3,
            gifts_given_paise: 400_000,
            gifts_budgeted_paise: 500_000,
            next_quarter_gifts_paise: 1_400_000,
            next_quarter_occasions: 5,
            ..Default::default()
        };
        notes(&mut r);
        assert_eq!(r.wins, vec!["You caught up 12 times this quarter.", "Gifts came to \u{20b9}4,000, within the \u{20b9}5,000 planned."]);
        assert_eq!(
            r.watch,
            vec!["3 people are past their catch-up date.", "\u{20b9}14,000 of gifts are due next quarter across 5 occasions."]
        );
        r.gifts_given_paise = 700_000;
        r.overdue_people = 0;
        notes(&mut r);
        assert_eq!(r.wins, vec!["You caught up 12 times this quarter."]);
        assert_eq!(r.watch.len(), 2);
        assert!(r.watch[1].contains("over the"));
    }

    #[test]
    fn report_notes_handle_one_and_none() {
        let mut r = PeopleReport { catch_ups: 1, overdue_people: 1, ..Default::default() };
        notes(&mut r);
        assert_eq!(r.wins, vec!["You caught up once this quarter."]);
        assert_eq!(r.watch, vec!["1 person is past their catch-up date."]);
        let mut quiet = PeopleReport::default();
        notes(&mut quiet);
        assert!(quiet.wins.is_empty() && quiet.watch.is_empty());
    }

    #[test]
    fn circle_order_and_names_round_trip() {
        for c in [Circle::Family, Circle::Close, Circle::Friend, Circle::Work, Circle::Other] {
            assert_eq!(Circle::parse(c.as_str()), Some(c));
        }
        assert!(Circle::Family.rank() < Circle::Close.rank() && Circle::Work.rank() < Circle::Other.rank());
        assert_eq!(Circle::parse("boss"), None);
        assert_eq!(DateKind::parse("birthday"), Some(DateKind::Birthday));
    }
}
