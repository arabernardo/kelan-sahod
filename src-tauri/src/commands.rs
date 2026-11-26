//! Thin Tauri command layer. The only place that knows about the clock and the disk.
//! It gets today's date in Asia/Manila ONCE, then hands it to the pure `payday` module.

use crate::holidays::{self, HolidayStatus, Holidays, ListSource};
use crate::layout::{self, LayoutMode};
use crate::payday::{payday_info, PaydayInfo, Schedule};
use crate::remote;
use chrono::{Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Asia::Manila;
use std::path::PathBuf;
use tauri::{AppHandle, LogicalSize, Manager, WebviewWindow};

/// A list confirmed against the remote file within this many hours counts as "verified".
const VERIFIED_HOURS: i64 = 24;

fn today_in_manila() -> NaiveDate {
    Utc::now().with_timezone(&Manila).date_naive()
}

/// DEBUG BUILDS ONLY (`tauri dev`): set the KELAN_SAHOD_TODAY environment variable to a date
/// like 2026-11-27 to pretend that is today, e.g. to see the payday screen.
/// This code is compiled out of release builds, so installed apps always use the real date.
#[cfg(debug_assertions)]
fn debug_today_override() -> Option<NaiveDate> {
    use crate::payday::parse_date_override;
    static ANNOUNCE: std::sync::Once = std::sync::Once::new();

    let value = std::env::var("KELAN_SAHOD_TODAY").ok()?;
    let date = parse_date_override(&value);
    ANNOUNCE.call_once(|| match date {
        Some(d) => eprintln!("[dev] KELAN_SAHOD_TODAY is set: pretending today is {d}"),
        None => eprintln!("[dev] Ignoring KELAN_SAHOD_TODAY={value:?}: use the format YYYY-MM-DD"),
    });
    date
}

#[cfg(not(debug_assertions))]
fn debug_today_override() -> Option<NaiveDate> {
    None
}

/// Today's date: the real one in Asia/Manila (or the debug test date).
fn today() -> NaiveDate {
    debug_today_override().unwrap_or_else(today_in_manila)
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

    let today = today();
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

/// Resizes the main window for a layout mode (sizes come from the pure `layout` module).
pub fn apply_layout(window: &WebviewWindow, mode: LayoutMode) -> tauri::Result<()> {
    let spec = layout::window_spec(mode);

    // Lift the old limits first so the new size is never clamped by them.
    window.set_min_size(None::<LogicalSize<f64>>)?;
    window.set_max_size(None::<LogicalSize<f64>>)?;
    window.set_size(LogicalSize::new(spec.width, spec.height))?;
    window.set_min_size(Some(LogicalSize::new(spec.min_width, spec.min_height)))?;
    window.set_max_size(spec.max.map(|(w, h)| LogicalSize::new(w, h)))?;
    window.center()
}

/// Called by the frontend when the user picks Compact or Full in Settings
/// (and once at startup, as a safety net).
#[tauri::command]
pub fn set_layout_mode(app: AppHandle, mode: LayoutMode) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;
    apply_layout(&window, mode).map_err(|e| e.to_string())
}
