//! The envelope: what we send each other on WhatsApp is a link to a page where the date waits in
//! a sealed envelope to be opened. The link carries a long random token and is the only way in,
//! so it opens without logging in (WhatsApp's browser has no bearer token). A surprise stays a
//! surprise: its envelope holds only the hints.
//!
//! The SPA page is `/joy/e/{token}`; it reads `GET /api/public/dates/e/{token}`.

use axum::{
    extract::{Path, State},
    http::{header, HeaderName, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

use super::{dayplan::StepView, load_ctx, url_encode, ActivityView, AppError};
use crate::{
    handlers::common::not_found,
    joy::{ctx::Ctx, db},
    models::AppState,
};

#[derive(Debug, Serialize)]
pub struct EnvelopeView {
    /// "Anjal" when we know who sent it, else empty.
    pub from: String,
    /// "Diya", whoever it's for, else empty.
    pub to: String,
    /// What the letter says first: "It's a date", "A surprise", "What about this?".
    pub kicker: String,
    /// The slip's title, or "Something's planned" when it's sealed.
    pub title: String,
    /// "Sat 7 Nov, 11:00 am", or empty.
    pub when: String,
    pub place: String,
    pub maps_url: Option<String>,
    /// "₹1,200 · 1–2 hours (about 5 h door to door)", or empty when sealed.
    pub meta: String,
    pub notes: String,
    pub steps: Vec<StepView>,
    /// A sealed surprise: the letter holds hints, not the date.
    pub sealed: bool,
    pub hours_note: String,
}

/// Whose envelope is this? The 24-character token alone says (the recipient has no login).
async fn couple_of_token(db: &SqlitePool, token: &str) -> Result<Option<String>, AppError> {
    if token.len() != 24 || !token.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Ok(None);
    }
    for table in ["joy_history", "joy_activities"] {
        let found: Option<String> = sqlx::query_scalar(&format!("SELECT couple_id FROM {table} WHERE share_token = ?"))
            .bind(token)
            .fetch_optional(db)
            .await?;
        if found.is_some() {
            return Ok(found);
        }
    }
    Ok(None)
}

/// `GET /e/{token}` (public): the envelope.
pub async fn page(State(s): State<AppState>, Path(token): Path<String>) -> Result<Response, AppError> {
    let missing = || not_found("This envelope isn't here any more.");
    let Some(couple) = couple_of_token(&s.db, &token).await? else { return Err(missing()) };
    let Some(env) = db::envelope(&s.db, &token).await? else { return Err(missing()) };
    let ctx = load_ctx(&s.db, &couple).await?;
    let view = build(&s.db, &couple, &ctx, env).await?;
    // Only those of us with the link: keep it out of search and out of other sites' logs.
    Ok((
        [
            (HeaderName::from_static("x-robots-tag"), HeaderValue::from_static("noindex")),
            (header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
            (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
        ],
        Json(view),
    )
        .into_response())
}

pub async fn build(db: &SqlitePool, couple: &str, ctx: &Ctx, env: db::Envelope) -> Result<EnvelopeView, AppError> {
    let learned = super::learned(db, couple).await?;
    let name = |i: Option<i64>| i.and_then(|i| ctx.people.get(i as usize)).map(|p| p.0.clone()).unwrap_or_default();
    Ok(match env {
        db::Envelope::Slip(a) => {
            let v = ActivityView::new(ctx, &a, &learned);
            EnvelopeView {
                from: String::new(),
                to: String::new(),
                kicker: "What about this?".into(),
                meta: meta(&v),
                title: v.title,
                when: String::new(),
                place: v.place,
                maps_url: v.maps_url,
                notes: v.notes,
                steps: vec![],
                sealed: false,
                hours_note: String::new(),
            }
        }
        db::Envelope::Date(row) => {
            // It's addressed to whoever didn't draw it: a surprise is sealed for them.
            let to = row.drawn_by.map(|d| (1 - d.clamp(0, 1)) as usize);
            let plan = super::dayplan::build(db, couple, ctx, &row, to).await?;
            let sealed = super::decide::sealed_for(&row, to);
            let a = db::get(db, couple, row.activity_id).await?;
            let v = a.as_ref().map(|a| ActivityView::new(ctx, a, &learned));
            let when = match (&plan, row.planned_local.as_deref()) {
                (Some(p), _) => p.when.clone(),
                (None, Some(l)) => super::planned_label(l),
                _ => String::new(),
            };
            let (steps, hours_note) = plan.map(|p| (p.steps, p.hours_note)).unwrap_or_default();
            EnvelopeView {
                from: name(row.drawn_by),
                to: name(to.map(|t| t as i64)),
                kicker: if sealed { "A surprise".into() } else { "It's a date".into() },
                title: if sealed { "Something's planned".into() } else { ctx.personalize(&row.title) },
                when,
                place: if sealed { String::new() } else { v.as_ref().map(|v| v.place.clone()).unwrap_or_default() },
                maps_url: if sealed { None } else { v.as_ref().and_then(|v| v.maps_url.clone()) },
                meta: if sealed { String::new() } else { v.as_ref().map(meta).unwrap_or_default() },
                notes: if sealed { String::new() } else { v.map(|v| v.notes).unwrap_or_default() },
                steps,
                sealed,
                hours_note,
            }
        }
    })
}

fn meta(v: &ActivityView) -> String {
    let time = if v.door.is_empty() { v.duration.clone() } else { format!("{} ({})", v.duration, v.door) };
    format!("{} · {time}", v.cost)
}

/// The WhatsApp message for an envelope: a line and the link, nothing that gives it away.
pub fn wa_text(ctx: &Ctx, token: &str, line: &str) -> String {
    format!("{line}\n{}/joy/e/{token}", ctx.public_url)
}

pub fn wa_url(text: &str) -> String {
    format!("https://wa.me/?text={}", url_encode(text))
}
