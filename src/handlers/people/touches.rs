//! Touches: when you last caught up with someone.

use super::*;

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
pub(super) async fn recompute_last_contact(db: &SqlitePool, user_id: &str, person_id: &str) -> Result<(), AppError> {
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

