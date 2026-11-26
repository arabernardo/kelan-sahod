//! Thin Tauri command layer. The only place that knows about the clock and the disk.
//! It gets today's date in Asia/Manila ONCE, then hands it to the pure `payday` module.

use crate::holidays::{self, HolidayStatus, Holidays, ListSource};
use crate::payday::{payday_info, PaydayInfo, Schedule};
use crate::remote;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Asia::Manila;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// A list confirmed against the remote file within this many hours counts as "verified".
const VERIFIED_HOURS: i64 = 24;

fn today_in_manila() -> NaiveDate {
    Utc::now().with_timezone(&Manila).date_naive()
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

fn embedded_list() -> Result<holidays::MovableFile, String> {
    holidays::parse_movable(holidays::EMBEDDED_MOVABLE_JSON)
        .map_err(|e| format!("embedded holidays.json is invalid: {e}"))
}

#[tauri::command]
pub fn get_payday_info(app: AppHandle, schedule: Schedule) -> Result<PaydayInfo, String> {
    schedule.validate()?;

    let today = today_in_manila();
    let year = today.year();

    // Downloaded list (if any, and valid) beats the embedded one only when it is not older.
    // A missing or broken cache is not an error: we just use the embedded list.
    let cached = data_dir(&app).ok().and_then(|dir| remote::load_cache(&dir));
    let fetched_at = cached.as_ref().map(|c| c.fetched_at);
    let (list, source) = holidays::resolve(embedded_list()?, cached.map(|c| c.file));

    let verified_recently = source == ListSource::Remote
        && fetched_at.is_some_and(|t| Utc::now() - t < Duration::hours(VERIFIED_HOURS));

    // A payday can move across a year boundary, so load a few years around today.
    let calendar = Holidays::build((year - 2)..=(year + 2), &list.holidays);
    let status = HolidayStatus {
        missing_for_year: holidays::movable_missing_for_year(&list.holidays, year),
        verified_recently,
        updated: list.updated,
        source,
    };

    Ok(payday_info(today, &schedule, &calendar, &status))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshOutcome {
    /// True when the holiday list was confirmed against GitHub (the UI should reload).
    pub synced: bool,
}

/// Downloads the latest holiday list in the background. Never fails the UI:
/// the frontend ignores errors, and offline use keeps working.
#[tauri::command]
pub async fn refresh_holidays(app: AppHandle) -> Result<RefreshOutcome, String> {
    let dir = data_dir(&app)?;
    let embedded = embedded_list()?;

    // The download is blocking, so keep it off the async runtime's main threads.
    let synced = tauri::async_runtime::spawn_blocking(move || remote::sync_configured(&dir, &embedded))
        .await
        .map_err(|e| e.to_string())??;

    Ok(RefreshOutcome { synced })
}
