// ALL user-facing strings live in this file. Edit freely.
// `{n}` in a message is replaced with the number of days left.

import type { AdjustReason, Schedule } from './types'

// ---------------------------------------------------------------------------
// Countdown messages (2-3 variants per bucket, one is picked at random)
// ---------------------------------------------------------------------------

export type Bucket = 'today' | 'tomorrow' | 'fewDays' | 'week' | 'twoWeeks' | 'long'

export const messages: Record<Bucket, string[]> = {
  // 0 days
  today: [
    'SAHOD NA TODAY, bes! 🎉 Treat yourself, but like, hindi sobra ha.',
    'Payday na, sis! 💸 Check mo na ang bank app, literal.',
    'Ngayon na ’to, ate/kuya! Bayad-bayad muna, tapos konting saya lang ha.',
  ],
  // 1 day
  tomorrow: [
    'Bukas na ang sahod! Konting tiis na lang, literal.',
    'Isang tulog na lang, bes! Wag ka muna mag-checkout ngayon, ha.',
    'Bukas na, sis! Last na tiis na ’to, kaya mo yan.',
  ],
  // 2-3 days
  fewDays: [
    '{n} days na lang, sis! Kaya pa ’to, chill ka lang muna.',
    'Malapit na, bes! {n} days na lang, so hold on ka lang muna.',
    '{n} days na lang, diba? Konting hinga pa, almost there na tayo.',
  ],
  // 4-7 days
  week: [
    '{n} days til sahod. Pwede pa mag-milk tea, pero isa lang ha.',
    '{n} days to go, bes. Mag-budget muna, wag muna mag-impulse buy.',
    '{n} days na lang, so like, kaya pa mag-ulam ng masarap, pero sakto lang.',
  ],
  // 8-15 days
  twoWeeks: [
    '{n} days pa, diba. Budget-budget muna, wag muna mag-Shopee.',
    'Medyo matagal pa, {n} days. Mag-baon ka muna, sis, mas tipid.',
    '{n} days pa, bes, so like, ipon-ipon muna tayo.',
  ],
  // 16+ days
  long: [
    'Ay, {n} days pa. Hang in there, ate/kuya, kaya natin ’to.',
    '{n} days pa, literal. Pero okay lang, bes, kaya natin ’to.',
    'Haba pa ng {n} days, sis. Deep breath muna, tapos tuloy lang.',
  ],
}

export function bucketFor(daysLeft: number): Bucket {
  const n = Math.max(0, Math.floor(daysLeft))
  if (n === 0) return 'today'
  if (n === 1) return 'tomorrow'
  if (n <= 3) return 'fewDays'
  if (n <= 7) return 'week'
  if (n <= 15) return 'twoWeeks'
  return 'long'
}

export function fillDays(template: string, n: number): string {
  return template.replace(/\{n\}/g, String(n))
}

/** Pick a random message for the bucket and fill in `{n}`. `rng` is injectable for tests. */
export function pickMessage(daysLeft: number, rng: () => number = Math.random): string {
  const n = Math.max(0, Math.floor(daysLeft))
  const variants = messages[bucketFor(n)]
  const index = Math.min(variants.length - 1, Math.floor(rng() * variants.length))
  return fillDays(variants[index], n)
}

// ---------------------------------------------------------------------------
// Dates (formatting only, no date math: Rust gives us ISO strings and weekday numbers)
// ---------------------------------------------------------------------------

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

const WEEKDAYS = ['Lunes', 'Martes', 'Miyerkules', 'Huwebes', 'Biyernes', 'Sabado', 'Linggo']

/** '2026-12-14' -> 'Dec 14' */
export function formatShortDate(iso: string): string {
  const [, month, day] = iso.split('-').map(Number)
  return `${MONTHS[month - 1] ?? '?'} ${day}`
}

/** 1 (Monday) ... 7 (Sunday) -> Filipino weekday name */
export function weekdayName(weekday: number): string {
  return WEEKDAYS[weekday - 1] ?? ''
}

// ---------------------------------------------------------------------------
// Adjustment note ("Naka-advance ang sahod mo kasi ...")
// ---------------------------------------------------------------------------

export function describeReason(reason: AdjustReason): string {
  const date = formatShortDate(reason.date)
  switch (reason.kind) {
    case 'saturday':
      return `Sabado ang ${date}`
    case 'sunday':
      return `Linggo ang ${date}`
    case 'holiday':
      return `holiday ang ${date} (${reason.name})`
  }
}

function joinWithAt(parts: string[]): string {
  if (parts.length <= 1) return parts.join('')
  return `${parts.slice(0, -1).join(', ')} at ${parts[parts.length - 1]}`
}

export function adjustmentNote(reasons: AdjustReason[]): string {
  return `Naka-advance ang sahod mo kasi ${joinWithAt(reasons.map(describeReason))}.`
}

// ---------------------------------------------------------------------------
// Presets
// ---------------------------------------------------------------------------

export type Preset = { id: string; schedule: Schedule; label: string; hint: string }

export const presets: Preset[] = [
  {
    id: 'preset-15-30',
    // 31 means "last day of the month" when the month is shorter
    schedule: { firstDay: 15, secondDay: 31 },
    label: '15 & 30/31',
    hint: 'Ika-15 at katapusan ng buwan',
  },
  {
    id: 'preset-5-20',
    schedule: { firstDay: 5, secondDay: 20 },
    label: '5 & 20',
    hint: 'Ika-5 at ika-20',
  },
  {
    id: 'preset-10-25',
    schedule: { firstDay: 10, secondDay: 25 },
    label: '10 & 25',
    hint: 'Ika-10 at ika-25',
  },
]

// ---------------------------------------------------------------------------
// Screens
// ---------------------------------------------------------------------------

export const copy = {
  loading: 'Sandali lang…',

  setup: {
    title: 'Kelan Sahod?',
    subtitle: 'Sabihin mo lang kung kelan ka sinasahuran, ako na bahala mag-count.',
    question: 'Kelan ka ba sasahod?',
    customLabel: 'Custom',
    customHint: 'Ikaw ang pipili ng dalawang araw',
    firstDay: 'Unang araw',
    secondDay: 'Pangalawang araw',
    customNote: 'Kung 29, 30, o 31 ang pinili mo at mas maikli ang buwan, last day na lang ng buwan ang gagamitin.',
    errorRange: 'Pumili ng araw mula 1 hanggang 31.',
    errorSame: 'Dapat magkaiba ang dalawang araw, bes.',
    save: 'I-save',
    cancel: 'Bumalik',
  },

  home: {
    unit: (n: number) => (n === 0 ? 'ngayon na!' : n === 1 ? 'day na lang' : 'days na lang'),
    dateLine: (weekday: string, date: string) => `Sa ${weekday}, ${date}`,
    announce: (n: number, dateLine: string) =>
      n === 0 ? `Sahod na ngayon. ${dateLine}.` : `${n} araw na lang bago sumahod. ${dateLine}.`,
    progressLabel: 'Progress papunta sa susunod na sahod',
    lastPayday: (date: string) => `Huling sahod: ${date}`,
    holidayHint: 'Baka kulang pa ang holiday list para sa taon na ’to.',
    editSchedule: 'Palitan ang sahod schedule',
  },

  error: {
    message: 'Ay, may nag-error. Subukan ulit.',
    retry: 'Subukan ulit',
  },
}
