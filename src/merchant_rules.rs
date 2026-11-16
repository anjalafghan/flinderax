//! Merchant memory (SQL side): the user's category corrections, remembered per merchant key in
//! `merchant_rules` so the next bank text or statement line from the same merchant is categorised
//! the way they last said. `pattern` holds the planner's `merchant_key` for these rows (older rows
//! hold a words-contained pattern and still work). The rules are read into `alerts::categorize`,
//! which puts them before the built-in seed (`planner::merchant_seed`) and the keyword lists.

use sqlx::SqlitePool;

use crate::{alerts::CATEGORIES, planner::merchant::merchant_key};

/// True when `category` is one a rule may point at: a real app category that says something
/// (remembering "uncategorized" would only hide the built-in guesses).
pub fn rememberable(category: &str) -> bool {
    category != "uncategorized" && CATEGORIES.contains(&category)
}

/// Insert or replace the rule for exactly `pattern`; a repeat counts as another confirmation.
pub async fn upsert_pattern(db: &SqlitePool, user_id: &str, pattern: &str, category: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO merchant_rules (user_id, pattern, category, updated_at, hits)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP, 1)
         ON CONFLICT(user_id, pattern) DO UPDATE
         SET category = excluded.category, updated_at = CURRENT_TIMESTAMP, hits = hits + 1",
    )
    .bind(user_id)
    .bind(pattern)
    .bind(category)
    .execute(db)
    .await?;
    Ok(())
}

/// Remember that `payee` (the raw merchant text) belongs in `category` for this user. Does nothing
/// and returns `false` when the payee has no merchant key (a person's phone number, all noise) or
/// the category is not one worth remembering.
pub async fn remember(db: &SqlitePool, user_id: &str, payee: &str, category: &str) -> Result<bool, sqlx::Error> {
    let key = merchant_key(payee);
    if key.is_empty() || !rememberable(category) {
        return Ok(false);
    }
    upsert_pattern(db, user_id, &key, category).await?;
    Ok(true)
}

/// The category this user has remembered for an exact merchant key. Never another user's.
pub async fn lookup(db: &SqlitePool, user_id: &str, key: &str) -> Result<Option<String>, sqlx::Error> {
    if key.is_empty() {
        return Ok(None);
    }
    sqlx::query_scalar("SELECT category FROM merchant_rules WHERE user_id = ? AND pattern = ?")
        .bind(user_id)
        .bind(key)
        .fetch_optional(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{categorize, Direction};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn db() -> SqlitePool {
        let db = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for id in ["u", "other"] {
            sqlx::query("INSERT INTO users (user_id, user_name, user_password, user_role) VALUES (?, ?, 'x', 'user')")
                .bind(id)
                .bind(id)
                .execute(&db)
                .await
                .unwrap();
        }
        db
    }

    async fn rules(db: &SqlitePool, user: &str) -> Vec<(String, String)> {
        sqlx::query_as("SELECT pattern, category FROM merchant_rules WHERE user_id = ?").bind(user).fetch_all(db).await.unwrap()
    }

    #[tokio::test]
    async fn upsert_replaces_the_category_for_the_same_key() {
        let db = db().await;
        assert!(remember(&db, "u", "UPI/CORNER CAFE@okaxis", "food").await.unwrap());
        assert!(remember(&db, "u", "Corner Cafe 4412", "dates").await.unwrap());
        assert_eq!(lookup(&db, "u", "corner cafe").await.unwrap().as_deref(), Some("dates"));
        assert_eq!(rules(&db, "u").await.len(), 1, "one row per key");
        let hits: i64 = sqlx::query_scalar("SELECT hits FROM merchant_rules WHERE user_id = 'u'").fetch_one(&db).await.unwrap();
        assert_eq!(hits, 2);
    }

    #[tokio::test]
    async fn an_empty_key_or_a_useless_category_is_ignored() {
        let db = db().await;
        for (payee, cat) in [("9876543210@ybl", "people"), ("", "food"), ("UPI/", "food"), ("Corner Cafe", ""), ("Corner Cafe", "nonsense"), ("Corner Cafe", "uncategorized")] {
            assert!(!remember(&db, "u", payee, cat).await.unwrap(), "{payee:?} {cat:?}");
        }
        assert!(rules(&db, "u").await.is_empty());
        assert_eq!(lookup(&db, "u", "").await.unwrap(), None);
    }

    #[tokio::test]
    async fn another_users_rule_is_never_used() {
        let db = db().await;
        remember(&db, "other", "Corner Cafe", "dates").await.unwrap();
        assert_eq!(lookup(&db, "u", "corner cafe").await.unwrap(), None);
        assert!(rules(&db, "u").await.is_empty());
    }

    #[tokio::test]
    async fn remembered_rules_beat_the_seed_and_the_seed_beats_keywords() {
        let db = db().await;
        let debit = |rules: &[(String, String)], m: &str| categorize(Some(m), Direction::Debit, false, rules);

        // Seed over keywords: "swiggy instamart" is groceries in the seed; the keyword list says swiggy is food.
        assert_eq!(debit(&[], "SWIGGY INSTAMART"), "groceries");
        assert_eq!(debit(&[], "SWIGGY"), "food");
        // "Zepto Cafe" would be food by keyword ("cafe") but the seed knows zepto.
        assert_eq!(debit(&[], "ZEPTO CAFE 4412"), "groceries");

        // The user's correction beats the seed.
        remember(&db, "u", "UPI/SWIGGY@icici", "dates").await.unwrap();
        let mine = rules(&db, "u").await;
        assert_eq!(debit(&mine, "SWIGGY BANGALORE 4412"), "dates");
        // ... and for someone else the seed still applies.
        assert_eq!(debit(&rules(&db, "other").await, "SWIGGY BANGALORE 4412"), "food");
    }
}
