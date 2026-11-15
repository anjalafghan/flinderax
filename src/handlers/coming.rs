//! `GET /api/coming-up?days=14`: bills, planned dates, birthdays and follow-ups in one timeline.

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use time::{Date, Duration};

use crate::{
    handlers::{
        autopost,
        common::{today_local, AppError},
        dashboard::load_input,
        followups, people,
    },
    models::AppState,
    planner::{
        bills::{calendar, BillKind},
        coming::{merge, Entry, Item, Kind},
        life::inr,
        people::{self as people_rules, GiftState},
    },
};

type User = Extension<(String, String)>;

#[derive(Deserialize)]
pub struct Q {
    pub days: Option<i64>,
}

#[derive(Serialize)]
pub struct ComingUp {
    pub items: Vec<Entry>,
}

pub async fn get(State(state): State<AppState>, Extension((user_id, _)): User, Query(q): Query<Q>) -> Result<Json<ComingUp>, AppError> {
    let days = q.days.unwrap_or(14).clamp(1, 60);
    Ok(Json(ComingUp { items: gather(&state.db, &user_id, today_local(), days).await? }))
}

/// Everything coming up for one person in the next `days` days, soonest first.
pub async fn gather(db: &sqlx::SqlitePool, user_id: &str, today: Date, days: i64) -> Result<Vec<Entry>, AppError> {
    let user_id = user_id.to_string();
    let mut items: Vec<Item> = Vec::new();

    autopost::apply(db, &user_id, today).await?;
    let input = load_input(db, &user_id, today).await?;
    for b in calendar(&input, today, today + Duration::days(days)) {
        let (kind, link) = if b.kind == BillKind::Date { (Kind::Date, "/joy") } else { (Kind::Bill, "/money") };
        items.push(Item { on: b.date, kind, title: b.label, detail: inr(b.amount_paise - b.covered_paise), link: link.into() });
    }

    let (dates, marks) = people::dates_in(db, &user_id).await?;
    for u in people_rules::upcoming(&dates, today, |id, year| {
        GiftState::best(marks.iter().filter(|(d, y, _)| d == id && *y == year).map(|(_, _, s)| *s))
    }) {
        if let Ok(on) = Date::parse(&u.on, time::macros::format_description!("[year]-[month]-[day]")) {
            let title = if u.label.is_empty() { format!("{}: {}", u.name, u.kind.as_str()) } else { format!("{}: {}", u.name, u.label) };
            items.push(Item { on, kind: Kind::Birthday, title, detail: String::new(), link: format!("/people/{}", u.person_id) });
        }
    }

    for f in followups::compute(db, &user_id, today).await?.open {
        if let Some(on) = f.due_on.as_deref().and_then(|d| Date::parse(d, time::macros::format_description!("[year]-[month]-[day]")).ok()) {
            items.push(Item { on, kind: Kind::FollowUp, title: format!("{}: {}", f.person_name, f.text), detail: String::new(), link: format!("/people/{}", f.person_id) });
        }
    }

    Ok(merge(items, today, days))
}
