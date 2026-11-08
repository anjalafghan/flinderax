//! Dates (Joy) JSON API, part 3: regulars, nights, photos, and the month and year pages.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode},
};
use joy::{app::build_router, models::AppState};
use rusty_paseto::prelude::{Key, PasetoSymmetricKey};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

struct T {
    app: Router,
    db: SqlitePool,
}

struct Res {
    status: StatusCode,
    headers: HeaderMap,
    json: Value,
    text: String,
}

async fn setup() -> T {
    static PHOTOS: std::sync::Once = std::sync::Once::new();
    PHOTOS.call_once(|| {
        let dir = std::env::temp_dir().join(format!("joy-regulars-test-{}", std::process::id()));
        // SAFETY: set once, before any request reads it.
        unsafe { std::env::set_var("JOY_PHOTOS_DIR", dir) };
    });
    let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();
    let key = PasetoSymmetricKey::from(Key::from(&[7u8; 32]));
    let app = build_router(AppState { db: db.clone(), redis: None, paseto_key: Arc::new(key) });
    T { app, db }
}

impl T {
    async fn call(&self, method: Method, path: &str, token: Option<&str>, body: Option<Value>) -> Res {
        let mut req = Request::builder().method(method).uri(path).header("fly-client-ip", "10.99.0.1");
        if let Some(t) = token {
            req = req.header("authorization", format!("Bearer {t}"));
        }
        let req = match body {
            Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let (status, headers) = (res.status(), res.headers().clone());
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        Res { status, headers, json: serde_json::from_str(&text).unwrap_or(Value::Null), text }
    }

    async fn get(&self, path: &str, token: &str) -> Res {
        self.call(Method::GET, path, Some(token), None).await
    }

    async fn multipart(&self, path: &str, token: &str, field: &str, bytes: &[u8]) -> Res {
        let boundary = "xBOUNDARYx";
        let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{field}\"; filename=\"p.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n").into_bytes();
        body.extend_from_slice(bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        let req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header("fly-client-ip", "10.99.0.1")
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(Body::from(body))
            .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let (status, headers) = (res.status(), res.headers().clone());
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        Res { status, headers, json: serde_json::from_str(&text).unwrap_or(Value::Null), text }
    }

    async fn post(&self, path: &str, token: &str, body: Value) -> Res {
        self.call(Method::POST, path, Some(token), Some(body)).await
    }

    /// A registered, logged-in user; returns their bearer token.
    async fn user(&self, label: &str) -> String {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let (name, ip) = (format!("{label}{n}"), format!("10.88.{}.{}", n / 250, n % 250));
        let creds = json!({"user_name": name, "user_password": "correct horse"});
        for (path, expect) in [("/api/common/register", StatusCode::OK), ("/api/common/login", StatusCode::OK)] {
            let req = Request::post(path)
                .header("content-type", "application/json")
                .header("fly-client-ip", &ip)
                .body(Body::from(creds.to_string()))
                .unwrap();
            let res = self.app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), expect, "{path}");
            if path.ends_with("login") {
                let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
                return serde_json::from_slice::<Value>(&bytes).unwrap()["access_token"].as_str().unwrap().to_string();
            }
        }
        unreachable!()
    }

    /// Two users who are a couple: (rohit's token, neha's token, couple id). Rohit is slot 0.
    async fn couple(&self) -> (String, String, String) {
        let (a, b) = (self.user("rohit").await, self.user("neha").await);
        assert!(self.post("/api/couple/create", &a, json!({})).await.status.is_success());
        let code = self.post("/api/couple/invite", &a, json!({})).await.json["code"].as_str().unwrap().to_string();
        let joined = self.post("/api/couple/join", &b, json!({"code": code})).await;
        assert!(joined.status.is_success(), "{}", joined.text);
        let id = self.text("SELECT couple_id FROM couples ORDER BY created_at DESC, rowid DESC LIMIT 1").await;
        (a, b, id)
    }

    /// Someone in a couple of their own (just themselves).
    async fn loner(&self) -> (String, String) {
        let c = self.user("carol").await;
        assert!(self.post("/api/couple/create", &c, json!({})).await.status.is_success());
        let id = self.text(&format!(
            "SELECT couple_id FROM couples WHERE user_0 = (SELECT user_id FROM users ORDER BY rowid DESC LIMIT 1)"
        ))
        .await;
        (c, id)
    }

    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(&self.db).await.unwrap();
    }

    async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.db).await.unwrap()
    }

    async fn text(&self, sql: &str) -> String {
        sqlx::query_scalar(sql).fetch_one(&self.db).await.unwrap()
    }
}

fn jpeg() -> Vec<u8> {
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 4, image::Rgb([200, 80, 40])))
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .unwrap();
    out
}

/// A regular on Tue and Thu costing ₹300, made through the API; returns its id.
async fn add_regular(t: &T, token: &str) -> i64 {
    let r = t.post("/api/dates/regulars", token, json!({"title": "Blue Tokai", "place": "Goregaon East", "days": [2, 4], "cost": "300"})).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.text);
    r.json["to"].as_str().unwrap().rsplit('/').next().unwrap().parse().unwrap()
}

async fn today(t: &T, offset: i64) -> String {
    t.text(&format!("SELECT date('now', 'localtime', '{offset} days')")).await
}

#[tokio::test]
async fn adding_a_regular_checks_the_form() {
    let t = setup().await;
    let (rohit, _, couple) = t.couple().await;

    for (body, msg) in [
        (json!({"title": "  ", "days": [2]}), "Give it a name."),
        (json!({"title": "Blue Tokai", "days": []}), "Pick at least one day."),
        (json!({"title": "Blue Tokai", "days": ["9", "x"]}), "Pick at least one day."),
        (json!({"title": "Blue Tokai"}), "Pick at least one day."),
    ] {
        let r = t.post("/api/dates/regulars", &rohit, body).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.text);
        assert_eq!(r.json["error"], msg);
    }
    assert_eq!(t.scalar("SELECT count(*) FROM joy_regulars").await, 0);

    // Only web addresses are kept; days are cleaned; numbers work where Joy's form had text.
    let r = t
        .post("/api/dates/regulars", &rohit, json!({"title": "Chai", "maps_url": "javascript:alert(1)", "days": [4, 2, 4], "cost": 250}))
        .await;
    let id = r.json["to"].as_str().unwrap().strip_prefix("/joy/regulars/").unwrap().to_string();
    let (maps, days, cost): (String, String, i64) =
        sqlx::query_as("SELECT maps_url, weekdays, usual_cost FROM joy_regulars").fetch_one(&t.db).await.unwrap();
    assert_eq!((maps.as_str(), days.as_str(), cost), ("", "2,4", 250));
    let ok = t.post("/api/dates/regulars", &rohit, json!({"title": "Maps", "maps_url": "https://maps.app.goo.gl/x", "days": "1"})).await;
    assert_eq!(ok.status, StatusCode::OK);
    assert_eq!(t.text("SELECT maps_url FROM joy_regulars WHERE title = 'Maps'").await, "https://maps.app.goo.gl/x");

    let list = t.get("/api/dates/regulars", &rohit).await;
    assert_eq!(list.status, StatusCode::OK, "{}", list.text);
    assert_eq!(list.json["items"].as_array().unwrap().len(), 2);
    assert_eq!(list.json["items"][0]["id"].to_string(), id);
    assert_eq!(list.json["items"][0]["days"], "Tuesdays and Thursdays");
    assert_eq!(list.json["days"].as_array().unwrap().len(), 7);
    assert_eq!(t.scalar(&format!("SELECT count(*) FROM joy_regulars WHERE couple_id = '{couple}'")).await, 2);
}

#[tokio::test]
async fn going_or_skipping_a_night_and_the_spread_totals() {
    let t = setup().await;
    let (rohit, neha, _) = t.couple().await;
    let id = add_regular(&t, &rohit).await;
    let (d0, d1, d2) = (today(&t, 0).await, today(&t, -1).await, today(&t, -2).await);

    // Blank amount = the usual; typed amounts may carry commas; "somewhere else" is kept.
    let went = |day: &str, body: Value| {
        let mut b = json!({"day": day});
        b.as_object_mut().unwrap().extend(body.as_object().unwrap().clone());
        b
    };
    let r = t.post(&format!("/api/dates/regulars/{id}/went"), &neha, went(&d0, json!({"back": "/joy"}))).await;
    assert_eq!((r.status, r.json["ok"].clone(), r.json["to"].clone()), (StatusCode::OK, json!(true), json!("/joy")), "{}", r.text);
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, went(&d1, json!({"amount": "1,200", "place": "Third Wave"}))).await;
    // Typing the regular's own name is just the regular; skipping costs nothing; junk is ignored.
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, went(&d2, json!({"amount": 100, "place": "blue tokai"}))).await;
    let skipped = today(&t, -3).await;
    t.post(&format!("/api/dates/regulars/{id}/skip"), &rohit, json!({"day": skipped})).await;
    let future = today(&t, 3).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, went(&future, json!({}))).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, went(&today(&t, -4).await, json!({"amount": "lots"}))).await;
    assert_eq!(t.scalar("SELECT count(*) FROM joy_regular_visits").await, 4);
    assert_eq!(t.scalar("SELECT count(*) FROM joy_regular_visits WHERE status = 'skipped' AND amount = 0").await, 1);

    // Answering again changes the answer.
    t.post(&format!("/api/dates/regulars/{id}/skip"), &rohit, json!({"day": d2})).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, went(&d2, json!({"amount": 100}))).await;

    let spread = t.get(&format!("/api/dates/regulars/{id}"), &rohit).await;
    assert_eq!(spread.status, StatusCode::OK, "{}", spread.text);
    let v = &spread.json;
    assert_eq!((v["title"].as_str(), v["days"].as_str()), (Some("Blue Tokai"), Some("Tuesdays and Thursdays")));
    assert_eq!(v["count"], 3);
    assert_eq!(v["elsewhere"], 1);
    assert_eq!(v["total"], "₹1,600");
    assert_eq!(v["average"], "₹533");
    assert_eq!(v["stamps"].as_array().unwrap().len(), 3);
    assert_eq!(v["stamps"][0]["amount"], "₹300", "newest first");
    assert_eq!(v["stamps"][1]["elsewhere"], "Third Wave");
    assert_eq!(v["stamps"][2]["elsewhere"], "");
    assert_eq!(t.get("/api/dates/regulars", &rohit).await.json["items"][0]["count"], 3);

    // Nobody home yet: no average.
    let other = add_regular(&t, &rohit).await;
    let empty = t.get(&format!("/api/dates/regulars/{other}"), &rohit).await.json;
    assert_eq!((empty["count"].clone(), empty["average"].clone(), empty["total"].clone()), (json!(0), json!(""), json!("Free")));
}

#[tokio::test]
async fn deleting_a_night_takes_its_photo_too() {
    let t = setup().await;
    let (rohit, _, _) = t.couple().await;
    let id = add_regular(&t, &rohit).await;
    let day = today(&t, 0).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, json!({"day": day})).await;
    let vid = t.scalar("SELECT id FROM joy_regular_visits").await;
    let up = t.multipart(&format!("/api/dates/nights/{vid}/photo"), &rohit, "photo", &jpeg()).await;
    assert_eq!(up.status, StatusCode::OK, "{}", up.text);
    assert_eq!(t.scalar("SELECT has_photo FROM joy_regular_visits").await, 1);

    let gone = t.post(&format!("/api/dates/nights/{vid}/delete"), &rohit, json!({})).await;
    assert_eq!((gone.status, gone.json["to"].clone()), (StatusCode::OK, json!(format!("/joy/regulars/{id}"))), "{}", gone.text);
    assert_eq!(t.scalar("SELECT count(*) FROM joy_regular_visits").await, 0);
    assert_eq!(t.post(&format!("/api/dates/nights/{vid}/delete"), &rohit, json!({})).await.status, StatusCode::NOT_FOUND);
    // The unsigned address and an unknown night give nothing.
    assert_eq!(t.call(Method::GET, &format!("/api/public/dates/nights/{vid}/photo"), None, None).await.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_night_photo_uploads_and_is_served_only_to_a_valid_signature() {
    let t = setup().await;
    let (rohit, _, _) = t.couple().await;
    let id = add_regular(&t, &rohit).await;
    let day = today(&t, 0).await;
    let skipped = today(&t, -1).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, json!({"day": day})).await;
    t.post(&format!("/api/dates/regulars/{id}/skip"), &rohit, json!({"day": skipped})).await;
    let vid = t.scalar(&format!("SELECT id FROM joy_regular_visits WHERE day = '{day}'")).await;
    let sid = t.scalar(&format!("SELECT id FROM joy_regular_visits WHERE day = '{skipped}'")).await;

    // Only a night we went to takes a photo; junk is refused with the reason.
    assert_eq!(t.multipart(&format!("/api/dates/nights/{sid}/photo"), &rohit, "photo", &jpeg()).await.status, StatusCode::NOT_FOUND);
    let bad = t.multipart(&format!("/api/dates/nights/{vid}/photo"), &rohit, "photo", b"definitely not a photo").await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad.json["error"], "That file doesn't look like a photo.");
    let wrong = t.multipart(&format!("/api/dates/nights/{vid}/photo"), &rohit, "file", &jpeg()).await;
    assert_eq!(wrong.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(t.scalar("SELECT has_photo FROM joy_regular_visits WHERE day = date('now', 'localtime')").await, 0);

    let up = t.multipart(&format!("/api/dates/nights/{vid}/photo"), &rohit, "photo", &jpeg()).await;
    assert_eq!(up.status, StatusCode::OK, "{}", up.text);
    let url = up.json["photo"].as_str().unwrap().to_string();
    assert!(url.starts_with(&format!("/api/public/dates/nights/{vid}/photo?v=")), "{url}");
    assert_eq!(t.get(&format!("/api/dates/regulars/{id}"), &rohit).await.json["stamps"][0]["photo"].as_str().unwrap().split("&exp").next(), url.split("&exp").next());

    // The signed link works with no login at all, and caches for good.
    let img = t.call(Method::GET, &url, None, None).await;
    assert_eq!(img.status, StatusCode::OK);
    assert_eq!(img.headers["content-type"], "image/jpeg");
    assert!(img.headers["cache-control"].to_str().unwrap().contains("immutable"));
    assert!(img.text.len() > 100);

    // Tampering with anything breaks it.
    let sig = url.rsplit("sig=").next().unwrap();
    let flipped = format!("{}{}", &sig[..sig.len() - 1], if sig.ends_with('0') { '1' } else { '0' });
    for bad in [
        url.replace(sig, &flipped),
        url.replace(&format!("nights/{vid}/"), &format!("nights/{}/", vid + 1)),
        url.replacen("&exp=", "&exp=9", 1),
        url.split("&exp").next().unwrap().to_string(),
        url.replace("?v=", "?v=9"),
    ] {
        assert_eq!(t.call(Method::GET, &bad, None, None).await.status, StatusCode::FORBIDDEN, "{bad}");
    }

    // Taking the photo off.
    assert_eq!(t.post(&format!("/api/dates/nights/{vid}/photo/delete"), &rohit, json!({})).await.status, StatusCode::OK);
    assert_eq!(t.call(Method::GET, &url, None, None).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.get(&format!("/api/dates/regulars/{id}"), &rohit).await.json["stamps"][0]["photo"], "");
}

/// A kept date this month: Dinner at Rohit's, five stars, cost ₹900 against a ₹500 guess.
async fn keep_a_date(t: &T, couple: &str, title: &str, rating: i64) {
    let activity = t.scalar(&format!("SELECT min(id) FROM joy_activities WHERE couple_id = '{couple}'")).await;
    t.exec(&format!("UPDATE joy_activities SET title = '{title}', venue = 'Rohit''s Kitchen' WHERE id = {activity}")).await;
    t.exec(&format!(
        "INSERT INTO joy_history (couple_id, activity_id, outcome, rating, note, est_cost, actual_cost, picked_at)
         VALUES ('{couple}', {activity}, 'did_it', {rating}, 'Lovely evening\nsecond line', 500, 900, unixepoch() - 60)"
    ))
    .await;
}

#[tokio::test]
async fn the_month_page_lists_the_night() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let id = add_regular(&t, &rohit).await;
    let day = today(&t, 0).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, json!({"day": day, "amount": 450})).await;
    let vid = t.scalar("SELECT id FROM joy_regular_visits").await;
    t.multipart(&format!("/api/dates/nights/{vid}/photo"), &rohit, "photo", &jpeg()).await;
    keep_a_date(&t, &couple, "Dinner at home", 5).await;
    t.exec(&format!("INSERT INTO joy_other_spending (couple_id, amount, note, spent_on) VALUES ('{couple}', 120, 'Ice cream', '{day}')")).await;

    let (y, m) = (&day[..4], day[5..7].trim_start_matches('0'));
    let page = t.get(&format!("/api/dates/year/{y}/{m}"), &neha).await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.text);
    let v = &page.json;
    assert_eq!(v["year"].to_string(), y);
    assert_eq!(v["entries"].as_array().unwrap().len(), 1);
    assert_eq!(v["entries"][0]["title"], "Dinner at home");
    assert_eq!((v["entries"][0]["est"].as_str(), v["entries"][0]["diff"].as_str()), (Some("₹500"), Some("₹400 over our guess")));
    assert_eq!(v["entries"][0]["note"], "Lovely evening");
    assert_eq!(v["best"]["rating"], 5);
    assert_eq!(v["avg_rating"], "5.0");
    assert_eq!(v["others"][0]["note"], "Ice cream");
    let nights = &v["nights"][0];
    assert_eq!((nights["title"].as_str(), nights["count"].as_u64(), nights["total"].as_str()), (Some("Blue Tokai"), Some(1), Some("₹450")));
    assert_eq!(nights["href"], format!("/joy/regulars/{id}"));
    let photo = nights["nights"][0]["photo"].as_str().unwrap();
    assert_eq!(t.call(Method::GET, photo, None, None).await.status, StatusCode::OK, "{photo}");
    // Dates + slips-free spending + the regular's nights, all in the month's spend.
    assert_eq!(v["spent"], "₹1,470");

    // Nonsense isn't a page; a month with nothing in it is just empty.
    assert_eq!(t.get(&format!("/api/dates/year/{y}/13"), &rohit).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.get(&format!("/api/dates/year/{y}/abc"), &rohit).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.get("/api/dates/year/1999/5", &rohit).await.status, StatusCode::NOT_FOUND);
    let quiet = t.get(&format!("/api/dates/year/2001/2"), &rohit).await;
    assert_eq!((quiet.status, quiet.json["entries"].as_array().unwrap().len()), (StatusCode::OK, 0));
}

#[tokio::test]
async fn the_year_page_adds_up_and_its_share_data_carries_no_names() {
    let t = setup().await;
    let (rohit, _, couple) = t.couple().await;
    t.exec(&format!("INSERT INTO joy_settings (couple_id, key, value) VALUES ('{couple}', 'PERSON_A', 'Rohit'), ('{couple}', 'PERSON_B', 'Neha'), ('{couple}', 'HOME_A_AREA', 'Bandra') ON CONFLICT(couple_id, key) DO UPDATE SET value = excluded.value")).await;
    keep_a_date(&t, &couple, "Dinner with Rohit and Neha", 5).await;
    t.exec(&format!("UPDATE joy_activities SET venue = 'Rohit''s Kitchen in Bandra' WHERE couple_id = '{couple}' AND id = (SELECT min(id) FROM joy_activities WHERE couple_id = '{couple}')")).await;
    let id = add_regular(&t, &rohit).await;
    let day = today(&t, 0).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, json!({"day": day, "amount": 300})).await;

    let year = &day[..4];
    let page = t.get(&format!("/api/dates/year/{year}"), &rohit).await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.text);
    let v = &page.json;
    assert_eq!(v["dates"], 1);
    assert_eq!(v["years"][0].to_string(), year);
    assert_eq!(v["best"]["title"], "Dinner with Rohit and Neha", "the page itself may use names");
    assert_eq!((v["best"]["rating"].clone(), v["avg_rating"].clone(), v["rated"].clone()), (json!(5), json!("5.0"), json!(1)));
    assert_eq!(v["money"]["spent"], "₹900");
    assert_eq!(v["money"]["diff"], "₹400 over what we guessed");
    assert_eq!(v["money"]["over"], true);
    assert_eq!(v["year_total"], "₹1,200", "the date plus the regular's night");
    assert_eq!(v["streak_weeks"], 1);
    let m: usize = day[5..7].parse().unwrap();
    assert_eq!(v["months"][m - 1]["n"], 1);
    assert_eq!(v["rating_line"], "1 5-star");

    let share = serde_json::to_string(&v["share"]).unwrap();
    for secret in ["Rohit", "Neha", "Bandra", "Lovely evening", "rohit", "neha"] {
        assert!(!share.contains(secret), "{secret} leaked into {share}");
    }
    assert_eq!(v["share"]["best_title"], "Dinner with us and us");
    assert_eq!(v["share"]["places"], json!(["us's Kitchen in"]));
    assert_eq!(v["share"]["dates"], 1);
    assert_eq!(v["share"]["spent"], "₹900");
    assert!(v["share"].get("note").is_none() && v["share"].get("me_name").is_none());

    // /year points at this year (or last, in January); the range is 2000-2100.
    let to = t.get("/api/dates/year", &rohit).await;
    let want = if &day[5..7] == "01" { year.parse::<i32>().unwrap() - 1 } else { year.parse().unwrap() };
    assert_eq!((to.json["ok"].clone(), to.json["to"].clone()), (json!(true), json!(format!("/joy/year/{want}"))));
    assert_eq!(t.get("/api/dates/year/2101", &rohit).await.status, StatusCode::NOT_FOUND);
    let empty = t.get("/api/dates/year/2005", &rohit).await.json;
    assert_eq!((empty["dates"].clone(), empty["best"].clone(), empty["share"]["dates"].clone()), (json!(0), Value::Null, json!(0)));
}

#[tokio::test]
async fn another_couples_regulars_nights_and_years_stay_theirs() {
    let t = setup().await;
    let (rohit, neha, couple) = t.couple().await;
    let (carol, _) = t.loner().await;
    let id = add_regular(&t, &rohit).await;
    let day = today(&t, 0).await;
    t.post(&format!("/api/dates/regulars/{id}/went"), &rohit, json!({"day": day, "amount": 300})).await;
    let vid = t.scalar("SELECT id FROM joy_regular_visits").await;
    t.multipart(&format!("/api/dates/nights/{vid}/photo"), &rohit, "photo", &jpeg()).await;
    keep_a_date(&t, &couple, "Our date", 4).await;

    assert_eq!(t.get(&format!("/api/dates/regulars/{id}"), &carol).await.status, StatusCode::NOT_FOUND);
    for path in [format!("/api/dates/regulars/{id}/went"), format!("/api/dates/regulars/{id}/skip")] {
        assert_eq!(t.post(&path, &carol, json!({"day": day})).await.status, StatusCode::NOT_FOUND, "{path}");
    }
    for path in [format!("/api/dates/nights/{vid}/delete"), format!("/api/dates/nights/{vid}/photo/delete")] {
        assert_eq!(t.post(&path, &carol, json!({})).await.status, StatusCode::NOT_FOUND, "{path}");
    }
    assert_eq!(t.multipart(&format!("/api/dates/nights/{vid}/photo"), &carol, "photo", &jpeg()).await.status, StatusCode::NOT_FOUND);
    assert_eq!(t.scalar("SELECT count(*) FROM joy_regular_visits").await, 1);
    assert_eq!(t.scalar("SELECT has_photo FROM joy_regular_visits").await, 1);

    // Her lists and pages hold none of ours.
    assert_eq!(t.get("/api/dates/regulars", &carol).await.json["items"], json!([]));
    let (y, m) = (&day[..4], day[5..7].trim_start_matches('0'));
    let year = t.get(&format!("/api/dates/year/{y}"), &carol).await.json;
    assert_eq!((year["dates"].clone(), year["year_total"].clone()), (json!(0), json!("")));
    let month = t.get(&format!("/api/dates/year/{y}/{m}"), &carol).await.json;
    assert_eq!((month["entries"].clone(), month["nights"].clone(), month["spent"].clone()), (json!([]), json!([]), json!("")));
    // Both of us see it.
    assert_eq!(t.get(&format!("/api/dates/year/{y}"), &neha).await.json["dates"], 1);

    // Logged out, or without a couple, none of it opens.
    let lonely = t.user("dave").await;
    for path in ["/api/dates/regulars", "/api/dates/year/2026", "/api/dates/year/2026/3"] {
        assert_eq!(t.call(Method::GET, path, None, None).await.status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(t.get(path, &lonely).await.status, StatusCode::FORBIDDEN, "{path}");
    }
    assert_eq!(t.post("/api/dates/regulars", &lonely, json!({"title": "x", "days": [1]})).await.status, StatusCode::FORBIDDEN);
}
