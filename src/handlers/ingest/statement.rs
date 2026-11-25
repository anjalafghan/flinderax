//! Statement import: preview, upload, matching against alerts.

use super::*;

// ---------- statement import ----------

pub(super) const MAX_STATEMENT_ROWS: usize = 5_000;
/// The first bytes of an Excel (OLE2) file. Only the delimited text export can be read.
pub(super) const OLE2_MAGIC: [u8; 4] = [0xD0, 0xCF, 0x11, 0xE0];
pub(super) const EXCEL_MESSAGE: &str = "This is the Excel export. Download the Delimited (.txt) format instead.";
/// A payee with a debit and a credit this large, and no holder names saved, is offered as the user.
pub(super) const HOLDER_SUGGESTION_PAISE: i64 = 500_000;
pub(super) const MAX_HOLDER_NAMES: usize = 5;
/// Balances may differ by a paisa from rounding.
pub(super) const CHAIN_TOLERANCE_PAISE: i64 = 1;

#[derive(Deserialize)]
pub struct StatementPayload {
    /// The CSV text of a bank or card statement export.
    pub csv: String,
    /// The bank account or card the statement belongs to (optional; sets last 4 and the balance).
    pub account_id: Option<String>,
    pub card_id: Option<String>,
    /// A card statement's summary box (total due, dates), when the client read one.
    #[serde(default)]
    pub card_summary: Option<CardSummaryText>,
}

#[derive(Serialize)]
pub struct StatementResult {
    pub total: usize,
    pub added: usize,
    /// Already recorded: the same file or an overlapping one imported before.
    pub duplicates: usize,
    /// Alert rows this statement confirmed. Same number as `upgraded`; kept for older clients.
    pub matched_alerts: usize,
    /// Bank-alert transactions that the statement confirmed (kept, with the statement's reference and text).
    pub upgraded: usize,
    /// Rows already in joy with no account or card that this import put on its account or card.
    pub linked: usize,
    /// Rows under the header that could not be read.
    pub skipped: usize,
    /// Rows this import left without a category.
    pub uncategorized: usize,
    /// Rows this import added or confirmed that got a category from structure, rules or keywords.
    pub categorized: usize,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
    pub balance_updated: Option<BalanceUpdated>,
    /// The account kept a balance entered after this statement ends, so the statement's closing one wasn't applied.
    pub balance_kept: Option<BalanceKept>,
    /// Rows from earlier imports whose category changed when this import re-checked them.
    pub older_recategorized: usize,
    /// Each row's balance follows from the one before it (in file order, or reversed).
    pub chain_ok: bool,
    pub opening_paise: Option<i64>,
    pub closing_paise: Option<i64>,
    /// Bank-alert transactions on this account or card inside the statement's dates that the statement never confirmed.
    pub not_on_statement: usize,
    /// A payee the user seems to pay and receive from, when no holder names are saved.
    pub holder_name_suggestion: Option<String>,
    /// A card statement's summary box (from a PDF), and what it did to the card's bill.
    pub card_bill: Option<CardBillFromStatement>,
}

/// A card statement's summary box as text read off the page (the PDF reader finds it; the server
/// reads the amounts and dates with the same rules as the rows).
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CardSummaryText {
    pub total_due: String,
    pub minimum_due: Option<String>,
    pub statement_date: String,
    pub due_date: Option<String>,
}

/// A card statement's summary, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CardSummary {
    pub total_due: i64,
    pub minimum_due: Option<i64>,
    pub statement_date: Date,
    pub due_date: Option<Date>,
}

/// An amount owed on a card: "₹58,239.00" -> 5823900. A credit balance ("1,200.00 Cr") owes nothing.
fn owed(raw: &str) -> Option<i64> {
    if !raw.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let lower = raw.trim().to_lowercase();
    if lower.ends_with("cr") || lower.ends_with("cr.") {
        return Some(0);
    }
    Some(crate::statement::parse_amount(raw).map_or(0, i64::abs))
}

impl CardSummaryText {
    /// `None` when the total due or the statement date can't be read: then nothing is recorded.
    pub(super) fn read(&self) -> Option<CardSummary> {
        Some(CardSummary {
            total_due: owed(&self.total_due)?,
            minimum_due: self.minimum_due.as_deref().and_then(owed),
            statement_date: crate::statement::parse_date(&self.statement_date)?,
            due_date: self.due_date.as_deref().and_then(crate::statement::parse_date),
        })
    }
}

/// Records a card statement's summary as the card's latest snapshot, dated on the statement date,
/// unless that statement was recorded already or a snapshot newer than it exists. Outstanding is the
/// total due plus the EMI principal not yet billed, so the next-bill estimate keeps the EMIs. A
/// card with no billing cycle saved gets the statement's (fills blanks only).
pub(super) async fn apply_card_summary(
    conn: &mut sqlx::SqliteConnection,
    user_id: &str,
    card_id: &str,
    summary: &CardSummary,
    today: Date,
) -> Result<CardBillFromStatement, AppError> {
    let card_name: String = sqlx::query_scalar("SELECT card_name FROM cards WHERE card_id = ? AND user_id = ?")
        .bind(card_id)
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await?;
    let statement_date = summary.statement_date.to_string();
    let mut out = CardBillFromStatement {
        card_name,
        status: "recorded",
        total_due_paise: summary.total_due,
        minimum_due_paise: summary.minimum_due,
        statement_date: statement_date.clone(),
        due_date: summary.due_date.map(|d| d.to_string()),
        kept_since: None,
    };

    let already: Option<i64> = sqlx::query_scalar("SELECT 1 FROM card_snapshots WHERE card_id = ? AND statement_on = ?")
        .bind(card_id)
        .bind(&statement_date)
        .fetch_optional(&mut *conn)
        .await?;
    if already.is_some() {
        out.status = "already";
        return Ok(out);
    }
    // A snapshot typed (or read from a statement) after this statement's date is newer: keep it.
    let newest: Option<String> = sqlx::query_scalar("SELECT MAX(substr(created_at, 1, 10)) FROM card_snapshots WHERE card_id = ?")
        .bind(card_id)
        .fetch_one(&mut *conn)
        .await?;
    if let Some(newest) = newest.filter(|d| *d > statement_date) {
        out.status = "kept_newer";
        out.kept_since = Some(newest);
        return Ok(out);
    }

    let emi_unbilled = crate::handlers::card_cycle::emi_remaining(&mut *conn, card_id, today).await?;
    let outstanding = summary.total_due + emi_unbilled;
    let starting: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM card_snapshots WHERE card_id = ?")
        .bind(card_id)
        .fetch_one(&mut *conn)
        .await?;
    // End of the statement's day in India (UTC is 5:30 behind), so it sorts after that day's typed ones.
    let dated = format!("{statement_date} 18:29:59");
    sqlx::query(
        "INSERT INTO card_snapshots (snapshot_id, card_id, total_due_paise, outstanding_paise, is_starting, statement_on, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(nanoid!())
    .bind(card_id)
    .bind(summary.total_due)
    .bind(outstanding)
    .bind(starting == 0)
    .bind(&statement_date)
    .bind(&dated)
    .execute(&mut *conn)
    .await?;
    // The legacy history needs the card's running row (a card restored from a backup may lack it).
    sqlx::query("INSERT OR IGNORE INTO card_running_state (card_id, last_total_due_paise, last_delta_paise) VALUES (?, 0, 0)")
        .bind(card_id)
        .execute(&mut *conn)
        .await?;
    crate::handlers::card::record_total(&mut *conn, card_id, outstanding).await?;
    if let Some(due) = summary.due_date {
        sqlx::query(
            "UPDATE cards SET statement_day = COALESCE(statement_day, ?), due_day = COALESCE(due_day, ?) WHERE card_id = ? AND user_id = ?",
        )
        .bind(i64::from(summary.statement_date.day()))
        .bind(i64::from(due.day()))
        .bind(card_id)
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(out)
}

/// The identity of a statement row. It deliberately leaves out the card or account the file is
/// imported to, so re-importing the same file against a different target is still a duplicate.
pub(super) fn statement_key(row: &StatementRow) -> String {
    format!("stmt:{}:{}:{}:{}", row.date, row.direction.as_str(), row.amount_paise, normalize(&row.narration))
}

/// Holder names: newline-separated in `user_settings.holder_names`.
pub(super) fn split_names(raw: &str) -> Vec<String> {
    raw.lines().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()
}

pub(super) async fn load_holder_names(db: &sqlx::SqlitePool, user_id: &str) -> Result<Vec<String>, AppError> {
    let raw: Option<String> = sqlx::query_scalar("SELECT holder_names FROM user_settings WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional(db)
        .await?;
    Ok(split_names(&raw.unwrap_or_default()))
}

/// Rows this statement row can confirm, nearest date first. Each is either a bank-alert row, or a
/// row an earlier statement import confirmed from the same narration (so re-imports are duplicates).
/// Bare rows from the statement itself (keyed 'stmt:') are never candidates: their own key already
/// answers "seen before".
pub(super) async fn find_by_reference(
    conn: &mut sqlx::SqliteConnection,
    user_id: &str,
    row: &StatementRow,
    rrn: Option<&str>,
) -> Result<Vec<(String, i64, String)>, sqlx::Error> {
    let on = row.date.to_string();
    sqlx::query_as(
        "SELECT txn_id, on_statement, category FROM transactions
         WHERE user_id = ?1 AND direction = ?2 AND amount_paise = ?3
           AND occurred_on BETWEEN date(?4, '-1 day') AND date(?4, '+1 day')
           AND dedupe_key NOT LIKE 'stmt:%' AND source != 'manual'
           AND (rrn = ?5 OR (on_statement = 1 AND raw_text = ?6))
         ORDER BY on_statement ASC, ABS(julianday(occurred_on) - julianday(?4)) ASC, created_at ASC",
    )
    .bind(user_id)
    .bind(row.direction.as_str())
    .bind(row.amount_paise)
    .bind(&on)
    .bind(rrn)
    .bind(&row.narration)
    .fetch_all(conn)
    .await
}

/// Bank-alert rows on this card or account, same direction and amount, within a day of the row,
/// that no statement has confirmed yet. An alert on one card must not swallow a row on another
/// account. An import with no target matches any alert.
pub(super) async fn find_alert_to_confirm(
    conn: &mut sqlx::SqliteConnection,
    user_id: &str,
    row: &StatementRow,
    account_id: Option<&str>,
    card_id: Option<&str>,
) -> Result<Vec<(String, i64, String)>, sqlx::Error> {
    let on = row.date.to_string();
    sqlx::query_as(
        "SELECT txn_id, on_statement, category FROM transactions
         WHERE user_id = ?1 AND direction = ?2 AND amount_paise = ?3
           AND occurred_on BETWEEN date(?4, '-1 day') AND date(?4, '+1 day')
           AND on_statement = 0 AND dedupe_key NOT LIKE 'stmt:%' AND source != 'manual'
           -- A targeted import matches alerts on the same card/account, plus alerts that never got one.
           AND ((?5 = '' AND ?6 = '')
                OR ((account_id IS NULL OR account_id = ?5) AND (card_id IS NULL OR card_id = ?6)))
         ORDER BY ABS(julianday(occurred_on) - julianday(?4)) ASC, created_at ASC",
    )
    .bind(user_id)
    .bind(row.direction.as_str())
    .bind(row.amount_paise)
    .bind(&on)
    .bind(account_id.unwrap_or(""))
    .bind(card_id.unwrap_or(""))
    .fetch_all(conn)
    .await
}

/// Checks that each balance follows from the one before it: prev - debit + credit == balance.
/// Tries file order first, then reversed (some exports list newest first). Returns
/// (chain_ok, opening, closing), opening and closing in chronological order. Without a balance on
/// every row there is nothing to check: (false, None, None).
pub(super) fn balance_chain(rows: &[StatementRow]) -> (bool, Option<i64>, Option<i64>) {
    if rows.is_empty() || rows.iter().any(|r| r.balance_paise.is_none()) {
        return (false, None, None);
    }
    let forward: Vec<&StatementRow> = rows.iter().collect();
    let reversed: Vec<&StatementRow> = rows.iter().rev().collect();
    let holds = |seq: &[&StatementRow]| {
        seq.windows(2).all(|w| {
            let prev = w[0].balance_paise.unwrap_or_default();
            let step = if w[1].direction == Direction::Credit { w[1].amount_paise } else { -w[1].amount_paise };
            (prev + step - w[1].balance_paise.unwrap_or_default()).abs() <= CHAIN_TOLERANCE_PAISE
        })
    };
    let opening_of = |seq: &[&StatementRow]| {
        let first = seq[0];
        let signed = if first.direction == Direction::Debit { first.amount_paise } else { -first.amount_paise };
        first.balance_paise.unwrap_or_default() + signed
    };
    for seq in [&forward, &reversed] {
        if holds(seq) {
            return (true, Some(opening_of(seq)), seq.last().and_then(|r| r.balance_paise));
        }
    }
    (false, Some(opening_of(&forward)), forward.last().and_then(|r| r.balance_paise))
}

/// The payee with a debit and a credit of at least the threshold and the largest total. Offered as
/// the user's own name, since they move money to and from themselves.
pub(super) fn holder_suggestion(rows: &[StatementRow]) -> Option<String> {
    let mut by_payee: BTreeMap<String, (bool, bool, i64)> = BTreeMap::new();
    for r in rows {
        let Some(payee) = crate::narration::parse(&r.narration).payee else { continue };
        let entry = by_payee.entry(payee).or_insert((false, false, 0));
        if r.amount_paise >= HOLDER_SUGGESTION_PAISE {
            match r.direction {
                Direction::Debit => entry.0 = true,
                Direction::Credit => entry.1 = true,
            }
        }
        entry.2 += r.amount_paise;
    }
    by_payee
        .into_iter()
        .filter(|(_, (debit, credit, _))| *debit && *credit)
        .max_by_key(|(_, (_, _, total))| *total)
        .map(|(name, _)| name)
}

/// What one statement import asks for. The bytes of an uploaded file are read into `csv` and dropped.
pub struct StatementInput {
    pub csv: String,
    pub account_id: Option<String>,
    pub card_id: Option<String>,
    /// The columns the user mapped. Without one, the layout is auto-detected.
    pub mapping: Option<Mapping>,
    /// A card statement's summary box, read off the PDF. Used only when importing to a card.
    pub card_summary: Option<CardSummaryText>,
}

/// The bank account or card a statement is imported to, and its last digits.
#[derive(Default)]
struct Target {
    account_id: Option<String>,
    card_id: Option<String>,
    last4: Option<String>,
}

impl Target {
    async fn resolve(db: &sqlx::SqlitePool, user_id: &str, input: &StatementInput) -> Result<Self, AppError> {
        let given = |id: &Option<String>| id.clone().filter(|s| !s.is_empty());
        let find = |sql: &'static str, id: String, missing: &'static str| async move {
            sqlx::query_as::<_, (String, Option<String>)>(sql)
                .bind(id)
                .bind(user_id)
                .fetch_optional(db)
                .await?
                .ok_or_else(|| not_found(missing))
        };
        if let Some(id) = given(&input.account_id) {
            let (account_id, last4) =
                find("SELECT account_id, last_4_digits FROM accounts WHERE account_id = ? AND user_id = ?", id, "Account not found").await?;
            return Ok(Target { account_id: Some(account_id), card_id: None, last4 });
        }
        if let Some(id) = given(&input.card_id) {
            let (card_id, last4) = find("SELECT card_id, last_4_digits FROM cards WHERE card_id = ? AND user_id = ?", id, "Card not found").await?;
            return Ok(Target { account_id: None, card_id: Some(card_id), last4 });
        }
        Ok(Target::default())
    }
}

/// Reads the statement with the user's mapping, or auto-detects the layout. Empty and oversized files are refused.
fn parse_statement(input: &StatementInput, card: bool) -> Result<crate::statement::Parsed, AppError> {
    let parsed = match &input.mapping {
        Some(mapping) => crate::statement::parse_with(&input.csv, mapping),
        None => crate::statement::parse_for(&input.csv, card)
            .ok_or_else(|| bad_request("Couldn't find the transaction table. The file needs a Date column and Debit/Credit or Amount columns."))?,
    };
    if parsed.rows.is_empty() {
        return Err(bad_request(if input.mapping.is_some() {
            "Couldn't read any transactions with these columns. Check the column mapping."
        } else {
            "No transactions found in that file."
        }));
    }
    if parsed.rows.len() > MAX_STATEMENT_ROWS {
        return Err(bad_request(format!("Too many rows (limit {MAX_STATEMENT_ROWS}). Import one period at a time.")));
    }
    Ok(parsed)
}

/// What is worked out about a statement row before it is matched or stored.
struct Prepared {
    on: String,
    credit: bool,
    category: String,
    merchant: Option<String>,
    raw: String,
    rrn: Option<String>,
}

/// An import in progress: the counts so far, and the rows this import has already used.
struct Importer<'a> {
    user_id: &'a str,
    target: &'a Target,
    card: bool,
    rules: &'a [(String, String)],
    owner: &'a crate::narration::Owner<'a>,
    /// How many times each statement key has appeared in this file.
    seen: BTreeMap<String, u32>,
    /// Rows this import has already confirmed or added: a second identical row must not claim them.
    claimed: HashSet<String>,
    res: StatementResult,
}

impl Importer<'_> {
    fn tally(&mut self, category: &str) {
        if category == "uncategorized" {
            self.res.uncategorized += 1;
        } else {
            self.res.categorized += 1;
        }
    }

    /// Two identical rows on one day are two purchases; the n-th gets its own key.
    fn dedupe_key(&mut self, row: &StatementRow) -> String {
        let base = statement_key(row);
        let n = self.seen.entry(base.clone()).and_modify(|n| *n += 1).or_insert(1);
        format!("{base}:{n}")
    }

    fn prepare(&self, row: &StatementRow) -> Prepared {
        let parts = crate::narration::parse(&row.narration);
        let credit = row.direction == Direction::Credit;
        // Category order: the structure of the narration (own accounts, SIPs, salary, fees...), then the
        // user's rules and the keywords, both read from the payee alone. The raw narration is never
        // matched, so app tags such as "PAID VIA CRED" cannot pick a category.
        let category = match statement_structure(&row.narration, credit, self.card, &parts, self.owner) {
            Some(c) => c.to_string(),
            None => categorize(parts.payee.as_deref(), row.direction, false, self.rules),
        };
        Prepared {
            on: row.date.to_string(),
            credit,
            category,
            merchant: parts.payee.clone().or_else(|| crate::statement::merchant_of(&row.narration)),
            raw: row.narration.chars().take(500).collect(),
            rrn: parts.rrn,
        }
    }

    /// The row this statement row confirms, if any.
    /// (1) An existing row with the same reference (or the same narration, already confirmed by an
    /// earlier statement), same direction and amount, date within a day. References are not unique,
    /// so the amount and date must agree too.
    /// (2) Otherwise a bank-alert row on the same card or account, within a day.
    async fn find_existing(&self, conn: &mut sqlx::SqliteConnection, row: &StatementRow, rrn: Option<&str>) -> Result<Option<(String, i64, String)>, AppError> {
        let unclaimed = |found: Vec<(String, i64, String)>| found.into_iter().find(|(id, _, _)| !self.claimed.contains(id));
        let by_reference = unclaimed(find_by_reference(conn, self.user_id, row, rrn).await?);
        if by_reference.is_some() {
            return Ok(by_reference);
        }
        let alerts = find_alert_to_confirm(conn, self.user_id, row, self.target.account_id.as_deref(), self.target.card_id.as_deref()).await?;
        Ok(unclaimed(alerts))
    }

    async fn row(&mut self, conn: &mut sqlx::SqliteConnection, row: &StatementRow) -> Result<(), AppError> {
        let on = row.date.to_string();
        self.res.first_date = Some(self.res.first_date.take().map_or(on.clone(), |d| d.min(on.clone())));
        self.res.last_date = Some(self.res.last_date.take().map_or(on.clone(), |d| d.max(on)));

        let dedupe_key = self.dedupe_key(row);
        let exists: Option<String> = sqlx::query_scalar("SELECT txn_id FROM transactions WHERE user_id = ? AND dedupe_key = ?")
            .bind(self.user_id)
            .bind(&dedupe_key)
            .fetch_optional(&mut *conn)
            .await?;
        if let Some(txn_id) = exists {
            self.res.duplicates += 1;
            return self.link(conn, &txn_id).await;
        }

        let prepared = self.prepare(row);
        match self.find_existing(conn, row, prepared.rrn.as_deref()).await? {
            Some((txn_id, on_statement, existing)) => self.confirm(conn, &prepared, txn_id, on_statement == 1, &existing).await,
            None => self.insert(conn, row, &prepared, &dedupe_key).await,
        }
    }

    /// A row this import found already in joy but on no account or card (an earlier import with no
    /// target, or an alert without the last digits) goes on this import's account or card. A row
    /// already on one is never moved.
    async fn link(&mut self, conn: &mut sqlx::SqliteConnection, txn_id: &str) -> Result<(), AppError> {
        if self.target.account_id.is_none() && self.target.card_id.is_none() {
            return Ok(());
        }
        let done = sqlx::query(
            "UPDATE transactions SET account_id = ?, card_id = ?, last4 = COALESCE(last4, ?)
             WHERE txn_id = ? AND user_id = ? AND account_id IS NULL AND card_id IS NULL",
        )
        .bind(&self.target.account_id)
        .bind(&self.target.card_id)
        .bind(&self.target.last4)
        .bind(txn_id)
        .bind(self.user_id)
        .execute(conn)
        .await?;
        self.res.linked += done.rows_affected() as usize;
        Ok(())
    }

    /// The statement is the source of truth for a row a bank alert recorded: its posting date,
    /// reference and narration replace the alert's, which is kept rather than doubled.
    async fn confirm(&mut self, conn: &mut sqlx::SqliteConnection, p: &Prepared, txn_id: String, on_statement: bool, existing: &str) -> Result<(), AppError> {
        self.claimed.insert(txn_id.clone());
        self.link(conn, &txn_id).await?;
        if on_statement {
            // Already confirmed by an earlier import of this statement.
            self.res.duplicates += 1;
            return Ok(());
        }
        // A card credit is never income: the statement's category replaces an alert's, unless the user locked it.
        let forced = self.card && p.credit;
        let kept = if forced || existing == "uncategorized" { p.category.as_str() } else { existing };
        sqlx::query(
            // The statement is the source of truth: its posting date replaces the alert's (±1 day).
            "UPDATE transactions SET on_statement = 1, occurred_on = ?, rrn = COALESCE(?, rrn), merchant = COALESCE(?, merchant), raw_text = ?,
                 category = CASE WHEN category_locked = 1 THEN category
                                 WHEN ? = 0 AND category <> 'uncategorized' THEN category
                                 ELSE ? END
             WHERE txn_id = ? AND user_id = ?",
        )
        .bind(&p.on)
        .bind(&p.rrn)
        .bind(&p.merchant)
        .bind(&p.raw)
        .bind(forced as i64)
        .bind(&p.category)
        .bind(&txn_id)
        .bind(self.user_id)
        .execute(conn)
        .await?;
        self.res.upgraded += 1;
        self.res.matched_alerts += 1;
        self.tally(kept);
        Ok(())
    }

    async fn insert(&mut self, conn: &mut sqlx::SqliteConnection, row: &StatementRow, p: &Prepared, dedupe_key: &str) -> Result<(), AppError> {
        self.tally(&p.category);
        let txn_id = nanoid!();
        sqlx::query(
            "INSERT INTO transactions
                (txn_id, user_id, occurred_on, direction, amount_paise, merchant, category, card_id, account_id, last4, source, raw_text, dedupe_key, rrn, on_statement)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'paste', ?, ?, ?, 1)",
        )
        .bind(&txn_id)
        .bind(self.user_id)
        .bind(&p.on)
        .bind(row.direction.as_str())
        .bind(row.amount_paise)
        .bind(&p.merchant)
        .bind(&p.category)
        .bind(&self.target.card_id)
        .bind(&self.target.account_id)
        .bind(&self.target.last4)
        .bind(&p.raw)
        .bind(dedupe_key)
        .bind(&p.rrn)
        .execute(conn)
        .await?;
        self.claimed.insert(txn_id);
        self.res.added += 1;
        Ok(())
    }
}

/// The statement's own balance, from its newest line, becomes the account balance unless a newer one exists.
async fn apply_closing_balance(
    conn: &mut sqlx::SqliteConnection,
    user_id: &str,
    account_id: &str,
    parsed: &crate::statement::Parsed,
    chain: (bool, Option<i64>),
    opening: Option<i64>,
    res: &mut StatementResult,
) -> Result<(), AppError> {
    let newest = parsed.rows.iter().enumerate().filter(|(_, r)| r.balance_paise.is_some()).max_by_key(|(i, r)| (r.date, *i));
    let Some((_, r)) = newest else { return Ok(()) };
    // When every balance follows from the one before, the chain's closing balance is the real last
    // one. Taking "the last row of the latest day" in file order is wrong for a newest-first export
    // with several lines on its last day (it picks the day's first).
    let bal = match chain {
        (true, Some(c)) => c,
        _ => r.balance_paise.expect("filtered"),
    };
    let on = r.date.to_string();
    // Statement balances are end-of-day figures. Kept whether or not the account itself moves.
    // Only a chain that holds makes them exact, as in the migration's backfill.
    if chain.0 {
        crate::handlers::account::record_snapshot(&mut *conn, user_id, account_id, &on, bal, "statement").await?;
        let day_before = parsed.rows.iter().map(|r| r.date).min().and_then(|d| d.previous_day());
        if let (Some(open), Some(day_before)) = (opening, day_before) {
            crate::handlers::account::record_snapshot(&mut *conn, user_id, account_id, &day_before.to_string(), open, "statement").await?;
        }
    }
    let done = sqlx::query(
        "UPDATE accounts SET balance_paise = ?, balance_set_on = ?, updated_at = CURRENT_TIMESTAMP
         WHERE account_id = ? AND user_id = ? AND (balance_set_on IS NULL OR balance_set_on <= ?)",
    )
    .bind(bal)
    .bind(&on)
    .bind(account_id)
    .bind(user_id)
    .bind(&on)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    if done == 1 {
        let name: String = sqlx::query_scalar("SELECT name FROM accounts WHERE account_id = ?").bind(account_id).fetch_one(&mut *conn).await?;
        res.balance_updated = Some(BalanceUpdated { account_name: name, balance_paise: bal });
    } else if let Some((name, kept, set_on)) = sqlx::query_as::<_, (String, i64, String)>(
        "SELECT name, balance_paise, balance_set_on FROM accounts
         WHERE account_id = ? AND user_id = ? AND balance_set_on IS NOT NULL",
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?
    {
        // Same number: nothing to point out.
        if kept != bal {
            res.balance_kept = Some(BalanceKept { account_name: name, balance_paise: kept, set_on, statement_paise: bal, statement_on: on });
        }
    }
    Ok(())
}

/// Bank-alert rows on this target, inside the statement's dates, that the statement never confirmed.
async fn count_unconfirmed(conn: &mut sqlx::SqliteConnection, user_id: &str, target: &Target, first: &str, last: &str) -> Result<usize, AppError> {
    let missing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM transactions
         WHERE user_id = ?1 AND on_statement = 0 AND dedupe_key NOT LIKE 'stmt:%' AND source != 'manual'
           AND occurred_on BETWEEN ?2 AND ?3
           AND ((?4 IS NOT NULL AND account_id = ?4) OR (?5 IS NOT NULL AND card_id = ?5))",
    )
    .bind(user_id)
    .bind(first)
    .bind(last)
    .bind(target.account_id.as_deref())
    .bind(target.card_id.as_deref())
    .fetch_one(conn)
    .await?;
    Ok(missing as usize)
}

/// Stores a parsed statement for `user_id`. Nothing is stored twice: a row already imported is a
/// duplicate; a row a bank alert recorded is confirmed (kept, with the statement's reference and
/// narration) rather than doubled; every other row is added.
pub async fn import_statement(db: &sqlx::SqlitePool, user_id: &str, input: StatementInput) -> Result<StatementResult, AppError> {
    let target = Target::resolve(db, user_id, &input).await?;
    let card = target.card_id.is_some();
    let parsed = parse_statement(&input, card)?;

    let rules = load_rules(db, user_id).await?;
    let holder_names = load_holder_names(db, user_id).await?;
    let owner = crate::narration::Owner { holder_names: &holder_names };
    let (chain_ok, opening_paise, closing_paise) = balance_chain(&parsed.rows);

    let mut import = Importer {
        user_id,
        target: &target,
        card,
        rules: &rules,
        owner: &owner,
        seen: BTreeMap::new(),
        claimed: HashSet::new(),
        res: StatementResult {
            total: parsed.rows.len(),
            added: 0,
            duplicates: 0,
            matched_alerts: 0,
            linked: 0,
            upgraded: 0,
            skipped: parsed.skipped,
            uncategorized: 0,
            categorized: 0,
            first_date: None,
            last_date: None,
            balance_updated: None,
            balance_kept: None,
            older_recategorized: 0,
            chain_ok,
            opening_paise,
            closing_paise,
            not_on_statement: 0,
            holder_name_suggestion: None,
            card_bill: None,
        },
    };
    let mut tx = db.begin().await?;
    for row in &parsed.rows {
        import.row(&mut tx, row).await?;
    }
    let mut res = import.res;

    if let (Some(card_id), Some(summary)) = (&target.card_id, input.card_summary.as_ref().and_then(CardSummaryText::read)) {
        res.card_bill = Some(apply_card_summary(&mut tx, user_id, card_id, &summary, today_local()).await?);
    }
    if let Some(account_id) = &target.account_id {
        apply_closing_balance(&mut tx, user_id, account_id, &parsed, (chain_ok, closing_paise), opening_paise, &mut res).await?;
    }
    if let (Some(first), Some(last)) = (&res.first_date, &res.last_date) {
        res.not_on_statement = count_unconfirmed(&mut tx, user_id, &target, first, last).await?;
    }
    if holder_names.is_empty() {
        res.holder_name_suggestion = holder_suggestion(&parsed.rows);
    }

    let import_id = nanoid!();
    sqlx::query(
        "INSERT INTO statement_imports (import_id, user_id, account_id, card_id, first_date, last_date, rows, opening_paise, closing_paise, chain_ok)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&import_id)
    .bind(user_id)
    .bind(&target.account_id)
    .bind(&target.card_id)
    .bind(&res.first_date)
    .bind(&res.last_date)
    .bind(parsed.rows.len() as i64)
    .bind(opening_paise)
    .bind(closing_paise)
    .bind(chain_ok as i64)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    // Only rows that were really new earn anything: a re-upload of the same file earns nothing.
    crate::handlers::activity::note(db, user_id, crate::planner::activity::Kind::Statement, &import_id, res.added).await;
    Ok(res)
}

/// `POST /api/ingest/statement`: the JSON form of the import (the body carries the CSV text).
pub async fn statement(
    State(state): State<AppState>,
    Extension((user_id, _)): User,
    Json(p): Json<StatementPayload>,
) -> Result<Json<StatementResult>, AppError> {
    let input = StatementInput { csv: p.csv, account_id: p.account_id, card_id: p.card_id, mapping: None, card_summary: p.card_summary };
    Ok(Json(import_and_catch_up(&state.db, &user_id, input).await?))
}

pub(super) fn multipart_error(e: MultipartError) -> AppError {
    AppError(e.status(), e.body_text())
}

/// The fields of a statement upload or preview, read in one pass.
#[derive(Default)]
pub(super) struct StatementForm {
    file: Option<Bytes>,
    account_id: Option<String>,
    card_id: Option<String>,
    /// The mapping as JSON (`statement::Mapping`), when the user chose one.
    mapping: Option<String>,
    save_profile: bool,
    label: String,
    /// A card statement's summary box as JSON (`CardSummaryText`).
    card_summary: Option<String>,
}

pub(super) async fn read_statement_form(mut multipart: Multipart) -> Result<StatementForm, AppError> {
    let mut form = StatementForm::default();
    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        match field.name().unwrap_or_default() {
            "file" => form.file = Some(field.bytes().await.map_err(multipart_error)?),
            "account_id" => form.account_id = Some(field.text().await.map_err(multipart_error)?),
            "card_id" => form.card_id = Some(field.text().await.map_err(multipart_error)?),
            "mapping" => form.mapping = Some(field.text().await.map_err(multipart_error)?),
            "save_profile" => form.save_profile = matches!(field.text().await.map_err(multipart_error)?.trim(), "true" | "1" | "on"),
            "label" => form.label = field.text().await.map_err(multipart_error)?,
            "card_summary" => form.card_summary = Some(field.text().await.map_err(multipart_error)?),
            _ => {}
        }
    }
    Ok(form)
}

/// The file's text, or 400 when there is no file or it is an Excel export.
pub(super) fn statement_text(file: Option<Bytes>) -> Result<String, AppError> {
    let bytes = file.ok_or_else(|| bad_request("Choose a statement file to upload."))?;
    if bytes.starts_with(&OLE2_MAGIC) {
        return Err(bad_request(EXCEL_MESSAGE));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub(super) fn is_card(card_id: Option<&str>) -> bool {
    card_id.is_some_and(|s| !s.trim().is_empty())
}

/// The saved layout whose header this file has, with the header row it has here. A saved layout
/// that reads no rows from this file does not count.
pub(super) async fn saved_layout(db: &sqlx::SqlitePool, user_id: &str, csv: &str) -> Result<Option<Mapping>, AppError> {
    let candidates = crate::statement::header_signatures(csv);
    if candidates.is_empty() {
        return Ok(None);
    }
    let saved: Vec<(String, String)> = sqlx::query_as("SELECT signature, mapping_json FROM statement_profiles WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(db)
        .await?;
    for (row, sig) in candidates {
        let Some((_, json)) = saved.iter().find(|(s, _)| *s == sig) else { continue };
        let Ok(mut mapping) = serde_json::from_str::<Mapping>(json) else { continue };
        mapping.header_row = row;
        if !crate::statement::parse_with(csv, &mapping).rows.is_empty() {
            return Ok(Some(mapping));
        }
    }
    Ok(None)
}

/// Keeps a layout under its header signature, replacing any saved one for it.
pub(super) async fn save_layout(db: &sqlx::SqlitePool, user_id: &str, signature: &str, mapping: &Mapping, label: &str) -> Result<(), AppError> {
    let json = serde_json::to_string(mapping).map_err(|e| AppError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let label: String = label.trim().chars().take(60).collect();
    let label = if label.is_empty() { "Statement layout".to_string() } else { label };
    sqlx::query(
        "INSERT INTO statement_profiles (user_id, signature, mapping_json, label) VALUES (?, ?, ?, ?)
         ON CONFLICT (user_id, signature) DO UPDATE SET mapping_json = excluded.mapping_json, label = excluded.label",
    )
    .bind(user_id)
    .bind(signature)
    .bind(json)
    .bind(label)
    .execute(db)
    .await?;
    Ok(())
}

/// `POST /api/ingest/statement/upload`: multipart with `file`, optional `account_id` or `card_id`,
/// and optionally a `mapping` (JSON) with `save_profile` and `label`. Without a mapping, a layout
/// saved for this header is used, else auto-detection. A file neither reads is a 422 with
/// `{ error: "needs_mapping", preview }`. The file is read into memory, imported and dropped. It is
/// never written to disk.
pub async fn statement_upload(State(state): State<AppState>, Extension((user_id, _)): User, multipart: Multipart) -> Result<Response, AppError> {
    let form = read_statement_form(multipart).await?;
    let card = is_card(form.card_id.as_deref());
    let csv = statement_text(form.file)?;

    let explicit = match form.mapping.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(raw) => Some(serde_json::from_str::<Mapping>(raw).map_err(|_| bad_request("The column mapping isn't valid."))?),
        None => None,
    };
    let mapping = match &explicit {
        Some(_) => explicit.clone(),
        None => saved_layout(&state.db, &user_id, &csv).await?,
    };
    if mapping.is_none() && crate::statement::parse_for(&csv, card).is_none() {
        let preview = crate::statement::preview_for(&csv, card);
        let body = serde_json::json!({ "error": "needs_mapping", "preview": preview });
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response());
    }

    // Only a mapping the user sent is kept as a layout; the signature comes from this file's header.
    let to_save = explicit.filter(|_| form.save_profile).map(|m| (crate::statement::signature(&csv, m.header_row), m));
    // A summary that can't be read is left out rather than failing the import.
    let card_summary = form.card_summary.as_deref().and_then(|raw| serde_json::from_str::<CardSummaryText>(raw).ok());
    let input = StatementInput { csv, account_id: form.account_id, card_id: form.card_id, mapping, card_summary };
    let result = import_and_catch_up(&state.db, &user_id, input).await?;
    if let Some((signature, mapping)) = to_save {
        save_layout(&state.db, &user_id, &signature, &mapping, &form.label).await?;
    }
    Ok(Json(result).into_response())
}

#[derive(Serialize)]
pub struct StatementPreview {
    pub preview: crate::statement::Preview,
    /// The header's signature: what a saved layout is found by.
    pub signature: String,
    /// Whether a layout is already saved for this file's header.
    pub known_profile: bool,
}

/// `POST /api/ingest/statement/preview`: the file's header and first rows, for mapping its columns.
/// Nothing is stored and the file is not imported.
pub async fn statement_preview(State(state): State<AppState>, Extension((user_id, _)): User, multipart: Multipart) -> Result<Json<StatementPreview>, AppError> {
    let form = read_statement_form(multipart).await?;
    let card = is_card(form.card_id.as_deref());
    let csv = statement_text(form.file)?;
    let preview = crate::statement::preview_for(&csv, card);
    let signature = crate::statement::signature(&csv, preview.header_row);
    let known_profile = saved_layout(&state.db, &user_id, &csv).await?.is_some();
    Ok(Json(StatementPreview { preview, signature, known_profile }))
}

/// Phrases that mark a credit on a card as a payment in. Anything else is a refund, reversal or cashback.
pub(super) const CARD_PAYMENT_PHRASES: &[&str] = &["PAYMENT", "THANK YOU", "BBPS", "NEFT", "IMPS", "UPI"];
/// Bill payments to a card from a bank account. A bank's name alone does not make one.
pub(super) const CARD_BILL_PHRASES: &[&str] = &["CRED CLUB", "CREDIT CARD", "CC PAYMENT"];
pub(super) const CARD_ISSUERS: &[&str] = &["HDFC", "ICICI", "SBI", "AXIS", "KOTAK", "CITI", "AMEX", "HSBC", "RBL", "INDUSIND", "IDFC", "YES BANK", "STANDARD CHARTERED"];

/// Whether `phrase` appears in `upper` as whole words: "UPI" is not part of "SUPIN".
pub(super) fn has_phrase(upper: &str, phrase: &str) -> bool {
    upper.match_indices(phrase).any(|(i, _)| {
        let before = upper[..i].chars().next_back().is_some_and(|c| c.is_ascii_alphanumeric());
        let after = upper[i + phrase.len()..].chars().next().is_some_and(|c| c.is_ascii_alphanumeric());
        !before && !after
    })
}

/// A credit on a card is never income: a payment in is a transfer (a bill paid), anything else a refund.
pub(super) fn card_credit_category(narration: &str) -> &'static str {
    let upper = narration.to_ascii_uppercase();
    if CARD_PAYMENT_PHRASES.iter().any(|p| has_phrase(&upper, p)) {
        "transfer"
    } else {
        "refund"
    }
}

/// A debit from a bank account that pays a credit card's bill. Not spending.
pub(super) fn is_card_bill_payment(narration: &str) -> bool {
    let upper = narration.to_ascii_uppercase();
    CARD_BILL_PHRASES.iter().any(|p| has_phrase(&upper, p))
        || (has_phrase(&upper, "BILLDESK") && has_phrase(&upper, "CARD"))
        || (has_phrase(&upper, "CARDS") && CARD_ISSUERS.iter().any(|b| has_phrase(&upper, b)))
}

/// The category a statement row gets from its structure: a card's credits and a bank's card bill
/// payments first, then the narration rules (own accounts, salary, fees...). `None` leaves the row
/// to the payee rules. Shared by the import and by recategorising, so both agree.
pub(super) fn statement_structure(
    narration: &str,
    credit: bool,
    on_card: bool,
    parts: &crate::narration::Narration,
    owner: &crate::narration::Owner,
) -> Option<&'static str> {
    if on_card && credit {
        return Some(card_credit_category(narration));
    }
    if !on_card && !credit && is_card_bill_payment(narration) {
        return Some("transfer");
    }
    crate::narration::structural_category(parts, credit, owner)
}

