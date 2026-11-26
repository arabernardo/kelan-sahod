//! Optional holiday sync: downloads `holidays.json` from GitHub and caches it on disk.
//!
//! This is the ONLY module that touches the network. The app works fully offline:
//! if anything fails here (no internet, bad JSON, wrong schema, stale file), the cached
//! copy or the list embedded in the binary is used instead and the user sees nothing.
//!
//! Only the holiday file is downloaded. No user data is ever sent.

use crate::holidays::{self, MovableFile};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

/// Raw URL of the SAME file that is embedded in the app (`src-tauri/data/holidays.json`).
/// Edit that file on GitHub and bump its "updated" date; installed apps pick it up.
/// TODO: replace OWNER/REPO (and the branch, if it is not `main`) with your repository.
pub const HOLIDAYS_URL: &str =
    "https://raw.githubusercontent.com/OWNER/REPO/main/src-tauri/data/holidays.json";

const PLACEHOLDER: &str = "OWNER/REPO";
const CACHE_FILE: &str = "holidays-cache.json";
const MAX_BYTES: u64 = 256 * 1024;
const TIMEOUT: Duration = Duration::from_secs(8);

/// What we store on disk: the raw downloaded text plus when we got it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheEntry {
    fetched_at: DateTime<Utc>,
    raw: String,
}

/// A validated cached list and the moment it was last confirmed against the remote file.
pub struct Cached {
    pub file: MovableFile,
    pub fetched_at: DateTime<Utc>,
}

/// `Ok(false)` = nothing to do (URL not configured yet). `Err` = refuse to fetch.
pub fn check_url(url: &str) -> Result<bool, String> {
    if url.contains(PLACEHOLDER) {
        return Ok(false);
    }
    if !url.starts_with("https://") {
        return Err("holiday URL must use https".into());
    }
    Ok(true)
}

/// Downloads the text at `url`: short timeout, hard size cap, no redirects to other schemes.
pub fn fetch_text(url: &str) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(TIMEOUT).build();
    let response = agent.get(url).call().map_err(|e| e.to_string())?;

    let mut body = String::new();
    response
        .into_reader()
        .take(MAX_BYTES + 1)
        .read_to_string(&mut body)
        .map_err(|e| e.to_string())?;
    if body.len() as u64 > MAX_BYTES {
        return Err("holiday file is too large".into());
    }
    Ok(body)
}

/// Reads the cache. Anything wrong with it (missing, corrupt, invalid) means "no cache".
pub fn load_cache(dir: &Path) -> Option<Cached> {
    let text = fs::read_to_string(dir.join(CACHE_FILE)).ok()?;
    let entry: CacheEntry = serde_json::from_str(&text).ok()?;
    let file = holidays::parse_movable(&entry.raw).ok()?;
    holidays::validate_remote(&file).ok()?;
    Some(Cached {
        file,
        fetched_at: entry.fetched_at,
    })
}

fn save_cache(dir: &Path, raw: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let entry = CacheEntry {
        fetched_at: Utc::now(),
        raw: raw.to_string(),
    };
    let json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;

    // Write to a temp file first so a crash can never leave a half-written cache.
    let tmp = dir.join(format!("{CACHE_FILE}.tmp"));
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, dir.join(CACHE_FILE)).map_err(|e| e.to_string())
}

/// Downloads `url`, validates it, and caches it if it is not older than what we already have.
/// Returns `Ok(true)` when the list was confirmed (and cached), `Ok(false)` when ignored.
pub fn sync(dir: &Path, url: &str, embedded: &MovableFile) -> Result<bool, String> {
    let raw = fetch_text(url)?;
    let file = holidays::parse_movable(&raw).map_err(|e| format!("invalid holidays.json: {e}"))?;
    holidays::validate_remote(&file)?;

    // Never go backwards: ignore a copy older than the embedded list or the cached list.
    let cached_updated = load_cache(dir).and_then(|c| c.file.updated);
    let baseline = cached_updated.max(embedded.updated);
    if baseline > file.updated {
        return Ok(false);
    }

    save_cache(dir, &raw)?;
    Ok(true)
}

/// Sync using the configured GitHub URL.
pub fn sync_configured(dir: &Path, embedded: &MovableFile) -> Result<bool, String> {
    if !check_url(HOLIDAYS_URL)? {
        return Ok(false);
    }
    sync(dir, HOLIDAYS_URL, embedded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_dir() -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("kelan-sahod-test-{}-{}", std::process::id(), n));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    /// One-shot local HTTP server. Serves `body` with `status` to the first request.
    fn serve_once(status: &str, body: &str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 2048];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(response.as_bytes());
            }
        });
        format!("http://127.0.0.1:{port}/holidays.json")
    }

    fn embedded(updated: &str) -> MovableFile {
        holidays::parse_movable(&format!(
            r#"{{ "updated": "{updated}", "holidays": [ {{ "date": "2026-04-03", "name": "Good Friday" }} ] }}"#
        ))
        .unwrap()
    }

    const NEWER: &str = r#"{ "updated": "2026-12-01", "holidays": [
        { "date": "2026-04-03", "name": "Good Friday" },
        { "date": "2026-12-24", "name": "Christmas Eve" } ] }"#;

    #[test]
    fn check_url_guards() {
        assert_eq!(check_url(HOLIDAYS_URL), Ok(false)); // placeholder not configured yet
        assert_eq!(check_url("https://example.com/h.json"), Ok(true));
        assert!(check_url("http://example.com/h.json").is_err());
    }

    #[test]
    fn downloads_validates_and_caches_a_newer_file() {
        let dir = temp_dir();
        let url = serve_once("200 OK", NEWER);
        assert_eq!(sync(&dir, &url, &embedded("2026-11-26")), Ok(true));

        let cached = load_cache(&dir).expect("cache should exist");
        assert_eq!(cached.file.holidays.len(), 2);
        assert_eq!(cached.file.updated, NaiveDate::from_ymd_opt(2026, 12, 1));
        assert!((Utc::now() - cached.fetched_at).num_seconds() < 60);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_equal_dated_file_counts_as_verified() {
        let dir = temp_dir();
        let url = serve_once("200 OK", NEWER);
        assert_eq!(sync(&dir, &url, &embedded("2026-12-01")), Ok(true));
        assert!(load_cache(&dir).is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ignores_a_file_older_than_the_embedded_list() {
        let dir = temp_dir();
        let url = serve_once("200 OK", NEWER);
        assert_eq!(sync(&dir, &url, &embedded("2027-01-15")), Ok(false));
        assert!(load_cache(&dir).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn never_replaces_a_newer_cache_with_an_older_download() {
        let dir = temp_dir();
        let newer_url = serve_once("200 OK", NEWER);
        assert_eq!(sync(&dir, &newer_url, &embedded("2026-11-26")), Ok(true));

        let older = r#"{ "updated": "2026-11-30", "holidays": [ { "date": "2026-04-03", "name": "Good Friday" } ] }"#;
        let older_url = serve_once("200 OK", older);
        assert_eq!(sync(&dir, &older_url, &embedded("2026-11-26")), Ok(false));
        assert_eq!(load_cache(&dir).unwrap().file.holidays.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_broken_or_invalid_files_and_writes_nothing() {
        for body in [
            "not json at all",
            r#"{ "updated": "2026-12-01", "holidays": [] }"#,
            r#"{ "holidays": [ { "date": "2026-12-24", "name": "No updated field" } ] }"#,
            r#"{ "updated": "2026-12-01", "holidays": [ { "date": "2026-13-45", "name": "Bad date" } ] }"#,
        ] {
            let dir = temp_dir();
            let url = serve_once("200 OK", body);
            assert!(sync(&dir, &url, &embedded("2026-11-26")).is_err(), "should reject: {body}");
            assert!(load_cache(&dir).is_none());
            let _ = fs::remove_dir_all(&dir);
        }
    }

    #[test]
    fn http_errors_and_dead_servers_are_errors_not_panics() {
        let dir = temp_dir();
        let url = serve_once("404 Not Found", "nope");
        assert!(sync(&dir, &url, &embedded("2026-11-26")).is_err());
        // Nothing listening on this port.
        assert!(sync(&dir, "http://127.0.0.1:9/holidays.json", &embedded("2026-11-26")).is_err());
        assert!(load_cache(&dir).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_responses_are_refused() {
        let dir = temp_dir();
        let big = format!(
            r#"{{ "updated": "2026-12-01", "pad": "{}", "holidays": [] }}"#,
            "x".repeat((MAX_BYTES as usize) + 10)
        );
        let url = serve_once("200 OK", &big);
        assert!(sync(&dir, &url, &embedded("2026-11-26")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_cache_is_treated_as_no_cache() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(CACHE_FILE), "{ this is not valid").unwrap();
        assert!(load_cache(&dir).is_none());
        let _ = fs::remove_dir_all(&dir);
    }
}
