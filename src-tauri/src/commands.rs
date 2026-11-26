//! Thin Tauri command layer. The only place that knows about the clock.
//! It gets today's date in Asia/Manila ONCE, then hands it to the pure `payday` module.

use crate::holidays::{self, Holidays};
use crate::payday::{payday_info, PaydayInfo, Schedule};
use chrono::{Datelike, NaiveDate, Utc};
use chrono_tz::Asia::Manila;

fn today_in_manila() -> NaiveDate {
    Utc::now().with_timezone(&Manila).date_naive()
}

#[tauri::command]
pub fn get_payday_info(schedule: Schedule) -> Result<PaydayInfo, String> {
    schedule.validate()?;

    let today = today_in_manila();
    let year = today.year();

    let movable = holidays::parse_movable(holidays::EMBEDDED_MOVABLE_JSON)
        .map_err(|e| format!("holidays.json is invalid: {e}"))?
        .holidays;

    // A payday can move across a year boundary, so load a few years around today.
    let calendar = Holidays::build((year - 2)..=(year + 2), &movable);
    let missing = holidays::movable_missing_for_year(&movable, year);

    Ok(payday_info(today, &schedule, &calendar, missing))
}
