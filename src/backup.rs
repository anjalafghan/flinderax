//! Backups. A daily copy of the database next to it (`backups/joy-YYYY-MM-DD.db`), keeping the last
//! seven. There's no scheduler (the machine sleeps when idle), so the first request of each day
//! makes the copy. `VACUUM INTO` writes a consistent snapshot while the app keeps running.
//!
//! Daily copies stay on the Fly volume. To get one off the volume, a scheduled job calls
//! `POST /api/internal/backup/run` (see `handlers::backup`), which takes a fresh snapshot and
//! streams it back; `fly ssh sftp` also works (see `docs/backups.md`).
//! `BACKUP_DIR` overrides the folder and `BACKUP_KEEP` the number of copies kept (default 7).

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use time::Date;

const DEFAULT_KEEP: usize = 7;

fn keep() -> usize {
    std::env::var("BACKUP_KEEP").ok().and_then(|v| v.parse().ok()).filter(|n| *n >= 1).unwrap_or(DEFAULT_KEEP)
}

/// The day the last copy was made in this process, so the check is free after the first call.
static LAST: Mutex<Option<Date>> = Mutex::new(None);

/// `backups/` next to the database file named in `DATABASE_URL`; `None` for in-memory databases.
pub fn backup_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var("BACKUP_DIR").ok().filter(|d| !d.trim().is_empty()) {
        return Some(PathBuf::from(dir));
    }
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

/// Makes the local copy and prunes the local folder. Returns the path of today's copy.
async fn run(db: &sqlx::SqlitePool, dir: &Path, today: Date) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let target = dir.join(format!("joy-{today}.db"));
    if !target.exists() {
        let path = target.to_string_lossy().replace('\'', "''");
        sqlx::query(&format!("VACUUM INTO '{path}'")).execute(db).await.map_err(|e| e.to_string())?;
    }
    prune(dir).map_err(|e| e.to_string())?;
    Ok(target)
}

/// Writes a consistent snapshot of the live database to `target` (replacing any file there).
/// `VACUUM INTO` runs while the app keeps serving. Used by the on-demand backup and the restore drill.
pub async fn snapshot_to(db: &sqlx::SqlitePool, target: &Path) -> Result<(), String> {
    if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if target.exists() {
        std::fs::remove_file(target).map_err(|e| e.to_string())?;
    }
    let path = target.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{path}'")).execute(db).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// A fresh snapshot as bytes, for the off-volume pull. The temporary file is removed afterwards
/// and never matches the `joy-*.db` pattern the daily pruning looks at.
pub async fn snapshot_bytes(db: &sqlx::SqlitePool) -> Result<Vec<u8>, String> {
    let dir = backup_dir().ok_or("no on-disk database")?;
    let tmp = dir.join(format!("pull-{}.tmp", std::process::id()));
    let made = snapshot_to(db, &tmp).await;
    let bytes = made.and_then(|_| std::fs::read(&tmp).map_err(|e| e.to_string()));
    let _ = std::fs::remove_file(&tmp);
    bytes
}

/// Deletes all but the newest `keep()` copies (names sort by date).
fn prune(dir: &Path) -> std::io::Result<()> {
    let mut copies: Vec<PathBuf> = std::fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("joy-") && n.ends_with(".db")))
        .collect();
    copies.sort();
    let extra = copies.len().saturating_sub(keep());
    for old in &copies[..extra] {
        std::fs::remove_file(old)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqliteConnectOptions;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("joy-backup-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

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
        let dir = temp("run");
        let file = dir.join("live.db");
        let opts = SqliteConnectOptions::new().filename(&file).create_if_missing(true);
        let db = sqlx::SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query("CREATE TABLE t (x INTEGER)").execute(&db).await.unwrap();
        run(&db, &dir.join("backups"), time::macros::date!(2026 - 11 - 08)).await.unwrap();
        assert!(dir.join("backups/joy-2026-11-08.db").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
