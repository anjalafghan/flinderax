//! `GET /api/dashboard/habits`: the habits to act on now, from the user's statements, transactions,
//! cards, accounts and saved life plan. The rules are pure in `planner::habits`; this fills their
//! input from the same sources the analytics, insights and life endpoints read.

use std::collections::BTreeMap;

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use time::{macros::format_description, Date, Duration};

use crate::{
    handlers::{
        analytics,
        common::{today_local, AppError},
        insights, life,
    },
    models::AppState,
    narration::{self, Rail},
    planner::{
        analytics::{AnalyticsOut, MonthOut},
        habits::{build, BankIn, CardIn, FoodIn, Habit, HabitsInput, LifeIn, SalaryIn, SpendIn},
    },
};

/// Transaction window for food, the same as the analytics and insights.
const HISTORY_DAYS: i64 = 400;
/// Window for the 90-day rules (categories, transfers, small payments).
const RECENT_DAYS: i64 = 90;

#[derive(Serialize)]
pub struct HabitsOut {
    pub habits: Vec<Habit>,
}

pub async fn get(
    State(state): State<AppState>,
    Extension((user_id, _)): Extension<(String, String)>,
) -> Result<Json<HabitsOut>, AppError> {
    Ok(Json(HabitsOut {
        habits: compute(&state, &user_id, today_local()).await?,
    }))
}

/// The habits for a user as of `today`.
pub async fn compute(state: &AppState, user_id: &str, today: Date) -> Result<Vec<Habit>, AppError> {
    let input = load_input(state, user_id, today).await?;
    Ok(build(&input, today))
}

/// Loads every input the habit rules read.
pub async fn load_input(
    state: &AppState,
    user_id: &str,
    today: Date,
) -> Result<HabitsInput, AppError> {
    let db = &state.db;
    let an = analytics::compute(db, user_id, today).await?;
    let ins = insights::compute(db, user_id, today).await?;
    let saved = life::get(
        State(state.clone()),
        Extension((user_id.to_string(), String::new())),
    )
    .await?
    .0;

    let current = month_key(today);
    let completed: Vec<&MonthOut> = an.months.iter().filter(|m| m.month < current).collect();
    let last_three: Vec<i64> = completed
        .iter()
        .rev()
        .take(3)
        .map(|m| m.spend_paise)
        .collect();
    let avg_monthly_spend_paise = match last_three.len() {
        0 => 0,
        n => last_three.iter().sum::<i64>() / n as i64,
    };

    let liquid_paise: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(balance_paise), 0) FROM accounts WHERE user_id = ? AND locked = 0",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;
    let own = own_transfers_out(db, user_id, today).await?;

    Ok(HabitsInput {
        banks: bank_accounts(db, user_id).await?,
        cards: cards(db, user_id).await?,
        spend: spend_input(
            &an,
            liquid_paise,
            ins.health.emergency_months_tenths,
            avg_monthly_spend_paise,
            own,
        ),
        salary: an.salary.as_ref().map(|s| SalaryIn {
            missing_months: s.missing_months.clone(),
            days: s.typical_days.clone(),
        }),
        sip_paise: an.sips.iter().map(|s| s.amount_paise).collect(),
        food: food(db, user_id, today).await?,
        life: saved.saved.then(|| LifeIn {
            fits: saved.plan.monthly.fits,
            shortfall_paise: saved.plan.monthly.shortfall_paise,
            top_goal: saved
                .plan
                .goals
                .iter()
                .filter(|g| g.monthly_sip_paise > 0)
                .max_by_key(|g| g.monthly_sip_paise)
                .map(|g| g.label.to_string()),
            term_gap_paise: saved.plan.insurance.term_gap_paise,
            health_gap_paise: saved.plan.insurance.health_gap_paise,
        }),
    })
}

fn spend_input(
    analytics: &AnalyticsOut,
    liquid_paise: i64,
    emergency_months_tenths: Option<i64>,
    avg_monthly_spend_paise: i64,
    own_transfers_out: (i64, i64),
) -> SpendIn {
    let spend_90d_paise: i64 = analytics.categories.iter().map(|c| c.paise).sum();
    let uncategorised_90d_paise = analytics
        .categories
        .iter()
        .find(|c| c.category == "uncategorized")
        .map_or(0, |c| c.paise);
    SpendIn {
        spend_90d_paise,
        uncategorised_90d_paise,
        any_spend: analytics.months.iter().any(|m| m.spend_paise > 0),
        avg_monthly_spend_paise,
        liquid_paise,
        emergency_months_tenths,
        own_transfers_out_count: own_transfers_out.0,
        own_transfers_out_paise: own_transfers_out.1,
        small_per_month_count: analytics.small_ticket.per_month_count,
        small_per_month_paise: analytics.small_ticket.per_month_paise,
    }
}

/// Bank accounts with the last day any of their statements covers.
async fn bank_accounts(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<BankIn>, AppError> {
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT a.account_id, a.name,
                (SELECT MAX(s.last_date) FROM statement_imports s WHERE s.account_id = a.account_id AND s.user_id = a.user_id)
         FROM accounts a WHERE a.user_id = ? AND a.kind = 'bank' ORDER BY a.name, a.account_id",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(account_id, name, last)| BankIn {
            account_id,
            name,
            last_covered: last
                .and_then(|s| Date::parse(&s, format_description!("[year]-[month]-[day]")).ok()),
        })
        .collect())
}

async fn cards(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<CardIn>, AppError> {
    let rows: Vec<(String, String, Option<i64>)> =
        sqlx::query_as("SELECT card_id, card_name, statement_day FROM cards WHERE user_id = ? ORDER BY card_name, card_id")
            .bind(user_id)
            .fetch_all(db)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(card_id, name, day)| CardIn {
            card_id,
            name,
            statement_day: day.and_then(|d| u8::try_from(d).ok()),
        })
        .collect())
}

/// Own-account transfers out over the last 90 days: debits categorised as transfers over UPI, IMPS
/// or NEFT, the same rule the analytics use for own transfers.
async fn own_transfers_out(
    db: &sqlx::SqlitePool,
    user_id: &str,
    today: Date,
) -> Result<(i64, i64), AppError> {
    let since = (today - Duration::days(RECENT_DAYS)).to_string();
    let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
        "SELECT amount_paise, raw_text FROM transactions
         WHERE user_id = ? AND direction = 'debit' AND category = 'transfer' AND occurred_on >= ? AND occurred_on <= ?",
    )
    .bind(user_id)
    .bind(since)
    .bind(today.to_string())
    .fetch_all(db)
    .await?;
    Ok(rows
        .iter()
        .filter(|(_, raw)| {
            matches!(
                narration::parse(raw.as_deref().unwrap_or("")).rail,
                Rail::Upi | Rail::Imps | Rail::Neft
            )
        })
        .fold((0, 0), |(count, paise), (amount, _)| {
            (count + 1, paise + amount)
        }))
}

/// Food and groceries spend by month over the history window, for this month and the three before.
async fn food(db: &sqlx::SqlitePool, user_id: &str, today: Date) -> Result<FoodIn, AppError> {
    let since = (today - Duration::days(HISTORY_DAYS)).to_string();
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT substr(occurred_on, 1, 7), SUM(amount_paise) FROM transactions
         WHERE user_id = ? AND direction = 'debit' AND category IN ('food', 'groceries')
           AND occurred_on >= ? AND occurred_on <= ?
         GROUP BY 1",
    )
    .bind(user_id)
    .bind(since)
    .bind(today.to_string())
    .fetch_all(db)
    .await?;
    let by_month: BTreeMap<String, i64> = rows.into_iter().collect();

    let mut prior = Vec::new();
    let mut cursor = today;
    for _ in 0..3 {
        cursor = previous_month_end(cursor);
        prior.push(by_month.get(&month_key(cursor)).copied().unwrap_or(0));
    }
    Ok(FoodIn {
        any_tagged: !by_month.is_empty(),
        this_month_paise: by_month.get(&month_key(today)).copied().unwrap_or(0),
        prior_avg_paise: prior.iter().sum::<i64>() / 3,
        day: today.day(),
        days_in_month: today.month().length(today.year()),
    })
}

/// "YYYY-MM" for a date, the form the analytics use for months.
fn month_key(d: Date) -> String {
    format!("{:04}-{:02}", d.year(), d.month() as u8)
}

/// The last day of the month before `d`'s month.
fn previous_month_end(d: Date) -> Date {
    d - Duration::days(i64::from(d.day()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::habits::Status;
    use sqlx::sqlite::SqlitePoolOptions;
    use time::Month;

    fn state(db: sqlx::SqlitePool) -> AppState {
        let key = rusty_paseto::prelude::PasetoSymmetricKey::from(
            rusty_paseto::prelude::Key::from(&[7u8; 32]),
        );
        AppState {
            db,
            redis: None,
            paseto_key: std::sync::Arc::new(key),
        }
    }

    #[tokio::test]
    async fn habits_read_only_this_users_data_and_fill_the_rules() {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        for q in [
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('u', 'u', 'x', 'user')",
            "INSERT INTO users (user_id, user_name, user_password, user_role) VALUES ('other', 'other', 'x', 'user')",
            "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise) VALUES ('acctA', 'u', 'HDFC', 'bank', 0)",
            "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise) VALUES ('acctB', 'u', 'Kotak', 'bank', 0)",
            "INSERT INTO accounts (account_id, user_id, name, kind, balance_paise) VALUES ('acctO', 'other', 'Other', 'bank', 0)",
            "INSERT INTO cards (card_id, user_id, card_name, card_bank, card_primary_color, card_secondary_color, statement_day)
             VALUES ('cardA', 'u', 'Millennia', 'HDFC', 0, 0, 20)",
            "INSERT INTO statement_imports (import_id, user_id, account_id, first_date, last_date, rows)
             VALUES ('imp1', 'u', 'acctA', '2026-07-01', '2026-08-20', 40)",
            "INSERT INTO statement_imports (import_id, user_id, account_id, first_date, last_date, rows)
             VALUES ('imp2', 'other', 'acctO', '2026-01-01', '2026-11-01', 40)",
            "INSERT INTO transactions (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, raw_text, source, dedupe_key)
             VALUES ('t1', 'u', '2026-10-05', 'debit', 900000, 'Corner Tea', 'food', 'UPI payment', 'manual', 'k1'),
                    ('t2', 'u', '2026-11-02', 'debit', 100000, 'Corner Tea', 'groceries', 'UPI payment', 'manual', 'k2'),
                    ('t3', 'other', '2026-10-06', 'debit', 999999, 'Other Shop', 'food', 'UPI payment', 'manual', 'k3'),
                    ('t4', 'u', '2026-10-07', 'debit', 500000, 'Shop', 'uncategorized', 'UPI payment', 'manual', 'k4')",
        ] {
            sqlx::query(q).execute(&db).await.unwrap();
        }
        let today = Date::from_calendar_date(2026, Month::November, 9).unwrap();
        let habits = compute(&state(db), "u", today)
            .await
            .ok()
            .expect("habits compute");
        let find = |key: &str| habits.iter().find(|h| h.key == key).cloned();

        // HDFC covered to 20 Aug: due 18 Nov. Kotak never uploaded. The other user's account is absent.
        let hdfc = find("bank_statement:acctA").expect("hdfc habit");
        assert_eq!(
            (hdfc.due_on.as_deref(), hdfc.status),
            (Some("2026-11-18"), Status::DueSoon)
        );
        let kotak = find("bank_statement:acctB").expect("kotak habit");
        assert_eq!(kotak.status, Status::Overdue);
        assert!(habits.iter().all(|h| !h.key.contains("acctO")));

        // Card statement day 20: due 23 Nov.
        let card = find("card_statement:cardA").expect("card habit");
        assert_eq!(card.due_on.as_deref(), Some("2026-11-23"));

        // Food is tagged, so the weekly budget shows, not the tagging habit. No life plan is saved.
        assert!(find("tag_food_payees").is_none());
        assert!(find("food_budget").is_some());
        assert!(find("life_plan_missing").is_some());
    }
}
