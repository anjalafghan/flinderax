//! Work API, mounted at `/api/work`: the job-search leads. Every query is scoped by `user_id`,
//! so one user can never reach another's lead by guessing an id.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::{
    handlers::common::{bad_request, not_found, today_local, AppError},
    models::AppState,
    planner::{
        life::parse_date,
        work::{LeadIn, Stage, WorkIn},
    },
    validate,
};

type User = Extension<(String, String)>;

/// Most leads one user may keep.
const MAX_LEADS: i64 = 100;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct LeadRow {
    pub id: String,
    pub company: String,
    pub role: String,
    pub stage: Stage,
    pub take_home_paise: Option<i64>,
    pub next_step: String,
    pub next_on: Option<String>,
    pub note: String,
    pub updated_on: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct LeadBody {
    pub company: String,
    #[serde(default)]
    pub role: String,
    pub stage: Option<Stage>,
    pub take_home_paise: Option<i64>,
    #[serde(default)]
    pub next_step: String,
    pub next_on: Option<String>,
    #[serde(default)]
    pub note: String,
}

#[derive(Serialize)]
pub struct WorkOut {
    pub leads: Vec<LeadRow>,
}

type Raw = (String, String, String, String, Option<i64>, String, Option<String>, String, String);

const COLUMNS: &str = "lead_id, company, role, stage, take_home_paise, next_step, next_on, note, updated_on";

fn row((id, company, role, stage, take_home_paise, next_step, next_on, note, updated_on): Raw) -> LeadRow {
    LeadRow { id, company, role, stage: Stage::parse(&stage).unwrap_or(Stage::Idea), take_home_paise, next_step, next_on, note, updated_on }
}

async fn leads(db: &SqlitePool, user_id: &str) -> Result<Vec<LeadRow>, AppError> {
    let rows: Vec<Raw> = sqlx::query_as(&format!("SELECT {COLUMNS} FROM job_leads WHERE user_id = ?"))
        .bind(user_id)
        .fetch_all(db)
        .await?;
    let mut out: Vec<LeadRow> = rows.into_iter().map(row).collect();
    // Furthest along first; within a stage, the most recently moved.
    out.sort_by(|a, b| a.stage.cmp(&b.stage).then(b.updated_on.cmp(&a.updated_on)).then(a.company.cmp(&b.company)));
    Ok(out)
}

/// What the habit rules read.
pub(crate) async fn habits_input(db: &SqlitePool, user_id: &str) -> Result<WorkIn, AppError> {
    let leads = leads(db, user_id)
        .await?
        .into_iter()
        .filter_map(|l| {
            Some(LeadIn {
                updated_on: parse_date(&l.updated_on)?,
                next_on: l.next_on.as_deref().and_then(parse_date),
                id: l.id,
                company: l.company,
                stage: l.stage,
                next_step: l.next_step,
            })
        })
        .collect();
    Ok(WorkIn { leads })
}

fn text(label: &str, value: &str, max: usize, required: bool) -> Result<String, AppError> {
    let value = value.trim();
    if required && value.is_empty() {
        return Err(bad_request(format!("{label} can't be blank.")));
    }
    if value.chars().count() > max {
        return Err(bad_request(format!("{label} must be {max} characters or fewer.")));
    }
    Ok(value.to_string())
}

/// A checked body, ready to store.
fn clean(body: LeadBody) -> Result<LeadBody, AppError> {
    let next_on = body.next_on.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    if let Some(day) = &next_on {
        parse_date(day).ok_or_else(|| bad_request("The next step's date must be YYYY-MM-DD."))?;
    }
    if let Some(pay) = body.take_home_paise {
        validate::positive_amount(pay).map_err(|e| bad_request(format!("Take-home: {e}")))?;
    }
    Ok(LeadBody {
        company: text("Company", &body.company, 60, true)?,
        role: text("Role", &body.role, 60, false)?,
        stage: body.stage,
        take_home_paise: body.take_home_paise,
        next_step: text("Next step", &body.next_step, 80, false)?,
        next_on,
        note: text("Note", &body.note, 280, false)?,
    })
}

pub async fn list(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<WorkOut>, AppError> {
    Ok(Json(WorkOut { leads: leads(&state.db, &user_id).await? }))
}

pub async fn create(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(body): Json<LeadBody>,
) -> Result<(StatusCode, Json<LeadRow>), AppError> {
    let db = &state.db;
    let body = clean(body)?;
    let count: i64 = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM job_leads WHERE user_id = ?").bind(&user_id).fetch_one(db).await?;
    if count >= MAX_LEADS {
        return Err(AppError(StatusCode::CONFLICT, format!("You can keep {MAX_LEADS} leads. Remove an old one first.")));
    }
    let lead = LeadRow {
        id: nanoid!(),
        company: body.company,
        role: body.role,
        stage: body.stage.unwrap_or(Stage::Idea),
        take_home_paise: body.take_home_paise,
        next_step: body.next_step,
        next_on: body.next_on,
        note: body.note,
        updated_on: today_local().to_string(),
    };
    sqlx::query(
        "INSERT INTO job_leads (lead_id, user_id, company, role, stage, take_home_paise, next_step, next_on, note, updated_on)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&lead.id)
    .bind(&user_id)
    .bind(&lead.company)
    .bind(&lead.role)
    .bind(lead.stage.name())
    .bind(lead.take_home_paise)
    .bind(&lead.next_step)
    .bind(&lead.next_on)
    .bind(&lead.note)
    .bind(&lead.updated_on)
    .execute(db)
    .await?;
    Ok((StatusCode::CREATED, Json(lead)))
}

/// Replaces a lead with the body. A field left out is cleared, so the client sends the whole lead.
pub async fn replace(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Path(id): Path<String>,
    Json(body): Json<LeadBody>,
) -> Result<Json<LeadRow>, AppError> {
    let body = clean(body)?;
    let lead = LeadRow {
        id,
        company: body.company,
        role: body.role,
        stage: body.stage.unwrap_or(Stage::Idea),
        take_home_paise: body.take_home_paise,
        next_step: body.next_step,
        next_on: body.next_on,
        note: body.note,
        updated_on: today_local().to_string(),
    };
    let done = sqlx::query(
        "UPDATE job_leads SET company = ?, role = ?, stage = ?, take_home_paise = ?, next_step = ?, next_on = ?, note = ?, updated_on = ?
         WHERE lead_id = ? AND user_id = ?",
    )
    .bind(&lead.company)
    .bind(&lead.role)
    .bind(lead.stage.name())
    .bind(lead.take_home_paise)
    .bind(&lead.next_step)
    .bind(&lead.next_on)
    .bind(&lead.note)
    .bind(&lead.updated_on)
    .bind(&lead.id)
    .bind(&user_id)
    .execute(&state.db)
    .await?;
    if done.rows_affected() == 0 {
        return Err(not_found("No such lead."));
    }
    Ok(Json(lead))
}

pub async fn delete(State(state): State<AppState>, Extension((user_id, _)): User, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let done = sqlx::query("DELETE FROM job_leads WHERE lead_id = ? AND user_id = ?").bind(&id).bind(&user_id).execute(&state.db).await?;
    if done.rows_affected() == 0 {
        return Err(not_found("No such lead."));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct Cleared {
    pub removed: u64,
}

/// `POST /api/work/clear`: starts the job search again with no leads.
pub async fn clear(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Cleared>, AppError> {
    let removed = sqlx::query("DELETE FROM job_leads WHERE user_id = ?").bind(&user_id).execute(&state.db).await?.rows_affected();
    Ok(Json(Cleared { removed }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn state() -> AppState {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for id in ["u", "other"] {
            sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')")
                .bind(id)
                .bind(id)
                .execute(&db)
                .await
                .unwrap();
        }
        let key = rusty_paseto::prelude::PasetoSymmetricKey::from(rusty_paseto::prelude::Key::from(&[7u8; 32]));
        AppState { db, redis: None, paseto_key: std::sync::Arc::new(key) }
    }

    fn user(id: &str) -> User {
        Extension((id.to_string(), "user".to_string()))
    }

    fn ok<T>(r: Result<T, AppError>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => panic!("{}: {}", e.0, e.1),
        }
    }

    fn body(company: &str, stage: Stage) -> LeadBody {
        LeadBody { company: company.into(), stage: Some(stage), ..Default::default() }
    }

    #[tokio::test]
    async fn leads_are_kept_listed_furthest_along_first_changed_and_removed() {
        let s = state().await;
        let (code, a) = ok(create(State(s.clone()), user("u"), Json(body("Company A", Stage::Applied))).await);
        assert_eq!(code, StatusCode::CREATED);
        let _ = ok(create(State(s.clone()), user("u"), Json(body("Company B", Stage::Offer))).await);
        let names = |out: WorkOut| out.leads.into_iter().map(|l| l.company).collect::<Vec<_>>();
        assert_eq!(names(ok(list(State(s.clone()), user("u")).await).0), vec!["Company B", "Company A"]);

        let change = LeadBody {
            take_home_paise: Some(22_000_000),
            next_step: " Prepare for round 2 ".into(),
            next_on: Some("2026-12-01".into()),
            ..body("Company A", Stage::Interview)
        };
        let changed = ok(replace(State(s.clone()), user("u"), Path(a.0.id.clone()), Json(change)).await).0;
        assert_eq!((changed.stage, changed.take_home_paise), (Stage::Interview, Some(22_000_000)));
        assert_eq!(changed.next_step, "Prepare for round 2");

        let input = ok(habits_input(&s.db, "u").await);
        assert_eq!(input.leads.len(), 2);
        assert!(input.leads.iter().any(|l| l.next_on.is_some()));

        assert_eq!(ok(delete(State(s.clone()), user("u"), Path(a.0.id)).await), StatusCode::NO_CONTENT);
        assert_eq!(names(ok(list(State(s), user("u")).await).0), vec!["Company B"]);
    }

    #[tokio::test]
    async fn bad_bodies_and_other_users_leads_are_refused() {
        let s = state().await;
        let code = |r: Result<(StatusCode, Json<LeadRow>), AppError>| r.err().map(|e| e.0);
        assert_eq!(code(create(State(s.clone()), user("u"), Json(body("  ", Stage::Idea))).await), Some(StatusCode::BAD_REQUEST));
        let bad_day = LeadBody { next_on: Some("soon".into()), ..body("Company A", Stage::Idea) };
        assert_eq!(code(create(State(s.clone()), user("u"), Json(bad_day)).await), Some(StatusCode::BAD_REQUEST));
        let bad_pay = LeadBody { take_home_paise: Some(-1), ..body("Company A", Stage::Idea) };
        assert_eq!(code(create(State(s.clone()), user("u"), Json(bad_pay)).await), Some(StatusCode::BAD_REQUEST));

        let (_, mine) = ok(create(State(s.clone()), user("u"), Json(body("Company A", Stage::Idea))).await);
        let theirs = replace(State(s.clone()), user("other"), Path(mine.0.id.clone()), Json(body("Taken", Stage::Offer))).await;
        assert_eq!(theirs.err().map(|e| e.0), Some(StatusCode::NOT_FOUND));
        assert_eq!(delete(State(s.clone()), user("other"), Path(mine.0.id)).await.err().map(|e| e.0), Some(StatusCode::NOT_FOUND));
        assert!(ok(list(State(s.clone()), user("other")).await).0.leads.is_empty());
        assert_eq!(ok(list(State(s), user("u")).await).0.leads[0].company, "Company A");
    }
}
