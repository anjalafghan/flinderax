//! Gifts, gift seasons, the gift plan and settings.

use super::*;

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

pub(super) fn check_title(t: &str) -> Result<String, AppError> {
    let t = t.trim();
    if t.is_empty() || too_long(t, 80) {
        return Err(unprocessable("A gift needs a title of 1 to 80 characters."));
    }
    Ok(t.to_string())
}

pub(super) fn check_for_year(y: Option<i64>, today: Date) -> Result<(), AppError> {
    if y.is_some_and(|y| y < 1900 || y > i64::from(today.year()) + 2) {
        return Err(unprocessable("That year is not right."));
    }
    Ok(())
}

pub(super) async fn check_txn(db: &SqlitePool, user_id: &str, txn_id: &Option<String>) -> Result<(), AppError> {
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

pub(super) fn check_season(label: &str, month: i64, budget: i64) -> Result<String, AppError> {
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

