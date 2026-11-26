//! Philippine holiday data. Pure module: no Tauri, no clock, no I/O.
//!
//! Holidays come from two places:
//!   1. FIXED-DATE holidays, generated here for any year (`fixed_holidays`).
//!   2. MOVABLE / PROCLAIMED-YEARLY holidays, a manual list in
//!      `src-tauri/data/holidays.json`, embedded into the binary at compile time.
//!
//! Both regular holidays and special non-working days count (banks are closed).
//! Special WORKING days do not count and must never be added.

use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::RangeInclusive;

/// The movable holiday list, embedded in the binary. Update it every year.
pub const EMBEDDED_MOVABLE_JSON: &str = include_str!("../data/holidays.json");

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Holiday {
    pub date: NaiveDate,
    pub name: String,
}

/// Shape of `data/holidays.json`. Unknown fields (like `_readme`) are ignored.
#[derive(Debug, Deserialize)]
pub struct MovableFile {
    #[serde(default)]
    pub holidays: Vec<Holiday>,
    /// Dates we were NOT sure about. Never used for calculations.
    #[allow(dead_code)]
    #[serde(default, rename = "todoVerify")]
    pub todo_verify: Vec<serde_json::Value>,
}

pub fn parse_movable(json: &str) -> Result<MovableFile, serde_json::Error> {
    serde_json::from_str(json)
}

// ---------------------------------------------------------------------------
// PART 1: FIXED-DATE holidays (same month/day every year, generated in code)
// ---------------------------------------------------------------------------
const FIXED: [(u32, u32, &str); 11] = [
    (1, 1, "New Year's Day"),
    (4, 9, "Araw ng Kagitingan"),
    (5, 1, "Labor Day"),
    (6, 12, "Independence Day"),
    (8, 21, "Ninoy Aquino Day"),
    (11, 1, "All Saints' Day"),
    (11, 30, "Bonifacio Day"),
    (12, 8, "Feast of the Immaculate Conception"),
    (12, 25, "Christmas Day"),
    (12, 30, "Rizal Day"),
    (12, 31, "Last Day of the Year"),
];

pub fn fixed_holidays(year: i32) -> Vec<Holiday> {
    FIXED
        .iter()
        .filter_map(|&(month, day, name)| {
            NaiveDate::from_ymd_opt(year, month, day).map(|date| Holiday {
                date,
                name: name.to_string(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// PART 2: MOVABLE / PROCLAIMED holidays come from the JSON file (see above)
// ---------------------------------------------------------------------------

/// True when the movable list has NO entry at all for `year`.
pub fn movable_missing_for_year(movable: &[Holiday], year: i32) -> bool {
    movable.iter().all(|h| h.date.year() != year)
}

/// Lookup table: date -> holiday name.
#[derive(Debug, Default)]
pub struct Holidays {
    by_date: HashMap<NaiveDate, String>,
}

impl Holidays {
    /// Build from explicit entries (used by tests). The first entry wins on duplicates.
    pub fn from_entries(entries: impl IntoIterator<Item = Holiday>) -> Self {
        let mut by_date = HashMap::new();
        for h in entries {
            by_date.entry(h.date).or_insert(h.name);
        }
        Self { by_date }
    }

    /// Fixed holidays for every year in `years`, plus the movable list.
    /// Pass a range wider than the current year: paydays can cross year boundaries.
    pub fn build(years: RangeInclusive<i32>, movable: &[Holiday]) -> Self {
        let fixed = years.flat_map(fixed_holidays);
        Self::from_entries(fixed.chain(movable.iter().cloned()))
    }

    pub fn name_on(&self, date: NaiveDate) -> Option<&str> {
        self.by_date.get(&date).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn fixed_holidays_has_all_eleven_for_any_year() {
        let list = fixed_holidays(2031);
        assert_eq!(list.len(), 11);
        assert!(list
            .iter()
            .any(|h| h.date == d(2031, 12, 30) && h.name == "Rizal Day"));
    }

    #[test]
    fn build_merges_fixed_and_movable_and_spans_years() {
        let movable = vec![Holiday {
            date: d(2026, 4, 3),
            name: "Good Friday".into(),
        }];
        let h = Holidays::build(2025..=2027, &movable);
        assert_eq!(h.name_on(d(2026, 4, 3)), Some("Good Friday"));
        assert_eq!(h.name_on(d(2025, 12, 25)), Some("Christmas Day"));
        assert_eq!(h.name_on(d(2027, 1, 1)), Some("New Year's Day"));
        assert_eq!(h.name_on(d(2026, 3, 3)), None);
    }

    #[test]
    fn missing_year_detection() {
        let movable = vec![Holiday {
            date: d(2026, 4, 3),
            name: "Good Friday".into(),
        }];
        assert!(!movable_missing_for_year(&movable, 2026));
        assert!(movable_missing_for_year(&movable, 2030));
    }

    #[test]
    fn embedded_json_is_valid_and_sane() {
        let file = parse_movable(EMBEDDED_MOVABLE_JSON).expect("holidays.json must parse");
        assert!(!file.holidays.is_empty());
        assert!(!file.todo_verify.is_empty());
        let mut dates: Vec<_> = file.holidays.iter().map(|h| h.date).collect();
        dates.sort();
        let before = dates.len();
        dates.dedup();
        assert_eq!(before, dates.len(), "duplicate dates in holidays.json");
    }
}
