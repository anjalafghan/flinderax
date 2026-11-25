//! `POST /api/internal/backup/run`: a scheduled job pulls a fresh snapshot of the database off the
//! Fly volume. Same secret as the push round (`BACKUP_RUN_TOKEN`, else `PUSH_RUN_TOKEN`), sent as
//! `Authorization: Bearer ...`; without either set the route answers 404.

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use tokio_util::io::ReaderStream;

use crate::{handlers::common::AppError, handlers::push::same, models::AppState};

pub async fn run(State(state): State<AppState>, headers: HeaderMap) -> Result<impl IntoResponse, AppError> {
    let token = ["BACKUP_RUN_TOKEN", "PUSH_RUN_TOKEN"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|t| t.len() >= 8))
        .ok_or_else(|| AppError(StatusCode::NOT_FOUND, "Not found".into()))?;
    let given = headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")).unwrap_or("");
    if !same(given.as_bytes(), token.as_bytes()) {
        return Err(AppError(StatusCode::UNAUTHORIZED, "Not allowed".into()));
    }
    let file = crate::backup::snapshot_file(&state.db).await.map_err(|e| {
        tracing::error!("on-demand backup failed: {e}");
        AppError(StatusCode::INTERNAL_SERVER_ERROR, "Backup failed".into())
    })?;
    let body = Body::from_stream(ReaderStream::new(file));
    Ok(([(header::CONTENT_TYPE, "application/octet-stream"), (header::CACHE_CONTROL, "no-store")], body))
}
