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
  /** The movable holiday list has no entry at all for the current year */
  holidayDataMissingForYear: boolean
  /** Payday is within ~10 days and the list was not confirmed against GitHub in the last 24h */
  holidayListMayBeOutdated: boolean
  /** Date the holiday list in use was last edited (YYYY-MM-DD) */
  holidayListUpdated: string | null
  holidayListSource: 'embedded' | 'remote'
}
