//! `/api/people`: the circle, important dates, catch-ups, gifts and festival seasons. Phase 5.
//! Private to each user: every query filters by `user_id`, including lookups by id, so an unknown or
//! foreign id is a 404. No contact details are stored. Money is paise.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::SqlitePool;
use time::Date;

use crate::{
    handlers::common::{not_found, today_local, AppError},
    models::AppState,
    planner::{
        checkin::Period,
        people::{
            self, Circle, DateIn, DateKind, GiftState, GiftStatus, MonthPlan, PeopleIn, PeopleReport, PersonIn, SeasonIn,
            Upcoming,
        },
        habits::Status,
    },
    validate,
};

type User = Extension<(String, String)>;

fn unprocessable(msg: impl Into<String>) -> AppError {
    AppError(StatusCode::UNPROCESSABLE_ENTITY, msg.into())
}

/// For PATCH bodies: a missing field is `None`, an explicit `null` is `Some(None)`.
fn double_option<'de, T, D>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(d)?))
}

fn too_long(s: &str, max: usize) -> bool {
    s.chars().count() > max
}

fn money(v: i64, what: &str) -> Result<i64, AppError> {
    validate::positive_amount(v).map_err(|e| unprocessable(format!("{what}: {e}")))
}

fn parse_day(s: &str, what: &str) -> Result<Date, AppError> {
    people::day_of(s).filter(|_| s.len() == 10).ok_or_else(|| unprocessable(format!("{what} must look like 2026-12-14.")))
}

// ---------- rows ----------

#[derive(sqlx::FromRow)]
struct PersonDb {
    person_id: String,
    name: String,
    circle: String,
    every_days: Option<i64>,
    last_contact_on: Option<String>,
    note: String,
    archived: i64,
    created_day: String,
}

impl PersonDb {
    fn to_in(&self, today: Date) -> PersonIn {
        PersonIn {
            id: self.person_id.clone(),
            name: self.name.clone(),
            circle: Circle::parse(&self.circle).unwrap_or(Circle::Other),
            every_days: self.every_days,
            last_contact_on: self.last_contact_on.as_deref().and_then(people::day_of),
            created_on: people::day_of(&self.created_day).unwrap_or(today),
        }
    }

    fn to_row(&self, today: Date) -> PersonRow {
        let p = self.to_in(today);
        PersonRow {
            id: self.person_id.clone(),
            name: self.name.clone(),
            circle: p.circle,
            every_days: self.every_days,
            last_contact_on: self.last_contact_on.clone(),
            next_due_on: p.next_due_on().map(|d| d.to_string()),
            status: p.status(today),
            note: self.note.clone(),
            archived: self.archived != 0,
        }
    }
}

const PERSON_COLS: &str =
    "person_id, name, circle, every_days, last_contact_on, note, archived, substr(created_at, 1, 10) AS created_day";

#[derive(Serialize, Clone)]
pub struct PersonRow {
    pub id: String,
    pub name: String,
    pub circle: Circle,
    pub every_days: Option<i64>,
    pub last_contact_on: Option<String>,
    pub next_due_on: Option<String>,
    pub status: Option<Status>,
    pub note: String,
    pub archived: bool,
}

#[derive(sqlx::FromRow)]
struct DateDb {
    date_id: String,
    person_id: String,
    kind: String,
    label: String,
    month: i64,
    day: i64,
    year: Option<i64>,
    remind_days: i64,
    gift_budget_paise: Option<i64>,
}

const DATE_COLS: &str = "date_id, person_id, kind, label, month, day, year, remind_days, gift_budget_paise";

#[derive(Serialize)]
pub struct PersonDate {
    pub id: String,
    pub person_id: String,
    pub kind: DateKind,
    pub label: String,
    pub month: i64,
    pub day: i64,
    pub year: Option<i64>,
    pub remind_days: i64,
    pub gift_budget_paise: Option<i64>,
    pub next_on: String,
    pub days_away: i64,
    pub turns: Option<i64>,
}

impl DateDb {
    fn to_out(&self, today: Date) -> PersonDate {
        let on = people::next_occurrence(self.month as u8, self.day as u8, today);
        PersonDate {
            id: self.date_id.clone(),
            person_id: self.person_id.clone(),
            kind: DateKind::parse(&self.kind).unwrap_or(DateKind::Other),
            label: self.label.clone(),
            month: self.month,
            day: self.day,
            year: self.year,
            remind_days: self.remind_days,
            gift_budget_paise: self.gift_budget_paise,
            next_on: on.to_string(),
            days_away: (on - today).whole_days(),
            turns: people::turns(self.year.map(|y| y as i32), on),
        }
    }
}

#[derive(sqlx::FromRow)]
struct TouchDb {
    touch_id: String,
    person_id: String,
    on_day: String,
    kind: String,
    cost_paise: Option<i64>,
    note: String,
}

#[derive(Serialize)]
pub struct Touch {
    pub id: String,
    pub person_id: String,
    pub on_day: String,
    pub kind: String,
    pub cost_paise: Option<i64>,
    pub note: String,
}

impl From<TouchDb> for Touch {
    fn from(t: TouchDb) -> Touch {
        Touch { id: t.touch_id, person_id: t.person_id, on_day: t.on_day, kind: t.kind, cost_paise: t.cost_paise, note: t.note }
    }
}

#[derive(sqlx::FromRow)]
struct GiftDb {
    gift_id: String,
    person_id: String,
    date_id: Option<String>,
    for_year: Option<i64>,
    status: String,
    title: String,
    amount_paise: Option<i64>,
    on_day: Option<String>,
    txn_id: Option<String>,
}

const GIFT_COLS: &str = "gift_id, person_id, date_id, for_year, status, title, amount_paise, on_day, txn_id";

#[derive(Serialize)]
pub struct Gift {
    pub id: String,
    pub person_id: String,
    pub date_id: Option<String>,
    pub for_year: Option<i64>,
    pub status: GiftStatus,
    pub title: String,
    pub amount_paise: Option<i64>,
    pub on_day: Option<String>,
    pub txn_id: Option<String>,
}

impl From<GiftDb> for Gift {
    fn from(g: GiftDb) -> Gift {
        Gift {
            id: g.gift_id,
            person_id: g.person_id,
            date_id: g.date_id,
            for_year: g.for_year,
            status: GiftStatus::parse(&g.status).unwrap_or(GiftStatus::Idea),
            title: g.title,
            amount_paise: g.amount_paise,
            on_day: g.on_day,
            txn_id: g.txn_id,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SeasonDb {
    season_id: String,
    label: String,
    month: i64,
    budget_paise: i64,
}

#[derive(Serialize)]
pub struct Season {
    pub id: String,
    pub label: String,
    pub month: i64,
    pub budget_paise: i64,
}

impl From<SeasonDb> for Season {
    fn from(s: SeasonDb) -> Season {
        Season { id: s.season_id, label: s.label, month: s.month, budget_paise: s.budget_paise }
    }
}

// ---------- loaders ----------

async fn find_person(db: &SqlitePool, user_id: &str, id: &str) -> Result<PersonDb, AppError> {
    sqlx::query_as::<_, PersonDb>(&format!("SELECT {PERSON_COLS} FROM people WHERE person_id = ? AND user_id = ?"))
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| not_found("Person not found"))
}

async fn find_date(db: &SqlitePool, user_id: &str, id: &str) -> Result<DateDb, AppError> {
    sqlx::query_as::<_, DateDb>(&format!("SELECT {DATE_COLS} FROM person_dates WHERE date_id = ? AND user_id = ?"))
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| not_found("Date not found"))
}

async fn find_gift(db: &SqlitePool, user_id: &str, id: &str) -> Result<GiftDb, AppError> {
    sqlx::query_as::<_, GiftDb>(&format!("SELECT {GIFT_COLS} FROM gifts WHERE gift_id = ? AND user_id = ?"))
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| not_found("Gift not found"))
}

async fn find_season(db: &SqlitePool, user_id: &str, id: &str) -> Result<SeasonDb, AppError> {
    sqlx::query_as::<_, SeasonDb>("SELECT season_id, label, month, budget_paise FROM gift_seasons WHERE season_id = ? AND user_id = ?")
        .bind(id)
        .bind(user_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| not_found("Season not found"))
}

async fn all_people(db: &SqlitePool, user_id: &str) -> Result<Vec<PersonDb>, AppError> {
    Ok(sqlx::query_as::<_, PersonDb>(&format!("SELECT {PERSON_COLS} FROM people WHERE user_id = ?"))
        .bind(user_id)
        .fetch_all(db)
        .await?)
}

async fn seasons_of(db: &SqlitePool, user_id: &str) -> Result<Vec<SeasonDb>, AppError> {
    Ok(sqlx::query_as::<_, SeasonDb>(
        "SELECT season_id, label, month, budget_paise FROM gift_seasons WHERE user_id = ? ORDER BY month, label, season_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

fn season_in(s: &SeasonDb) -> SeasonIn {
    SeasonIn { id: s.season_id.clone(), label: s.label.clone(), month: s.month as u8, budget_paise: s.budget_paise }
}

/// (date id, year, status) of every gift tied to a date.
async fn gift_marks(db: &SqlitePool, user_id: &str) -> Result<Vec<(String, i32, GiftStatus)>, AppError> {
    let rows: Vec<(String, Option<i64>, String)> =
        sqlx::query_as("SELECT date_id, for_year, status FROM gifts WHERE user_id = ? AND date_id IS NOT NULL AND for_year IS NOT NULL")
            .bind(user_id)
            .fetch_all(db)
            .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(d, y, s)| Some((d, y? as i32, GiftStatus::parse(&s)?)))
        .collect())
}

/// Dates of people who are not archived, with the years that already have a gift bought or given.
async fn dates_in(db: &SqlitePool, user_id: &str) -> Result<(Vec<DateIn>, Vec<(String, i32, GiftStatus)>), AppError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, String, i64, i64, Option<i64>, i64, Option<i64>)>(
        "SELECT d.date_id, d.person_id, p.name, p.circle, d.kind, d.label, d.month, d.day, d.year, d.remind_days, d.gift_budget_paise \
         FROM person_dates d JOIN people p ON p.person_id = d.person_id AND p.user_id = d.user_id \
         WHERE d.user_id = ? AND p.archived = 0 ORDER BY d.month, d.day, d.date_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    let marks = gift_marks(db, user_id).await?;
    let dates = rows
        .into_iter()
        .map(|(date_id, person_id, name, circle, kind, label, month, day, year, remind, budget)| {
            let settled_years = marks.iter().filter(|(d, _, s)| *d == date_id && s.settled()).map(|(_, y, _)| *y).collect();
            DateIn {
                date_id,
                person_id,
                name,
                circle: Circle::parse(&circle).unwrap_or(Circle::Other),
                kind: DateKind::parse(&kind).unwrap_or(DateKind::Other),
                label,
                month: month as u8,
                day: day as u8,
                year: year.map(|y| y as i32),
                remind_days: remind,
                gift_budget_paise: budget,
                settled_years,
            }
        })
        .collect();
    Ok((dates, marks))
}

async fn gifts_goal_id(db: &SqlitePool, user_id: &str) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar(
        "SELECT goal_id FROM goals WHERE user_id = ? AND archived = 0 AND lower(trim(name)) = 'gifts' ORDER BY created_at LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?)
}

/// The habit rules' input for one user.
pub async fn habits_input(db: &SqlitePool, user_id: &str, today: Date) -> Result<PeopleIn, AppError> {
    let people: Vec<PersonIn> = all_people(db, user_id)
        .await?
        .iter()
        .filter(|p| p.archived == 0)
        .map(|p| p.to_in(today))
        .collect();
    let (dates, _) = dates_in(db, user_id).await?;
    let seasons = seasons_of(db, user_id).await?.iter().map(season_in).collect();
    Ok(PeopleIn { people, dates, seasons, has_gifts_goal: gifts_goal_id(db, user_id).await?.is_some() })
}

/// Dates and seasons for the calendar feed.
pub async fn feed_input(db: &SqlitePool, user_id: &str) -> Result<(Vec<DateIn>, Vec<SeasonIn>), AppError> {
    let (dates, _) = dates_in(db, user_id).await?;
    let seasons = seasons_of(db, user_id).await?.iter().map(season_in).collect();
    Ok((dates, seasons))
}

/// Whether the calendar feed may show names.
pub async fn feed_names(db: &SqlitePool, user_id: &str) -> Result<bool, AppError> {
    let v: Option<i64> = sqlx::query_scalar("SELECT feed_names FROM user_settings WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(v.unwrap_or(0) != 0)
}

// ---------- people ----------

#[derive(Serialize)]
pub struct PeopleOut {
    pub people: Vec<PersonRow>,
    pub archived: Vec<PersonRow>,
    pub upcoming: Vec<Upcoming>,
    pub catch_up: Vec<PersonRow>,
}

fn by_circle_then_name(a: &PersonRow, b: &PersonRow) -> std::cmp::Ordering {
    (a.circle.rank(), a.name.to_lowercase(), &a.id).cmp(&(b.circle.rank(), b.name.to_lowercase(), &b.id))
}

pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<PeopleOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    let rows: Vec<PersonRow> = all_people(db, &user_id).await?.iter().map(|p| p.to_row(today)).collect();
    let (mut people, mut archived): (Vec<_>, Vec<_>) = rows.into_iter().partition(|p| !p.archived);
    people.sort_by(by_circle_then_name);
    archived.sort_by(by_circle_then_name);
    let mut catch_up: Vec<PersonRow> =
        people.iter().filter(|p| matches!(p.status, Some(Status::Overdue | Status::DueSoon))).cloned().collect();
    catch_up.sort_by(|a, b| (&a.next_due_on, &a.id).cmp(&(&b.next_due_on, &b.id)));
    let (dates, marks) = dates_in(db, &user_id).await?;
    let upcoming = people::upcoming(&dates, today, |date_id, year| {
        GiftState::best(marks.iter().filter(|(d, y, _)| d == date_id && *y == year).map(|(_, _, s)| *s))
    });
    Ok(Json(PeopleOut { people, archived, upcoming, catch_up }))
}

#[derive(Deserialize)]
pub struct NewPerson {
    pub name: String,
    pub circle: Circle,
    pub every_days: Option<i64>,
    pub note: Option<String>,
}

fn check_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() || too_long(name, 40) {
        return Err(unprocessable("A name needs 1 to 40 characters."));
    }
    Ok(name.to_string())
}

fn check_every(v: Option<i64>) -> Result<Option<i64>, AppError> {
    match v {
        Some(d) if !(7..=365).contains(&d) => Err(unprocessable("Catch up every 7 to 365 days.")),
        v => Ok(v),
    }
}

fn check_note(note: &str, max: usize) -> Result<String, AppError> {
    if too_long(note, max) {
        return Err(unprocessable(format!("Notes can be up to {max} characters.")));
    }
    Ok(note.to_string())
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<NewPerson>,
) -> Result<Json<PersonRow>, AppError> {
    let name = check_name(&p.name)?;
    let every = check_every(p.every_days)?;
    let note = check_note(p.note.as_deref().unwrap_or(""), 280)?;
    let id = nanoid!();
    sqlx::query("INSERT INTO people (person_id, user_id, name, circle, every_days, note) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user_id)
        .bind(&name)
        .bind(p.circle.as_str())
        .bind(every)
        .bind(&note)
        .execute(&state.db)
        .await?;
    let person = find_person(&state.db, &user_id, &id).await?;
    Ok(Json(person.to_row(today_local())))
}

#[derive(Serialize)]
pub struct PersonOut {
    pub person: PersonRow,
    pub dates: Vec<PersonDate>,
    pub gifts: Vec<Gift>,
    pub touches: Vec<Touch>,
}

pub async fn get(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<Json<PersonOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    let person = find_person(db, &user_id, &id).await?;
    let mut dates: Vec<PersonDate> =
        sqlx::query_as::<_, DateDb>(&format!("SELECT {DATE_COLS} FROM person_dates WHERE person_id = ? AND user_id = ?"))
            .bind(&id)
            .bind(&user_id)
            .fetch_all(db)
            .await?
            .iter()
            .map(|d| d.to_out(today))
            .collect();
    dates.sort_by(|a, b| (&a.next_on, &a.id).cmp(&(&b.next_on, &b.id)));
    let gifts = sqlx::query_as::<_, GiftDb>(&format!(
        "SELECT {GIFT_COLS} FROM gifts WHERE person_id = ? AND user_id = ? ORDER BY created_at DESC, rowid DESC"
    ))
    .bind(&id)
    .bind(&user_id)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(Gift::from)
    .collect();
    let touches = sqlx::query_as::<_, TouchDb>(
        "SELECT touch_id, person_id, on_day, kind, cost_paise, note FROM touches WHERE person_id = ? AND user_id = ? \
         ORDER BY on_day DESC, created_at DESC, rowid DESC LIMIT 20",
    )
    .bind(&id)
    .bind(&user_id)
    .fetch_all(db)
    .await?
    .into_iter()
    .map(Touch::from)
    .collect();
    Ok(Json(PersonOut { person: person.to_row(today), dates, gifts, touches }))
}

#[derive(Deserialize)]
pub struct PersonPatch {
    pub name: Option<String>,
    pub circle: Option<Circle>,
    #[serde(default, deserialize_with = "double_option")]
    pub every_days: Option<Option<i64>>,
    pub note: Option<String>,
    pub archived: Option<bool>,
}

pub async fn patch(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(p): Json<PersonPatch>,
) -> Result<Json<PersonRow>, AppError> {
    let cur = find_person(&state.db, &user_id, &id).await?;
    let name = match &p.name {
        Some(n) => check_name(n)?,
        None => cur.name.clone(),
    };
    let circle = p.circle.map(|c| c.as_str().to_string()).unwrap_or_else(|| cur.circle.clone());
    let every = match p.every_days {
        Some(v) => check_every(v)?,
        None => cur.every_days,
    };
    let note = match &p.note {
        Some(n) => check_note(n, 280)?,
        None => cur.note.clone(),
    };
    let archived = p.archived.map(i64::from).unwrap_or(cur.archived);
    sqlx::query("UPDATE people SET name = ?, circle = ?, every_days = ?, note = ?, archived = ? WHERE person_id = ? AND user_id = ?")
        .bind(&name)
        .bind(&circle)
        .bind(every)
        .bind(&note)
        .bind(archived)
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(find_person(&state.db, &user_id, &id).await?.to_row(today_local())))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM people WHERE person_id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Person not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------- touches ----------

#[derive(Deserialize)]
pub struct NewTouch {
    pub kind: TouchKind,
    pub on_day: Option<String>,
    pub cost_paise: Option<i64>,
    pub note: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TouchKind {
    Call,
    Visit,
    Message,
    Met,
}

impl TouchKind {
    fn as_str(self) -> &'static str {
        match self {
            TouchKind::Call => "call",
            TouchKind::Visit => "visit",
            TouchKind::Message => "message",
            TouchKind::Met => "met",
        }
    }
}

/// `last_contact_on` is the latest touch, or nothing when there are none.
async fn recompute_last_contact(db: &SqlitePool, user_id: &str, person_id: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE people SET last_contact_on = (SELECT MAX(on_day) FROM touches WHERE person_id = ? AND user_id = ?) \
         WHERE person_id = ? AND user_id = ?",
    )
    .bind(person_id)
    .bind(user_id)
    .bind(person_id)
    .bind(user_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn add_touch(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(p): Json<NewTouch>,
) -> Result<Json<PersonRow>, AppError> {
    let today = today_local();
    find_person(&state.db, &user_id, &id).await?;
    let on = match &p.on_day {
        Some(s) => parse_day(s, "The day")?,
        None => today,
    };
    if on > today {
        return Err(unprocessable("That day is still ahead."));
    }
    if let Some(c) = p.cost_paise {
        if p.kind != TouchKind::Visit {
            return Err(unprocessable("Only a visit has a cost."));
        }
        money(c, "The cost")?;
    }
    let note = check_note(p.note.as_deref().unwrap_or(""), 140)?;
    sqlx::query("INSERT INTO touches (touch_id, user_id, person_id, on_day, kind, cost_paise, note) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(nanoid!())
        .bind(&user_id)
        .bind(&id)
        .bind(on.to_string())
        .bind(p.kind.as_str())
        .bind(p.cost_paise)
        .bind(&note)
        .execute(&state.db)
        .await?;
    recompute_last_contact(&state.db, &user_id, &id).await?;
    Ok(Json(find_person(&state.db, &user_id, &id).await?.to_row(today)))
}

/// `DELETE /api/people/{id}/touches`: forgets every catch-up with this person, however old (the
/// person's page only shows the latest ones).
pub async fn clear_touches(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<Json<PersonRow>, AppError> {
    find_person(&state.db, &user_id, &id).await?;
    sqlx::query("DELETE FROM touches WHERE person_id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    recompute_last_contact(&state.db, &user_id, &id).await?;
    Ok(Json(find_person(&state.db, &user_id, &id).await?.to_row(today_local())))
}

pub async fn delete_touch(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(touch_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let person_id: Option<String> = sqlx::query_scalar("SELECT person_id FROM touches WHERE touch_id = ? AND user_id = ?")
        .bind(&touch_id)
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?;
    let person_id = person_id.ok_or_else(|| not_found("Catch-up not found"))?;
    sqlx::query("DELETE FROM touches WHERE touch_id = ? AND user_id = ?")
        .bind(&touch_id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    recompute_last_contact(&state.db, &user_id, &person_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- dates ----------

#[derive(Deserialize)]
pub struct NewDate {
    pub kind: DateKind,
    pub label: Option<String>,
    pub month: i64,
    pub day: i64,
    pub year: Option<i64>,
    pub remind_days: Option<i64>,
    pub gift_budget_paise: Option<i64>,
}

#[derive(Deserialize)]
pub struct DatePatch {
    pub kind: Option<DateKind>,
    pub label: Option<String>,
    pub month: Option<i64>,
    pub day: Option<i64>,
    #[serde(default, deserialize_with = "double_option")]
    pub year: Option<Option<i64>>,
    pub remind_days: Option<i64>,
    #[serde(default, deserialize_with = "double_option")]
    pub gift_budget_paise: Option<Option<i64>>,
}

struct DateFields {
    kind: DateKind,
    label: String,
    month: i64,
    day: i64,
    year: Option<i64>,
    remind_days: i64,
    budget: Option<i64>,
}

fn check_date(f: &DateFields, today: Date) -> Result<(), AppError> {
    if !(1..=12).contains(&f.month) || !(1..=31).contains(&f.day) || !people::possible_day(f.month as u8, f.day as u8) {
        return Err(unprocessable("That day does not exist in that month."));
    }
    if f.year.is_some_and(|y| y < 1900 || y > i64::from(today.year())) {
        return Err(unprocessable(format!("The year must be between 1900 and {}.", today.year())));
    }
    if !(0..=60).contains(&f.remind_days) {
        return Err(unprocessable("Remind between 0 and 60 days ahead."));
    }
    if too_long(&f.label, 40) {
        return Err(unprocessable("A label can be up to 40 characters."));
    }
    if let Some(b) = f.budget {
        money(b, "The gift budget")?;
    }
    Ok(())
}

pub async fn add_date(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(p): Json<NewDate>,
) -> Result<Json<PersonDate>, AppError> {
    let today = today_local();
    find_person(&state.db, &user_id, &id).await?;
    let f = DateFields {
        kind: p.kind,
        label: p.label.as_deref().unwrap_or("").trim().to_string(),
        month: p.month,
        day: p.day,
        year: p.year,
        remind_days: p.remind_days.unwrap_or(7),
        budget: p.gift_budget_paise,
    };
    check_date(&f, today)?;
    let date_id = nanoid!();
    sqlx::query(
        "INSERT INTO person_dates (date_id, user_id, person_id, kind, label, month, day, year, remind_days, gift_budget_paise) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&date_id)
    .bind(&user_id)
    .bind(&id)
    .bind(f.kind.as_str())
    .bind(&f.label)
    .bind(f.month)
    .bind(f.day)
    .bind(f.year)
    .bind(f.remind_days)
    .bind(f.budget)
    .execute(&state.db)
    .await?;
    Ok(Json(find_date(&state.db, &user_id, &date_id).await?.to_out(today)))
}

pub async fn patch_date(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(date_id): Path<String>,
    Json(p): Json<DatePatch>,
) -> Result<Json<PersonDate>, AppError> {
    let today = today_local();
    let cur = find_date(&state.db, &user_id, &date_id).await?;
    let f = DateFields {
        kind: p.kind.unwrap_or_else(|| DateKind::parse(&cur.kind).unwrap_or(DateKind::Other)),
        label: p.label.as_deref().map(|l| l.trim().to_string()).unwrap_or_else(|| cur.label.clone()),
        month: p.month.unwrap_or(cur.month),
        day: p.day.unwrap_or(cur.day),
        year: p.year.unwrap_or(cur.year),
        remind_days: p.remind_days.unwrap_or(cur.remind_days),
        budget: p.gift_budget_paise.unwrap_or(cur.gift_budget_paise),
    };
    check_date(&f, today)?;
    sqlx::query(
        "UPDATE person_dates SET kind = ?, label = ?, month = ?, day = ?, year = ?, remind_days = ?, gift_budget_paise = ? \
         WHERE date_id = ? AND user_id = ?",
    )
    .bind(f.kind.as_str())
    .bind(&f.label)
    .bind(f.month)
    .bind(f.day)
    .bind(f.year)
    .bind(f.remind_days)
    .bind(f.budget)
    .bind(&date_id)
    .bind(&user_id)
    .execute(&state.db)
    .await?;
    Ok(Json(find_date(&state.db, &user_id, &date_id).await?.to_out(today)))
}

pub async fn delete_date(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(date_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM person_dates WHERE date_id = ? AND user_id = ?")
        .bind(&date_id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Date not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------- gifts ----------

#[derive(Deserialize)]
pub struct NewGift {
    pub person_id: String,
    pub date_id: Option<String>,
    pub for_year: Option<i64>,
    pub title: String,
    pub status: Option<GiftStatus>,
    pub amount_paise: Option<i64>,
    pub on_day: Option<String>,
    pub txn_id: Option<String>,
}

#[derive(Deserialize)]
pub struct GiftPatch {
    pub title: Option<String>,
    pub status: Option<GiftStatus>,
    #[serde(default, deserialize_with = "double_option")]
    pub amount_paise: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub on_day: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    pub for_year: Option<Option<i64>>,
    #[serde(default, deserialize_with = "double_option")]
    pub txn_id: Option<Option<String>>,
}

fn check_title(t: &str) -> Result<String, AppError> {
    let t = t.trim();
    if t.is_empty() || too_long(t, 80) {
        return Err(unprocessable("A gift needs a title of 1 to 80 characters."));
    }
    Ok(t.to_string())
}

fn check_for_year(y: Option<i64>, today: Date) -> Result<(), AppError> {
    if y.is_some_and(|y| y < 1900 || y > i64::from(today.year()) + 2) {
        return Err(unprocessable("That year is not right."));
    }
    Ok(())
}

async fn check_txn(db: &SqlitePool, user_id: &str, txn_id: &Option<String>) -> Result<(), AppError> {
    if let Some(t) = txn_id {
        let found: Option<String> = sqlx::query_scalar("SELECT txn_id FROM transactions WHERE txn_id = ? AND user_id = ?")
            .bind(t)
            .bind(user_id)
            .fetch_optional(db)
            .await?;
        if found.is_none() {
            return Err(not_found("Transaction not found"));
        }
    }
    Ok(())
}

pub async fn add_gift(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<NewGift>,
) -> Result<Json<Gift>, AppError> {
    let db = &state.db;
    let today = today_local();
    find_person(db, &user_id, &p.person_id).await?;
    let title = check_title(&p.title)?;
    let status = p.status.unwrap_or(GiftStatus::Idea);
    if let Some(a) = p.amount_paise {
        money(a, "The amount")?;
    }
    check_for_year(p.for_year, today)?;
    let mut for_year = p.for_year;
    if let Some(date_id) = &p.date_id {
        let date = find_date(db, &user_id, date_id).await?;
        if date.person_id != p.person_id {
            return Err(not_found("Date not found"));
        }
        if for_year.is_none() {
            for_year = Some(i64::from(people::next_occurrence(date.month as u8, date.day as u8, today).year()));
        }
    }
    check_txn(db, &user_id, &p.txn_id).await?;
    let on_day = match &p.on_day {
        Some(s) => Some(parse_day(s, "The day")?.to_string()),
        None if status.settled() => Some(today.to_string()),
        None => None,
    };
    let id = nanoid!();
    sqlx::query(
        "INSERT INTO gifts (gift_id, user_id, person_id, date_id, for_year, status, title, amount_paise, on_day, txn_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&p.person_id)
    .bind(&p.date_id)
    .bind(for_year)
    .bind(status.as_str())
    .bind(&title)
    .bind(p.amount_paise)
    .bind(&on_day)
    .bind(&p.txn_id)
    .execute(db)
    .await?;
    Ok(Json(find_gift(db, &user_id, &id).await?.into()))
}

pub async fn patch_gift(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(gift_id): Path<String>,
    Json(p): Json<GiftPatch>,
) -> Result<Json<Gift>, AppError> {
    let db = &state.db;
    let today = today_local();
    let cur = find_gift(db, &user_id, &gift_id).await?;
    let title = match &p.title {
        Some(t) => check_title(t)?,
        None => cur.title.clone(),
    };
    let status = p.status.unwrap_or_else(|| GiftStatus::parse(&cur.status).unwrap_or(GiftStatus::Idea));
    let amount = p.amount_paise.unwrap_or(cur.amount_paise);
    if let Some(a) = amount {
        money(a, "The amount")?;
    }
    let for_year = p.for_year.unwrap_or(cur.for_year);
    check_for_year(for_year, today)?;
    let mut txn_id = p.txn_id.clone().unwrap_or(cur.txn_id.clone());
    if p.txn_id.is_some() {
        check_txn(db, &user_id, &txn_id).await?;
    }
    // Moving a settled gift back to an idea lets go of when it was given and the payment it was
    // matched to (unless the request sets them), so it really is an idea again.
    let back_to_idea = p.status == Some(GiftStatus::Idea) && GiftStatus::parse(&cur.status).is_some_and(|s| s.settled());
    let mut on_day = match &p.on_day {
        Some(Some(s)) => Some(parse_day(s, "The day")?.to_string()),
        Some(None) => None,
        None if back_to_idea => None,
        None => cur.on_day.clone(),
    };
    if back_to_idea && p.txn_id.is_none() {
        txn_id = Default::default();
    }
    if on_day.is_none() && status.settled() {
        on_day = Some(today.to_string());
    }
    sqlx::query(
        "UPDATE gifts SET title = ?, status = ?, amount_paise = ?, on_day = ?, for_year = ?, txn_id = ? WHERE gift_id = ? AND user_id = ?",
    )
    .bind(&title)
    .bind(status.as_str())
    .bind(amount)
    .bind(&on_day)
    .bind(for_year)
    .bind(&txn_id)
    .bind(&gift_id)
    .bind(&user_id)
    .execute(db)
    .await?;
    Ok(Json(find_gift(db, &user_id, &gift_id).await?.into()))
}

pub async fn delete_gift(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(gift_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM gifts WHERE gift_id = ? AND user_id = ?")
        .bind(&gift_id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Gift not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------- seasons ----------

#[derive(Serialize)]
pub struct SeasonsOut {
    pub seasons: Vec<Season>,
}

#[derive(Deserialize)]
pub struct NewSeason {
    pub label: String,
    pub month: i64,
    pub budget_paise: i64,
}

#[derive(Deserialize)]
pub struct SeasonPatch {
    pub label: Option<String>,
    pub month: Option<i64>,
    pub budget_paise: Option<i64>,
}

fn check_season(label: &str, month: i64, budget: i64) -> Result<String, AppError> {
    let label = label.trim();
    if label.is_empty() || too_long(label, 40) {
        return Err(unprocessable("A season needs a label of 1 to 40 characters."));
    }
    if !(1..=12).contains(&month) {
        return Err(unprocessable("The month must be between 1 and 12."));
    }
    money(budget, "The budget")?;
    Ok(label.to_string())
}

pub async fn list_seasons(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<SeasonsOut>, AppError> {
    let seasons = seasons_of(&state.db, &user_id).await?.into_iter().map(Season::from).collect();
    Ok(Json(SeasonsOut { seasons }))
}

pub async fn add_season(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<NewSeason>,
) -> Result<Json<Season>, AppError> {
    let label = check_season(&p.label, p.month, p.budget_paise)?;
    let id = nanoid!();
    sqlx::query("INSERT INTO gift_seasons (season_id, user_id, label, month, budget_paise) VALUES (?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user_id)
        .bind(&label)
        .bind(p.month)
        .bind(p.budget_paise)
        .execute(&state.db)
        .await?;
    Ok(Json(find_season(&state.db, &user_id, &id).await?.into()))
}

pub async fn patch_season(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(p): Json<SeasonPatch>,
) -> Result<Json<Season>, AppError> {
    let cur = find_season(&state.db, &user_id, &id).await?;
    let label = check_season(
        p.label.as_deref().unwrap_or(&cur.label),
        p.month.unwrap_or(cur.month),
        p.budget_paise.unwrap_or(cur.budget_paise),
    )?;
    sqlx::query("UPDATE gift_seasons SET label = ?, month = ?, budget_paise = ? WHERE season_id = ? AND user_id = ?")
        .bind(&label)
        .bind(p.month.unwrap_or(cur.month))
        .bind(p.budget_paise.unwrap_or(cur.budget_paise))
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(find_season(&state.db, &user_id, &id).await?.into()))
}

pub async fn delete_season(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let res = sqlx::query("DELETE FROM gift_seasons WHERE season_id = ? AND user_id = ?")
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(not_found("Season not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------- gift plan ----------

#[derive(Serialize)]
pub struct GiftPlanOut {
    pub months: Vec<MonthPlan>,
    pub total_paise: i64,
    pub monthly_paise: i64,
    pub given_this_year_paise: i64,
    pub goal_id: Option<String>,
}

pub async fn gift_plan(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<GiftPlanOut>, AppError> {
    let db = &state.db;
    let today = today_local();
    let (dates, seasons) = feed_input(db, &user_id).await?;
    let months = people::gift_months(&dates, &seasons, today);
    let total_paise = people::yearly_total(&dates, &seasons);
    let year = today.year();
    let given: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_paise), 0) FROM gifts WHERE user_id = ? AND status = 'given' \
         AND (substr(on_day, 1, 4) = ? OR (on_day IS NULL AND for_year = ?))",
    )
    .bind(&user_id)
    .bind(format!("{year:04}"))
    .bind(i64::from(year))
    .fetch_one(db)
    .await?;
    Ok(Json(GiftPlanOut {
        months,
        total_paise,
        monthly_paise: people::monthly_share(total_paise),
        given_this_year_paise: given,
        goal_id: gifts_goal_id(db, &user_id).await?,
    }))
}

// ---------- settings ----------

#[derive(Serialize, Deserialize)]
pub struct SettingsBody {
    pub feed_names: bool,
}

pub async fn get_settings(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<SettingsBody>, AppError> {
    Ok(Json(SettingsBody { feed_names: feed_names(&state.db, &user_id).await? }))
}

pub async fn put_settings(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<SettingsBody>,
) -> Result<Json<SettingsBody>, AppError> {
    sqlx::query(
        "INSERT INTO user_settings (user_id, feed_names) VALUES (?, ?) ON CONFLICT(user_id) DO UPDATE SET feed_names = excluded.feed_names",
    )
    .bind(&user_id)
    .bind(i64::from(p.feed_names))
    .execute(&state.db)
    .await?;
    Ok(Json(p))
}

// ---------- report ----------

/// The people section for a check-in period, or None when the user has no people and no seasons.
pub async fn report(db: &SqlitePool, user_id: &str, period: Period, today: Date) -> Result<Option<PeopleReport>, AppError> {
    let people_rows = all_people(db, user_id).await?;
    let seasons = seasons_of(db, user_id).await?;
    if people_rows.is_empty() && seasons.is_empty() {
        return Ok(None);
    }
    let seasons: Vec<SeasonIn> = seasons.iter().map(season_in).collect();
    let (dates, _) = dates_in(db, user_id).await?;
    let (from, to) = (period.start.to_string(), period.end.to_string());
    let prev = period.previous();
    let (pfrom, pto) = (prev.start.to_string(), prev.end.to_string());

    let (catch_ups, visit_cost): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(cost_paise), 0) FROM touches WHERE user_id = ? AND on_day BETWEEN ? AND ?",
    )
    .bind(user_id)
    .bind(&from)
    .bind(&to)
    .fetch_one(db)
    .await?;
    let given: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_paise), 0) FROM gifts WHERE user_id = ? AND status = 'given' AND on_day BETWEEN ? AND ?",
    )
    .bind(user_id)
    .bind(&from)
    .bind(&to)
    .fetch_one(db)
    .await?;
    let sent = |a: String, b: String| async move {
        sqlx::query_as::<_, (i64, i64)>(
            "SELECT COUNT(*), COALESCE(SUM(amount_paise), 0) FROM transactions \
             WHERE user_id = ? AND direction = 'debit' AND category = 'people' AND occurred_on BETWEEN ? AND ?",
        )
        .bind(user_id)
        .bind(a)
        .bind(b)
        .fetch_one(db)
        .await
    };
    let (_, sent_now) = sent(from.clone(), to.clone()).await?;
    let (prev_count, sent_prev) = sent(pfrom, pto).await?;

    let (budgeted, _) = people::occasions_between(&dates, &seasons, period.start, period.end);
    let (nq_from, nq_to) = people::next_quarter(period);
    let (nq_paise, nq_count) = people::occasions_between(&dates, &seasons, nq_from, nq_to);
    let overdue = people_rows
        .iter()
        .filter(|p| p.archived == 0)
        .filter(|p| p.to_in(today).status(today) == Some(Status::Overdue))
        .count() as i64;

    let mut r = PeopleReport {
        catch_ups,
        overdue_people: overdue,
        gifts_given_paise: given,
        gifts_budgeted_paise: budgeted,
        next_quarter_gifts_paise: nq_paise,
        next_quarter_occasions: nq_count,
        visit_cost_paise: visit_cost,
        sent_to_people_paise: sent_now,
        prev_sent_to_people_paise: (prev_count > 0).then_some(sent_prev),
        wins: Vec::new(),
        watch: Vec::new(),
    };
    people::notes(&mut r);
    Ok(Some(r))
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::{Duration, Month};

    async fn test_state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::<rusty_paseto::core::V4, rusty_paseto::core::Local>::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32][..]),
        );
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user(id: &str) -> User {
        Extension((id.to_string(), String::new()))
    }

    fn st(s: &AppState) -> State<AppState> {
        State(s.clone())
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("handler failed: {} {}", e.0, e.1),
        }
    }

    /// The status a failing handler returned.
    fn code<T>(r: Result<T, AppError>) -> StatusCode {
        match r {
            Ok(_) => panic!("expected an error"),
            Err(e) => e.0,
        }
    }

    fn day(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }

    async fn person(s: &AppState, who: &str, name: &str, circle: Circle, every: Option<i64>) -> PersonRow {
        ok(create(st(s), user(who), Json(NewPerson { name: name.into(), circle, every_days: every, note: None })).await).0
    }

    fn new_date(kind: DateKind, month: i64, d: i64) -> NewDate {
        NewDate { kind, label: None, month, day: d, year: None, remind_days: None, gift_budget_paise: None }
    }

    async fn date_for(s: &AppState, who: &str, person_id: &str, nd: NewDate) -> PersonDate {
        ok(add_date(st(s), user(who), Path(person_id.into()), Json(nd)).await).0
    }

    fn new_touch(kind: TouchKind, on: Option<&str>) -> NewTouch {
        NewTouch { kind, on_day: on.map(Into::into), cost_paise: None, note: None }
    }

    fn new_gift(person_id: &str, title: &str) -> NewGift {
        NewGift {
            person_id: person_id.into(),
            date_id: None,
            for_year: None,
            title: title.into(),
            status: None,
            amount_paise: None,
            on_day: None,
            txn_id: None,
        }
    }

    async fn run(s: &AppState, q: &str) {
        sqlx::query(q).execute(&s.db).await.unwrap_or_else(|e| panic!("{q}: {e}"));
    }

    // ----- people -----

    #[tokio::test]
    async fn creating_a_person_validates_and_trims() {
        let s = test_state().await;
        let bad = |name: &str, every, note: Option<&str>| NewPerson {
            name: name.into(),
            circle: Circle::Friend,
            every_days: every,
            note: note.map(Into::into),
        };
        for p in [
            bad("   ", None, None),
            bad(&"a".repeat(41), None, None),
            bad("Person A", Some(6), None),
            bad("Person A", Some(366), None),
            bad("Person A", None, Some(&"n".repeat(281))),
        ] {
            assert_eq!(code(create(st(&s), user("u"), Json(p)).await), StatusCode::UNPROCESSABLE_ENTITY);
        }
        let row = ok(create(st(&s), user("u"), Json(bad("  Person A ", Some(28), Some("Likes tea")))).await).0;
        assert_eq!((row.name.as_str(), row.every_days, row.note.as_str()), ("Person A", Some(28), "Likes tea"));
        assert_eq!(row.circle, Circle::Friend);
        assert!(!row.archived && row.last_contact_on.is_none());
        let edge = ok(create(st(&s), user("u"), Json(bad(&"a".repeat(40), Some(7), Some(&"n".repeat(280))))).await).0;
        assert_eq!(edge.every_days, Some(7));
        assert!(person_names(&s, "u").await.contains(&"Person A".to_string()));
    }

    async fn person_names(s: &AppState, who: &str) -> Vec<String> {
        ok(list(st(s), user(who)).await).0.people.into_iter().map(|p| p.name).collect()
    }

    #[tokio::test]
    async fn the_list_groups_by_circle_then_name_and_splits_archived() {
        let s = test_state().await;
        person(&s, "u", "Person D", Circle::Work, None).await;
        person(&s, "u", "person c", Circle::Friend, None).await;
        person(&s, "u", "Person B", Circle::Family, None).await;
        person(&s, "u", "Person A", Circle::Family, None).await;
        let close = person(&s, "u", "Person E", Circle::Close, None).await;
        let gone = person(&s, "u", "Person F", Circle::Family, None).await;
        person(&s, "other", "Person X", Circle::Family, None).await;
        ok(patch(st(&s), user("u"), Path(gone.id.clone()), Json(PersonPatch { name: None, circle: None, every_days: None, note: None, archived: Some(true) })).await);
        let out = ok(list(st(&s), user("u")).await).0;
        let names: Vec<&str> = out.people.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["Person A", "Person B", "Person E", "person c", "Person D"]);
        assert_eq!(out.archived.len(), 1);
        assert_eq!(out.archived[0].name, "Person F");
        assert!(out.archived[0].archived);
        assert_eq!(close.circle, Circle::Close);
        assert!(out.catch_up.is_empty() && out.upcoming.is_empty());
    }

    #[tokio::test]
    async fn catch_up_lists_overdue_and_due_soon_most_overdue_first() {
        let s = test_state().await;
        let today = today_local();
        let a = person(&s, "u", "Person A", Circle::Friend, Some(28)).await;
        let b = person(&s, "u", "Person B", Circle::Friend, Some(28)).await;
        let c = person(&s, "u", "Person C", Circle::Friend, Some(28)).await;
        person(&s, "u", "Person D", Circle::Friend, Some(28)).await; // never contacted: overdue since it was added? added today, so ok
        person(&s, "u", "Person E", Circle::Friend, None).await;
        let set = |id: String, ago: i64| {
            let s = s.clone();
            async move {
                let d = (today - Duration::days(ago)).to_string();
                sqlx::query("UPDATE people SET last_contact_on = ? WHERE person_id = ?").bind(d).bind(id).execute(&s.db).await.unwrap();
            }
        };
        set(a.id.clone(), 40).await;
        set(b.id.clone(), 60).await;
        set(c.id.clone(), 24).await;
        let out = ok(list(st(&s), user("u")).await).0;
        let ids: Vec<&str> = out.catch_up.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(ids, vec!["Person B", "Person A", "Person C"]);
        assert_eq!(out.catch_up[0].status, Some(Status::Overdue));
        assert_eq!(out.catch_up[2].status, Some(Status::DueSoon));
        assert_eq!(out.catch_up[0].next_due_on, Some((today - Duration::days(32)).to_string()));
        let d = out.people.iter().find(|p| p.name == "Person D").unwrap();
        assert_eq!(d.status, Some(Status::Ok), "the clock starts the day they were added");
        let e = out.people.iter().find(|p| p.name == "Person E").unwrap();
        assert_eq!((e.status, e.next_due_on.clone()), (None, None));
    }

    #[tokio::test]
    async fn another_users_person_is_a_404_everywhere() {
        let s = test_state().await;
        let theirs = person(&s, "other", "Person X", Circle::Family, Some(28)).await;
        let id = theirs.id.clone();
        let nf = StatusCode::NOT_FOUND;
        assert_eq!(code(get(st(&s), user("u"), Path(id.clone())).await), nf);
        assert_eq!(
            code(patch(st(&s), user("u"), Path(id.clone()), Json(PersonPatch { name: Some("Mine".into()), circle: None, every_days: None, note: None, archived: None })).await),
            nf
        );
        assert_eq!(code(add_touch(st(&s), user("u"), Path(id.clone()), Json(new_touch(TouchKind::Call, None))).await), nf);
        assert_eq!(code(add_date(st(&s), user("u"), Path(id.clone()), Json(new_date(DateKind::Birthday, 3, 3))).await), nf);
        assert_eq!(code(add_gift(st(&s), user("u"), Json(new_gift(&id, "Scarf"))).await), nf);
        assert_eq!(code(delete(st(&s), user("u"), Path(id.clone())).await), nf);
        let still = ok(get(st(&s), user("other"), Path(id)).await).0;
        assert_eq!(still.person.name, "Person X", "untouched");
    }

    #[tokio::test]
    async fn patching_changes_only_what_is_sent_and_null_clears_the_cadence() {
        let s = test_state().await;
        let p = person(&s, "u", "Person A", Circle::Friend, Some(28)).await;
        let body = |json: &str| Json(serde_json::from_str::<PersonPatch>(json).unwrap());
        let r = ok(patch(st(&s), user("u"), Path(p.id.clone()), body(r#"{"note":"Moved to Pune","circle":"close"}"#)).await).0;
        assert_eq!((r.note.as_str(), r.circle, r.every_days, r.name.as_str()), ("Moved to Pune", Circle::Close, Some(28), "Person A"));
        let r = ok(patch(st(&s), user("u"), Path(p.id.clone()), body(r#"{"every_days":null}"#)).await).0;
        assert_eq!((r.every_days, r.status), (None, None));
        let r = ok(patch(st(&s), user("u"), Path(p.id.clone()), body(r#"{"every_days":14}"#)).await).0;
        assert_eq!(r.every_days, Some(14));
        assert_eq!(code(patch(st(&s), user("u"), Path(p.id.clone()), body(r#"{"every_days":3}"#)).await), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(code(patch(st(&s), user("u"), Path(p.id.clone()), body(r#"{"name":""}"#)).await), StatusCode::UNPROCESSABLE_ENTITY);
        let r = ok(patch(st(&s), user("u"), Path(p.id.clone()), body(r#"{"archived":true}"#)).await).0;
        assert!(r.archived);
    }

    #[tokio::test]
    async fn deleting_a_person_removes_their_dates_touches_and_gifts() {
        let s = test_state().await;
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let d = date_for(&s, "u", &p.id, new_date(DateKind::Birthday, 3, 3)).await;
        ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Call, None))).await);
        let mut g = new_gift(&p.id, "Scarf");
        g.date_id = Some(d.id.clone());
        ok(add_gift(st(&s), user("u"), Json(g)).await);
        assert_eq!(ok(delete(st(&s), user("u"), Path(p.id.clone())).await), StatusCode::NO_CONTENT);
        for t in ["people", "person_dates", "touches", "gifts"] {
            let n: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {t}")).fetch_one(&s.db).await.unwrap();
            assert_eq!(n, 0, "{t}");
        }
        assert_eq!(code(delete(st(&s), user("u"), Path(p.id)).await), StatusCode::NOT_FOUND);
    }

    // ----- touches -----

    #[tokio::test]
    async fn touches_set_last_contact_to_the_latest_day_and_deleting_recomputes() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, Some(28)).await;
        let older = (today - Duration::days(10)).to_string();
        let oldest = (today - Duration::days(30)).to_string();
        let r = ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Call, Some(&older)))).await).0;
        assert_eq!(r.last_contact_on.as_deref(), Some(older.as_str()));
        let r = ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Message, None))).await).0;
        assert_eq!(r.last_contact_on, Some(today.to_string()));
        assert_eq!(r.status, Some(Status::Ok));
        // An older touch added later does not move last contact back.
        let r = ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Met, Some(&oldest)))).await).0;
        assert_eq!(r.last_contact_on, Some(today.to_string()));
        let touches = ok(get(st(&s), user("u"), Path(p.id.clone())).await).0.touches;
        assert_eq!(touches.len(), 3);
        assert_eq!(touches[0].on_day, today.to_string(), "newest first");
        assert_eq!(touches[2].on_day, oldest);
        // Delete the newest: back to the ten-days-ago call.
        assert_eq!(ok(delete_touch(st(&s), user("u"), Path(touches[0].id.clone())).await), StatusCode::NO_CONTENT);
        let r = ok(get(st(&s), user("u"), Path(p.id.clone())).await).0.person;
        assert_eq!(r.last_contact_on.as_deref(), Some(older.as_str()));
        for t in ok(get(st(&s), user("u"), Path(p.id.clone())).await).0.touches {
            ok(delete_touch(st(&s), user("u"), Path(t.id)).await);
        }
        let r = ok(get(st(&s), user("u"), Path(p.id)).await).0.person;
        assert_eq!(r.last_contact_on, None, "no touches, no last contact");
    }

    #[tokio::test]
    async fn touch_validation() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let tomorrow = (today + Duration::days(1)).to_string();
        let unp = StatusCode::UNPROCESSABLE_ENTITY;
        assert_eq!(code(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Call, Some(&tomorrow)))).await), unp);
        assert_eq!(code(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Call, Some("14 Dec")))).await), unp);
        let mut t = new_touch(TouchKind::Call, None);
        t.cost_paise = Some(5000);
        assert_eq!(code(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(t)).await), unp, "only visits have a cost");
        let mut t = new_touch(TouchKind::Visit, None);
        t.cost_paise = Some(0);
        assert_eq!(code(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(t)).await), unp);
        let mut t = new_touch(TouchKind::Visit, None);
        t.note = Some("n".repeat(141));
        assert_eq!(code(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(t)).await), unp);
        let mut t = new_touch(TouchKind::Visit, None);
        t.cost_paise = Some(150_000);
        t.note = Some("n".repeat(140));
        ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(t)).await);
        let touches = ok(get(st(&s), user("u"), Path(p.id)).await).0.touches;
        assert_eq!((touches.len(), touches[0].cost_paise), (1, Some(150_000)));
    }

    #[tokio::test]
    async fn only_the_last_twenty_touches_are_listed() {
        let s = test_state().await;
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        for _ in 0..22 {
            ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Call, None))).await);
        }
        assert_eq!(ok(get(st(&s), user("u"), Path(p.id)).await).0.touches.len(), 20);
    }

    #[tokio::test]
    async fn another_users_touch_cannot_be_deleted() {
        let s = test_state().await;
        let theirs = person(&s, "other", "Person X", Circle::Family, None).await;
        ok(add_touch(st(&s), user("other"), Path(theirs.id.clone()), Json(new_touch(TouchKind::Call, None))).await);
        let tid = ok(get(st(&s), user("other"), Path(theirs.id.clone())).await).0.touches[0].id.clone();
        assert_eq!(code(delete_touch(st(&s), user("u"), Path(tid.clone())).await), StatusCode::NOT_FOUND);
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM touches").fetch_one(&s.db).await.unwrap();
        assert_eq!(n, 1);
        assert_eq!(code(delete_touch(st(&s), user("other"), Path("nope".into())).await), StatusCode::NOT_FOUND);
    }

    // ----- dates -----

    #[tokio::test]
    async fn date_validation() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let unp = StatusCode::UNPROCESSABLE_ENTITY;
        let add = |nd: NewDate| add_date(st(&s), user("u"), Path(p.id.clone()), Json(nd));
        assert_eq!(code(add(new_date(DateKind::Birthday, 4, 31)).await), unp);
        assert_eq!(code(add(new_date(DateKind::Birthday, 2, 30)).await), unp);
        assert_eq!(code(add(new_date(DateKind::Birthday, 13, 1)).await), unp);
        assert_eq!(code(add(new_date(DateKind::Birthday, 1, 0)).await), unp);
        let mut nd = new_date(DateKind::Birthday, 3, 3);
        nd.year = Some(1899);
        assert_eq!(code(add(nd).await), unp);
        let mut nd = new_date(DateKind::Birthday, 3, 3);
        nd.year = Some(i64::from(today.year()) + 1);
        assert_eq!(code(add(nd).await), unp);
        let mut nd = new_date(DateKind::Birthday, 3, 3);
        nd.remind_days = Some(61);
        assert_eq!(code(add(nd).await), unp);
        let mut nd = new_date(DateKind::Birthday, 3, 3);
        nd.remind_days = Some(-1);
        assert_eq!(code(add(nd).await), unp);
        let mut nd = new_date(DateKind::Other, 3, 3);
        nd.label = Some("l".repeat(41));
        assert_eq!(code(add(nd).await), unp);
        let mut nd = new_date(DateKind::Birthday, 3, 3);
        nd.gift_budget_paise = Some(0);
        assert_eq!(code(add(nd).await), unp);
        // Valid edges.
        let mut nd = new_date(DateKind::Birthday, 2, 29);
        nd.year = Some(1900);
        nd.remind_days = Some(60);
        let d = ok(add(nd).await).0;
        assert_eq!((d.month, d.day, d.remind_days), (2, 29, 60));
        let mut nd = new_date(DateKind::Anniversary, 12, 31);
        nd.year = Some(i64::from(today.year()));
        nd.remind_days = Some(0);
        ok(add(nd).await);
    }

    #[tokio::test]
    async fn a_date_reports_next_day_days_away_and_turns() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let target = today + Duration::days(5);
        let mut nd = new_date(DateKind::Birthday, target.month() as i64, target.day() as i64);
        nd.year = Some(1966);
        nd.gift_budget_paise = Some(200_000);
        let d = date_for(&s, "u", &p.id, nd).await;
        assert_eq!(d.next_on, target.to_string());
        assert_eq!(d.days_away, 5);
        assert_eq!(d.turns, Some(i64::from(target.year()) - 1966));
        assert_eq!(d.remind_days, 7, "the default");
        assert_eq!(d.kind, DateKind::Birthday);
        // It shows up in the circle's upcoming list with no gift yet.
        let up = ok(list(st(&s), user("u")).await).0.upcoming;
        assert_eq!(up.len(), 1);
        assert_eq!((up[0].days_away, up[0].gift_status, up[0].name.as_str()), (5, GiftState::None, "Person A"));
        assert_eq!(up[0].gift_budget_paise, Some(200_000));
    }

    #[tokio::test]
    async fn dates_sort_soonest_first_on_the_person_page() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        for n in [30, 3, 12] {
            let t = today + Duration::days(n);
            date_for(&s, "u", &p.id, new_date(DateKind::Other, t.month() as i64, t.day() as i64)).await;
        }
        let days: Vec<i64> = ok(get(st(&s), user("u"), Path(p.id)).await).0.dates.iter().map(|d| d.days_away).collect();
        assert_eq!(days, vec![3, 12, 30]);
    }

    #[tokio::test]
    async fn patching_a_date_merges_and_can_clear_year_and_budget() {
        let s = test_state().await;
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let mut nd = new_date(DateKind::Birthday, 3, 3);
        nd.year = Some(1980);
        nd.gift_budget_paise = Some(100_000);
        let d = date_for(&s, "u", &p.id, nd).await;
        let body = |json: &str| Json(serde_json::from_str::<DatePatch>(json).unwrap());
        let r = ok(patch_date(st(&s), user("u"), Path(d.id.clone()), body(r#"{"day":4,"remind_days":10}"#)).await).0;
        assert_eq!((r.month, r.day, r.year, r.remind_days, r.gift_budget_paise), (3, 4, Some(1980), 10, Some(100_000)));
        let r = ok(patch_date(st(&s), user("u"), Path(d.id.clone()), body(r#"{"year":null,"gift_budget_paise":null}"#)).await).0;
        assert_eq!((r.year, r.gift_budget_paise, r.turns), (None, None, None));
        // A combined month and day that cannot exist is refused.
        assert_eq!(code(patch_date(st(&s), user("u"), Path(d.id.clone()), body(r#"{"month":2,"day":30}"#)).await), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(code(patch_date(st(&s), user("u"), Path(d.id.clone()), body(r#"{"month":4,"day":31}"#)).await), StatusCode::UNPROCESSABLE_ENTITY);
        let r = ok(patch_date(st(&s), user("u"), Path(d.id.clone()), body(r#"{"kind":"other","label":" Housewarming "}"#)).await).0;
        assert_eq!((r.kind, r.label.as_str()), (DateKind::Other, "Housewarming"));
    }

    #[tokio::test]
    async fn another_users_date_is_a_404() {
        let s = test_state().await;
        let theirs = person(&s, "other", "Person X", Circle::Family, None).await;
        let d = date_for(&s, "other", &theirs.id, new_date(DateKind::Birthday, 3, 3)).await;
        let empty = || Json(serde_json::from_str::<DatePatch>(r#"{"day":5}"#).unwrap());
        assert_eq!(code(patch_date(st(&s), user("u"), Path(d.id.clone()), empty()).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete_date(st(&s), user("u"), Path(d.id.clone())).await), StatusCode::NOT_FOUND);
        assert_eq!(ok(delete_date(st(&s), user("other"), Path(d.id.clone())).await), StatusCode::NO_CONTENT);
        assert_eq!(code(delete_date(st(&s), user("other"), Path(d.id)).await), StatusCode::NOT_FOUND);
    }

    // ----- gifts -----

    #[tokio::test]
    async fn a_gift_for_a_date_defaults_its_year_and_a_bought_gift_its_day() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let target = today + Duration::days(20);
        let d = date_for(&s, "u", &p.id, new_date(DateKind::Birthday, target.month() as i64, target.day() as i64)).await;
        let mut g = new_gift(&p.id, " Scarf ");
        g.date_id = Some(d.id.clone());
        let idea = ok(add_gift(st(&s), user("u"), Json(g)).await).0;
        assert_eq!((idea.title.as_str(), idea.status, idea.for_year), ("Scarf", GiftStatus::Idea, Some(i64::from(target.year()))));
        assert_eq!(idea.on_day, None);
        let mut g = new_gift(&p.id, "Book");
        g.status = Some(GiftStatus::Bought);
        g.amount_paise = Some(45_000);
        let bought = ok(add_gift(st(&s), user("u"), Json(g)).await).0;
        assert_eq!(bought.on_day, Some(today.to_string()));
        assert_eq!(bought.amount_paise, Some(45_000));
        assert_eq!(bought.for_year, None);
        let page = ok(get(st(&s), user("u"), Path(p.id.clone())).await).0;
        assert_eq!(page.gifts[0].title, "Book", "newest first");
        // The upcoming list shows the best status; buying the scarf lifts it.
        assert_eq!(ok(list(st(&s), user("u")).await).0.upcoming[0].gift_status, GiftState::Idea);
        let body = Json(serde_json::from_str::<GiftPatch>(r#"{"status":"bought"}"#).unwrap());
        let moved = ok(patch_gift(st(&s), user("u"), Path(idea.id.clone()), body).await).0;
        assert_eq!((moved.status, moved.on_day), (GiftStatus::Bought, Some(today.to_string())));
        assert_eq!(ok(list(st(&s), user("u")).await).0.upcoming[0].gift_status, GiftState::Bought);
        let body = Json(serde_json::from_str::<GiftPatch>(r#"{"status":"given","amount_paise":120000,"on_day":"2026-03-04"}"#).unwrap());
        let given = ok(patch_gift(st(&s), user("u"), Path(idea.id.clone()), body).await).0;
        assert_eq!((given.status, given.amount_paise, given.on_day.as_deref()), (GiftStatus::Given, Some(120_000), Some("2026-03-04")));
        assert_eq!(ok(list(st(&s), user("u")).await).0.upcoming[0].gift_status, GiftState::Given);
        let body = Json(serde_json::from_str::<GiftPatch>(r#"{"amount_paise":null}"#).unwrap());
        assert_eq!(ok(patch_gift(st(&s), user("u"), Path(idea.id), body).await).0.amount_paise, None);
    }

    #[tokio::test]
    async fn moving_a_given_gift_back_to_an_idea_lets_go_of_its_day_and_clearing_catch_ups_resets_last_contact() {
        let s = test_state().await;
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let mut g = new_gift(&p.id, "Book");
        g.status = Some(GiftStatus::Given);
        g.amount_paise = Some(45_000);
        let given = ok(add_gift(st(&s), user("u"), Json(g)).await).0;
        assert!(given.on_day.is_some());
        let body = Json(serde_json::from_str::<GiftPatch>(r#"{"status":"idea"}"#).unwrap());
        let idea = ok(patch_gift(st(&s), user("u"), Path(given.id.clone()), body).await).0;
        assert_eq!((idea.status, idea.on_day.clone(), idea.amount_paise), (GiftStatus::Idea, None, Some(45_000)));

        // Every catch-up goes, however many there are, and last contact goes with them.
        for _ in 0..25 {
            ok(add_touch(st(&s), user("u"), Path(p.id.clone()), Json(new_touch(TouchKind::Call, None))).await);
        }
        let cleared = ok(clear_touches(st(&s), user("u"), Path(p.id.clone())).await).0;
        assert_eq!(cleared.last_contact_on, None);
        let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM touches WHERE person_id = ?").bind(&p.id).fetch_one(&s.db).await.unwrap();
        assert_eq!(left, 0);
        assert_eq!(code(clear_touches(st(&s), user("other"), Path(p.id)).await), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn gift_validation() {
        let s = test_state().await;
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let unp = StatusCode::UNPROCESSABLE_ENTITY;
        assert_eq!(code(add_gift(st(&s), user("u"), Json(new_gift(&p.id, " "))).await), unp);
        assert_eq!(code(add_gift(st(&s), user("u"), Json(new_gift(&p.id, &"t".repeat(81)))).await), unp);
        let mut g = new_gift(&p.id, "Scarf");
        g.amount_paise = Some(0);
        assert_eq!(code(add_gift(st(&s), user("u"), Json(g)).await), unp);
        let mut g = new_gift(&p.id, "Scarf");
        g.on_day = Some("yesterday".into());
        assert_eq!(code(add_gift(st(&s), user("u"), Json(g)).await), unp);
        ok(add_gift(st(&s), user("u"), Json(new_gift(&p.id, &"t".repeat(80)))).await);
    }

    #[tokio::test]
    async fn a_gift_can_only_point_at_the_users_own_person_date_and_transaction() {
        let s = test_state().await;
        let mine = person(&s, "u", "Person A", Circle::Family, None).await;
        let mine2 = person(&s, "u", "Person B", Circle::Family, None).await;
        let theirs = person(&s, "other", "Person X", Circle::Family, None).await;
        let my_date = date_for(&s, "u", &mine.id, new_date(DateKind::Birthday, 3, 3)).await;
        let their_date = date_for(&s, "other", &theirs.id, new_date(DateKind::Birthday, 3, 3)).await;
        run(&s, "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key) VALUES ('t-mine', 'u', '2026-03-01', 'debit', 100000, 'Shop', 'shopping', 'x', 'manual', 'k1')").await;
        run(&s, "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key) VALUES ('t-theirs', 'other', '2026-03-01', 'debit', 100000, 'Shop', 'shopping', 'x', 'manual', 'k2')").await;
        let nf = StatusCode::NOT_FOUND;
        let mut g = new_gift(&mine.id, "Scarf");
        g.date_id = Some(their_date.id);
        assert_eq!(code(add_gift(st(&s), user("u"), Json(g)).await), nf, "another user's date");
        let mut g = new_gift(&mine2.id, "Scarf");
        g.date_id = Some(my_date.id.clone());
        assert_eq!(code(add_gift(st(&s), user("u"), Json(g)).await), nf, "a date of a different person");
        let mut g = new_gift(&mine.id, "Scarf");
        g.txn_id = Some("t-theirs".into());
        assert_eq!(code(add_gift(st(&s), user("u"), Json(g)).await), nf, "another user's transaction");
        let mut g = new_gift(&mine.id, "Scarf");
        g.date_id = Some(my_date.id);
        g.txn_id = Some("t-mine".into());
        let gift = ok(add_gift(st(&s), user("u"), Json(g)).await).0;
        assert_eq!(gift.txn_id.as_deref(), Some("t-mine"));
        // Patching the transaction is checked the same way.
        let body = Json(serde_json::from_str::<GiftPatch>(r#"{"txn_id":"t-theirs"}"#).unwrap());
        assert_eq!(code(patch_gift(st(&s), user("u"), Path(gift.id.clone()), body).await), nf);
        let body = Json(serde_json::from_str::<GiftPatch>(r#"{"txn_id":null}"#).unwrap());
        assert_eq!(ok(patch_gift(st(&s), user("u"), Path(gift.id), body).await).0.txn_id, None);
    }

    #[tokio::test]
    async fn another_users_gift_is_a_404() {
        let s = test_state().await;
        let theirs = person(&s, "other", "Person X", Circle::Family, None).await;
        let g = ok(add_gift(st(&s), user("other"), Json(new_gift(&theirs.id, "Scarf"))).await).0;
        let body = || Json(serde_json::from_str::<GiftPatch>(r#"{"title":"Mine now"}"#).unwrap());
        assert_eq!(code(patch_gift(st(&s), user("u"), Path(g.id.clone()), body()).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete_gift(st(&s), user("u"), Path(g.id.clone())).await), StatusCode::NOT_FOUND);
        assert_eq!(ok(delete_gift(st(&s), user("other"), Path(g.id.clone())).await), StatusCode::NO_CONTENT);
        assert_eq!(code(delete_gift(st(&s), user("other"), Path(g.id)).await), StatusCode::NOT_FOUND);
    }

    // ----- seasons -----

    #[tokio::test]
    async fn seasons_crud_validation_and_isolation() {
        let s = test_state().await;
        let unp = StatusCode::UNPROCESSABLE_ENTITY;
        let new = |l: &str, m, b| Json(NewSeason { label: l.into(), month: m, budget_paise: b });
        assert_eq!(code(add_season(st(&s), user("u"), new("", 10, 100)).await), unp);
        assert_eq!(code(add_season(st(&s), user("u"), new(&"l".repeat(41), 10, 100)).await), unp);
        assert_eq!(code(add_season(st(&s), user("u"), new("Diwali", 0, 100)).await), unp);
        assert_eq!(code(add_season(st(&s), user("u"), new("Diwali", 13, 100)).await), unp);
        assert_eq!(code(add_season(st(&s), user("u"), new("Diwali", 10, 0)).await), unp);
        let diwali = ok(add_season(st(&s), user("u"), new(" Diwali ", 10, 500_000)).await).0;
        let rakhi = ok(add_season(st(&s), user("u"), new("Rakhi", 8, 100_000)).await).0;
        let theirs = ok(add_season(st(&s), user("other"), new("Eid", 4, 200_000)).await).0;
        assert_eq!(diwali.label, "Diwali");
        let list = ok(list_seasons(st(&s), user("u")).await).0.seasons;
        assert_eq!(list.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), vec!["Rakhi", "Diwali"], "by month");
        let body = |j: &str| Json(serde_json::from_str::<SeasonPatch>(j).unwrap());
        let r = ok(patch_season(st(&s), user("u"), Path(diwali.id.clone()), body(r#"{"budget_paise":600000}"#)).await).0;
        assert_eq!((r.budget_paise, r.month, r.label.as_str()), (600_000, 10, "Diwali"));
        assert_eq!(code(patch_season(st(&s), user("u"), Path(diwali.id.clone()), body(r#"{"month":14}"#)).await), unp);
        assert_eq!(code(patch_season(st(&s), user("u"), Path(theirs.id.clone()), body(r#"{"month":5}"#)).await), StatusCode::NOT_FOUND);
        assert_eq!(code(delete_season(st(&s), user("u"), Path(theirs.id.clone())).await), StatusCode::NOT_FOUND);
        assert_eq!(ok(delete_season(st(&s), user("u"), Path(rakhi.id)).await), StatusCode::NO_CONTENT);
        assert_eq!(ok(list_seasons(st(&s), user("u")).await).0.seasons.len(), 1);
        assert_eq!(ok(list_seasons(st(&s), user("other")).await).0.seasons.len(), 1);
    }

    // ----- gift plan -----

    #[tokio::test]
    async fn the_gift_plan_sums_dates_and_seasons_and_finds_the_gifts_goal() {
        let s = test_state().await;
        let today = today_local();
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let gone = person(&s, "u", "Person B", Circle::Family, None).await;
        let x = person(&s, "other", "Person X", Circle::Family, None).await;
        let in_month = |n: i64| {
            let d = today + Duration::days(n);
            (d.month() as i64, d.day() as i64)
        };
        let (m, d) = in_month(0);
        let mut nd = new_date(DateKind::Birthday, m, d);
        nd.gift_budget_paise = Some(200_000);
        date_for(&s, "u", &p.id, nd).await;
        let mut nd = new_date(DateKind::Anniversary, m, d);
        nd.gift_budget_paise = Some(100_000);
        date_for(&s, "u", &gone.id, nd).await;
        let mut nd = new_date(DateKind::Birthday, 5, 5);
        nd.gift_budget_paise = None;
        date_for(&s, "u", &p.id, nd).await;
        let mut nd = new_date(DateKind::Birthday, m, d);
        nd.gift_budget_paise = Some(999_999);
        date_for(&s, "other", &x.id, nd).await;
        ok(patch(st(&s), user("u"), Path(gone.id.clone()), Json(PersonPatch { name: None, circle: None, every_days: None, note: None, archived: Some(true) })).await);
        ok(add_season(st(&s), user("u"), Json(NewSeason { label: "Diwali".into(), month: m, budget_paise: 300_000 })).await);
        let out = ok(gift_plan(st(&s), user("u")).await).0;
        assert_eq!(out.months.len(), 12);
        assert_eq!(out.months[0].month, format!("{:04}-{:02}", today.year(), today.month() as u8));
        assert_eq!((out.months[0].budget_paise, out.months[0].occasions), (500_000, 2), "archived people's dates do not count");
        assert_eq!(out.total_paise, 500_000);
        assert_eq!(out.monthly_paise, 41_667);
        assert_eq!(out.goal_id, None);
        assert_eq!(out.given_this_year_paise, 0);

        // Goals: another user's, an archived one and a differently named one do not count.
        run(&s, "INSERT INTO goals (goal_id, user_id, name, target_paise) VALUES ('g-other', 'other', 'Gifts', 100000)").await;
        run(&s, "INSERT INTO goals (goal_id, user_id, name, target_paise, archived) VALUES ('g-old', 'u', 'Gifts', 100000, 1)").await;
        run(&s, "INSERT INTO goals (goal_id, user_id, name, target_paise) VALUES ('g-trip', 'u', 'Gifts trip', 100000)").await;
        assert_eq!(ok(gift_plan(st(&s), user("u")).await).0.goal_id, None);
        run(&s, "INSERT INTO goals (goal_id, user_id, name, target_paise) VALUES ('g-mine', 'u', ' gIFts ', 100000)").await;
        assert_eq!(ok(gift_plan(st(&s), user("u")).await).0.goal_id.as_deref(), Some("g-mine"));

        // Given this year: this year's given gifts, not ideas, not last year's, not other users'.
        let mut g = new_gift(&p.id, "Scarf");
        g.status = Some(GiftStatus::Given);
        g.amount_paise = Some(150_000);
        ok(add_gift(st(&s), user("u"), Json(g)).await);
        let mut g = new_gift(&p.id, "Book");
        g.status = Some(GiftStatus::Given);
        g.amount_paise = Some(50_000);
        g.on_day = Some(format!("{}-01-02", today.year() - 1));
        ok(add_gift(st(&s), user("u"), Json(g)).await);
        let mut g = new_gift(&p.id, "Idea");
        g.amount_paise = Some(70_000);
        ok(add_gift(st(&s), user("u"), Json(g)).await);
        let mut g = new_gift(&x.id, "Theirs");
        g.status = Some(GiftStatus::Given);
        g.amount_paise = Some(900_000);
        ok(add_gift(st(&s), user("other"), Json(g)).await);
        assert_eq!(ok(gift_plan(st(&s), user("u")).await).0.given_this_year_paise, 150_000);
    }

    // ----- settings -----

    #[tokio::test]
    async fn feed_names_defaults_off_and_upserts() {
        let s = test_state().await;
        assert!(!ok(get_settings(st(&s), user("u")).await).0.feed_names);
        assert!(ok(put_settings(st(&s), user("u"), Json(SettingsBody { feed_names: true })).await).0.feed_names);
        assert!(ok(get_settings(st(&s), user("u")).await).0.feed_names);
        assert!(!ok(get_settings(st(&s), user("other")).await).0.feed_names, "per user");
        // With an existing settings row the other columns stay put.
        run(&s, "INSERT INTO user_settings (user_id, week_start) VALUES ('other', 3)").await;
        ok(put_settings(st(&s), user("other"), Json(SettingsBody { feed_names: true })).await);
        let ws: i64 = sqlx::query_scalar("SELECT week_start FROM user_settings WHERE user_id = 'other'").fetch_one(&s.db).await.unwrap();
        assert_eq!(ws, 3);
        ok(put_settings(st(&s), user("u"), Json(SettingsBody { feed_names: false })).await);
        assert!(!ok(get_settings(st(&s), user("u")).await).0.feed_names);
    }

    // ----- habits input and report -----

    #[tokio::test]
    async fn habit_input_leaves_out_archived_people_and_marks_settled_gift_years() {
        let s = test_state().await;
        let today = today_local();
        let a = person(&s, "u", "Person A", Circle::Family, Some(28)).await;
        let b = person(&s, "u", "Person B", Circle::Family, Some(28)).await;
        person(&s, "other", "Person X", Circle::Family, Some(28)).await;
        let target = today + Duration::days(10);
        let mut nd = new_date(DateKind::Birthday, target.month() as i64, target.day() as i64);
        nd.gift_budget_paise = Some(200_000);
        let da = date_for(&s, "u", &a.id, nd).await;
        let mut nd = new_date(DateKind::Birthday, 1, 1);
        nd.gift_budget_paise = Some(100_000);
        date_for(&s, "u", &b.id, nd).await;
        ok(patch(st(&s), user("u"), Path(b.id.clone()), Json(PersonPatch { name: None, circle: None, every_days: None, note: None, archived: Some(true) })).await);
        let mut g = new_gift(&a.id, "Scarf");
        g.date_id = Some(da.id.clone());
        g.status = Some(GiftStatus::Bought);
        ok(add_gift(st(&s), user("u"), Json(g)).await);
        let mut g = new_gift(&a.id, "Idea only");
        g.date_id = Some(da.id.clone());
        g.for_year = Some(i64::from(target.year()) - 1);
        ok(add_gift(st(&s), user("u"), Json(g)).await);
        let input = ok(habits_input(&s.db, "u", today).await);
        assert_eq!(input.people.len(), 1);
        assert_eq!(input.people[0].name, "Person A");
        assert_eq!(input.dates.len(), 1);
        assert_eq!(input.dates[0].name, "Person A");
        assert_eq!(input.dates[0].settled_years, vec![target.year()]);
        assert!(!input.has_gifts_goal);
        // The rules then stay quiet about that gift.
        let habits = crate::planner::people::habits(&input, today);
        assert!(habits.iter().all(|h| !h.key.starts_with("gift:")), "{habits:?}");
        assert!(ok(habits_input(&s.db, "nobody", today).await).people.is_empty());
    }

    #[tokio::test]
    async fn the_report_is_none_without_people_or_seasons() {
        let s = test_state().await;
        let period = Period { start: day(2026, 7, 1), end: day(2026, 9, 30) };
        assert!(ok(report(&s.db, "u", period, day(2026, 10, 1)).await).is_none());
        person(&s, "other", "Person X", Circle::Family, None).await;
        assert!(ok(report(&s.db, "u", period, day(2026, 10, 1)).await).is_none(), "another user's people do not count");
        ok(add_season(st(&s), user("u"), Json(NewSeason { label: "Diwali".into(), month: 10, budget_paise: 100_000 })).await);
        let r = ok(report(&s.db, "u", period, day(2026, 10, 1)).await).unwrap();
        assert_eq!((r.next_quarter_gifts_paise, r.next_quarter_occasions), (100_000, 1));
    }

    #[tokio::test]
    async fn the_report_numbers() {
        let s = test_state().await;
        let today = day(2026, 10, 1);
        let period = Period { start: day(2026, 7, 1), end: day(2026, 9, 30) };
        let a = person(&s, "u", "Person A", Circle::Family, Some(28)).await;
        let b = person(&s, "u", "Person B", Circle::Friend, Some(28)).await;
        let c = person(&s, "u", "Person C", Circle::Friend, None).await;
        let x = person(&s, "other", "Person X", Circle::Friend, Some(28)).await;
        let a_id = a.id.clone();
        // Overdue: A was last seen long ago and B was added long ago. Archived people are not counted.
        for id in [&a.id, &b.id, &c.id, &x.id] {
            sqlx::query("UPDATE people SET created_at = '2026-01-01 10:00:00' WHERE person_id = ?").bind(id).execute(&s.db).await.unwrap();
        }
        let touch = |who: &'static str, pid: String, kind: &'static str, on: &'static str, cost: Option<i64>| {
            let s = s.clone();
            async move {
                sqlx::query("INSERT INTO touches (touch_id, user_id, person_id, on_day, kind, cost_paise) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(nanoid!()).bind(who).bind(pid).bind(on).bind(kind).bind(cost)
                    .execute(&s.db).await.unwrap();
            }
        };
        touch("u", a_id.clone(), "call", "2026-07-01", None).await; // first day counts
        touch("u", a_id.clone(), "visit", "2026-08-10", Some(250_000)).await;
        touch("u", a_id.clone(), "visit", "2026-09-30", Some(50_000)).await; // last day counts
        touch("u", a_id.clone(), "message", "2026-06-30", None).await; // before the period
        touch("u", a_id.clone(), "message", "2026-10-01", None).await; // after the period
        touch("other", x.id.clone(), "call", "2026-08-01", None).await;
        sqlx::query("UPDATE people SET last_contact_on = '2026-06-01' WHERE person_id = ?").bind(&a_id).execute(&s.db).await.unwrap();
        ok(patch(st(&s), user("u"), Path(c.id.clone()), Json(PersonPatch { name: None, circle: None, every_days: Some(Some(30)), note: None, archived: Some(true) })).await);
        // Dates: one in the period (budget 2000), one next quarter (budget 1000), one without budget.
        let mut nd = new_date(DateKind::Birthday, 8, 15);
        nd.gift_budget_paise = Some(200_000);
        date_for(&s, "u", &a.id, nd).await;
        let mut nd = new_date(DateKind::Birthday, 11, 20);
        nd.gift_budget_paise = Some(100_000);
        date_for(&s, "u", &b.id, nd).await;
        date_for(&s, "u", &b.id, new_date(DateKind::Birthday, 11, 21)).await;
        ok(add_season(st(&s), user("u"), Json(NewSeason { label: "Diwali".into(), month: 11, budget_paise: 400_000 })).await);
        ok(add_season(st(&s), user("u"), Json(NewSeason { label: "Rakhi".into(), month: 8, budget_paise: 150_000 })).await);
        // Gifts given: one inside the period, one outside, an idea, and another user's.
        for (title, status, amount, on) in [
            ("In", GiftStatus::Given, 180_000, "2026-08-15"),
            ("Out", GiftStatus::Given, 90_000, "2026-10-02"),
            ("Bought", GiftStatus::Bought, 70_000, "2026-08-20"),
        ] {
            let mut g = new_gift(&a_id, title);
            g.status = Some(status);
            g.amount_paise = Some(amount);
            g.on_day = Some(on.into());
            ok(add_gift(st(&s), user("u"), Json(g)).await);
        }
        // Money sent to people.
        for (id, who, on, dir, cat, amt) in [
            ("t1", "u", "2026-07-02", "debit", "people", 500_000),
            ("t2", "u", "2026-09-29", "debit", "people", 250_000),
            ("t3", "u", "2026-09-30", "credit", "people", 900_000),
            ("t4", "u", "2026-08-01", "debit", "food", 900_000),
            ("t5", "other", "2026-08-01", "debit", "people", 900_000),
            ("t6", "u", "2026-04-15", "debit", "people", 300_000),
        ] {
            sqlx::query("INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key) VALUES (?, ?, ?, ?, ?, 'M', ?, 'x', 'manual', ?)")
                .bind(id).bind(who).bind(on).bind(dir).bind(amt).bind(cat).bind(id)
                .execute(&s.db).await.unwrap();
        }
        let r = ok(report(&s.db, "u", period, today).await).unwrap();
        assert_eq!(r.catch_ups, 3);
        assert_eq!(r.visit_cost_paise, 300_000);
        assert_eq!(r.overdue_people, 2, "A and B; C is archived");
        assert_eq!(r.gifts_given_paise, 180_000);
        assert_eq!(r.gifts_budgeted_paise, 350_000, "A's 15 Aug and the Rakhi season on 1 Aug");
        assert_eq!((r.next_quarter_gifts_paise, r.next_quarter_occasions), (500_000, 2), "B's 20 Nov and Diwali on 1 Nov");
        assert_eq!(r.sent_to_people_paise, 750_000);
        assert_eq!(r.prev_sent_to_people_paise, Some(300_000), "1 Apr to 30 Jun");
        assert_eq!(r.wins, vec!["You caught up 3 times this quarter.", "Gifts came to \u{20b9}1,800, within the \u{20b9}3,500 planned."]);
        assert_eq!(r.watch, vec!["2 people are past their catch-up date.", "\u{20b9}5,000 of gifts are due next quarter across 2 occasions."]);
        let text = r.wins.iter().chain(&r.watch).cloned().collect::<String>();
        assert!(!text.contains("Person"), "no names in the report");
        // No spend before the period: previous is None. Another user sees only their own.
        let later = Period { start: day(2026, 1, 1), end: day(2026, 3, 31) };
        assert_eq!(ok(report(&s.db, "u", later, today).await).unwrap().prev_sent_to_people_paise, None);
        let theirs = ok(report(&s.db, "other", period, today).await).unwrap();
        assert_eq!((theirs.catch_ups, theirs.sent_to_people_paise, theirs.gifts_given_paise), (1, 900_000, 0));
    }

    // ----- routes -----

    #[tokio::test]
    async fn routes_resolve_fixed_paths_before_ids() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;
        let s = test_state().await;
        let app = crate::routes::people::routes(s.clone()).layer(Extension(("u".to_string(), String::new())));
        let p = person(&s, "u", "Person A", Circle::Family, None).await;
        let send = |method: &str, path: String, body: &str| {
            let app = app.clone();
            let req = Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap();
            async move { app.oneshot(req).await.unwrap().status() }
        };
        assert_eq!(send("GET", "/".into(), "").await, StatusCode::OK);
        assert_eq!(send("GET", "/seasons".into(), "").await, StatusCode::OK);
        assert_eq!(send("GET", "/gift-plan".into(), "").await, StatusCode::OK);
        assert_eq!(send("GET", "/settings".into(), "").await, StatusCode::OK);
        assert_eq!(send("GET", format!("/{}", p.id), "").await, StatusCode::OK);
        assert_eq!(send("GET", "/missing".into(), "").await, StatusCode::NOT_FOUND);
        assert_eq!(send("POST", format!("/{}/touches", p.id), r#"{"kind":"call"}"#).await, StatusCode::OK);
        assert_eq!(send("POST", format!("/{}/dates", p.id), r#"{"kind":"birthday","month":3,"day":3}"#).await, StatusCode::OK);
        assert_eq!(send("POST", "/gifts".into(), &format!(r#"{{"person_id":"{}","title":"Scarf"}}"#, p.id)).await, StatusCode::OK);
        assert_eq!(send("POST", "/seasons".into(), r#"{"label":"Diwali","month":10,"budget_paise":100}"#).await, StatusCode::OK);
        assert_eq!(send("PATCH", "/dates/nope".into(), "{}").await, StatusCode::NOT_FOUND);
        assert_eq!(send("DELETE", "/touches/nope".into(), "").await, StatusCode::NOT_FOUND);
        assert_eq!(send("DELETE", "/gifts/nope".into(), "").await, StatusCode::NOT_FOUND);
        assert_eq!(send("DELETE", "/seasons/nope".into(), "").await, StatusCode::NOT_FOUND);
        assert_eq!(send("POST", "/".into(), r#"{"name":"Person B","circle":"boss"}"#).await, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(send("DELETE", format!("/{}", p.id), "").await, StatusCode::NO_CONTENT);
    }
}
