//! Web Push, twice a day. A morning nudge about today and an evening one about tomorrow, from the
//! same "coming up" list the Today page shows. What is sent is only a count by kind ("Today: 2
//! bills, 1 date"): no names, no amounts, so a lock screen reveals nothing. Whatever starts the day
//! (a scheduled job calling `/api/internal/push/run`) wakes the sleeping machine; nothing here runs on its own.

use serde::Serialize;
use sqlx::SqlitePool;
use time::Date;

use crate::planner::coming::{Entry, Kind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Morning,
    Evening,
    /// Once a week: how many things need a decision (see `planner::week_digest`).
    Weekly,
}

impl Slot {
    pub fn parse(s: &str) -> Option<Slot> {
        match s {
            "morning" => Some(Slot::Morning),
            "evening" => Some(Slot::Evening),
            "weekly" => Some(Slot::Weekly),
            _ => None,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Message {
    pub title: String,
    pub body: String,
    pub url: String,
}

fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The nudge for a slot, or nothing when there is nothing to say. Morning is what is due today
/// (and follow-ups that are overdue); evening is what is on tomorrow.
pub fn digest(items: &[Entry], slot: Slot) -> Option<Message> {
    let (picked, lead): (Vec<&Entry>, &str) = match slot {
        Slot::Morning => (items.iter().filter(|e| e.days_away <= 0).collect(), "Today"),
        Slot::Evening => (items.iter().filter(|e| e.days_away == 1).collect(), "Tomorrow"),
        // The weekly nudge is built from decisions, not the coming-up list.
        Slot::Weekly => return None,
    };
    if picked.is_empty() {
        return None;
    }
    let of = |k: Kind| picked.iter().filter(|e| e.item.kind == k).count();
    let mut parts = Vec::new();
    for (kind, one, many) in [
        (Kind::Bill, "bill", "bills"),
        (Kind::Date, "date", "dates"),
        (Kind::Birthday, "birthday", "birthdays"),
        (Kind::FollowUp, "follow-up", "follow-ups"),
    ] {
        let n = of(kind);
        if n > 0 {
            parts.push(count(n, one, many));
        }
    }
    Some(Message { title: "joy".into(), body: format!("{lead}: {}.", parts.join(", ")), url: "/".into() })
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Subscription {
    pub id: i64,
    pub user_id: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Delivery {
    Sent,
    /// The browser says this subscription is gone (404/410): forget it.
    Gone,
    Failed,
}

/// Something that can deliver one message to one subscription. The real one speaks Web Push; tests record.
pub trait Sender: Send + Sync {
    fn send(&self, sub: &Subscription, message: &Message) -> impl std::future::Future<Output = Delivery> + Send;
}

#[derive(Serialize, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub users: usize,
    pub sent: usize,
    pub removed: usize,
    pub failed: usize,
}

/// Send the slot's nudge to everyone who has turned notifications on and has something coming up.
pub async fn run(db: &SqlitePool, sender: &impl Sender, slot: Slot, today: Date) -> Result<Report, crate::handlers::common::AppError> {
    let subs: Vec<Subscription> = sqlx::query_as("SELECT id, user_id, endpoint, p256dh, auth FROM push_subscriptions ORDER BY user_id, id").fetch_all(db).await?;
    let mut report = Report::default();
    let mut current: Option<(String, Option<Message>)> = None;
    for sub in subs {
        let message = match &current {
            Some((u, m)) if *u == sub.user_id => m.clone(),
            _ => {
                let m = if slot == Slot::Weekly {
                    let c = crate::handlers::decisions::counts(db, &sub.user_id, today).await?;
                    crate::planner::week_digest::weekly_body(&crate::planner::week_digest::decisions(&c))
                        .map(|body| Message { title: "joy".into(), body, url: "/".into() })
                } else {
                    let items = crate::handlers::coming::gather(db, &sub.user_id, today, 2).await?;
                    digest(&items, slot)
                };
                if m.is_some() {
                    report.users += 1;
                }
                current = Some((sub.user_id.clone(), m.clone()));
                m
            }
        };
        let Some(message) = message else { continue };
        match sender.send(&sub, &message).await {
            Delivery::Sent => report.sent += 1,
            Delivery::Gone => {
                sqlx::query("DELETE FROM push_subscriptions WHERE id = ?").bind(sub.id).execute(db).await?;
                report.removed += 1;
            }
            Delivery::Failed => report.failed += 1,
        }
    }
    Ok(report)
}

/// The uncompressed SEC1 point (65 bytes) of a signing key's public half: what a browser's `applicationServerKey` wants.
fn uncompressed_public(kp: &web_push_native::jwt_simple::algorithms::ES256KeyPair) -> Vec<u8> {
    use web_push_native::p256::elliptic_curve::sec1::ToEncodedPoint;
    let compressed = kp.public_key().to_bytes();
    web_push_native::p256::PublicKey::from_sec1_bytes(&compressed)
        .map(|k| k.to_encoded_point(false).as_bytes().to_vec())
        .unwrap_or_default()
}

/// The real sender: encrypts for the browser and posts to its push service, signed with our VAPID key.
pub struct WebPushSender {
    key_pair: web_push_native::jwt_simple::algorithms::ES256KeyPair,
    contact: String,
    client: reqwest::Client,
}

impl WebPushSender {
    /// From `VAPID_PRIVATE_KEY` (the key's 32 bytes, base64url) and `VAPID_CONTACT` ("mailto:..."). None when not configured.
    pub fn from_env() -> Option<Self> {
        let raw = std::env::var("VAPID_PRIVATE_KEY").ok()?;
        let contact = std::env::var("VAPID_CONTACT").unwrap_or_else(|_| "mailto:admin@joy.fly.dev".into());
        Self::new(&raw, contact)
    }

    pub fn new(private_key_b64url: &str, contact: String) -> Option<Self> {
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(private_key_b64url.trim()).ok()?;
        let key_pair = web_push_native::jwt_simple::algorithms::ES256KeyPair::from_bytes(&bytes).ok()?;
        Some(Self { key_pair, contact, client: reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().ok()? })
    }

    /// The public half browsers need to subscribe, base64url (uncompressed point).
    pub fn public_key(&self) -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(uncompressed_public(&self.key_pair))
    }

    /// A fresh key pair, for `joy vapid-keys`: (private, public), both base64url.
    pub fn generate() -> (String, String) {
        use base64::Engine as _;
        let kp = web_push_native::jwt_simple::algorithms::ES256KeyPair::generate();
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        (e.encode(kp.to_bytes()), e.encode(uncompressed_public(&kp)))
    }
}

impl Sender for WebPushSender {
    async fn send(&self, sub: &Subscription, message: &Message) -> Delivery {
        use base64::Engine as _;
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let built = (|| -> Option<_> {
            let public = web_push_native::p256::PublicKey::from_sec1_bytes(&e.decode(&sub.p256dh).ok()?).ok()?;
            let auth = web_push_native::Auth::clone_from_slice(&e.decode(&sub.auth).ok()?);
            let builder = web_push_native::WebPushBuilder::new(sub.endpoint.parse().ok()?, public, auth).with_vapid(&self.key_pair, &self.contact);
            builder.build(serde_json::to_vec(message).ok()?).ok()
        })();
        let Some(request) = built else { return Delivery::Gone };
        let (parts, body) = request.into_parts();
        let mut req = self.client.post(parts.uri.to_string()).body(body);
        for (k, v) in &parts.headers {
            req = req.header(k.as_str(), v.as_bytes());
        }
        match req.send().await {
            Ok(r) if r.status().is_success() => Delivery::Sent,
            Ok(r) if matches!(r.status().as_u16(), 404 | 410) => Delivery::Gone,
            Ok(_) | Err(_) => Delivery::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::coming::{Entry, Item};
    use time::macros::date;

    fn entry(kind: Kind, days_away: i64) -> Entry {
        Entry {
            item: Item { on: date!(2026 - 11 - 15) + time::Duration::days(days_away), kind, title: "SECRET NAME".into(), detail: "₹99,999".into(), link: String::new() },
            days_away,
            when: String::new(),
        }
    }

    #[test]
    fn the_morning_nudge_counts_what_is_due_today_and_never_names_anything() {
        let items = [entry(Kind::Bill, 0), entry(Kind::Bill, 0), entry(Kind::Date, 0), entry(Kind::Bill, 1), entry(Kind::FollowUp, -2)];
        let m = digest(&items, Slot::Morning).unwrap();
        assert_eq!(m.body, "Today: 2 bills, 1 date, 1 follow-up.");
        assert!(!m.body.contains("SECRET") && !m.body.contains("99"), "no names or amounts on a lock screen");
        assert_eq!(m.url, "/");
    }

    #[test]
    fn the_evening_nudge_is_about_tomorrow() {
        let items = [entry(Kind::Birthday, 1), entry(Kind::Bill, 0), entry(Kind::Bill, 3)];
        assert_eq!(digest(&items, Slot::Evening).unwrap().body, "Tomorrow: 1 birthday.");
    }

    #[test]
    fn nothing_to_say_means_no_message() {
        assert!(digest(&[], Slot::Morning).is_none());
        assert!(digest(&[entry(Kind::Bill, 3)], Slot::Evening).is_none());
        assert!(digest(&[entry(Kind::Bill, 1)], Slot::Morning).is_none());
    }

    #[test]
    fn slots_parse() {
        assert_eq!(Slot::parse("morning"), Some(Slot::Morning));
        assert_eq!(Slot::parse("evening"), Some(Slot::Evening));
        assert_eq!(Slot::parse("noon"), None);
    }

    #[test]
    fn a_generated_key_round_trips_and_has_a_public_half_browsers_accept() {
        let (private, public) = WebPushSender::generate();
        let sender = WebPushSender::new(&private, "mailto:a@b.c".into()).expect("the key loads back");
        assert_eq!(sender.public_key(), public);
        use base64::Engine as _;
        let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(public).unwrap();
        assert_eq!((raw.len(), raw[0]), (65, 4), "an uncompressed P-256 point");
        assert!(WebPushSender::new("not a key", "mailto:a@b.c".into()).is_none());
    }

    use std::sync::Mutex;

    struct Fake {
        sent: Mutex<Vec<(String, String)>>,
        gone: &'static str,
    }

    impl Sender for Fake {
        async fn send(&self, sub: &Subscription, message: &Message) -> Delivery {
            if sub.endpoint.contains(self.gone) {
                return Delivery::Gone;
            }
            self.sent.lock().unwrap().push((sub.endpoint.clone(), message.body.clone()));
            Delivery::Sent
        }
    }

    async fn db() -> SqlitePool {
        let db = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for u in ["u1", "u2"] {
            sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')").bind(u).bind(u).execute(&db).await.unwrap();
        }
        db
    }

    async fn subscribe(db: &SqlitePool, user: &str, endpoint: &str) {
        sqlx::query("INSERT INTO push_subscriptions (user_id, endpoint, p256dh, auth) VALUES (?, ?, 'k', 'a')").bind(user).bind(endpoint).execute(db).await.unwrap();
    }

    #[tokio::test]
    async fn the_run_sends_to_each_browser_of_each_person_with_something_coming_and_forgets_dead_ones() {
        let db = db().await;
        let today = crate::handlers::common::today_local();
        // u1 has a follow-up due today.
        sqlx::query("INSERT INTO people (person_id, user_id, name, circle) VALUES ('p1', 'u1', 'Asha', 'close')").execute(&db).await.unwrap();
        sqlx::query("INSERT INTO followups (followup_id, user_id, person_id, text, due_on) VALUES ('f1', 'u1', 'p1', 'Call back', ?)").bind(today.to_string()).execute(&db).await.unwrap();
        subscribe(&db, "u1", "https://push.example/phone").await;
        subscribe(&db, "u1", "https://push.example/laptop").await;
        subscribe(&db, "u1", "https://push.example/dead").await;
        subscribe(&db, "u2", "https://push.example/other").await;

        let fake = Fake { sent: Mutex::new(vec![]), gone: "dead" };
        let report = run(&db, &fake, Slot::Morning, today).await.ok().unwrap();
        assert_eq!(report, Report { users: 1, sent: 2, removed: 1, failed: 0 });
        let sent = fake.sent.lock().unwrap().clone();
        assert_eq!(sent.iter().map(|(e, _)| e.as_str()).collect::<Vec<_>>(), ["https://push.example/phone", "https://push.example/laptop"]);
        assert!(sent.iter().all(|(_, b)| b == "Today: 1 follow-up."));
        let left: i64 = sqlx::query_scalar("SELECT count(*) FROM push_subscriptions").fetch_one(&db).await.unwrap();
        assert_eq!(left, 3, "the dead one is forgotten; the quiet person's stays");

        // In the evening there is nothing on for tomorrow: nobody is bothered.
        let fake = Fake { sent: Mutex::new(vec![]), gone: "never" };
        assert_eq!(run(&db, &fake, Slot::Evening, today).await.ok().unwrap(), Report::default());
        assert!(fake.sent.lock().unwrap().is_empty());
    }
}
