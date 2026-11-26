//! Payday logic. Pure module: no Tauri, no UI, no clock.
//! "Today" is always a function PARAMETER so tests can use fixed dates.
//!
//! Rule: if the nominal payday falls on a Saturday, Sunday or holiday, the actual
//! payday is the nearest PREVIOUS working day. Paydays never move forward.

use crate::holidays::Holidays;
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// Twice-a-month schedule. A day of 29-31 means "last day of the month" when the
/// month is shorter (so the preset "15 & 30/31" is stored as 15 and 31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub first_day: u32,
    pub second_day: u32,
}

impl Schedule {
    pub fn validate(&self) -> Result<(), String> {
        let in_range = |d: u32| (1..=31).contains(&d);
        if !in_range(self.first_day) || !in_range(self.second_day) {
            return Err("payday days must be between 1 and 31".into());
        }
        if self.first_day == self.second_day {
            return Err("the two payday days must be different".into());
        }
        Ok(())
    }
}

/// Why a payday was moved earlier. Structured data only; the UI writes the words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AdjustReason {
    Saturday { date: NaiveDate },
    Sunday { date: NaiveDate },
    Holiday { date: NaiveDate, name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustedPayday {
    pub actual_date: NaiveDate,
    pub nominal_date: NaiveDate,
    pub was_adjusted: bool,
    /// 1 = Monday ... 7 = Sunday (so the UI never needs date math to name the day).
    pub actual_weekday: u32,
    /// Skipped days, starting from the nominal date and walking backwards.
    pub reasons: Vec<AdjustReason>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaydayInfo {
    pub today: NaiveDate,
    pub days_left: i64,
    pub next_payday: AdjustedPayday,
    pub previous_payday: AdjustedPayday,
    /// 0.0 = just got paid, 1.0 = payday is today.
    pub progress: f64,
    pub holiday_data_missing_for_year: bool,
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|first_of_next| first_of_next.pred_opt())
        .map(|last| last.day())
        .expect("valid year/month")
}

/// The date from the schedule, clamped to the last day of short months.
pub fn nominal_payday(year: i32, month: u32, day: u32) -> NaiveDate {
    let day = day.min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).expect("valid year/month/day")
}

/// Step back one day at a time until we land on a working day
/// (Monday-Friday and not a holiday).
pub fn adjust_payday(nominal: NaiveDate, holidays: &Holidays) -> AdjustedPayday {
    let mut reasons = Vec::new();
    let mut date = nominal;

    // The bound is only a safety net; real calendars need a handful of steps at most.
    for _ in 0..366 {
        match date.weekday() {
            Weekday::Sat => reasons.push(AdjustReason::Saturday { date }),
            Weekday::Sun => reasons.push(AdjustReason::Sunday { date }),
            _ => match holidays.name_on(date) {
                Some(name) => reasons.push(AdjustReason::Holiday {
                    date,
                    name: name.to_string(),
                }),
                None => break,
            },
        }
        match date.pred_opt() {
            Some(previous) => date = previous,
            None => break,
        }
    }

    AdjustedPayday {
        actual_date: date,
        nominal_date: nominal,
        was_adjusted: date != nominal,
        actual_weekday: date.weekday().number_from_monday(),
        reasons,
    }
}

fn shift_month(year: i32, month: u32, offset: i32) -> (i32, u32) {
    let index = year * 12 + (month as i32 - 1) + offset;
    (index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
}

/// Every payday around `today` (two months back to two months ahead), ADJUSTED first
/// and then sorted by ACTUAL date. An adjusted payday can land earlier than its
/// nominal date, even in an earlier month or year, so we never sort by nominal date.
fn candidates(today: NaiveDate, schedule: &Schedule, holidays: &Holidays) -> Vec<AdjustedPayday> {
    let mut all = Vec::with_capacity(10);
    for offset in -2..=2 {
        let (year, month) = shift_month(today.year(), today.month(), offset);
        for day in [schedule.first_day, schedule.second_day] {
            all.push(adjust_payday(nominal_payday(year, month, day), holidays));
        }
    }
    all.sort_by_key(|p| p.actual_date);
    // Two nominal days can collapse to the same actual day (e.g. 30 & 31 in February).
    all.dedup_by_key(|p| p.actual_date);
    all
}

/// The earliest ACTUAL payday that is today or later, plus the calendar days left.
/// If today is the actual payday, days left is 0.
pub fn next_payday(
    today: NaiveDate,
    schedule: &Schedule,
    holidays: &Holidays,
) -> (AdjustedPayday, i64) {
    let next = candidates(today, schedule, holidays)
        .into_iter()
        .find(|p| p.actual_date >= today)
        // Unreachable: candidates reach two months ahead and an adjustment moves a
        // payday back by a few days at most.
        .expect("a payday within two months ahead");
    let days_left = (next.actual_date - today).num_days();
    (next, days_left)
}

/// The latest ACTUAL payday strictly before today (start of the current cycle).
pub fn previous_payday(
    today: NaiveDate,
    schedule: &Schedule,
    holidays: &Holidays,
) -> AdjustedPayday {
    candidates(today, schedule, holidays)
        .into_iter()
        .rev()
        .find(|p| p.actual_date < today)
        .expect("a payday within two months back")
}

/// Everything the UI needs, computed from `today`.
pub fn payday_info(
    today: NaiveDate,
    schedule: &Schedule,
    holidays: &Holidays,
    holiday_data_missing_for_year: bool,
) -> PaydayInfo {
    let (next, days_left) = next_payday(today, schedule, holidays);
    let previous = previous_payday(today, schedule, holidays);

    let cycle_days = (next.actual_date - previous.actual_date).num_days().max(1);
    let elapsed_days = (today - previous.actual_date).num_days();
    let progress = (elapsed_days as f64 / cycle_days as f64).clamp(0.0, 1.0);

    PaydayInfo {
        today,
        days_left,
        next_payday: next,
        previous_payday: previous,
        progress,
        holiday_data_missing_for_year,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holidays::Holiday;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    /// Test holiday list. Never the real calendar.
    fn hols(entries: &[(i32, u32, u32, &str)]) -> Holidays {
        Holidays::from_entries(entries.iter().map(|&(y, m, day, name)| Holiday {
            date: d(y, m, day),
            name: name.to_string(),
        }))
    }

    const TWICE: Schedule = Schedule { first_day: 15, second_day: 31 };

    // Sanity check on the calendar facts the tests rely on.
    #[test]
    fn calendar_facts() {
        assert_eq!(d(2025, 11, 15).weekday(), Weekday::Sat);
        assert_eq!(d(2025, 12, 15).weekday(), Weekday::Mon);
        assert_eq!(d(2026, 2, 15).weekday(), Weekday::Sun);
        assert_eq!(d(2026, 2, 28).weekday(), Weekday::Sat);
        assert_eq!(d(2032, 2, 29).weekday(), Weekday::Sun);
        assert_eq!(d(2028, 12, 31).weekday(), Weekday::Sun);
        assert_eq!(d(2026, 2, 1).weekday(), Weekday::Sun);
    }

    #[test]
    fn nominal_handles_short_months() {
        assert_eq!(nominal_payday(2026, 2, 31), d(2026, 2, 28));
        assert_eq!(nominal_payday(2024, 2, 30), d(2024, 2, 29));
        assert_eq!(nominal_payday(2026, 4, 31), d(2026, 4, 30));
        assert_eq!(nominal_payday(2026, 12, 31), d(2026, 12, 31));
    }

    #[test]
    fn schedule_validation() {
        assert!(TWICE.validate().is_ok());
        assert!(Schedule { first_day: 5, second_day: 5 }.validate().is_err());
        assert!(Schedule { first_day: 0, second_day: 5 }.validate().is_err());
        assert!(Schedule { first_day: 5, second_day: 32 }.validate().is_err());
    }

    #[test]
    fn saturday_15th_moves_to_friday_14th_and_counts_days() {
        let h = hols(&[]);
        let (next, days_left) = next_payday(d(2025, 11, 10), &TWICE, &h);
        assert_eq!(next.actual_date, d(2025, 11, 14));
        assert_eq!(next.nominal_date, d(2025, 11, 15));
        assert_eq!(days_left, 4);
    }

    #[test]
    fn on_the_actual_payday_days_left_is_zero() {
        let h = hols(&[]);
        let (next, days_left) = next_payday(d(2025, 11, 14), &TWICE, &h);
        assert_eq!(next.actual_date, d(2025, 11, 14));
        assert_eq!(days_left, 0);
    }

    #[test]
    fn day_after_adjusted_payday_points_to_the_next_scheduled_payday() {
        let h = hols(&[]);
        // Today is Saturday the 15th: the 14th already passed.
        // Nov 30 2025 is a Sunday, so the month-end payday is Friday Nov 28.
        let (next, days_left) = next_payday(d(2025, 11, 15), &TWICE, &h);
        assert_eq!(next.actual_date, d(2025, 11, 28));
        assert_eq!(days_left, 13);
    }

    #[test]
    fn after_both_paydays_of_a_month_it_rolls_into_next_month() {
        let h = hols(&[]);
        // Nov 28 was payday, so on Nov 29 the next one is Mon Dec 15.
        let (next, days_left) = next_payday(d(2025, 11, 29), &TWICE, &h);
        assert_eq!(next.actual_date, d(2025, 12, 15));
        assert_eq!(days_left, 16);
    }

    #[test]
    fn saturday_15th_with_holiday_14th_goes_to_thursday_13th() {
        let h = hols(&[(2025, 11, 14, "Test Holiday")]);
        let adjusted = adjust_payday(d(2025, 11, 15), &h);
        assert_eq!(adjusted.actual_date, d(2025, 11, 13));
        assert_eq!(adjusted.actual_weekday, 4); // Thursday
    }

    #[test]
    fn monday_holiday_15th_goes_to_friday_12th() {
        let h = hols(&[(2025, 12, 15, "Test Monday Holiday")]);
        let adjusted = adjust_payday(d(2025, 12, 15), &h);
        assert_eq!(adjusted.actual_date, d(2025, 12, 12));
        assert_eq!(adjusted.actual_weekday, 5);
    }

    #[test]
    fn sunday_nominal_with_friday_holiday_goes_to_thursday() {
        let h = hols(&[(2026, 2, 13, "Test Friday Holiday")]);
        let adjusted = adjust_payday(d(2026, 2, 15), &h);
        assert_eq!(adjusted.actual_date, d(2026, 2, 12));
        assert_eq!(adjusted.actual_weekday, 4);
    }

    #[test]
    fn february_non_leap_year_second_payday() {
        let h = hols(&[]);
        // Feb 2026 has 28 days and the 28th is a Saturday -> Friday the 27th.
        let (next, days_left) = next_payday(d(2026, 2, 20), &TWICE, &h);
        assert_eq!(next.nominal_date, d(2026, 2, 28));
        assert_eq!(next.actual_date, d(2026, 2, 27));
        assert_eq!(days_left, 7);
    }

    #[test]
    fn february_leap_year_second_payday() {
        let h = hols(&[]);
        // Feb 2024: the 29th is a Thursday, no adjustment.
        let (next, _) = next_payday(d(2024, 2, 20), &TWICE, &h);
        assert_eq!(next.nominal_date, d(2024, 2, 29));
        assert_eq!(next.actual_date, d(2024, 2, 29));
        assert!(!next.was_adjusted);
        // Feb 2032: the 29th is a Sunday -> Friday the 27th.
        let (next, _) = next_payday(d(2032, 2, 20), &TWICE, &h);
        assert_eq!(next.nominal_date, d(2032, 2, 29));
        assert_eq!(next.actual_date, d(2032, 2, 27));
    }

    #[test]
    fn dec_31_holiday_plus_weekend_moves_back_correctly() {
        // Dec 31 2028 is a Sunday; Dec 30 (Saturday) and Dec 31 are holidays too.
        let h = hols(&[(2028, 12, 30, "Rizal Day"), (2028, 12, 31, "Last Day of the Year")]);
        let (next, days_left) = next_payday(d(2028, 12, 20), &TWICE, &h);
        assert_eq!(next.actual_date, d(2028, 12, 29));
        assert_eq!(days_left, 9);
    }

    #[test]
    fn five_and_twenty_rolls_over_to_january() {
        let schedule = Schedule { first_day: 5, second_day: 20 };
        let h = hols(&[]);
        // Dec 21 2025 -> Jan 5 2026 (a Monday).
        let (next, days_left) = next_payday(d(2025, 12, 21), &schedule, &h);
        assert_eq!(next.actual_date, d(2026, 1, 5));
        assert_eq!(days_left, 15);
    }

    #[test]
    fn five_and_twenty_rollover_is_adjusted_when_needed() {
        let schedule = Schedule { first_day: 5, second_day: 20 };
        let h = hols(&[(2026, 1, 5, "Test Holiday")]);
        // Jan 5 is now a holiday -> Jan 4 (Sunday), Jan 3 (Saturday), Fri Jan 2.
        let (next, _) = next_payday(d(2025, 12, 21), &schedule, &h);
        assert_eq!(next.actual_date, d(2026, 1, 2));
    }

    #[test]
    fn custom_day_1_on_a_sunday_moves_into_previous_month() {
        let schedule = Schedule { first_day: 1, second_day: 15 };
        let h = hols(&[]);
        // Feb 1 2026 is a Sunday -> last working day of January: Friday Jan 30.
        let (next, days_left) = next_payday(d(2026, 1, 20), &schedule, &h);
        assert_eq!(next.nominal_date, d(2026, 2, 1));
        assert_eq!(next.actual_date, d(2026, 1, 30));
        assert_eq!(days_left, 10);
    }

    #[test]
    fn custom_day_1_on_a_holiday_moves_into_previous_month() {
        let schedule = Schedule { first_day: 1, second_day: 15 };
        // May 1 2026 is a Friday and a holiday -> Thursday Apr 30.
        let h = hols(&[(2026, 5, 1, "Labor Day")]);
        let (next, _) = next_payday(d(2026, 4, 20), &schedule, &h);
        assert_eq!(next.actual_date, d(2026, 4, 30));
    }

    #[test]
    fn nominal_jan_1_holiday_moves_into_previous_year() {
        let schedule = Schedule { first_day: 1, second_day: 15 };
        let h = hols(&[(2026, 1, 1, "New Year's Day")]);
        let (next, days_left) = next_payday(d(2025, 12, 20), &schedule, &h);
        assert_eq!(next.nominal_date, d(2026, 1, 1));
        assert_eq!(next.actual_date, d(2025, 12, 31));
        assert_eq!(days_left, 11);
    }

    #[test]
    fn nominal_jan_1_with_holiday_dec_30_and_31_goes_further_back() {
        let schedule = Schedule { first_day: 1, second_day: 15 };
        let h = hols(&[
            (2026, 1, 1, "New Year's Day"),
            (2025, 12, 31, "Last Day of the Year"),
            (2025, 12, 30, "Rizal Day"),
        ]);
        let adjusted = adjust_payday(nominal_payday(2026, 1, 1), &h);
        assert_eq!(adjusted.actual_date, d(2025, 12, 29));
        let (next, _) = next_payday(d(2025, 12, 20), &schedule, &h);
        assert_eq!(next.actual_date, d(2025, 12, 29));
    }

    #[test]
    fn custom_31_and_15_in_a_30_day_month() {
        let schedule = Schedule { first_day: 31, second_day: 15 };
        let h = hols(&[]);
        // April 2026: the 15th is a Wednesday, the 31st clamps to Thursday Apr 30.
        let (next, days_left) = next_payday(d(2026, 4, 16), &schedule, &h);
        assert_eq!(next.nominal_date, d(2026, 4, 30));
        assert_eq!(next.actual_date, d(2026, 4, 30));
        assert_eq!(days_left, 14);
    }

    #[test]
    fn collapsed_paydays_do_not_break_next_payday() {
        // 29 & 31 both clamp to Feb 28 in a non-leap year.
        let schedule = Schedule { first_day: 29, second_day: 31 };
        let h = hols(&[]);
        let (next, _) = next_payday(d(2026, 2, 10), &schedule, &h);
        assert_eq!(next.actual_date, d(2026, 2, 27)); // Feb 28 2026 is a Saturday
    }

    #[test]
    fn unadjusted_payday_has_no_reasons() {
        let h = hols(&[]);
        let adjusted = adjust_payday(d(2025, 11, 14), &h);
        assert!(!adjusted.was_adjusted);
        assert!(adjusted.reasons.is_empty());
        assert_eq!(adjusted.actual_date, d(2025, 11, 14));
    }

    #[test]
    fn reasons_saturday() {
        let h = hols(&[]);
        let adjusted = adjust_payday(d(2025, 11, 15), &h);
        assert!(adjusted.was_adjusted);
        assert_eq!(adjusted.reasons, vec![AdjustReason::Saturday { date: d(2025, 11, 15) }]);
    }

    #[test]
    fn reasons_holiday_rizal_day() {
        // Dec 30 2025 is a Tuesday.
        let h = hols(&[(2025, 12, 30, "Rizal Day")]);
        let adjusted = adjust_payday(d(2025, 12, 30), &h);
        assert!(adjusted.was_adjusted);
        assert_eq!(adjusted.actual_date, d(2025, 12, 29));
        assert_eq!(
            adjusted.reasons,
            vec![AdjustReason::Holiday { date: d(2025, 12, 30), name: "Rizal Day".into() }]
        );
    }

    #[test]
    fn reasons_are_listed_from_nominal_backwards() {
        // Saturday 15th, Friday 14th is a holiday -> Thursday 13th.
        let h = hols(&[(2025, 11, 14, "Test Holiday")]);
        let adjusted = adjust_payday(d(2025, 11, 15), &h);
        assert_eq!(
            adjusted.reasons,
            vec![
                AdjustReason::Saturday { date: d(2025, 11, 15) },
                AdjustReason::Holiday { date: d(2025, 11, 14), name: "Test Holiday".into() },
            ]
        );
        // Monday holiday: holiday, then Sunday, then Saturday.
        let h = hols(&[(2025, 12, 15, "Test Monday Holiday")]);
        let adjusted = adjust_payday(d(2025, 12, 15), &h);
        assert_eq!(
            adjusted.reasons,
            vec![
                AdjustReason::Holiday { date: d(2025, 12, 15), name: "Test Monday Holiday".into() },
                AdjustReason::Sunday { date: d(2025, 12, 14) },
                AdjustReason::Saturday { date: d(2025, 12, 13) },
            ]
        );
    }

    #[test]
    fn progress_runs_from_last_payday_to_next() {
        let h = hols(&[]);
        // Cycle: Fri Oct 31 2025 -> Fri Nov 14 2025 (14 days).
        let mid = payday_info(d(2025, 11, 7), &TWICE, &h, false);
        assert_eq!(mid.previous_payday.actual_date, d(2025, 10, 31));
        assert_eq!(mid.next_payday.actual_date, d(2025, 11, 14));
        assert!((mid.progress - 0.5).abs() < 1e-9);
        // On payday the bar is full.
        let on_payday = payday_info(d(2025, 11, 14), &TWICE, &h, false);
        assert_eq!(on_payday.days_left, 0);
        assert!((on_payday.progress - 1.0).abs() < 1e-9);
        // The day after payday it starts again.
        let after = payday_info(d(2025, 11, 15), &TWICE, &h, false);
        assert_eq!(after.previous_payday.actual_date, d(2025, 11, 14));
        assert!(after.progress > 0.0 && after.progress < 0.2);
    }

    #[test]
    fn serializes_with_camel_case_and_iso_dates() {
        let h = hols(&[]);
        let info = payday_info(d(2025, 11, 10), &TWICE, &h, true);
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["daysLeft"], 4);
        assert_eq!(json["holidayDataMissingForYear"], true);
        assert_eq!(json["nextPayday"]["actualDate"], "2025-11-14");
        assert_eq!(json["nextPayday"]["nominalDate"], "2025-11-15");
        assert_eq!(json["nextPayday"]["wasAdjusted"], true);
        assert_eq!(json["nextPayday"]["actualWeekday"], 5);
        assert_eq!(json["nextPayday"]["reasons"][0]["kind"], "saturday");
        assert_eq!(json["nextPayday"]["reasons"][0]["date"], "2025-11-15");
    }
}
