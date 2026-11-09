//! One-off move of an existing Joy database into the Dates tables:
//! `joy import-joy <joy.db> <user name> [--as a|b] [--photos <dir>] [--dry-run]`.
//!
//! `--as` says which of the two people in Joy you were (PERSON_A = a, the default), so every
//! date, memory, job and check-in stays with the right person. Joy kept homes, the office, date
//! days and commutes in its environment, not its database: any of those set in this process's
//! environment (as they are on Joy's own machine) are copied into the couple's settings.
//!
//! Copies every table (ids are renumbered and every reference follows), keeps envelope share
//! tokens so links already sent still open, keeps the `monthly_budget` / `saving_for` settings and
//! drops Joy's passcode hashes and cookie tokens. Photos are copied to `<photos root>/<couple>/`.
//! Money stays whole rupees, as in Joy. Runs in one transaction: all or nothing.

use std::{collections::HashMap, path::Path};

use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};

#[derive(Debug, Default)]
pub struct Report {
    pub counts: Vec<(&'static str, usize)>,
    pub photos: usize,
}

fn err(msg: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Protocol(msg.into())
}

/// Imports `joy_db` for the couple of `user_name` (started if they have none). With `dry_run`
/// nothing is written and the report says what would be copied.
/// Joy's settings that lived in its environment (see `joy::schedule` and `joy::ctx`).
pub const ENV_SETTINGS: &[&str] = &[
    "PERSON_A", "PERSON_B", "HOME_A", "HOME_B", "HOME_A_AREA", "HOME_B_AREA", "CURRENCY_SYMBOL",
    "DATE_DAYS", "OFFICE", "OFFICE_NAME", "WEEKNIGHT_START", "WEEKEND_HOURS", "WEEKEND_MAX_TRAVEL_MIN",
    "LATEST_A", "LATEST_A_AT", "LATEST_A_PLACE", "LATEST_B", "LATEST_B_AT", "LATEST_B_PLACE",
    "COMMUTE_A_MIN", "COMMUTE_B_MIN", "TRAVEL_A", "TRAVEL_B",
];

#[allow(clippy::too_many_arguments)]
pub async fn run(
    db: &SqlitePool,
    joy_db: &Path,
    user_name: &str,
    slot: usize,
    env: &[(String, String)],
    photos_from: Option<&Path>,
    photos_root: &Path,
    dry_run: bool,
) -> Result<Report, sqlx::Error> {
    let src = SqlitePool::connect_with(SqliteConnectOptions::new().filename(joy_db).read_only(true)).await?;
    let user_id: String = sqlx::query_scalar("SELECT user_id FROM users WHERE user_name = ?")
        .bind(user_name)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| err(format!("no Flinderax user called {user_name}")))?;

    let mut tx = db.begin().await?;
    let existing: Option<String> = sqlx::query_scalar("SELECT couple_id FROM couples WHERE user_0 = ? OR user_1 = ?")
        .bind(&user_id)
        .bind(&user_id)
        .fetch_optional(&mut *tx)
        .await?;
    let couple = match existing {
        Some(c) => {
            let has: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM joy_history WHERE couple_id = ?").bind(&c).fetch_one(&mut *tx).await?;
            if has > 0 {
                return Err(err("this couple already has dates; import only into a fresh Dates"));
            }
            // A fresh couple has the starter jar: the imported slips replace it.
            sqlx::query("DELETE FROM joy_activities WHERE couple_id = ?").bind(&c).execute(&mut *tx).await?;
            c
        }
        None => {
            let c = nanoid::nanoid!();
            let sql = if slot == 1 { "INSERT INTO couples (couple_id, user_1) VALUES (?, ?)" } else { "INSERT INTO couples (couple_id, user_0) VALUES (?, ?)" };
            sqlx::query(sql).bind(&c).bind(&user_id).execute(&mut *tx).await?;
            c
        }
    };
    // An existing couple keeps the slot the user already has; say so if it differs from --as.
    let actual: i64 = sqlx::query_scalar("SELECT CASE WHEN user_1 = ? THEN 1 ELSE 0 END FROM couples WHERE couple_id = ?")
        .bind(&user_id)
        .bind(&couple)
        .fetch_one(&mut *tx)
        .await?;
    if actual as usize != slot {
        return Err(err(format!(
            "{user_name} is person {} in Dates already, not {}; leave Dates first or import with --as {}",
            if actual == 1 { "b" } else { "a" },
            if slot == 1 { "b" } else { "a" },
            if actual == 1 { "b" } else { "a" }
        )));
    }
    let mut report = Report::default();

    // activities
    let mut act: HashMap<i64, i64> = HashMap::new();
    let cols = "title, notes, est_cost, setting, area, venue, lat, lng, duration, tags, energy, added_by, archived, times_done,
        last_done_at, last_skipped_at, created_at, generated, closed_days, season_from, season_to, est_from_actual, hours,
        best_time, minutes, booking, book_ahead_days, cover, bring, station, station_walk_min, share_token";
    let n_cols = cols.split(',').count();
    let rows = sqlx::query(&format!("SELECT id, {cols} FROM activities ORDER BY id")).fetch_all(&src).await?;
    for r in &rows {
        let placeholders = vec!["?"; n_cols].join(", ");
        let sql = format!("INSERT INTO joy_activities (couple_id, {cols}) VALUES (?, {placeholders})");
        let mut q = sqlx::query(&sql).bind(&couple);
        for i in 1..=n_cols {
            q = bind_any(q, r, i);
        }
        let new_id = q.execute(&mut *tx).await?.last_insert_rowid();
        act.insert(r.get::<i64, _>(0), new_id);
    }
    report.counts.push(("slips", rows.len()));

    // pairs and vetoes (reference slips only)
    let pairs = sqlx::query("SELECT a_id, b_id FROM pairs").fetch_all(&src).await?;
    for p in &pairs {
        let (a, b) = (act[&p.get::<i64, _>(0)], act[&p.get::<i64, _>(1)]);
        sqlx::query("INSERT INTO joy_pairs (a_id, b_id) VALUES (?, ?)").bind(a.min(b)).bind(a.max(b)).execute(&mut *tx).await?;
    }
    report.counts.push(("pairs", pairs.len()));
    let vetoes = sqlx::query("SELECT activity_id, person, created_at FROM vetoes").fetch_all(&src).await?;
    for v in &vetoes {
        sqlx::query("INSERT INTO joy_vetoes (activity_id, person, created_at) VALUES (?, ?, ?)")
            .bind(act[&v.get::<i64, _>(0)])
            .bind(v.get::<i64, _>(1))
            .bind(v.get::<i64, _>(2))
            .execute(&mut *tx)
            .await?;
    }
    report.counts.push(("vetoes", vetoes.len()));

    // history
    let mut hist: HashMap<i64, i64> = HashMap::new();
    let hrows = sqlx::query(
        "SELECT id, activity_id, picked_at, outcome, rating, note, drawn_by, planned_at, surprise, revealed, has_photo, est_cost,
                actual_cost, cab_fare_0, cab_est_0, cab_fare_1, cab_est_1, from_savings, plan_b, day_kind, share_token
         FROM history ORDER BY id",
    )
    .fetch_all(&src)
    .await?;
    for h in &hrows {
        let plan_b = h.get::<Option<i64>, _>(18).and_then(|p| act.get(&p).copied());
        let new_id = sqlx::query(
            "INSERT INTO joy_history (couple_id, activity_id, picked_at, outcome, rating, note, drawn_by, planned_at, surprise,
                revealed, has_photo, est_cost, actual_cost, cab_fare_0, cab_est_0, cab_fare_1, cab_est_1, from_savings, plan_b,
                day_kind, share_token)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&couple)
        .bind(act[&h.get::<i64, _>(1)])
        .bind(h.get::<i64, _>(2))
        .bind(h.get::<String, _>(3))
        .bind(h.get::<Option<i64>, _>(4))
        .bind(h.get::<String, _>(5))
        .bind(h.get::<Option<i64>, _>(6))
        .bind(h.get::<Option<i64>, _>(7))
        .bind(h.get::<i64, _>(8))
        .bind(h.get::<i64, _>(9))
        .bind(h.get::<i64, _>(10))
        .bind(h.get::<Option<i64>, _>(11))
        .bind(h.get::<Option<i64>, _>(12))
        .bind(h.get::<Option<i64>, _>(13))
        .bind(h.get::<Option<i64>, _>(14))
        .bind(h.get::<Option<i64>, _>(15))
        .bind(h.get::<Option<i64>, _>(16))
        .bind(h.get::<Option<i64>, _>(17))
        .bind(plan_b)
        .bind(h.get::<String, _>(19))
        .bind(h.get::<Option<String>, _>(20))
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        hist.insert(h.get::<i64, _>(0), new_id);
        // Joy's one photo per date arrives as <new id>.jpg (copied below): its first photo here.
        if h.get::<i64, _>(10) != 0 {
            sqlx::query("INSERT INTO joy_history_photos (history_id, file, added_at) VALUES (?, ?, ?)")
                .bind(new_id)
                .bind(format!("{new_id}.jpg"))
                .bind(h.get::<i64, _>(2))
                .execute(&mut *tx)
                .await?;
        }
    }
    report.counts.push(("dates", hrows.len()));

    let mem = sqlx::query("SELECT history_id, who, line, at FROM memories").fetch_all(&src).await?;
    let mut kept_mem = 0;
    for m in &mem {
        // Joy didn't enforce this foreign key: skip lines whose date is gone.
        let Some(h) = hist.get(&m.get::<i64, _>(0)) else { continue };
        sqlx::query("INSERT INTO joy_memories (history_id, who, line, at) VALUES (?, ?, ?, ?)")
            .bind(h)
            .bind(m.get::<i64, _>(1))
            .bind(m.get::<String, _>(2))
            .bind(m.get::<i64, _>(3))
            .execute(&mut *tx)
            .await?;
        kept_mem += 1;
    }
    report.counts.push(("memory lines", kept_mem));

    let tasks = sqlx::query("SELECT history_id, who, task, due, done_at FROM tasks").fetch_all(&src).await?;
    for t in &tasks {
        sqlx::query("INSERT INTO joy_tasks (history_id, who, task, due, done_at) VALUES (?, ?, ?, ?, ?)")
            .bind(hist[&t.get::<i64, _>(0)])
            .bind(t.get::<i64, _>(1))
            .bind(t.get::<String, _>(2))
            .bind(t.get::<String, _>(3))
            .bind(t.get::<Option<i64>, _>(4))
            .execute(&mut *tx)
            .await?;
    }
    report.counts.push(("jobs", tasks.len()));

    let checkins = sqlx::query("SELECT who, day, energy, budget, max_duration, reach, vibe, at FROM checkins").fetch_all(&src).await?;
    for c in &checkins {
        sqlx::query(
            "INSERT INTO joy_checkins (couple_id, who, day, energy, budget, max_duration, reach, vibe, at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&couple)
        .bind(c.get::<i64, _>(0))
        .bind(c.get::<String, _>(1))
        .bind(c.get::<String, _>(2))
        .bind(c.get::<Option<i64>, _>(3))
        .bind(c.get::<String, _>(4))
        .bind(c.get::<String, _>(5))
        .bind(c.get::<String, _>(6))
        .bind(c.get::<i64, _>(7))
        .execute(&mut *tx)
        .await?;
    }
    report.counts.push(("check-ins", checkins.len()));

    let other = sqlx::query("SELECT amount, note, spent_on FROM other_spending ORDER BY id").fetch_all(&src).await?;
    for o in &other {
        sqlx::query("INSERT INTO joy_other_spending (couple_id, amount, note, spent_on) VALUES (?, ?, ?, ?)")
            .bind(&couple)
            .bind(o.get::<i64, _>(0))
            .bind(o.get::<String, _>(1))
            .bind(o.get::<String, _>(2))
            .execute(&mut *tx)
            .await?;
    }
    report.counts.push(("other spending", other.len()));

    let months = sqlx::query("SELECT month, base, carry FROM month_budget").fetch_all(&src).await?;
    for m in &months {
        sqlx::query("INSERT INTO joy_month_budget (couple_id, month, base, carry) VALUES (?, ?, ?, ?)")
            .bind(&couple)
            .bind(m.get::<String, _>(0))
            .bind(m.get::<i64, _>(1))
            .bind(m.get::<i64, _>(2))
            .execute(&mut *tx)
            .await?;
    }
    report.counts.push(("budget months", months.len()));

    let savings = sqlx::query("SELECT amount, kind, note, on_day FROM savings ORDER BY id").fetch_all(&src).await?;
    for s in &savings {
        sqlx::query("INSERT INTO joy_savings (couple_id, amount, kind, note, on_day) VALUES (?, ?, ?, ?, ?)")
            .bind(&couple)
            .bind(s.get::<i64, _>(0))
            .bind(s.get::<String, _>(1))
            .bind(s.get::<String, _>(2))
            .bind(s.get::<String, _>(3))
            .execute(&mut *tx)
            .await?;
    }
    report.counts.push(("savings entries", savings.len()));

    let mut reg: HashMap<i64, i64> = HashMap::new();
    let regs = sqlx::query("SELECT id, title, place, maps_url, weekdays, usual_cost, active, since FROM regulars ORDER BY id")
        .fetch_all(&src)
        .await?;
    for r in &regs {
        let id = sqlx::query(
            "INSERT INTO joy_regulars (couple_id, title, place, maps_url, weekdays, usual_cost, active, since) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&couple)
        .bind(r.get::<String, _>(1))
        .bind(r.get::<String, _>(2))
        .bind(r.get::<String, _>(3))
        .bind(r.get::<String, _>(4))
        .bind(r.get::<i64, _>(5))
        .bind(r.get::<i64, _>(6))
        .bind(r.get::<String, _>(7))
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        reg.insert(r.get::<i64, _>(0), id);
    }
    report.counts.push(("regulars", regs.len()));

    let mut visit: HashMap<i64, i64> = HashMap::new();
    let visits = sqlx::query("SELECT id, regular_id, day, status, place, amount, has_photo FROM regular_visits ORDER BY id")
        .fetch_all(&src)
        .await?;
    for v in &visits {
        let id = sqlx::query(
            "INSERT INTO joy_regular_visits (regular_id, day, status, place, amount, has_photo) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(reg[&v.get::<i64, _>(1)])
        .bind(v.get::<String, _>(2))
        .bind(v.get::<String, _>(3))
        .bind(v.get::<String, _>(4))
        .bind(v.get::<i64, _>(5))
        .bind(v.get::<i64, _>(6))
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        visit.insert(v.get::<i64, _>(0), id);
    }
    report.counts.push(("regular nights", visits.len()));

    // settings: the budget and the savings goal; never the passcodes or their tokens
    let settings = sqlx::query("SELECT key, value FROM settings WHERE key IN ('monthly_budget', 'saving_for')").fetch_all(&src).await?;
    for s in &settings {
        let (key, mut value) = (s.get::<String, _>(0), s.get::<String, _>(1));
        if key == "saving_for" {
            match value.parse::<i64>().ok().and_then(|id| act.get(&id)) {
                Some(new) => value = new.to_string(),
                None => continue,
            }
        }
        sqlx::query(
            "INSERT INTO joy_settings (couple_id, key, value) VALUES (?, ?, ?) ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value",
        )
        .bind(&couple)
        .bind(key)
        .bind(value)
        .execute(&mut *tx)
        .await?;
    }
    let mut n_settings = settings.len();
    for (key, value) in env.iter().filter(|(k, v)| ENV_SETTINGS.contains(&k.as_str()) && !v.trim().is_empty()) {
        sqlx::query(
            "INSERT INTO joy_settings (couple_id, key, value) VALUES (?, ?, ?) ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value",
        )
        .bind(&couple)
        .bind(key)
        .bind(value)
        .execute(&mut *tx)
        .await?;
        n_settings += 1;
    }
    report.counts.push(("settings", n_settings));

    // photos: <history id>.jpg and v<visit id>.jpg, renamed to the new ids
    if let Some(from) = photos_from {
        let to = photos_root.join(&couple);
        if !dry_run {
            std::fs::create_dir_all(&to).map_err(|e| err(e.to_string()))?;
        }
        let pairs = hist
            .iter()
            .map(|(o, n)| (format!("{o}.jpg"), format!("{n}.jpg")))
            .chain(visit.iter().map(|(o, n)| (format!("v{o}.jpg"), format!("v{n}.jpg"))));
        for (old, new) in pairs {
            let src_path = from.join(&old);
            if src_path.exists() {
                if !dry_run {
                    std::fs::copy(&src_path, to.join(&new)).map_err(|e| err(e.to_string()))?;
                }
                report.photos += 1;
            }
        }
    }

    if dry_run {
        tx.rollback().await?;
    } else {
        tx.commit().await?;
    }
    Ok(report)
}

/// Binds column `i` of `row` whatever its SQLite type (NULL, integer, real or text).
fn bind_any<'q>(
    q: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    row: &sqlx::sqlite::SqliteRow,
    i: usize,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
        return q.bind(v);
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
        return q.bind(v);
    }
    q.bind(row.try_get::<Option<String>, _>(i).unwrap_or(None))
}

/// Where Dates photos live: `JOY_PHOTOS_DIR`, else `joy-photos` next to the database file.
pub fn photos_root() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("JOY_PHOTOS_DIR") {
        return dir.into();
    }
    let url = std::env::var("DATABASE_URL").unwrap_or_default();
    let file = url.trim_start_matches("sqlite:").trim_start_matches("//").split('?').next().unwrap_or("");
    Path::new(file).parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")).join("joy-photos")
}
