//! Fills gaps in slips from what the database already knows. It only ever fills blanks (never
//! overwrites what a person typed), so it is safe to run again. Run it with
//! `joy enrich-slips [--dry-run]`.
//!
//! - closed days: worked out from opening hours ("wed-sun 09:00-17:30" → closed Monday and Tuesday)
//! - coordinates: copied from another slip with the same venue
//! - nearest station: borrowed from a slip with a station within about 500 m
//! - best time: "sunset" / "dusk" in the title or notes → evening

use sqlx::SqlitePool;

use crate::joy::{ctx::road_km, hours::Hours, picker::WEEKDAYS};

/// A neighbour this close (road km, which is 1.3 × straight line) shares the nearest station.
const SAME_STATION_KM: f64 = 0.65;

#[derive(sqlx::FromRow, Clone)]
pub struct Row {
    pub id: i64,
    pub title: String,
    pub notes: String,
    pub venue: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub hours: String,
    pub closed_days: String,
    pub best_time: String,
    pub station: String,
    pub duration: String,
}

#[derive(Debug, PartialEq)]
pub enum Fill {
    Coords(f64, f64),
    Station(String),
    ClosedDays(String),
    BestTime(&'static str),
}

/// Everything that can be filled in, per slip id. Pure: reads the rows, changes nothing.
pub fn plan(rows: &[Row]) -> Vec<(i64, Fill)> {
    let mut out = vec![];
    for r in rows {
        // Coordinates from a twin (same venue text), when this one has none.
        let mut at = r.lat.zip(r.lng);
        if at.is_none() && !r.venue.is_empty() {
            let twin = rows.iter().find(|o| o.id != r.id && o.venue == r.venue && o.lat.is_some() && o.lng.is_some());
            if let Some(o) = twin {
                let (lat, lng) = (o.lat.unwrap(), o.lng.unwrap());
                out.push((r.id, Fill::Coords(lat, lng)));
                at = Some((lat, lng));
            }
        }
        if r.station.is_empty() {
            let near = at.and_then(|p| {
                rows.iter()
                    .filter(|o| o.id != r.id && !o.station.is_empty())
                    .filter_map(|o| Some((o, road_km(p, o.lat.zip(o.lng)?))))
                    .filter(|(_, km)| *km <= SAME_STATION_KM)
                    .min_by(|a, b| a.1.total_cmp(&b.1))
            });
            if let Some((o, _)) = near {
                out.push((r.id, Fill::Station(o.station.clone())));
            }
        }
        if r.closed_days.is_empty()
            && !crosses_midnight(&r.hours)
            && let Some(h) = Hours::parse(&r.hours)
        {
            let closed: Vec<&str> = (0..7u8).filter(|d| !h.open_on(*d)).map(|d| WEEKDAYS[d as usize].0).collect();
            if !closed.is_empty() && closed.len() < 7 {
                out.push((r.id, Fill::ClosedDays(closed.join(","))));
            }
        }
        // A time of day only means something for an outing, not a weekend away.
        if r.best_time.is_empty() && matches!(r.duration.as_str(), "short" | "half_day") {
            let text = format!("{} {}", r.title, r.notes).to_lowercase();
            if text.contains("sunset") || text.contains("dusk") {
                out.push((r.id, Fill::BestTime("evening")));
            }
        }
    }
    out
}

/// Any "HH:MM-HH:MM" stretch that ends at or before it starts runs past midnight, which the
/// weekday list can't describe, so those slips are left alone.
fn crosses_midnight(hours: &str) -> bool {
    hours.split([';', ' ']).any(|seg| {
        let Some((a, b)) = seg.split_once('-') else { return false };
        let t = |x: &str| -> Option<u32> {
            let (h, m) = x.split_once(':')?;
            Some(h.parse::<u32>().ok()? * 60 + m.parse::<u32>().ok()?)
        };
        matches!((t(a), t(b)), (Some(a), Some(b)) if b <= a)
    })
}

impl Fill {
    /// What was filled, in words, for the note after a save.
    pub fn describe(&self) -> String {
        match self {
            Fill::Coords(..) => "Location copied from another slip at the same venue".into(),
            Fill::Station(s) => format!("Nearest station: {s} (from a slip nearby)"),
            Fill::ClosedDays(d) => {
                let names: Vec<&str> = d
                    .split(',')
                    .filter_map(|k| WEEKDAYS.iter().find(|(key, _)| key == &k).map(|(_, n)| &n[..3]))
                    .collect();
                format!("Closed {} (worked out from the hours)", names.join(", "))
            }
            Fill::BestTime(t) => format!("Best time: {t} (it mentions the sunset)"),
        }
    }
}

/// Fills the blanks of one slip from what its couple's other slips know. Returns what was filled.
pub async fn run_one(db: &SqlitePool, couple: &str, id: i64) -> sqlx::Result<Vec<Fill>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, title, notes, venue, lat, lng, hours, closed_days, best_time, station, duration
         FROM joy_activities WHERE archived = 0 AND couple_id = ?",
    )
    .bind(couple)
    .fetch_all(db)
    .await?;
    let fills: Vec<Fill> = plan(&rows).into_iter().filter(|(i, _)| *i == id).map(|(_, f)| f).collect();
    for fill in &fills {
        match fill {
            Fill::Coords(lat, lng) => sqlx::query("UPDATE joy_activities SET lat = ?, lng = ? WHERE id = ? AND lat IS NULL").bind(lat).bind(lng).bind(id),
            Fill::Station(s) => sqlx::query("UPDATE joy_activities SET station = ? WHERE id = ? AND station = ''").bind(s).bind(id),
            Fill::ClosedDays(d) => sqlx::query("UPDATE joy_activities SET closed_days = ? WHERE id = ? AND closed_days = ''").bind(d).bind(id),
            Fill::BestTime(t) => sqlx::query("UPDATE joy_activities SET best_time = ? WHERE id = ? AND best_time = ''").bind(t).bind(id),
        }
        .execute(db)
        .await?;
    }
    Ok(fills)
}

/// Applies the plan to every slip. Returns how many fills were made (or would be, when `dry`).
pub async fn run(db: &SqlitePool, dry: bool) -> sqlx::Result<Vec<(i64, String, Fill)>> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, title, notes, venue, lat, lng, hours, closed_days, best_time, station, duration
         FROM joy_activities WHERE archived = 0",
    )
    .fetch_all(db)
    .await?;
    let titles: std::collections::HashMap<i64, String> = rows.iter().map(|r| (r.id, r.title.clone())).collect();
    let fills = plan(&rows);
    if !dry {
        let mut tx = db.begin().await?;
        for (id, fill) in &fills {
            match fill {
                Fill::Coords(lat, lng) => {
                    sqlx::query("UPDATE joy_activities SET lat = ?, lng = ? WHERE id = ? AND lat IS NULL").bind(lat).bind(lng).bind(id)
                }
                Fill::Station(s) => sqlx::query("UPDATE joy_activities SET station = ? WHERE id = ? AND station = ''").bind(s).bind(id),
                Fill::ClosedDays(d) => sqlx::query("UPDATE joy_activities SET closed_days = ? WHERE id = ? AND closed_days = ''").bind(d).bind(id),
                Fill::BestTime(t) => sqlx::query("UPDATE joy_activities SET best_time = ? WHERE id = ? AND best_time = ''").bind(t).bind(id),
            }
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
    }
    Ok(fills.into_iter().map(|(id, f)| (id, titles.get(&id).cloned().unwrap_or_default(), f)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: i64) -> Row {
        Row {
            id,
            title: format!("slip {id}"),
            notes: String::new(),
            venue: String::new(),
            lat: None,
            lng: None,
            hours: String::new(),
            closed_days: String::new(),
            best_time: String::new(),
            station: String::new(),
            duration: "short".into(),
        }
    }

    #[test]
    fn closed_days_come_from_hours() {
        let mut r = row(1);
        r.hours = "wed-sun 09:00-17:30".into();
        assert_eq!(plan(&[r]), vec![(1, Fill::ClosedDays("mon,tue".into()))]);
        let mut every = row(2);
        every.hours = "mon-sun 11:00-22:00".into();
        assert!(plan(&[every]).is_empty());
    }

    #[test]
    fn never_overwrites_what_is_there() {
        let mut r = row(1);
        r.hours = "wed-sun 09:00-17:30".into();
        r.closed_days = "mon".into();
        r.best_time = "morning".into();
        r.title = "Sunset walk".into();
        assert!(plan(&[r]).is_empty());
    }

    #[test]
    fn coordinates_and_station_already_set_are_never_replaced() {
        let mut lender = row(1);
        lender.venue = "Blue Tokai".into();
        lender.lat = Some(19.1647);
        lender.lng = Some(72.8493);
        lender.station = "Goregaon".into();
        // Same venue and a few metres apart, but each has its own coordinates and station already.
        let mut own = row(2);
        own.venue = "Blue Tokai".into();
        own.lat = Some(19.1648);
        own.lng = Some(72.8494);
        own.station = "Malad".into();
        assert!(plan(&[lender.clone(), own]).is_empty());
        // One blank field among filled ones: only that one is planned. It has coordinates (a few
        // metres from the lender) but no station.
        let mut half = row(3);
        half.lat = Some(19.1648);
        half.lng = Some(72.8494);
        assert_eq!(plan(&[lender, half]), vec![(3, Fill::Station("Goregaon".into()))]);
    }

    #[test]
    fn twin_venue_lends_coordinates_and_a_close_neighbour_lends_its_station() {
        let mut a = row(1);
        a.venue = "Blue Tokai".into();
        a.lat = Some(19.1647);
        a.lng = Some(72.8493);
        a.station = "Goregaon".into();
        let mut b = row(2);
        b.venue = "Blue Tokai".into();
        let mut far = row(3);
        far.lat = Some(19.30);
        far.lng = Some(72.90);
        let got = plan(&[a, b, far]);
        assert_eq!(got, vec![(2, Fill::Coords(19.1647, 72.8493)), (2, Fill::Station("Goregaon".into()))]);
    }

    #[test]
    fn overnight_hours_and_weekends_away_are_left_alone() {
        let mut night = row(1);
        night.hours = "fri-sat 22:00-05:30".into();
        let mut trip = row(2);
        trip.notes = "sunset on the beach".into();
        trip.duration = "weekend".into();
        assert!(plan(&[night, trip]).is_empty());
    }

    #[test]
    fn sunset_means_evening() {
        let mut r = row(1);
        r.notes = "Watch the sunset from the rocks".into();
        assert_eq!(plan(&[r]), vec![(1, Fill::BestTime("evening"))]);
    }
}
