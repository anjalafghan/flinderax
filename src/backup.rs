//! A daily copy of the database next to it (`backups/joy-YYYY-MM-DD.db`), keeping the last
//! seven. There's no scheduler (the machine sleeps when idle), so the first request of each day
//! makes the copy. `VACUUM INTO` writes a consistent snapshot while the app keeps running.

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use time::Date;

const KEEP: usize = 7;

/// The day the last copy was made in this process, so the check is free after the first call.
static LAST: Mutex<Option<Date>> = Mutex::new(None);

/// `backups/` next to the database file named in `DATABASE_URL`; `None` for in-memory databases.
pub fn backup_dir() -> Option<PathBuf> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let file = url.trim_start_matches("sqlite:").trim_start_matches("//").split('?').next()?.to_string();
    if file.is_empty() || file.contains(":memory:") {
        return None;
    }
    let parent = Path::new(&file).parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    Some(parent.join("backups"))
}

/// Makes today's copy if it doesn't exist yet. Failures are logged, never returned: a backup
/// problem must not break the page that triggered it.
pub async fn maybe_backup(db: &sqlx::SqlitePool, today: Date) {
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if *last == Some(today) {
            return;
        }
        *last = Some(today);
    }
    let Some(dir) = backup_dir() else { return };
    if let Err(e) = run(db, &dir, today).await {
        tracing::error!("daily backup failed: {e}");
    }
}

async fn run(db: &sqlx::SqlitePool, dir: &Path, today: Date) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let target = dir.join(format!("joy-{today}.db"));
    if !target.exists() {
        let path = target.to_string_lossy().replace('\'', "''");
        sqlx::query(&format!("VACUUM INTO '{path}'")).execute(db).await.map_err(|e| e.to_string())?;
    }
    prune(dir).map_err(|e| e.to_string())
}

/// Deletes all but the newest `KEEP` copies (names sort by date).
fn prune(dir: &Path) -> std::io::Result<()> {
    let mut copies: Vec<PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("joy-") && n.ends_with(".db")))
        .collect();
    copies.sort();
    let extra = copies.len().saturating_sub(KEEP);
    for old in &copies[..extra] {
        std::fs::remove_file(old)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_newest_seven() {
        let dir = std::env::temp_dir().join(format!("joy-backup-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for d in 1..=9 {
            std::fs::write(dir.join(format!("joy-2026-11-0{d}.db")), b"x").unwrap();
        }
        std::fs::write(dir.join("other.txt"), b"x").unwrap();
        prune(&dir).unwrap();
        let mut left: Vec<String> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        assert_eq!(left.len(), 8, "7 copies + the unrelated file");
        assert_eq!(left[0], "joy-2026-11-03.db");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn vacuum_into_makes_a_copy() {
        let dir = std::env::temp_dir().join(format!("joy-backup-run-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("live.db");
        let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(&file).create_if_missing(true);
        let db = sqlx::SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query("CREATE TABLE t (x INTEGER)").execute(&db).await.unwrap();
        run(&db, &dir.join("backups"), time::macros::date!(2026 - 11 - 08)).await.unwrap();
        assert!(dir.join("backups/joy-2026-11-08.db").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
