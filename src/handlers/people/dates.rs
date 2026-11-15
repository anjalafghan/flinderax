//! Dates that matter: birthdays and anniversaries.

#![allow(unused_imports)]
use super::*;

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

pub(super) struct DateFields {
    kind: DateKind,
    label: String,
    month: i64,
    day: i64,
    year: Option<i64>,
    remind_days: i64,
    budget: Option<i64>,
}

pub(super) fn check_date(f: &DateFields, today: Date) -> Result<(), AppError> {
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

