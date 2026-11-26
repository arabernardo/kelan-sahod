// ALL user-facing strings live in this file, in every language. Edit freely.
//
// - `tl` is Tagalog (Filipino with a conyo / Taglish tone), `en` is English.
// - `en` is typed as `Copy` (the shape of `tl`), so a missing translation is a type error.
// - `copy` always points at the language chosen in Settings, so components just write `copy.home.settings`.
// - `{n}` in a countdown message is replaced with the number of days left.
//
// To add a language: add its code to `Language` in types.ts, write a full object like `tl` below,
// add it to `copies`, and add a choice for it in SettingsScreen.vue.

import { ref } from 'vue'
import type { AdjustReason, Language, Schedule } from './types'

/** The language shown right now. Settings changes it; the choice is saved by the app. */
export const language = ref<Language>('tl')

export function setLanguage(next: Language): void {
  language.value = next
}

const AUTHOR = 'Arianna Maouna Bernardo' // also in package.json, src-tauri/Cargo.toml and tauri.conf.json

export type Bucket = 'today' | 'tomorrow' | 'fewDays' | 'week' | 'twoWeeks' | 'long'

export type Preset = { id: string; schedule: Schedule; label: string; hint: string }

// The presets are the same in every language; only the hint text is translated.
function makePresets(hints: [string, string, string]): Preset[] {
  return [
    {
      id: 'preset-15-30',
      // 31 means "last day of the month" when the month is shorter
      schedule: { firstDay: 15, secondDay: 31 },
      label: '15 & 30/31',
      hint: hints[0],
    },
    { id: 'preset-5-20', schedule: { firstDay: 5, secondDay: 20 }, label: '5 & 20', hint: hints[1] },
    { id: 'preset-10-25', schedule: { firstDay: 10, secondDay: 25 }, label: '10 & 25', hint: hints[2] },
  ]
}

// ---------------------------------------------------------------------------
// Tagalog (conyo)
// ---------------------------------------------------------------------------

const tl = {
  loading: 'Sandali lang…',

  // Countdown messages (2-3 variants per bucket, one is picked at random)
  messages: {
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
  } as Record<Bucket, string[]>,

  weekdays: ['Lunes', 'Martes', 'Miyerkules', 'Huwebes', 'Biyernes', 'Sabado', 'Linggo'],

  // Adjustment note ("Naka-advance ang sahod mo kasi ...")
  and: 'at',
  reason: {
    saturday: (date: string) => `Sabado ang ${date}`,
    sunday: (date: string) => `Linggo ang ${date}`,
    holiday: (date: string, name: string) => `holiday ang ${date} (${name})`,
  },
  adjustmentNote: (reasons: string) => `Naka-advance ang sahod mo kasi ${reasons}.`,

  presets: makePresets(['Ika-15 at katapusan ng buwan', 'Ika-5 at ika-20', 'Ika-10 at ika-25']),

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
    // Big heading on payday (instead of the number 0)
    paydayTitle: 'Sahod na!',
    unit: (n: number): string => (n === 1 ? 'day na lang' : 'days na lang'),
    dateLine: (weekday: string, date: string) => `Sa ${weekday}, ${date}`,
    announce: (n: number, dateLine: string) =>
      n === 0 ? 'Sahod na ngayon.' : `${n} araw na lang bago sumahod. ${dateLine}.`,
    progressLabel: 'Progress papunta sa susunod na sahod',
    lastPayday: (date: string) => `Huling sahod: ${date}`,
    holidayHint: 'Baka kulang pa ang holiday list para sa taon na ’to.',
    holidayMaybeOutdated: 'Baka may bagong holiday na hindi pa nakalista.',
    holidayListAsOf: (date: string) => `Holiday list as of ${date}`,
    holidaysTitle: (month: string) => `Mga holiday ngayong ${month}`,
    noHolidays: (month: string) => `Walang holiday ngayong ${month}.`,
    tagUpcoming: 'Upcoming',
    tagToday: 'Today',
    // Read by screen readers for finished holidays (they are only greyed out visually)
    pastLabel: 'tapos na',
    editSchedule: 'Palitan ang sahod schedule',
    settings: 'Settings',
  },

  settings: {
    title: 'Settings',
    layoutLegend: 'Layout',
    compactLabel: 'Compact',
    compactHint: 'Maliit na window: countdown at sahod date lang.',
    fullLabel: 'Full',
    fullHint: 'Buong layout: kasama ang mga holiday ngayong buwan at iba pang detalye.',
    languageLegend: 'Wika',
    tagalogLabel: 'Tagalog',
    tagalogHint: 'Filipino na may konting conyo.',
    englishLabel: 'English',
    englishHint: 'Plain English, walang conyo.',
    back: 'Bumalik',
  },

  about: {
    title: 'About',
    appLine: (version: string) => `Kelan Sahod v${version}`,
    author: AUTHOR,
    madeBy: (name: string) => `Gawa ni ${name}`,
  },

  error: {
    message: 'Ay, may nag-error. Subukan ulit.',
    retry: 'Subukan ulit',
  },
}

export type Copy = typeof tl

// ---------------------------------------------------------------------------
// English
// ---------------------------------------------------------------------------

const en: Copy = {
  loading: 'Just a sec…',

  messages: {
    today: [
      'PAYDAY TODAY! 🎉 Treat yourself, but maybe not too much.',
      'It’s payday! 💸 Go check that bank app.',
      'Today’s the day! Pay the bills first, then have a little fun.',
    ],
    tomorrow: [
      'Payday is tomorrow! Just hang on a little longer.',
      'One more sleep! Maybe skip the online checkout tonight.',
      'Tomorrow, finally. You’ve got this.',
    ],
    fewDays: [
      '{n} days to go! You can do this, stay calm for now.',
      'Almost there! Just {n} more days.',
      '{n} days left. Take a deep breath, we’re close.',
    ],
    week: [
      '{n} days until payday. One milk tea is fine, but just one.',
      '{n} days to go. Budget mode on, no impulse buys yet.',
      '{n} days left. You can still have a nice meal, just keep it reasonable.',
    ],
    twoWeeks: [
      '{n} days to go. Budget first, and stay away from Shopee for now.',
      'Still a while: {n} days. Packing lunch will save you.',
      '{n} days left. Time to save a little.',
    ],
    long: [
      'Ugh, {n} days to go. Hang in there, we’ll get through it.',
      '{n} days left, honestly. But it’s okay, we’ve got this.',
      '{n} days is a long way off. Deep breath, then keep going.',
    ],
  },

  weekdays: ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'],

  and: 'and',
  reason: {
    saturday: (date: string) => `${date} is a Saturday`,
    sunday: (date: string) => `${date} is a Sunday`,
    holiday: (date: string, name: string) => `${date} is a holiday (${name})`,
  },
  adjustmentNote: (reasons: string) => `Your payday was moved earlier because ${reasons}.`,

  presets: makePresets([
    '15th and the last day of the month',
    '5th and 20th',
    '10th and 25th',
  ]),

  setup: {
    title: 'When’s payday?',
    subtitle: 'Tell me your pay schedule and I’ll do the counting.',
    question: 'When do you get paid?',
    customLabel: 'Custom',
    customHint: 'Pick your own two days',
    firstDay: 'First day',
    secondDay: 'Second day',
    customNote: 'If you pick 29, 30 or 31 and the month is shorter, the last day of the month is used.',
    errorRange: 'Pick a day from 1 to 31.',
    errorSame: 'The two days must be different.',
    save: 'Save',
    cancel: 'Back',
  },

  home: {
    paydayTitle: 'Payday!',
    unit: (n: number): string => (n === 1 ? 'day to go' : 'days to go'),
    dateLine: (weekday: string, date: string) => `On ${weekday}, ${date}`,
    announce: (n: number, dateLine: string) =>
      n === 0 ? 'It’s payday today.' : `${n} ${n === 1 ? 'day' : 'days'} until payday. ${dateLine}.`,
    progressLabel: 'Progress to the next payday',
    lastPayday: (date: string) => `Last payday: ${date}`,
    holidayHint: 'The holiday list may be incomplete for this year.',
    holidayMaybeOutdated: 'A newly declared holiday may not be listed yet.',
    holidayListAsOf: (date: string) => `Holiday list as of ${date}`,
    holidaysTitle: (month: string) => `Holidays this ${month}`,
    noHolidays: (month: string) => `No holidays this ${month}.`,
    tagUpcoming: 'Upcoming',
    tagToday: 'Today',
    pastLabel: 'already passed',
    editSchedule: 'Change pay schedule',
    settings: 'Settings',
  },

  settings: {
    title: 'Settings',
    layoutLegend: 'Layout',
    compactLabel: 'Compact',
    compactHint: 'Small window: just the countdown and payday date.',
    fullLabel: 'Full',
    fullHint: 'Full layout: includes this month’s holidays and more details.',
    languageLegend: 'Language',
    tagalogLabel: 'Tagalog',
    tagalogHint: 'Filipino with a bit of conyo.',
    englishLabel: 'English',
    englishHint: 'Plain English, no conyo.',
    back: 'Back',
  },

  about: {
    title: 'About',
    appLine: (version: string) => `Kelan Sahod v${version}`,
    author: AUTHOR,
    madeBy: (name: string) => `Made by ${name}`,
  },

  error: {
    message: 'Oops, something went wrong. Please try again.',
    retry: 'Try again',
  },
}

export const copies: Record<Language, Copy> = { tl, en }

/**
 * The strings for the language chosen in Settings. Reading from it inside a component
 * re-renders that component when the language changes.
 */
export const copy: Copy = new Proxy(tl, {
  get: (_target, key) => Reflect.get(copies[language.value], key),
})

// ---------------------------------------------------------------------------
// Countdown messages
// ---------------------------------------------------------------------------

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

/** Pick a random message for the bucket and fill in `{n}`. `rng` and `lang` are injectable for tests. */
export function pickMessage(
  daysLeft: number,
  rng: () => number = Math.random,
  lang: Language = language.value,
): string {
  const n = Math.max(0, Math.floor(daysLeft))
  const variants = copies[lang].messages[bucketFor(n)]
  const index = Math.min(variants.length - 1, Math.floor(rng() * variants.length))
  return fillDays(variants[index], n)
}

// ---------------------------------------------------------------------------
// Dates (formatting only, no date math: Rust gives us ISO strings and weekday numbers).
// Months are written the same way in every language: "Dec 14", "December".
// ---------------------------------------------------------------------------

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']

const MONTHS_FULL = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
]

/** '2026-12-14' -> 'Dec 14' */
export function formatShortDate(iso: string): string {
  const [, month, day] = iso.split('-').map(Number)
  return `${MONTHS[month - 1] ?? '?'} ${day}`
}

/** '2026-12-14' -> 'December' */
export function monthName(iso: string): string {
  const month = Number(iso.split('-')[1])
  return MONTHS_FULL[month - 1] ?? ''
}

/** 1 (Monday) ... 7 (Sunday) -> weekday name in the current language */
export function weekdayName(weekday: number, lang: Language = language.value): string {
  return copies[lang].weekdays[weekday - 1] ?? ''
}

// ---------------------------------------------------------------------------
// Adjustment note
// ---------------------------------------------------------------------------

export function describeReason(reason: AdjustReason, lang: Language = language.value): string {
  const c = copies[lang]
  const date = formatShortDate(reason.date)
  switch (reason.kind) {
    case 'saturday':
      return c.reason.saturday(date)
    case 'sunday':
      return c.reason.sunday(date)
    case 'holiday':
      return c.reason.holiday(date, reason.name)
  }
}

function joinWithAnd(parts: string[], and: string): string {
  if (parts.length <= 1) return parts.join('')
  return `${parts.slice(0, -1).join(', ')} ${and} ${parts[parts.length - 1]}`
}

export function adjustmentNote(reasons: AdjustReason[], lang: Language = language.value): string {
  const c = copies[lang]
  return c.adjustmentNote(joinWithAnd(reasons.map((r) => describeReason(r, lang)), c.and))
}
