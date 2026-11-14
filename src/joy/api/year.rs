//! "Our 2026": a year in review, worked out on request from what's already stored. Nothing new is
//! saved; the share picture is drawn on the phone from [`ShareData`]. (Joy's `routes/year.rs`.)
//!
//! `GET /year` → `Go` to this year (last year in January); `GET /year/{year}` (2000-2100, else
//! 404). The SPA pages are `/joy/year/{y}` and `/joy/year/{y}/{m}`.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Serialize;
use sqlx::FromRow;

use super::{go, AppError, Go, Me};
use crate::{
    handlers::common::not_found,
    joy::{ctx::Ctx, db, picker},
    models::AppState,
};

type User = Extension<(String, String)>;

/// A kept date that happened this year and that both of us may see (no unrevealed surprises).
#[derive(Debug, Clone, FromRow)]
pub struct YearRow {
    pub id: i64,
    pub title: String,
    pub setting: String,
    pub tags: String,
    pub place: String,
    pub rating: Option<i64>,
    pub note: String,
    pub has_photo: bool,
    /// The date's photo ids, oldest first, comma-separated ("" for none). The first is the cover.
    pub photo_ids: String,
    pub est: i64,
    pub actual: Option<i64>,
    pub cab_0: Option<i64>,
    pub cab_1: Option<i64>,
    /// 1-12, local time.
    pub month: i64,
    /// Local julian day number, for week streaks.
    pub jd: i64,
    /// "16 Oct"
    pub day_label: String,
    /// "16·10·26", the date stamp on a print.
    pub stamp: String,
}

pub(crate) async fn rows(db: &sqlx::SqlitePool, couple: &str, year: i32) -> sqlx::Result<Vec<YearRow>> {
    sqlx::query_as(
        "SELECT h.id, a.title, a.setting, a.tags,
                CASE WHEN a.venue != '' THEN a.venue ELSE a.area END AS place,
                h.rating, h.note, h.has_photo,
                COALESCE((SELECT group_concat(id) FROM (SELECT p.id FROM joy_history_photos p WHERE p.history_id = h.id ORDER BY p.id)), '') AS photo_ids,
                COALESCE(h.est_cost, a.est_cost) AS est, h.actual_cost AS actual, h.cab_fare_0 AS cab_0, h.cab_fare_1 AS cab_1,
                CAST(strftime('%m', COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') AS INTEGER) AS month,
                CAST(julianday(date(COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime')) AS INTEGER) AS jd,
                CAST(strftime('%d', COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') AS INTEGER) || ' ' ||
                  substr('JanFebMarAprMayJunJulAugSepOctNovDec',
                         1 + 3 * (CAST(strftime('%m', COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') AS INTEGER) - 1), 3) AS day_label,
                strftime('%d·%m·%y', COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') AS stamp
         FROM joy_history h JOIN joy_activities a ON a.id = h.activity_id
         WHERE h.couple_id = ?2 AND h.outcome = 'did_it'
           AND (h.surprise = 0 OR h.revealed = 1)
           AND COALESCE(h.planned_at, h.picked_at) <= unixepoch()
           AND CAST(strftime('%Y', COALESCE(h.planned_at, h.picked_at), 'unixepoch', 'localtime') AS INTEGER) = ?1
         ORDER BY COALESCE(h.planned_at, h.picked_at), h.id",
    )
    .bind(year)
    .bind(couple)
    .fetch_all(db)
    .await
}

/// Dates this year still to come, and (for the planner only) surprises still sealed.
async fn later(db: &sqlx::SqlitePool, couple: &str, year: i32, who: usize) -> sqlx::Result<(i64, i64)> {
    let to_come: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM joy_history WHERE couple_id = ?2 AND outcome = 'did_it' AND (surprise = 0 OR revealed = 1)
           AND planned_at > unixepoch()
           AND CAST(strftime('%Y', planned_at, 'unixepoch', 'localtime') AS INTEGER) = ?1",
    )
    .bind(year)
    .bind(couple)
    .fetch_one(db)
    .await?;
    let sealed_mine: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM joy_history WHERE couple_id = ?3 AND outcome = 'did_it' AND surprise = 1 AND revealed = 0 AND drawn_by = ?2
           AND CAST(strftime('%Y', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') AS INTEGER) = ?1",
    )
    .bind(year)
    .bind(who as i64)
    .bind(couple)
    .fetch_one(db)
    .await?;
    Ok((to_come, sealed_mine))
}

#[derive(Debug, Serialize)]
pub struct MonthBar {
    /// "J", "F", … for the strip; `name` for screen readers.
    pub letter: String,
    pub name: String,
    pub n: i64,
    /// 0-100, relative to the busiest month.
    pub pct: i64,
}

#[derive(Debug, Serialize)]
pub struct BestNight {
    pub title: String,
    pub when: String,
    pub rating: i64,
    /// First line of the note (page only, never in the share picture).
    pub note: String,
    /// Other nights with the same top rating.
    pub more: usize,
    /// Its signed photo link, when it has one.
    pub photo: String,
    /// The kept date, to open all its photos.
    pub date_id: i64,
}

/// One kind of night (home, local, …) and its share of the year, for the paper strip.
#[derive(Debug, Serialize)]
pub struct Kind {
    pub key: String,
    pub label: String,
    pub n: usize,
    /// 1-100 of the year's dates, at least 1 so every kind shows.
    pub pct: i64,
}

/// A month's blot on the money line.
#[derive(Debug, Serialize)]
pub struct MonthBlot {
    pub letter: String,
    /// Radius in px, 0 when nothing was spent.
    pub r: String,
    pub over: bool,
    /// "March: ₹6,800 of ₹6,000", for screen readers.
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct Money {
    /// What the dates with a real cost came to, e.g. "₹42,300".
    pub spent: String,
    /// "₹3,100 under what we guessed", "₹1,800 over", "right on what we guessed".
    pub diff: String,
    pub over: bool,
    /// How many dates had a real cost.
    pub n: usize,
}

/// A folded slip lying in the header's jar (one per date, coloured by kind of night).
#[derive(Debug, Serialize)]
pub struct JarSlip {
    pub x: i64,
    pub y: i64,
    pub rot: i64,
    pub kind: String,
}

/// "★★★★★ × 3" under "stars we gave".
#[derive(Debug, Serialize)]
pub struct RatingRow {
    pub stars: i64,
    pub n: usize,
}

#[derive(Debug, Serialize)]
pub struct YearPhoto {
    pub id: i64,
    /// The kept date it belongs to, so a viewer can step through that date's photos.
    pub date_id: i64,
    /// Signed photo link.
    pub url: String,
    pub title: String,
    pub when: String,
}

#[derive(Debug, Serialize)]
pub struct SharePhoto {
    pub id: i64,
    pub url: String,
}

/// Exactly what the share picture may contain: no names, home areas, notes or app address.
/// Places, money and photos are only used if the person ticks them on the page.
#[derive(Debug, Serialize)]
pub struct ShareData {
    pub year: i32,
    pub dates: usize,
    pub months: Vec<i64>,
    pub vibe: String,
    pub tags: Vec<String>,
    pub best_title: String,
    pub best_rating: i64,
    /// Other nights with the same top rating ("and 4 more").
    pub best_more: usize,
    /// "home", "local", "day_trip" or "travel", for the picture's tint.
    pub vibe_key: String,
    pub avg_rating: String,
    pub streak_weeks: i64,
    /// Only drawn when "show where we went" is ticked.
    pub places: Vec<String>,
    /// Only drawn when "show what we spent" is ticked.
    pub spent: String,
    /// Photo ids → signed URLs; only drawn when ticked one by one.
    pub photos: Vec<SharePhoto>,
}

/// `GET /year/{year}`
#[derive(Debug, Serialize)]
pub struct YearView {
    pub me_name: String,
    pub year: i32,
    /// Years with kept dates, newest first (for "← 2025").
    pub years: Vec<i32>,
    pub dates: usize,
    pub to_come: i64,
    /// Planner only: their own surprises this year, still sealed.
    pub sealed_mine: i64,
    pub months: Vec<MonthBar>,
    pub busiest: String,
    pub vibe_label: String,
    pub tags: Vec<String>,
    pub best: Option<BestNight>,
    pub avg_rating: String,
    pub rated: usize,
    pub streak_weeks: i64,
    pub money: Option<Money>,
    /// "₹2,400 in cabs", when any real fares were logged.
    pub cabs: String,
    pub photos: Vec<YearPhoto>,
    /// How many dates the busiest month held.
    pub busiest_n: i64,
    /// Tally marks: groups of five (the last group may be shorter), at most 24 groups.
    pub tally: Vec<u8>,
    /// Dates beyond what the tally has room for.
    pub tally_more: usize,
    pub kinds: Vec<Kind>,
    /// "our best kind of night: day trips (4.8★)", or empty.
    pub best_kind: String,
    /// "4 five-star nights, 3 four-star", or empty.
    pub rating_line: String,
    /// The two most-given ratings, most stars first.
    pub rating_rows: Vec<RatingRow>,
    /// At most 14 slips in the header's jar.
    pub jar_slips: Vec<JarSlip>,
    pub month_blots: Vec<MonthBlot>,
    /// Everything spent this year, slips or not; empty when nothing was.
    pub year_total: String,
    /// "12 regular nights, ₹3,600 of it", or empty when none were had this year.
    pub nights_line: String,
    /// "₹6,000 a month", when a monthly budget is set.
    pub budget_note: String,
    /// What the share picture is drawn from (the page's `year-share` JSON).
    pub share: ShareData,
}

pub(crate) const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// `GET /year` → `Go` to this year (or last year, in January when this one has barely started).
pub async fn current(State(s): State<AppState>, Extension(_): User) -> Result<Json<Go>, AppError> {
    let (y, m): (i32, i32) = sqlx::query_as(
        "SELECT CAST(strftime('%Y', 'now', 'localtime') AS INTEGER), CAST(strftime('%m', 'now', 'localtime') AS INTEGER)",
    )
    .fetch_one(&s.db)
    .await?;
    Ok(go(format!("/joy/year/{}", if m == 1 { y - 1 } else { y })))
}

pub async fn page(State(s): State<AppState>, Extension((user_id, _)): User, Path(year): Path<i32>) -> Result<Json<YearView>, AppError> {
    if !(2000..=2100).contains(&year) {
        return Err(not_found("No such year"));
    }
    let j = super::joy(&s, &user_id).await?;
    let (couple, ctx) = (j.couple.as_str(), &j.ctx);
    let rows = rows(&s.db, couple, year).await?;
    let (to_come, sealed_mine) = later(&s.db, couple, year, j.who).await?;
    // Same rule as the recap itself: dates that happened and that both of us may see, so a year
    // whose only date is a sealed surprise doesn't show up in the switcher.
    let years: Vec<i32> = sqlx::query_scalar(
        "SELECT DISTINCT CAST(strftime('%Y', COALESCE(planned_at, picked_at), 'unixepoch', 'localtime') AS INTEGER) AS y
         FROM joy_history WHERE couple_id = ? AND outcome = 'did_it' AND (surprise = 0 OR revealed = 1)
           AND COALESCE(planned_at, picked_at) <= unixepoch()
         ORDER BY y DESC",
    )
    .bind(couple)
    .fetch_all(&s.db)
    .await?;
    let r = Recap::from(&rows);
    let money = r.money.map(|(spent, guessed, n)| {
        let d = spent - guessed;
        Money {
            spent: ctx.money(spent),
            diff: match d {
                0 => "right on what we guessed".into(),
                d if d > 0 => format!("{} over what we guessed", ctx.money(d)),
                d => format!("{} under what we guessed", ctx.money(-d)),
            },
            over: d > 0,
            n,
        }
    });
    let photos: Vec<YearPhoto> = rows
        .iter()
        .flat_map(|x| {
            x.photos().into_iter().map(move |pid| YearPhoto { id: pid, date_id: x.id, url: history_photo_url(x.id, pid), title: ctx.personalize(&x.title), when: x.day_label.clone() })
        })
        .collect();
    let best = r.best.map(|(i, more)| {
        let b = &rows[i];
        BestNight {
            title: ctx.personalize(&b.title),
            when: b.day_label.clone(),
            rating: b.rating.unwrap_or(0),
            note: b.note.lines().next().unwrap_or("").to_string(),
            more,
            photo: b.cover_url(),
            date_id: b.id,
        }
    });
    let share = ShareData {
        year,
        dates: rows.len(),
        months: r.months.to_vec(),
        vibe: r.vibe.as_deref().map(|v| picker::setting_label(v).to_string()).unwrap_or_default(),
        tags: r.tags.clone(),
        best_title: r.best.map(|(i, _)| share_safe(ctx, &rows[i].title)).unwrap_or_default(),
        best_rating: best.as_ref().map_or(0, |b| b.rating),
        best_more: best.as_ref().map_or(0, |b| b.more),
        vibe_key: r.vibe.clone().unwrap_or_default(),
        avg_rating: r.avg_rating.clone(),
        streak_weeks: r.streak_weeks,
        places: {
            let mut p: Vec<String> = rows.iter().map(|x| share_safe(ctx, &x.place)).filter(|p| !p.is_empty()).collect();
            p.dedup();
            p.truncate(8);
            p
        },
        spent: money.as_ref().map(|m| m.spent.clone()).unwrap_or_default(),
        photos: photos.iter().map(|p| SharePhoto { id: p.id, url: p.url.clone() }).collect(),
    };
    let max = r.months.iter().copied().max().unwrap_or(0).max(1);
    let busiest = r.months.iter().enumerate().max_by_key(|(i, n)| (**n, -(*i as i64))).filter(|(_, n)| **n > 0).map(|(i, _)| MONTHS[i].to_string());
    let busiest_n = r.months.iter().copied().max().unwrap_or(0);
    let tally: Vec<u8> = (0..rows.len().min(120)).step_by(5).map(|i| (rows.len().min(120) - i).min(5) as u8).collect();
    let kinds: Vec<Kind> = r
        .kinds
        .iter()
        .map(|(k, n)| Kind {
            key: k.clone(),
            label: picker::setting_label(k).to_string(),
            n: *n,
            pct: ((*n * 100 / rows.len().max(1)) as i64).max(1),
        })
        .collect();
    let best_kind = r
        .best_kind
        .as_ref()
        .map(|(k, avg)| format!("our best kind of night: {} ({avg}★)", picker::setting_label(k).to_lowercase()))
        .unwrap_or_default();
    let top_ratings = || r.rating_hist.iter().enumerate().rev().filter(|(_, n)| **n > 0).take(2);
    let rating_line = top_ratings().map(|(i, n)| format!("{n} {}-star", i + 1)).collect::<Vec<_>>().join(", ");
    let rating_rows: Vec<RatingRow> = top_ratings().map(|(i, n)| RatingRow { stars: i as i64 + 1, n: *n }).collect();
    // Slips settle in rows from the bottom of the jar; positions are fixed so the jar never jumps.
    const SLIP_SPOTS: [(i64, i64, i64); 14] = [
        (52, 246, -14), (96, 252, 6), (140, 240, -8), (62, 262, 3), (150, 258, 12), (78, 224, 20), (124, 218, -26),
        (50, 206, 9), (156, 210, -4), (100, 196, -12), (136, 186, 17), (66, 182, -20), (112, 166, 5), (150, 160, -9),
    ];
    let jar_slips: Vec<JarSlip> = rows
        .iter()
        .zip(SLIP_SPOTS)
        .map(|(x, (sx, sy, rot))| JarSlip { x: sx, y: sy, rot, kind: x.setting.clone() })
        .collect();
    let spent = db::spent_by_month(&s.db, couple, year).await?;
    let standing: Option<i64> = db::setting(&s.db, couple, "monthly_budget").await?.and_then(|v| v.parse().ok()).filter(|b: &i64| *b > 0);
    let mut month_blots = vec![];
    let top = spent.iter().copied().max().unwrap_or(0).max(1);
    for (i, sp) in spent.iter().copied().enumerate() {
        let budget = db::month_budget(&s.db, couple, &format!("{year}-{:02}", i + 1)).await?.map(|(b, c)| b + c).or(standing);
        let r_px = match (sp, budget) {
            (0, _) => 0.0,
            (sp, Some(b)) => (sp as f64 / b as f64 * 9.0).clamp(3.0, 13.0),
            (sp, None) => (sp as f64 / top as f64 * 12.0).clamp(3.0, 13.0),
        };
        month_blots.push(MonthBlot {
            letter: MONTHS[i][..1].into(),
            r: format!("{r_px:.1}"),
            over: budget.is_some_and(|b| sp > b),
            text: match (sp, budget) {
                (0, _) => format!("{}: nothing logged", MONTHS[i]),
                (sp, Some(b)) => format!("{}: {} of {}", MONTHS[i], ctx.money(sp), ctx.money(b)),
                (sp, None) => format!("{}: {}", MONTHS[i], ctx.money(sp)),
            },
        });
    }
    let year_spent: i64 = spent.iter().sum();
    let (night_count, night_total): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(v.amount), 0) FROM joy_regular_visits v JOIN joy_regulars r ON r.id = v.regular_id
          WHERE r.couple_id = ? AND v.status = 'went' AND v.history_id IS NULL AND CAST(strftime('%Y', v.day) AS INTEGER) = ?",
    )
    .bind(couple)
    .bind(year)
    .fetch_one(&s.db)
    .await?;
    let nights_line = match (night_count, night_total) {
        (0, _) => String::new(),
        (n, 0) => format!("{n} regular {}", if n == 1 { "night" } else { "nights" }),
        (n, t) => format!("{n} regular {}, {} of it", if n == 1 { "night" } else { "nights" }, ctx.money(t)),
    };
    Ok(Json(YearView {
        busiest_n,
        tally_more: rows.len().saturating_sub(120),
        tally,
        kinds,
        best_kind,
        rating_line,
        rating_rows,
        jar_slips,
        month_blots,
        nights_line,
        year_total: if year_spent > 0 { ctx.money(year_spent) } else { String::new() },
        budget_note: standing.map(|b| format!("{} a month", ctx.money(b))).unwrap_or_default(),
        me_name: Me::new(ctx, Some(j.who)).name,
        year,
        years,
        dates: rows.len(),
        to_come,
        sealed_mine,
        months: r
            .months
            .iter()
            .enumerate()
            .map(|(i, n)| MonthBar { letter: MONTHS[i][..1].into(), name: MONTHS[i].into(), n: *n, pct: n * 100 / max })
            .collect(),
        busiest: busiest.unwrap_or_default(),
        vibe_label: r.vibe.as_deref().map(|v| picker::setting_label(v).to_string()).unwrap_or_default(),
        tags: r.tags,
        best,
        avg_rating: r.avg_rating,
        rated: r.rated,
        streak_weeks: r.streak_weeks,
        money,
        cabs: if r.cabs > 0 { format!("{} in cabs", ctx.money(r.cabs)) } else { String::new() },
        photos,
        share,
    }))
}

/// Signed link (a week) to one photo of a kept date. The history module serves it.
pub fn history_photo_url(history_id: i64, photo_id: i64) -> String {
    super::history::open_photo_url(history_id, photo_id, 7 * 24 * 3600)
}

impl YearRow {
    /// Photo ids, oldest first.
    pub fn photos(&self) -> Vec<i64> {
        self.photo_ids.split(',').filter_map(|p| p.trim().parse().ok()).collect()
    }

    /// The cover photo's signed link, or empty.
    pub fn cover_url(&self) -> String {
        self.photos().first().map(|&pid| history_photo_url(self.id, pid)).unwrap_or_default()
    }
}

/// Text that's safe to put in a picture others may see: personal placeholders and our names
/// become "us", home areas are dropped (old slips may carry them literally).
fn share_safe(ctx: &Ctx, text: &str) -> String {
    let mut t = text.replace("{a}", "us").replace("{b}", "us");
    // Optional [[…]] blocks only ever hold home-area hints: drop them.
    while let (Some(a), Some(b)) = (t.find("[["), t.find("]]")) {
        if b < a {
            break;
        }
        t.replace_range(a..b + 2, "");
    }
    t = t.replace("{a_area}", "").replace("{b_area}", "");
    for (name, _) in &ctx.people {
        t = replace_word(&t, name, "us");
    }
    for area in ctx.areas.iter().flatten() {
        t = t.replace(area.as_str(), "");
    }
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Replace whole-word, case-insensitive occurrences of `word`.
fn replace_word(text: &str, word: &str, with: &str) -> String {
    let wl: Vec<char> = word.chars().flat_map(char::to_lowercase).collect();
    if wl.is_empty() {
        return text.to_string();
    }
    // Every lowercased char, tagged with the byte span of the original char it came from. Matching
    // runs over this list and `text` is only ever sliced at these spans, so lowercasing changing
    // byte lengths (e.g. "İ" -> "i̇") cannot misalign the output.
    let folded: Vec<(char, usize, usize)> = text
        .char_indices()
        .flat_map(|(s, c)| {
            let end = s + c.len_utf8();
            c.to_lowercase().map(move |lc| (lc, s, end))
        })
        .collect();
    let mut out = String::with_capacity(text.len());
    // Byte offset in `text` up to which the original text has been copied to `out`.
    let mut copied = 0;
    let mut k = 0;
    while k + wl.len() <= folded.len() {
        let end_idx = k + wl.len();
        // A match must start and end on whole original chars, not inside one char's lowercase form.
        let starts_cleanly = k == 0 || folded[k - 1].2 == folded[k].1;
        let ends_cleanly = end_idx == folded.len() || folded[end_idx - 1].2 == folded[end_idx].1;
        let same = folded[k..end_idx].iter().zip(&wl).all(|(f, c)| f.0 == *c);
        if !(starts_cleanly && ends_cleanly && same) {
            k += 1;
            continue;
        }
        let before_ok = k == 0 || !folded[k - 1].0.is_alphanumeric();
        let after_ok = end_idx == folded.len() || !folded[end_idx].0.is_alphanumeric();
        if before_ok && after_ok {
            let span_start = folded[k].1;
            let span_end = folded[end_idx - 1].2;
            out.push_str(&text[copied..span_start]);
            out.push_str(with);
            copied = span_end;
        }
        // Like the original scan, continue after the match whether or not it was replaced.
        k = end_idx;
    }
    out.push_str(&text[copied..]);
    out
}

/// The numbers, worked out from this year's rows (pure; unit-tested).
#[derive(Debug, Default)]
pub struct Recap {
    pub months: [i64; 12],
    pub vibe: Option<String>,
    pub tags: Vec<String>,
    /// (index into rows, how many other nights share the top rating)
    pub best: Option<(usize, usize)>,
    pub avg_rating: String,
    pub rated: usize,
    pub streak_weeks: i64,
    /// (spent, what we guessed for those, how many) over dates with a real cost.
    pub money: Option<(i64, i64, usize)>,
    pub cabs: i64,
    /// Every kind of night with its count, most common first.
    pub kinds: Vec<(String, usize)>,
    /// How many nights got 1, 2, … 5 stars.
    pub rating_hist: [usize; 5],
    /// The kind with the best average rating (key, "4.8"), only when at least two kinds have two
    /// or more rated nights to compare.
    pub best_kind: Option<(String, String)>,
}

impl Recap {
    pub fn from(rows: &[YearRow]) -> Self {
        let mut r = Recap::default();
        for x in rows {
            if let Some(m) = usize::try_from(x.month - 1).ok().filter(|m| *m < 12) {
                r.months[m] += 1;
            }
            r.cabs += x.cab_0.unwrap_or(0) + x.cab_1.unwrap_or(0);
        }
        let count = |items: Vec<String>| -> Vec<(String, usize)> {
            let mut c: Vec<(String, usize)> = vec![];
            for i in items {
                match c.iter_mut().find(|(k, _)| *k == i) {
                    Some(e) => e.1 += 1,
                    None => c.push((i, 1)),
                }
            }
            c.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            c
        };
        r.kinds = count(rows.iter().map(|x| x.setting.clone()).collect());
        r.vibe = r.kinds.first().map(|(k, _)| k.clone());
        for x in rows {
            if let Some(v) = x.rating.filter(|v| (1..=5).contains(v)) {
                r.rating_hist[(v - 1) as usize] += 1;
            }
        }
        let mut by_kind: Vec<(String, f64, usize)> = vec![];
        for (kind, _) in &r.kinds {
            let rated: Vec<i64> = rows.iter().filter(|x| x.setting == *kind).filter_map(|x| x.rating).collect();
            if rated.len() >= 2 {
                by_kind.push((kind.clone(), rated.iter().sum::<i64>() as f64 / rated.len() as f64, rated.len()));
            }
        }
        if by_kind.len() >= 2 {
            by_kind.sort_by(|a, b| b.1.total_cmp(&a.1).then(b.2.cmp(&a.2)));
            r.best_kind = by_kind.first().map(|(k, avg, _)| (k.clone(), format!("{avg:.1}")));
        }
        r.tags = count(
            rows.iter()
                .flat_map(|x| x.tags.split(',').map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()).collect::<Vec<_>>())
                .collect(),
        )
        .into_iter()
        .take(3)
        .map(|(k, _)| k)
        .collect();
        let rated: Vec<(usize, i64)> = rows.iter().enumerate().filter_map(|(i, x)| x.rating.map(|v| (i, v))).collect();
        r.rated = rated.len();
        if let Some(top) = rated.iter().map(|(_, v)| *v).max() {
            let tops: Vec<usize> = rated.iter().filter(|(_, v)| *v == top).map(|(i, _)| *i).collect();
            // The most recent of the best nights.
            r.best = tops.last().map(|i| (*i, tops.len() - 1));
            let avg = rated.iter().map(|(_, v)| *v as f64).sum::<f64>() / rated.len() as f64;
            r.avg_rating = format!("{avg:.1}");
        }
        // Longest run of consecutive weeks (Monday-based) with at least one date.
        let mut weeks: Vec<i64> = rows.iter().map(|x| (x.jd + 1).div_euclid(7)).collect();
        weeks.sort_unstable();
        weeks.dedup();
        let (mut best, mut run) = (0, 0);
        for (i, w) in weeks.iter().enumerate() {
            run = if i > 0 && *w == weeks[i - 1] + 1 { run + 1 } else { 1 };
            best = best.max(run);
        }
        r.streak_weeks = best;
        let costed: Vec<&YearRow> = rows.iter().filter(|x| x.actual.is_some()).collect();
        if !costed.is_empty() {
            r.money = Some((
                costed.iter().map(|x| x.actual.unwrap_or(0)).sum(),
                costed.iter().map(|x| x.est).sum(),
                costed.len(),
            ));
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(month: i64, jd: i64, setting: &str, tags: &str, rating: Option<i64>, actual: Option<i64>) -> YearRow {
        YearRow {
            id: jd,
            title: format!("t{jd}"),
            setting: setting.into(),
            tags: tags.into(),
            place: String::new(),
            rating,
            note: String::new(),
            has_photo: false,
            photo_ids: String::new(),
            est: 1000,
            actual,
            cab_0: Some(200),
            cab_1: None,
            month,
            jd,
            day_label: String::new(),
            stamp: String::new(),
        }
    }

    #[test]
    fn recap_numbers() {
        // jd 2461000 is a Monday-based week boundary-agnostic run: three weeks in a row, a gap, then one.
        let rows = vec![
            row(3, 2461000, "local", "foodie,cozy", Some(4), Some(1500)),
            row(3, 2461007, "local", "foodie", Some(5), None),
            row(3, 2461014, "home", "cozy,games", Some(5), Some(800)),
            row(7, 2461100, "local", "foodie", None, None),
        ];
        let r = Recap::from(&rows);
        assert_eq!(r.months[2], 3);
        assert_eq!(r.months[6], 1);
        assert_eq!(r.vibe.as_deref(), Some("local"));
        assert_eq!(r.tags, vec!["foodie", "cozy", "games"]);
        assert_eq!(r.best, Some((2, 1)), "most recent five-star night, and one more like it");
        assert_eq!(r.avg_rating, "4.7");
        assert_eq!(r.streak_weeks, 3);
        assert_eq!(r.money, Some((2300, 2000, 2)), "only dates with a real cost are compared");
        assert_eq!(r.cabs, 800);
        assert_eq!(r.rating_hist, [0, 0, 0, 1, 2]);
        assert_eq!(r.kinds, vec![("local".to_string(), 3), ("home".to_string(), 1)]);
        assert_eq!(r.best_kind, None, "home has only one rated night, so there's nothing to compare");
    }

    #[test]
    fn best_kind_needs_two_kinds_with_two_rated_nights_each() {
        let rows = vec![
            row(1, 2461000, "local", "", Some(3), None),
            row(1, 2461001, "local", "", Some(4), None),
            row(2, 2461040, "travel", "", Some(5), None),
            row(2, 2461041, "travel", "", Some(5), None),
        ];
        assert_eq!(Recap::from(&rows).best_kind, Some(("travel".into(), "5.0".into())));
    }

    #[test]
    fn whole_word_name_replacement() {
        assert_eq!(replace_word("watch Neha react", "Neha", "us"), "watch us react");
        assert_eq!(replace_word("Nehal's place", "Neha", "us"), "Nehal's place");
        assert_eq!(replace_word("neha & NEHA", "Neha", "us"), "us & us");
    }

    #[test]
    fn whole_word_replacement_keeps_ascii_behaviour() {
        assert_eq!(replace_word("Priyanka and priya", "priya", "us"), "Priyanka and us");
        assert_eq!(replace_word("priya, PRIYA!", "Priya", "us"), "us, us!");
        assert_eq!(replace_word("aaaa", "aa", "x"), "aaaa", "overlapping runs inside a word are not whole words");
        assert_eq!(replace_word("aa aa", "aa", "x"), "x x");
    }

    #[test]
    fn replacement_after_char_whose_lowercase_is_longer() {
        // "İ" is 2 bytes but lowercases to 3 bytes ("i" + combining dot), which used to shift the
        // offsets and garble or panic on the slice.
        assert_eq!(replace_word("Cafe İzmir with Priya", "priya", "us"), "Cafe İzmir with us");
        assert_eq!(replace_word("İ priya İ", "priya", "us"), "İ us İ");
    }

    #[test]
    fn multi_byte_chars_around_and_inside_matches() {
        assert_eq!(replace_word("héllo priya ünïcode", "priya", "us"), "héllo us ünïcode");
        assert_eq!(replace_word("ÑANDÚ and ñandú!", "ñandú", "us"), "us and us!");
        assert_eq!(replace_word("日本 priya 語", "priya", "us"), "日本 us 語");
        // A non-ASCII letter directly after the name means it is not the whole word.
        assert_eq!(replace_word("priyaï priya", "priya", "us"), "priyaï us");
        // A match must not end inside the lowercase form of one char.
        assert_eq!(replace_word("İ i", "i", "us"), "İ us");
    }

    #[test]
    fn word_not_present_leaves_text_unchanged() {
        assert_eq!(replace_word("no names here, İzmir", "priya", "us"), "no names here, İzmir");
        assert_eq!(replace_word("", "priya", "us"), "");
        assert_eq!(replace_word("priy", "priya", "us"), "priy", "a word longer than the text");
    }

    #[test]
    fn empty_word_is_a_no_op() {
        assert_eq!(replace_word("abc", "", "us"), "abc");
        assert_eq!(replace_word("Cafe İzmir", "", "us"), "Cafe İzmir");
        assert_eq!(replace_word("", "", "us"), "");
    }
}
