//! One month of the year in review: prints and ticket stubs for each date, receipts for what it
//! cost, the best night as a sticker. Read-only, like the year page. (Joy's `routes/month.rs`.)
//!
//! `GET /year/{year}/{month}`; the SPA page is `/joy/year/{year}/{month}`.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Serialize;

use super::{
    regular::visit_photo_url,
    year::{history_photo_url, rows, MONTHS},
    AppError, Me,
};
use crate::{
    handlers::common::not_found,
    joy::{db, picker},
    models::AppState,
};

type User = Extension<(String, String)>;

/// One kept date on the spread.
#[derive(Debug, Serialize)]
pub struct Entry {
    pub title: String,
    pub place: String,
    /// "home", "local", … for the ticket's colour.
    pub kind: String,
    pub kind_label: String,
    /// "16 Oct"
    pub day: String,
    /// "16·10·26"
    pub stamp: String,
    /// Signed photo link, or empty.
    pub photo: String,
    /// 0 when not rated.
    pub rating: i64,
    /// First line of the note.
    pub note: String,
    /// What we guessed, e.g. "₹1,500".
    pub est: String,
    /// What it really cost, empty when nobody filled it in.
    pub actual: String,
    /// "₹400 over our guess", "₹200 under our guess", or empty.
    pub diff: String,
    pub over: bool,
    /// "₹350", when any real cab fares were logged.
    pub cabs: String,
    /// -2..=2, so each piece sits at its own angle.
    pub tilt: i64,
}

/// Money spent on a date that wasn't a slip.
#[derive(Debug, Serialize)]
pub struct Other {
    pub amount: String,
    pub note: String,
    pub day: String,
}

/// One regular's nights this month, as a strip of stamps and prints.
#[derive(Debug, Serialize)]
pub struct NightGroup {
    pub title: String,
    /// SPA path of the regular's spread.
    pub href: String,
    pub count: usize,
    pub total: String,
    pub nights: Vec<NightMark>,
}

#[derive(Debug, Serialize)]
pub struct NightMark {
    /// "3 Nov"
    pub day: String,
    /// Signed photo link, or empty.
    pub photo: String,
}

#[derive(Debug, Serialize)]
pub struct Best {
    pub title: String,
    pub rating: i64,
    pub day: String,
}

/// A neighbouring month with dates in it.
#[derive(Debug, Serialize)]
pub struct Nav {
    /// SPA path.
    pub href: String,
    pub label: String,
}

/// `GET /year/{year}/{month}`
#[derive(Debug, Serialize)]
pub struct MonthView {
    pub me_name: String,
    pub year: i32,
    pub month: u32,
    /// "October"
    pub month_name: String,
    /// Letters for the sticker title: the month, then the year.
    pub month_letters: Vec<char>,
    pub year_letters: Vec<char>,
    pub entries: Vec<Entry>,
    pub others: Vec<Other>,
    pub nights: Vec<NightGroup>,
    pub best: Option<Best>,
    /// Everything spent this month, slips or not; empty when nothing was.
    pub spent: String,
    /// "4.3", empty when nothing was rated.
    pub avg_rating: String,
    pub prev: Option<Nav>,
    pub next: Option<Nav>,
    /// Dates this month still to come.
    pub to_come: i64,
}

pub async fn page(
    State(s): State<AppState>,
    Extension((user_id, _)): User,
    Path((year, month)): Path<(String, String)>,
) -> Result<Json<MonthView>, AppError> {
    // Typed-in nonsense ("/year/2026/abc") isn't a page.
    let Some(year) = year.parse::<i32>().ok().filter(|y| (2000..=2100).contains(y)) else {
        return Err(not_found("No such year"));
    };
    let Some(month) = month.parse::<u32>().ok().filter(|m| (1..=12).contains(m)) else {
        return Err(not_found("No such month"));
    };
    let j = super::joy(&s, &user_id).await?;
    let (couple, ctx) = (j.couple.as_str(), &j.ctx);
    let key = format!("{year}-{month:02}");
    let rows: Vec<_> = rows(&s.db, couple, year).await?.into_iter().filter(|r| r.month == i64::from(month)).collect();

    let best = rows
        .iter()
        .filter(|r| r.rating.is_some())
        .max_by_key(|r| (r.rating, std::cmp::Reverse(r.id)))
        .map(|r| Best { title: ctx.personalize(&r.title), rating: r.rating.unwrap_or(0), day: r.day_label.clone() });
    let rated: Vec<i64> = rows.iter().filter_map(|r| r.rating).collect();
    let avg_rating = if rated.is_empty() { String::new() } else { format!("{:.1}", rated.iter().sum::<i64>() as f64 / rated.len() as f64) };

    let entries = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let cabs = r.cab_0.unwrap_or(0) + r.cab_1.unwrap_or(0);
            let (diff, over) = match r.actual {
                Some(a) if a > r.est => (format!("{} over our guess", ctx.money(a - r.est)), true),
                Some(a) if a < r.est => (format!("{} under our guess", ctx.money(r.est - a)), false),
                Some(_) => ("right on our guess".into(), false),
                None => (String::new(), false),
            };
            Entry {
                title: ctx.personalize(&r.title),
                place: ctx.personalize(&r.place),
                kind: r.setting.clone(),
                kind_label: picker::setting_label(&r.setting).to_string(),
                day: r.day_label.clone(),
                stamp: r.stamp.clone(),
                photo: if r.has_photo { history_photo_url(ctx, r.id) } else { String::new() },
                rating: r.rating.unwrap_or(0),
                note: r.note.lines().next().unwrap_or("").to_string(),
                est: ctx.money(r.est),
                actual: r.actual.map(|a| ctx.money(a)).unwrap_or_default(),
                diff,
                over,
                cabs: if cabs > 0 { ctx.money(cabs) } else { String::new() },
                tilt: [-2, 1, -1, 2, 0][i % 5],
            }
        })
        .collect();

    let others = db::other_spending_in_month(&s.db, couple, &key)
        .await?
        .into_iter()
        .map(|(amount, note, on)| Other { amount: ctx.money(amount), note, day: day_label(&on) })
        .collect();
    let mut nights: Vec<NightGroup> = vec![];
    for (v, title) in db::visits_in_month(&s.db, couple, &key).await? {
        let mark = NightMark { day: day_label(&v.day), photo: if v.has_photo { visit_photo_url(ctx, v.id) } else { String::new() } };
        let href = format!("/joy/regulars/{}", v.regular_id);
        match nights.iter_mut().find(|g| g.href == href) {
            Some(g) => {
                g.count += 1;
                g.nights.push(mark);
            }
            None => nights.push(NightGroup { title, href, count: 1, total: String::new(), nights: vec![mark] }),
        }
    }
    // Totals are filled in once every night is counted.
    for g in &mut nights {
        let id: i64 = g.href.rsplit('/').next().and_then(|i| i.parse().ok()).unwrap_or(0);
        let total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(v.amount), 0) FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
              WHERE r.couple_id = ? AND v.regular_id = ? AND v.status = 'went' AND strftime('%Y-%m', v.day) = ?",
        )
        .bind(couple)
        .bind(id)
        .bind(&key)
        .fetch_one(&s.db)
        .await?;
        g.total = ctx.money(total);
    }
    let spent = db::spent_by_month(&s.db, couple, year).await?[month as usize - 1];

    // Months with dates either side of this one, across every year, so the spreads read like pages.
    let kept: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT strftime('%Y-%m', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') AS m
         FROM joy_history WHERE couple_id = ? AND outcome = 'did_it' AND (surprise = 0 OR revealed = 1)
           AND COALESCE(planned_at, picked_at) <= unixepoch()
         ORDER BY m",
    )
    .bind(couple)
    .fetch_all(&s.db)
    .await?;
    let nav = |m: &String| Nav { href: format!("/joy/year/{}/{}", &m[..4], m[5..].trim_start_matches('0')), label: month_label(m) };
    let prev = kept.iter().rfind(|m| **m < key).map(nav);
    let next = kept.iter().find(|m| **m > key).map(nav);

    let to_come: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM joy_history WHERE couple_id = ? AND outcome = 'did_it' AND (surprise = 0 OR revealed = 1)
           AND planned_at > unixepoch() AND strftime('%Y-%m', planned_at, 'unixepoch', 'localtime') = ?",
    )
    .bind(couple)
    .bind(&key)
    .fetch_one(&s.db)
    .await?;

    let month_name = MONTHS[month as usize - 1].to_string();
    Ok(Json(MonthView {
        me_name: Me::new(ctx, Some(j.who)).name,
        year,
        month,
        month_letters: month_name.chars().collect(),
        year_letters: year.to_string().chars().collect(),
        month_name,
        entries,
        others,
        nights,
        best,
        spent: if spent > 0 { ctx.money(spent) } else { String::new() },
        avg_rating,
        prev,
        next,
        to_come,
    }))
}

/// "2026-10" → "October 2026"
fn month_label(ym: &str) -> String {
    let m: usize = ym.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(1);
    format!("{} {}", MONTHS[m.clamp(1, 12) - 1], ym.get(..4).unwrap_or(""))
}

/// "2026-10-16" → "16 Oct"
pub fn day_label(ymd: &str) -> String {
    let m: usize = ymd.get(5..7).and_then(|m| m.parse().ok()).unwrap_or(1);
    let d: u32 = ymd.get(8..10).and_then(|d| d.parse().ok()).unwrap_or(1);
    format!("{d} {}", &MONTHS[m.clamp(1, 12) - 1][..3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(month_label("2026-10"), "October 2026");
        assert_eq!(month_label("2025-01"), "January 2025");
        assert_eq!(day_label("2026-10-06"), "6 Oct");
    }
}
