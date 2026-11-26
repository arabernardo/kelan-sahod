//! Philippine holiday data. Pure module: no Tauri, no clock, no I/O.
//!
//! Holidays come from two places:
//!   1. GENERATED holidays that work for ANY year with no proclamation needed:
//!      a. fixed-date holidays (`fixed_holidays`), e.g. Dec 25;
//!      b. rule-based holidays (`computed_holidays`): Maundy Thursday, Good Friday and
//!         Black Saturday (from Easter) and National Heroes Day (last Monday of August).
//!   2. The MOVABLE / PROCLAIMED list in `src-tauri/data/holidays.json`, embedded into the
//!      binary at compile time and refreshed from GitHub: Eid'l Fitr, Eid'l Adha, Chinese
//!      New Year, Maundy Thursday, Good Friday and any special non-working day.
//!      A JSON entry REPLACES the generated holiday with the same name in the same year,
//!      so the JSON is always the final say. The generated Holy Week dates are the fallback
//!      for years the JSON doesn't list yet.
//!
//! Both regular holidays and special non-working days count (banks are closed).
//! Special WORKING days do not count and must never be added.

use chrono::{Datelike, Days, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    /// Date this list was last edited (YYYY-MM-DD). A downloaded list is only
    /// accepted when this is the same or newer than the embedded one.
    #[serde(default)]
    pub updated: Option<NaiveDate>,
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
const FIXED: [(u32, u32, &str); 13] = [
    (1, 1, "New Year's Day"),
    (4, 9, "Araw ng Kagitingan"),
    (5, 1, "Labor Day"),
    (6, 12, "Independence Day"),
    (8, 21, "Ninoy Aquino Day"),
    // National Heroes Day is NOT here: it is the last Monday of August (see `computed_holidays`).
    (11, 1, "All Saints' Day"),
    (11, 2, "All Souls' Day"),
    (11, 30, "Bonifacio Day"),
    (12, 8, "Feast of the Immaculate Conception"),
    (12, 24, "Christmas Eve"),
    (12, 25, "Christmas Day"),
    (12, 30, "Rizal Day"),
    (12, 31, "New Year's Eve"),
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
// PART 1b: RULE-BASED holidays (the date moves, but a fixed rule gives it)
// ---------------------------------------------------------------------------

/// Easter Sunday (Gregorian calendar), by the Meeus/Jones/Butcher algorithm.
pub fn easter_sunday(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).expect("valid Easter date")
}

/// The last `weekday` of the month (e.g. the last Monday of August).
pub fn last_weekday_of_month(year: i32, month: u32, weekday: Weekday) -> NaiveDate {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let mut date = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|first_of_next| first_of_next.pred_opt())
        .expect("valid year/month");
    while date.weekday() != weekday {
        date = date.pred_opt().expect("date in range");
    }
    date
}

/// Holidays that follow a rule instead of a fixed date (the JSON list can override them):
///   - Maundy Thursday, Good Friday (regular holidays) and Black Saturday (special
///     non-working day): the 3, 2 and 1 days before Easter Sunday.
///   - National Heroes Day: last Monday of August (RA 9492).
pub fn computed_holidays(year: i32) -> Vec<Holiday> {
    let easter = easter_sunday(year);
    let before_easter = |days: u64, name: &str| Holiday {
        date: easter.checked_sub_days(Days::new(days)).expect("date in range"),
        name: name.to_string(),
    };
    vec![
        before_easter(3, "Maundy Thursday"),
        before_easter(2, "Good Friday"),
        before_easter(1, "Black Saturday"),
        Holiday {
            date: last_weekday_of_month(year, 8, Weekday::Mon),
            name: "National Heroes Day".to_string(),
        },
    ]
}

/// Everything we can know for a year without waiting for a proclamation.
pub fn recurring_holidays(year: i32) -> Vec<Holiday> {
    let mut all = fixed_holidays(year);
    all.extend(computed_holidays(year));
    all
}

// ---------------------------------------------------------------------------
// PART 2: PROCLAIMED holidays come from the JSON file (see above)
// ---------------------------------------------------------------------------

/// True when the movable list has NO entry at all for `year`.
pub fn movable_missing_for_year(movable: &[Holiday], year: i32) -> bool {
    movable.iter().all(|h| h.date.year() != year)
}

/// Where the holiday list in use came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ListSource {
    /// Built into the app.
    Embedded,
    /// Downloaded from the remote holidays.json (cached on disk).
    Remote,
}

/// What we know about the list in use. Built by the command layer, read by `payday_info`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HolidayStatus {
    pub missing_for_year: bool,
    /// True when the list was confirmed against the remote file in the last 24 hours.
    pub verified_recently: bool,
    pub updated: Option<NaiveDate>,
    pub source: ListSource,
}

/// Checks a downloaded list before we trust it. Strict on purpose: the UI shows these names.
pub fn validate_remote(file: &MovableFile) -> Result<(), String> {
    if file.updated.is_none() {
        return Err("missing \"updated\" date".into());
    }
    if file.holidays.is_empty() {
        return Err("no holidays in file".into());
    }
    if file.holidays.len() > 500 {
        return Err("too many holidays".into());
    }
    for h in &file.holidays {
        if !(2000..=2100).contains(&h.date.year()) {
            return Err(format!("date out of range: {}", h.date));
        }
        let name = h.name.trim();
        if name.is_empty() || name.chars().count() > 80 {
            return Err(format!("bad holiday name for {}", h.date));
        }
    }
    Ok(())
}

/// Picks the list to use. A validated cached (downloaded) list wins only when it is the
/// same age or newer than the embedded one, so an app update is never beaten by an old cache.
pub fn resolve(embedded: MovableFile, cached: Option<MovableFile>) -> (MovableFile, ListSource) {
    match cached {
        Some(c) if c.updated >= embedded.updated => (c, ListSource::Remote),
        _ => (embedded, ListSource::Embedded),
    }
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

    /// Generated holidays (fixed + rule-based) for every year in `years`, plus the proclaimed list.
    /// Pass a range wider than the current year: paydays can cross year boundaries.
    ///
    /// A proclaimed entry REPLACES the generated holiday with the same name in the same
    /// year (names compared ignoring case). So if a proclamation moves Good Friday, the
    /// JSON date wins and the computed one disappears for that year only.
    pub fn build(years: RangeInclusive<i32>, proclaimed: &[Holiday]) -> Self {
        let overridden: HashSet<(i32, String)> = proclaimed
            .iter()
            .map(|h| (h.date.year(), h.name.trim().to_lowercase()))
            .collect();

        let generated = years
            .flat_map(recurring_holidays)
            .filter(|h| !overridden.contains(&(h.date.year(), h.name.trim().to_lowercase())));

        // Proclaimed entries come first, so they also win if two entries share a date.
        Self::from_entries(proclaimed.iter().cloned().chain(generated))
    }

    pub fn name_on(&self, date: NaiveDate) -> Option<&str> {
        self.by_date.get(&date).map(String::as_str)
    }

    /// All holidays in the given month, in date order.
    pub fn in_month(&self, year: i32, month: u32) -> Vec<(NaiveDate, &str)> {
        let mut found: Vec<(NaiveDate, &str)> = self
            .by_date
            .iter()
            .filter(|(date, _)| date.year() == year && date.month() == month)
            .map(|(date, name)| (*date, name.as_str()))
            .collect();
        found.sort_by_key(|(date, _)| *date);
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn fixed_holidays_has_all_thirteen_for_any_year() {
        let list = fixed_holidays(2031);
        assert_eq!(list.len(), 13);
        assert!(list
            .iter()
            .any(|h| h.date == d(2031, 12, 30) && h.name == "Rizal Day"));
        assert!(list.iter().any(|h| h.date == d(2031, 11, 2) && h.name == "All Souls' Day"));
        assert!(list.iter().any(|h| h.date == d(2031, 12, 24) && h.name == "Christmas Eve"));
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
    fn easter_sunday_matches_known_dates() {
        let known = [
            (2000, 4, 23),
            (2019, 4, 21),
            (2024, 3, 31),
            (2025, 4, 20),
            (2026, 4, 5),
            (2027, 3, 28),
            (2028, 4, 16),
            (2029, 4, 1),
            (2030, 4, 21),
        ];
        for (y, m, day) in known {
            assert_eq!(easter_sunday(y), d(y, m, day), "Easter {y}");
        }
    }

    #[test]
    fn last_monday_of_august() {
        assert_eq!(last_weekday_of_month(2024, 8, Weekday::Mon), d(2024, 8, 26));
        assert_eq!(last_weekday_of_month(2025, 8, Weekday::Mon), d(2025, 8, 25));
        assert_eq!(last_weekday_of_month(2026, 8, Weekday::Mon), d(2026, 8, 31));
        assert_eq!(last_weekday_of_month(2027, 8, Weekday::Mon), d(2027, 8, 30));
        // December rolls over the year boundary when looking for "the day before next month".
        assert_eq!(last_weekday_of_month(2026, 12, Weekday::Thu), d(2026, 12, 31));
    }

    #[test]
    fn computed_holidays_cover_holy_week_and_heroes_day() {
        let year = computed_holidays(2026);
        let find = |name: &str| year.iter().find(|h| h.name == name).unwrap().date;
        assert_eq!(find("Maundy Thursday"), d(2026, 4, 2));
        assert_eq!(find("Good Friday"), d(2026, 4, 3));
        assert_eq!(find("Black Saturday"), d(2026, 4, 4));
        assert_eq!(find("National Heroes Day"), d(2026, 8, 31));

        let year = computed_holidays(2027);
        let find = |name: &str| year.iter().find(|h| h.name == name).unwrap().date;
        assert_eq!(find("Maundy Thursday"), d(2027, 3, 25));
        assert_eq!(find("Good Friday"), d(2027, 3, 26));
        assert_eq!(find("Black Saturday"), d(2027, 3, 27));
        assert_eq!(find("National Heroes Day"), d(2027, 8, 30));
    }

    #[test]
    fn recurring_holidays_need_no_json_at_all() {
        // Far-future year with an EMPTY proclaimed list: everything recurring is still there.
        let h = Holidays::build(2035..=2035, &[]);
        assert_eq!(h.name_on(d(2035, 12, 25)), Some("Christmas Day")); // fixed
        assert_eq!(h.name_on(d(2035, 8, 27)), Some("National Heroes Day")); // last Monday of Aug 2035
        assert_eq!(h.name_on(easter_sunday(2035).pred_opt().unwrap().pred_opt().unwrap()), Some("Good Friday"));
        assert_eq!(h.name_on(d(2035, 3, 3)), None);
    }

    #[test]
    fn proclaimed_list_can_still_add_to_recurring_ones() {
        let movable = vec![Holiday { date: d(2026, 12, 24), name: "Christmas Eve".into() }];
        let h = Holidays::build(2026..=2026, &movable);
        assert_eq!(h.name_on(d(2026, 12, 24)), Some("Christmas Eve"));
        assert_eq!(h.name_on(d(2026, 12, 25)), Some("Christmas Day"));
    }

    #[test]
    fn in_month_lists_holidays_in_date_order() {
        let h = Holidays::build(2026..=2026, &[Holiday { date: d(2026, 12, 24), name: "Christmas Eve".into() }]);
        let december: Vec<_> = h.in_month(2026, 12).into_iter().map(|(date, name)| (date, name.to_string())).collect();
        assert_eq!(
            december,
            vec![
                (d(2026, 12, 8), "Feast of the Immaculate Conception".to_string()),
                (d(2026, 12, 24), "Christmas Eve".to_string()),
                (d(2026, 12, 25), "Christmas Day".to_string()),
                (d(2026, 12, 30), "Rizal Day".to_string()),
                (d(2026, 12, 31), "New Year's Eve".to_string()),
            ]
        );
        assert!(h.in_month(2026, 3).is_empty());
    }

    #[test]
    fn national_heroes_day_is_the_last_monday_not_a_fixed_date() {
        let h = Holidays::build(2026..=2028, &[]);
        assert_eq!(h.name_on(d(2026, 8, 31)), Some("National Heroes Day"));
        assert_eq!(h.name_on(d(2027, 8, 30)), Some("National Heroes Day"));
        assert_eq!(h.name_on(d(2027, 8, 31)), None); // a Tuesday: must NOT be a holiday
        assert_eq!(h.name_on(d(2028, 8, 28)), Some("National Heroes Day"));
        assert_eq!(h.name_on(d(2028, 8, 31)), None);
    }

    #[test]
    fn proclaimed_entry_replaces_the_generated_holiday_with_the_same_name() {
        // A (made-up) proclamation moves Good Friday 2026 to Apr 10.
        let moved = vec![Holiday { date: d(2026, 4, 10), name: "Good Friday".into() }];
        let h = Holidays::build(2026..=2027, &moved);
        assert_eq!(h.name_on(d(2026, 4, 10)), Some("Good Friday"));
        assert_eq!(h.name_on(d(2026, 4, 3)), None); // computed date dropped for 2026
        // Other years are untouched.
        assert_eq!(h.name_on(d(2027, 3, 26)), Some("Good Friday"));
        // Other holy days of 2026 are untouched.
        assert_eq!(h.name_on(d(2026, 4, 2)), Some("Maundy Thursday"));
        assert_eq!(h.name_on(d(2026, 4, 4)), Some("Black Saturday"));
    }

    #[test]
    fn name_override_ignores_case_and_spaces() {
        let moved = vec![Holiday { date: d(2026, 4, 10), name: "  good friday ".into() }];
        let h = Holidays::build(2026..=2026, &moved);
        assert_eq!(h.name_on(d(2026, 4, 3)), None);
        assert_eq!(h.name_on(d(2026, 4, 10)), Some("  good friday "));
    }

    #[test]
    fn generated_holy_week_is_the_fallback_when_json_has_no_entry() {
        // JSON lists only 2026; 2027 still gets Maundy Thursday and Good Friday.
        let listed = vec![
            Holiday { date: d(2026, 4, 2), name: "Maundy Thursday".into() },
            Holiday { date: d(2026, 4, 3), name: "Good Friday".into() },
        ];
        let h = Holidays::build(2026..=2027, &listed);
        assert_eq!(h.name_on(d(2026, 4, 3)), Some("Good Friday"));
        assert_eq!(h.name_on(d(2027, 3, 25)), Some("Maundy Thursday"));
        assert_eq!(h.name_on(d(2027, 3, 26)), Some("Good Friday"));
    }

    #[test]
    fn same_date_conflict_prefers_the_proclaimed_name() {
        let listed = vec![Holiday { date: d(2026, 12, 25), name: "Pasko".into() }];
        let h = Holidays::build(2026..=2026, &listed);
        assert_eq!(h.name_on(d(2026, 12, 25)), Some("Pasko"));
    }

    #[test]
    fn official_2027_list_is_fully_covered_with_no_extras() {
        let proclaimed = parse_movable(EMBEDDED_MOVABLE_JSON).unwrap().holidays;
        let h = Holidays::build(2027..=2027, &proclaimed);

        // The official 2027 list (regular holidays + special non-working days).
        // Eid'l Fitr and Eid'l Adha are not in it yet: they are proclaimed separately.
        let official = [
            (1, 1, "New Year's Day"),
            (2, 6, "Chinese New Year"),
            (3, 25, "Maundy Thursday"),
            (3, 26, "Good Friday"),
            (3, 27, "Black Saturday"),
            (4, 9, "Araw ng Kagitingan"),
            (5, 1, "Labor Day"),
            (6, 12, "Independence Day"),
            (8, 21, "Ninoy Aquino Day"),
            (8, 30, "National Heroes Day"),
            (11, 1, "All Saints' Day"),
            (11, 2, "All Souls' Day"),
            (11, 30, "Bonifacio Day"),
            (12, 8, "Feast of the Immaculate Conception"),
            (12, 24, "Christmas Eve"),
            (12, 25, "Christmas Day"),
            (12, 30, "Rizal Day"),
            (12, 31, "New Year's Eve"), // the official list calls it "Last Day of the Year"
        ];
        for (m, day, name) in official {
            assert_eq!(h.name_on(d(2027, m, day)), Some(name), "2027-{m:02}-{day:02}");
        }

        // No extra holidays: the app must not invent any date the official list lacks.
        let total: usize = (1..=12).map(|m| h.in_month(2027, m).len()).sum();
        assert_eq!(total, official.len());
    }

    fn file(updated: Option<(i32, u32, u32)>, entries: &[(i32, u32, u32, &str)]) -> MovableFile {
        MovableFile {
            updated: updated.map(|(y, m, day)| d(y, m, day)),
            holidays: entries
                .iter()
                .map(|&(y, m, day, name)| Holiday { date: d(y, m, day), name: name.into() })
                .collect(),
            todo_verify: vec![],
        }
    }

    #[test]
    fn embedded_json_passes_remote_validation() {
        let embedded = parse_movable(EMBEDDED_MOVABLE_JSON).unwrap();
        assert_eq!(validate_remote(&embedded), Ok(()));
    }

    #[test]
    fn validate_remote_accepts_a_good_file() {
        let f = file(Some((2026, 12, 1)), &[(2026, 12, 24, "Christmas Eve")]);
        assert_eq!(validate_remote(&f), Ok(()));
    }

    #[test]
    fn validate_remote_rejects_bad_files() {
        assert!(validate_remote(&file(None, &[(2026, 12, 24, "X")])).is_err());
        assert!(validate_remote(&file(Some((2026, 12, 1)), &[])).is_err());
        assert!(validate_remote(&file(Some((2026, 12, 1)), &[(1999, 1, 2, "Old")])).is_err());
        assert!(validate_remote(&file(Some((2026, 12, 1)), &[(2026, 12, 24, "   ")])).is_err());
        let long = "x".repeat(81);
        assert!(validate_remote(&file(Some((2026, 12, 1)), &[(2026, 12, 24, &long)])).is_err());
    }

    #[test]
    fn resolve_prefers_newer_or_equal_cache() {
        let embedded = || file(Some((2026, 11, 26)), &[(2026, 4, 3, "Good Friday")]);
        // newer cache wins
        let (list, source) = resolve(embedded(), Some(file(Some((2026, 12, 1)), &[(2026, 12, 24, "Christmas Eve")])));
        assert_eq!(source, ListSource::Remote);
        assert_eq!(list.holidays[0].name, "Christmas Eve");
        // equal cache counts as remote (it was verified against the remote file)
        let (_, source) = resolve(embedded(), Some(file(Some((2026, 11, 26)), &[(2026, 4, 3, "Good Friday")])));
        assert_eq!(source, ListSource::Remote);
        // older cache loses to an app update
        let (list, source) = resolve(embedded(), Some(file(Some((2026, 10, 1)), &[(2026, 10, 2, "Old")])));
        assert_eq!(source, ListSource::Embedded);
        assert_eq!(list.holidays[0].name, "Good Friday");
        // no cache
        let (_, source) = resolve(embedded(), None);
        assert_eq!(source, ListSource::Embedded);
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