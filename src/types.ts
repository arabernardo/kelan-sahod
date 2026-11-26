// Mirrors the Rust response types in src-tauri/src/payday.rs (serde camelCase).
// Dates are ISO strings: YYYY-MM-DD.

export type Schedule = {
  firstDay: number
  secondDay: number
}

export type AdjustReason =
  | { kind: 'saturday'; date: string }
  | { kind: 'sunday'; date: string }
  | { kind: 'holiday'; date: string; name: string }

export type AdjustedPayday = {
  actualDate: string
  nominalDate: string
  wasAdjusted: boolean
  /** 1 = Monday ... 7 = Sunday */
  actualWeekday: number
  /** Skipped days, from the nominal date going backwards */
  reasons: AdjustReason[]
}

export type PaydayInfo = {
  today: string
  daysLeft: number
  nextPayday: AdjustedPayday
  previousPayday: AdjustedPayday
  /** 0 = just got paid, 1 = payday is today */
  progress: number
  holidayDataMissingForYear: boolean
}
