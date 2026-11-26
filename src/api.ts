import { invoke } from '@tauri-apps/api/core'
import { load, type Store } from '@tauri-apps/plugin-store'
import type { PaydayInfo, Schedule } from './types'

const SCHEDULE_KEY = 'schedule'
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
