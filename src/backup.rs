//! Backups. A daily copy of the database next to it (`backups/joy-YYYY-MM-DD.db`), keeping the last
//! seven. There's no scheduler (the machine sleeps when idle), so the first request of each day
//! makes the copy. `VACUUM INTO` writes a consistent snapshot while the app keeps running.
//!
//! When `BACKUP_KEY` and S3 settings are present, that copy is also gzipped, encrypted and uploaded
//! off-site (see `docs/backups.md`). Without the key, only the local copy is made.

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Mutex, Once},
    time::Duration,
};

use base64::Engine as _;
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use rand::RngCore;
use reqwest::Url;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle, actions::ListObjectsV2};
use sqlx::{Connection, sqlite::SqliteConnectOptions};
use time::Date;

const KEEP: usize = 7;
/// Remote copies kept in the bucket (about a month of daily copies).
const REMOTE_KEEP: usize = 30;
/// Off-site file name: `joy-YYYY-MM-DD.db.gz.enc`.
const REMOTE_SUFFIX: &str = ".db.gz.enc";
/// Encrypted file header: 4-byte magic, then a 1-byte format version.
const MAGIC: &[u8; 4] = b"JOYB";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 5;
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
/// How long a presigned S3 URL stays valid. Each one is used right after it is signed.
const URL_TTL: Duration = Duration::from_secs(15 * 60);

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

/// Makes today's copy if it doesn't exist yet, then uploads it off-site in the background when
/// that is configured. Failures are logged, never returned: a backup problem must not break the
/// page that triggered it.
pub async fn maybe_backup(db: &sqlx::SqlitePool, today: Date) {
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if *last == Some(today) {
            return;
        }
        *last = Some(today);
    }
    let Some(dir) = backup_dir() else { return };
    let copy = match run(db, &dir, today).await {
        Ok(copy) => copy,
        Err(e) => {
            tracing::error!("daily backup failed: {e}");
            return;
        }
    };
    if let Some(cfg) = offsite_settings() {
        let name = format!("joy-{today}{REMOTE_SUFFIX}");
        // Detached: the request that triggered the backup doesn't wait for the upload.
        tokio::spawn(async move {
            match upload(&cfg, &copy, &name).await {
                Ok(bytes) => {
                    tracing::info!("off-site backup {name} uploaded ({bytes} bytes)");
                    if let Err(e) = prune_remote(&cfg).await {
                        tracing::error!("off-site backup pruning failed: {e}");
                    }
                }
                Err(e) => tracing::error!("off-site backup {name} failed: {e}"),
            }
        });
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

// ---------------------------------------------------------------------------------------------
// Encryption format
// ---------------------------------------------------------------------------------------------

/// Gzips and encrypts a database copy. Layout: `JOYB` | version (1) | nonce (24, random) |
/// XChaCha20-Poly1305 ciphertext + tag. The 5-byte header is authenticated as associated data,
/// so changing it fails decryption as well.
pub fn seal(key: &[u8; 32], plain: &[u8]) -> Result<Vec<u8>, String> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(plain).map_err(|e| e.to_string())?;
    let zipped = gz.finish().map_err(|e| e.to_string())?;

    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);
    let mut header = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(MAGIC);
    header.push(VERSION);

    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| "bad key length".to_string())?;
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), Payload { msg: &zipped, aad: &header })
        .map_err(|_| "encryption failed".to_string())?;

    let mut out = header;
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// The inverse of [`seal`]: checks magic and version, decrypts, then gunzips.
pub fn open(key: &[u8; 32], blob: &[u8]) -> Result<Vec<u8>, String> {
    if blob.len() < HEADER_LEN + NONCE_LEN + TAG_LEN {
        return Err("file is too short to be a Joy backup".into());
    }
    let (header, rest) = blob.split_at(HEADER_LEN);
    if header[..4] != MAGIC[..] {
        return Err("not a Joy backup (bad magic)".into());
    }
    if header[4] != VERSION {
        return Err(format!("unsupported backup version {} (this build reads version {VERSION})", header[4]));
    }
    let (nonce, ciphertext) = rest.split_at(NONCE_LEN);
    let cipher = XChaCha20Poly1305::new_from_slice(key).map_err(|_| "bad key length".to_string())?;
    let zipped = cipher
        .decrypt(XNonce::from_slice(nonce), Payload { msg: ciphertext, aad: header })
        .map_err(|_| "decryption failed: wrong BACKUP_KEY or a corrupted file".to_string())?;

    let mut plain = Vec::new();
    flate2::read::GzDecoder::new(&zipped[..]).read_to_end(&mut plain).map_err(|e| format!("gunzip failed: {e}"))?;
    Ok(plain)
}

/// Decodes a `BACKUP_KEY` value: standard base64 of exactly 32 bytes.
pub fn parse_key(b64: &str) -> Result<[u8; 32], String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(b64.trim())
        .map_err(|_| "BACKUP_KEY is not valid base64".to_string())?;
    raw.try_into().map_err(|_| "BACKUP_KEY must decode to exactly 32 bytes".to_string())
}

/// A fresh random key, base64 encoded, for `BACKUP_KEY`.
pub fn new_key_b64() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    base64::engine::general_purpose::STANDARD.encode(key)
}

// ---------------------------------------------------------------------------------------------
// Off-site (S3-compatible) upload
// ---------------------------------------------------------------------------------------------

/// Everything the off-site upload needs. Built only when the key and the S3 settings are all there.
struct OffsiteConfig {
    key: [u8; 32],
    bucket: Bucket,
    creds: Credentials,
}

/// The off-site settings from the environment, or `None` when they're missing or invalid. The
/// reason is logged once per process, so a machine without `BACKUP_KEY` logs it on the first day.
fn offsite_settings() -> Option<OffsiteConfig> {
    match offsite_config(|name| std::env::var(name).ok()) {
        Ok(cfg) => Some(cfg),
        Err(why) => {
            static WARNED: Once = Once::new();
            WARNED.call_once(|| tracing::warn!("off-site backups disabled: {why}"));
            None
        }
    }
}

/// Reads the settings through `get` so tests can supply them. Accepts the names Tigris sets on
/// Fly (`AWS_ENDPOINT_URL_S3`, `AWS_REGION`, `BUCKET_NAME`) as well as the `BACKUP_S3_*` ones.
fn offsite_config(get: impl Fn(&str) -> Option<String>) -> Result<OffsiteConfig, String> {
    let first = |names: &[&str]| names.iter().find_map(|n| get(n).filter(|v| !v.trim().is_empty()));

    let key_b64 = first(&["BACKUP_KEY"]).ok_or("BACKUP_KEY is not set (local copies only)")?;
    let key = parse_key(&key_b64)?;
    let endpoint = first(&["BACKUP_S3_ENDPOINT", "AWS_ENDPOINT_URL_S3", "AWS_ENDPOINT_URL"]);
    let bucket = first(&["BACKUP_S3_BUCKET", "BUCKET_NAME"]);
    let key_id = first(&["AWS_ACCESS_KEY_ID"]);
    let secret = first(&["AWS_SECRET_ACCESS_KEY"]);
    let (Some(endpoint), Some(bucket), Some(key_id), Some(secret)) = (endpoint, bucket, key_id, secret) else {
        return Err("S3 settings incomplete: need BACKUP_S3_ENDPOINT (or AWS_ENDPOINT_URL_S3), BACKUP_S3_BUCKET (or BUCKET_NAME), AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY".into());
    };
    let region = first(&["BACKUP_S3_REGION", "AWS_REGION"]).unwrap_or_else(|| "auto".into());
    let endpoint: Url = endpoint.parse().map_err(|_| "S3 endpoint is not a valid URL".to_string())?;
    let bucket = Bucket::new(endpoint, UrlStyle::Path, bucket, region).map_err(|e| format!("S3 settings invalid: {e}"))?;
    Ok(OffsiteConfig { key, bucket, creds: Credentials::new(key_id, secret) })
}

/// A presigned PUT for one object in the bucket.
fn put_url(cfg: &OffsiteConfig, name: &str) -> Url {
    cfg.bucket.put_object(Some(&cfg.creds), name).sign(URL_TTL)
}

fn http() -> Result<reqwest::Client, String> {
    reqwest::Client::builder().timeout(Duration::from_secs(120)).build().map_err(|e| e.without_url().to_string())
}

/// Encrypts the local copy and PUTs it. Returns the uploaded size in bytes.
async fn upload(cfg: &OffsiteConfig, copy: &Path, name: &str) -> Result<usize, String> {
    let plain = std::fs::read(copy).map_err(|e| e.to_string())?;
    let key = cfg.key;
    let blob = tokio::task::spawn_blocking(move || seal(&key, &plain)).await.map_err(|e| e.to_string())??;
    let len = blob.len();
    http()?
        .put(put_url(cfg, name))
        .body(blob)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.without_url().to_string())?;
    Ok(len)
}

/// Lists the remote copies and deletes all but the newest `REMOTE_KEEP`. Names sort by date.
async fn prune_remote(cfg: &OffsiteConfig) -> Result<(), String> {
    let client = http()?;
    let mut keys: Vec<String> = Vec::new();
    let mut token: Option<String> = None;
    loop {
        let mut list = cfg.bucket.list_objects_v2(Some(&cfg.creds));
        list.with_prefix("joy-");
        if let Some(t) = token.take() {
            list.with_continuation_token(t);
        }
        let body = client
            .get(list.sign(URL_TTL))
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.without_url().to_string())?
            .text()
            .await
            .map_err(|e| e.without_url().to_string())?;
        let page = ListObjectsV2::parse_response(&body).map_err(|e| e.to_string())?;
        keys.extend(page.contents.into_iter().map(|c| c.key).filter(|k| k.ends_with(REMOTE_SUFFIX)));
        match page.next_continuation_token {
            Some(next) => token = Some(next),
            None => break,
        }
    }
    keys.sort();
    let extra = keys.len().saturating_sub(REMOTE_KEEP);
    for key in &keys[..extra] {
        let url = cfg.bucket.delete_object(Some(&cfg.creds), key).sign(URL_TTL);
        client
            .delete(url)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| e.without_url().to_string())?;
    }
    if extra > 0 {
        tracing::info!("off-site backups: deleted {extra} old copies");
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// CLI: `joy gen-backup-key`, `joy restore-backup <in.enc> <out.db>`
// ---------------------------------------------------------------------------------------------

/// Runs a backup subcommand and returns the exit code. Needs no database and no server.
pub async fn cli(args: &[String]) -> i32 {
    match args.get(1).map(String::as_str) {
        Some("gen-backup-key") => {
            println!("{}", new_key_b64());
            0
        }
        Some("restore-backup") => restore_cli(args).await,
        _ => 2,
    }
}

async fn restore_cli(args: &[String]) -> i32 {
    let (Some(input), Some(output)) = (args.get(2), args.get(3)) else {
        eprintln!("usage: joy restore-backup <in.db.gz.enc> <out.db>");
        return 2;
    };
    let key = match std::env::var("BACKUP_KEY").map_err(|_| "BACKUP_KEY is not set".to_string()).and_then(|k| parse_key(&k)) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let blob = match std::fs::read(input) {
        Ok(blob) => blob,
        Err(e) => {
            eprintln!("cannot read {input}: {e}");
            return 1;
        }
    };
    let plain = match open(&key, &blob) {
        Ok(plain) => plain,
        Err(e) => {
            eprintln!("{e}");
            return 1;
        }
    };
    // create_new: a restore never overwrites an existing database.
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .and_then(|mut f| f.write_all(&plain));
    if let Err(e) = written {
        eprintln!("cannot write {output}: {e} (a restore never overwrites an existing file)");
        return 1;
    }
    match integrity(output).await {
        Ok(()) => {
            println!("ok");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

/// `PRAGMA integrity_check` on a file, read-only. Ok only when SQLite reports exactly `ok`.
async fn integrity(path: &str) -> Result<(), String> {
    let opts = SqliteConnectOptions::new().filename(path).read_only(true);
    let mut conn = sqlx::SqliteConnection::connect_with(&opts).await.map_err(|e| e.to_string())?;
    let rows = sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
        .fetch_all(&mut conn)
        .await
        .map_err(|e| e.to_string())?;
    let _ = conn.close().await;
    if rows == ["ok"] {
        Ok(())
    } else {
        Err(format!("integrity check failed: {}", rows.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7u8; 32];

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

    #[test]
    fn seal_then_open_round_trips() {
        let plain = b"SQLite format 3\0 pretend database contents ".repeat(200);
        let blob = seal(&KEY, &plain).unwrap();
        assert_eq!(&blob[..4], MAGIC);
        assert_eq!(blob[4], VERSION);
        assert!(!blob.windows(20).any(|w| w == &plain[..20]), "plaintext must not appear in the blob");
        assert_eq!(open(&KEY, &blob).unwrap(), plain);
    }

    #[test]
    fn wrong_key_fails() {
        let blob = seal(&KEY, b"data").unwrap();
        let err = open(&[8u8; 32], &blob).unwrap_err();
        assert!(err.contains("decryption failed"), "{err}");
    }

    #[test]
    fn tampered_bytes_fail() {
        let blob = seal(&KEY, b"data that must not be altered in transit").unwrap();
        // A nonce byte, a ciphertext byte, the last tag byte, and the version byte, one at a time.
        for i in [HEADER_LEN + 3, HEADER_LEN + NONCE_LEN + 1, blob.len() - 1, 4] {
            let mut bad = blob.clone();
            bad[i] ^= 0x01;
            assert!(open(&KEY, &bad).is_err(), "flipping byte {i} must fail");
        }
    }

    #[test]
    fn magic_and_version_are_checked() {
        let blob = seal(&KEY, b"data").unwrap();
        let mut bad_magic = blob.clone();
        bad_magic[0] = b'X';
        assert!(open(&KEY, &bad_magic).unwrap_err().contains("bad magic"));

        let mut bad_version = blob.clone();
        bad_version[4] = VERSION + 1;
        assert!(open(&KEY, &bad_version).unwrap_err().contains("unsupported backup version"));

        assert!(open(&KEY, &blob[..10]).unwrap_err().contains("too short"));
    }

    #[test]
    fn key_must_be_32_bytes_of_base64() {
        assert_eq!(parse_key(&new_key_b64()).unwrap().len(), 32);
        assert!(parse_key("not base64!!").is_err());
        assert!(parse_key(&base64::engine::general_purpose::STANDARD.encode([1u8; 16])).is_err());
    }

    fn env_of(pairs: &[(&str, String)]) -> impl Fn(&str) -> Option<String> + use<> {
        let map: std::collections::HashMap<String, String> = pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn offsite_needs_key_and_s3_settings() {
        let key = ("BACKUP_KEY", new_key_b64());
        let s3 = [
            ("AWS_ENDPOINT_URL_S3", "https://fly.storage.tigris.dev".to_string()),
            ("BUCKET_NAME", "joy-backups".to_string()),
            ("AWS_ACCESS_KEY_ID", "test-id".to_string()),
            ("AWS_SECRET_ACCESS_KEY", "test-secret".to_string()),
        ];

        let no_key = offsite_config(env_of(&s3));
        assert!(no_key.err().unwrap().contains("BACKUP_KEY is not set"));

        let no_s3 = offsite_config(env_of(std::slice::from_ref(&key)));
        assert!(no_s3.err().unwrap().contains("S3 settings incomplete"));

        let mut all = s3.to_vec();
        all.push((key.0, key.1.clone()));
        let cfg = offsite_config(env_of(&all)).unwrap();
        assert_eq!(cfg.bucket.region(), "auto", "Tigris uses region auto");
        assert_eq!(cfg.bucket.name(), "joy-backups");
    }

    #[test]
    fn presigned_put_names_bucket_and_key() {
        let cfg = OffsiteConfig {
            key: KEY,
            bucket: Bucket::new(Url::parse("https://s3.example.test").unwrap(), UrlStyle::Path, "joy-backups", "auto").unwrap(),
            creds: Credentials::new("test-id", "test-secret"),
        };
        let url = put_url(&cfg, "joy-2026-11-08.db.gz.enc").to_string();
        assert!(url.starts_with("https://s3.example.test/joy-backups/joy-2026-11-08.db.gz.enc?"), "{url}");
        assert!(url.contains("X-Amz-Signature="));
    }

    #[tokio::test]
    async fn vacuum_copy_survives_encrypt_and_integrity_check() {
        // A file-backed DB in a temp dir, like production. Not `sqlite::memory:`: in this test a
        // `VACUUM INTO` from an in-memory pool returned Ok but wrote no file, and the cause was not
        // investigated. In-memory databases get no backups anyway (`backup_dir` is None).
        let dir = temp("roundtrip");
        let opts = SqliteConnectOptions::new().filename(dir.join("live.db")).create_if_missing(true);
        let db = sqlx::SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query("CREATE TABLE t (x INTEGER)").execute(&db).await.unwrap();
        for i in 0..500 {
            sqlx::query("INSERT INTO t (x) VALUES (?)").bind(i).execute(&db).await.unwrap();
        }

        let copy = run(&db, &dir.join("backups"), time::macros::date!(2026 - 11 - 08)).await.unwrap();
        let plain = std::fs::read(&copy).unwrap();
        let blob = seal(&KEY, &plain).unwrap();
        let restored = dir.join("restored.db");
        std::fs::write(&restored, open(&KEY, &blob).unwrap()).unwrap();

        assert!(integrity(restored.to_str().unwrap()).await.is_ok());
        let mut conn = sqlx::SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&restored)).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM t").fetch_one(&mut conn).await.unwrap();
        assert_eq!(count, 500);
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
