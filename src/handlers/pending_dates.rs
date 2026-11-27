use axum::{Extension, Json, extract::State};
use serde::Serialize;

use crate::{handlers::{common::AppError, couple::couple_of}, models::AppState};

type User = Extension<(String, String)>;

#[derive(Serialize)]
pub struct PendingDate {
    pub id: i64,
    pub day: String,
    pub title: String,
    pub sealed: bool,
}

pub async fn pending_did_it_happen(State(state): State<AppState>, Extension((user_id, _)): User) -> Result<Json<Vec<PendingDate>>, AppError> {
    let Some((couple, who)) = couple_of(&state.db, &user_id).await? else { return Ok(Json(Vec::new())) };
    let rows: Vec<(i64, String, String, bool)> = sqlx::query_as(
        "SELECT h.id, date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime'),
                CASE WHEN h.surprise = 1 AND h.revealed = 0 AND COALESCE(h.drawn_by, -1) <> ?2 THEN '' ELSE a.title END,
                (h.surprise = 1 AND h.revealed = 0 AND COALESCE(h.drawn_by, -1) <> ?2)
         FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ?1 AND h.outcome = 'did_it' AND h.confirmed_at IS NULL
           AND date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') < date('now', 'localtime')
         ORDER BY COALESCE(h.planned_at, h.picked_at), h.id",
    )
    .bind(couple)
    .bind(who as i64)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(|(id, day, title, sealed)| PendingDate { id, day, title, sealed }).collect()))
}
