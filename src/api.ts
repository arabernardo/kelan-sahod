import { invoke } from '@tauri-apps/api/core'
import { load, type Store } from '@tauri-apps/plugin-store'
import type { Language, LayoutMode, PaydayInfo, Schedule } from './types'

const SCHEDULE_KEY = 'schedule'
const LAYOUT_KEY = 'layout'
const LANGUAGE_KEY = 'language'
let storePromise: Promise<Store> | null = null

function getStore(): Promise<Store> {
  if (!storePromise) {
    storePromise = load('settings.json', { defaults: {}, autoSave: false }).catch((err) => {
      storePromise = null // allow a retry after a failure
      throw err
    })
  }
  return storePromise
}

function isSchedule(value: unknown): value is Schedule {
  if (typeof value !== 'object' || value === null) return false
  const { firstDay, secondDay } = value as Record<string, unknown>
  return (
    Number.isInteger(firstDay) &&
    Number.isInteger(secondDay) &&
    (firstDay as number) >= 1 &&
    (firstDay as number) <= 31 &&
    (secondDay as number) >= 1 &&
    (secondDay as number) <= 31 &&
    firstDay !== secondDay
  )
}

/** Returns the saved schedule, or null on first launch (or if the saved value is unusable). */
export async function loadSchedule(): Promise<Schedule | null> {
  const store = await getStore()
  const saved = await store.get<unknown>(SCHEDULE_KEY)
  return isSchedule(saved) ? saved : null
}

export async function saveSchedule(schedule: Schedule): Promise<void> {
  const store = await getStore()
  await store.set(SCHEDULE_KEY, schedule)
  await store.save()
}

/** All date math happens in Rust. */
export function getPaydayInfo(schedule: Schedule): Promise<PaydayInfo> {
  return invoke<PaydayInfo>('get_payday_info', { schedule })
}

/**
 * Asks Rust to download the latest holiday list from GitHub (cached on disk).
 * Resolves true when the list was confirmed, so the caller can reload the countdown.
 * Offline or failing is fine: the cached or built-in list keeps working.
 */
export async function refreshHolidays(): Promise<boolean> {
  const result = await invoke<{ synced: boolean }>('refresh_holidays')
  return result.synced
}

/** The saved layout. Anything missing or unknown means the full layout. */
export async function loadLayout(): Promise<LayoutMode> {
  const store = await getStore()
  const saved = await store.get<unknown>(LAYOUT_KEY)
  return saved === 'compact' ? 'compact' : 'full'
}

export async function saveLayout(mode: LayoutMode): Promise<void> {
  const store = await getStore()
  await store.set(LAYOUT_KEY, mode)
  await store.save()
}

/** Asks Rust to resize the window for this layout (sizes live in src-tauri/src/layout.rs). */
export function applyLayout(mode: LayoutMode): Promise<void> {
  return invoke<void>('set_layout_mode', { mode })
}

/** The saved language. Anything missing or unknown means Tagalog. */
export async function loadLanguage(): Promise<Language> {
  const store = await getStore()
  const saved = await store.get<unknown>(LANGUAGE_KEY)
  return saved === 'en' ? 'en' : 'tl'
}

export async function saveLanguage(language: Language): Promise<void> {
  const store = await getStore()
  await store.set(LANGUAGE_KEY, language)
  await store.save()
}
