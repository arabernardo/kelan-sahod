import { afterEach, describe, expect, it } from 'vitest'
import { computed, nextTick, watchEffect } from 'vue'
import {
  adjustmentNote,
  bucketFor,
  copies,
  copy,
  describeReason,
  formatShortDate,
  language,
  monthName,
  pickMessage,
  setLanguage,
  weekdayName,
} from './copy'
import type { AdjustReason, Language } from './types'

const LANGUAGES = Object.keys(copies) as Language[]

// Always go back to Tagalog (the default) so tests don't affect each other.
afterEach(() => setLanguage('tl'))

/** The "shape" of a copy object: every key, and the type of every value. */
function shape(value: unknown): unknown {
  if (Array.isArray(value)) return ['array', value.length > 0 ? typeof value[0] : 'empty']
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).map(([key, v]) => [key, shape(v)]))
  }
  return typeof value
}

describe('languages', () => {
  it('has Tagalog and English', () => {
    expect(LANGUAGES.sort()).toEqual(['en', 'tl'])
  })

  it('every language has exactly the same keys as Tagalog (no missing translations)', () => {
    for (const lang of LANGUAGES) {
      expect(shape(copies[lang]), lang).toEqual(shape(copies.tl))
    }
  })

  it('no string is empty in any language', () => {
    const walk = (value: unknown, path: string) => {
      if (typeof value === 'string') expect(value.trim().length, path).toBeGreaterThan(0)
      else if (Array.isArray(value)) value.forEach((v, i) => walk(v, `${path}[${i}]`))
      else if (value && typeof value === 'object') {
        Object.entries(value).forEach(([k, v]) => walk(v, `${path}.${k}`))
      }
    }
    for (const lang of LANGUAGES) walk(copies[lang], lang)
  })

  it('`copy` follows the language that is set', () => {
    expect(language.value).toBe('tl')
    expect(copy.setup.save).toBe('I-save')
    setLanguage('en')
    expect(copy.setup.save).toBe('Save')
    expect(copy.home.unit(5)).toBe('days to go')
    setLanguage('tl')
    expect(copy.home.unit(5)).toBe('days na lang')
  })

  it('the language name labels are the same in both languages', () => {
    expect(copies.tl.settings.tagalogLabel).toBe('Tagalog')
    expect(copies.en.settings.tagalogLabel).toBe('Tagalog')
    expect(copies.tl.settings.englishLabel).toBe('English')
    expect(copies.en.settings.englishLabel).toBe('English')
  })

  it('presets keep the same ids, days and labels; only hints are translated', () => {
    const tl = copies.tl.presets
    const en = copies.en.presets
    expect(en.map((p) => [p.id, p.schedule, p.label])).toEqual(tl.map((p) => [p.id, p.schedule, p.label]))
    expect(en.map((p) => p.hint)).not.toEqual(tl.map((p) => p.hint))
    expect(tl[0].schedule).toEqual({ firstDay: 15, secondDay: 31 })
  })
})

describe('bucketFor', () => {
  it('picks the right bucket for each range', () => {
    expect(bucketFor(0)).toBe('today')
    expect(bucketFor(1)).toBe('tomorrow')
    expect(bucketFor(2)).toBe('fewDays')
    expect(bucketFor(3)).toBe('fewDays')
    expect(bucketFor(4)).toBe('week')
    expect(bucketFor(7)).toBe('week')
    expect(bucketFor(8)).toBe('twoWeeks')
    expect(bucketFor(15)).toBe('twoWeeks')
    expect(bucketFor(16)).toBe('long')
    expect(bucketFor(31)).toBe('long')
  })

  it('never goes below the "today" bucket', () => {
    expect(bucketFor(-3)).toBe('today')
  })
})

describe('pickMessage (both languages)', () => {
  it('uses a message from the matching bucket for every range', () => {
    const cases = [
      [0, 'today'],
      [1, 'tomorrow'],
      [2, 'fewDays'],
      [3, 'fewDays'],
      [5, 'week'],
      [12, 'twoWeeks'],
      [20, 'long'],
    ] as const
    for (const lang of LANGUAGES) {
      for (const [days, bucket] of cases) {
        const variants = copies[lang].messages[bucket].map((m) => m.replace(/\{n\}/g, String(days)))
        for (let i = 0; i < variants.length; i++) {
          // rng that lands on variant i
          expect(pickMessage(days, () => (i + 0.5) / variants.length, lang), `${lang} ${days}`).toBe(variants[i])
        }
      }
    }
  })

  it('fills in {n} and never leaves a placeholder behind', () => {
    for (const lang of LANGUAGES) {
      expect(pickMessage(5, () => 0, lang)).toContain('5')
      for (const days of [0, 1, 2, 3, 4, 7, 8, 15, 16, 40]) {
        for (let i = 0; i < 3; i++) {
          expect(pickMessage(days, () => i / 3, lang)).not.toContain('{n}')
        }
      }
    }
  })

  it('has 2-3 variants per bucket in every language', () => {
    for (const lang of LANGUAGES) {
      for (const variants of Object.values(copies[lang].messages)) {
        expect(variants.length).toBeGreaterThanOrEqual(2)
        expect(variants.length).toBeLessThanOrEqual(3)
      }
    }
  })

  it('uses the current language by default', () => {
    expect(pickMessage(0, () => 0)).toBe(copies.tl.messages.today[0])
    setLanguage('en')
    expect(pickMessage(0, () => 0)).toBe(copies.en.messages.today[0])
  })

  it('keeps the sample Tagalog copy from the brief', () => {
    expect(copies.tl.messages.today).toContain('SAHOD NA TODAY, bes! 🎉 Treat yourself, but like, hindi sobra ha.')
    expect(copies.tl.messages.tomorrow).toContain('Bukas na ang sahod! Konting tiis na lang, literal.')
  })
})

describe('dates', () => {
  it('formats ISO dates without timezone shifts (same in every language)', () => {
    expect(formatShortDate('2026-12-14')).toBe('Dec 14')
    expect(formatShortDate('2026-01-01')).toBe('Jan 1')
    expect(formatShortDate('2028-02-29')).toBe('Feb 29')
  })

  it('names the month in full', () => {
    expect(monthName('2026-12-14')).toBe('December')
    expect(monthName('2027-01-01')).toBe('January')
    expect(monthName('2026-05-31')).toBe('May')
  })

  it('names weekdays from 1 (Monday) to 7 (Sunday) in each language', () => {
    expect(weekdayName(1, 'tl')).toBe('Lunes')
    expect(weekdayName(5, 'tl')).toBe('Biyernes')
    expect(weekdayName(7, 'tl')).toBe('Linggo')
    expect(weekdayName(1, 'en')).toBe('Monday')
    expect(weekdayName(5, 'en')).toBe('Friday')
    expect(weekdayName(7, 'en')).toBe('Sunday')
  })

  it('follows the current language when none is given', () => {
    expect(weekdayName(5)).toBe('Biyernes')
    setLanguage('en')
    expect(weekdayName(5)).toBe('Friday')
  })
})

describe('adjustment note', () => {
  const saturday: AdjustReason = { kind: 'saturday', date: '2025-11-15' }
  const sunday: AdjustReason = { kind: 'sunday', date: '2025-12-14' }
  const holiday: AdjustReason = { kind: 'holiday', date: '2025-12-30', name: 'Rizal Day' }

  it('describes each reason type in Tagalog', () => {
    expect(describeReason(saturday, 'tl')).toBe('Sabado ang Nov 15')
    expect(describeReason(sunday, 'tl')).toBe('Linggo ang Dec 14')
    expect(describeReason(holiday, 'tl')).toBe('holiday ang Dec 30 (Rizal Day)')
  })

  it('describes each reason type in English', () => {
    expect(describeReason(saturday, 'en')).toBe('Nov 15 is a Saturday')
    expect(describeReason(sunday, 'en')).toBe('Dec 14 is a Sunday')
    expect(describeReason(holiday, 'en')).toBe('Dec 30 is a holiday (Rizal Day)')
  })

  it('builds the Tagalog note', () => {
    expect(adjustmentNote([saturday], 'tl')).toBe('Naka-advance ang sahod mo kasi Sabado ang Nov 15.')
    expect(adjustmentNote([holiday], 'tl')).toBe(
      'Naka-advance ang sahod mo kasi holiday ang Dec 30 (Rizal Day).',
    )
  })

  it('builds the English note', () => {
    expect(adjustmentNote([saturday], 'en')).toBe('Your payday was moved earlier because Nov 15 is a Saturday.')
    expect(adjustmentNote([holiday], 'en')).toBe(
      'Your payday was moved earlier because Dec 30 is a holiday (Rizal Day).',
    )
  })

  it('joins several reasons with commas and "at" / "and"', () => {
    const friday: AdjustReason = { kind: 'holiday', date: '2025-11-14', name: 'Test Holiday' }
    expect(adjustmentNote([saturday, friday], 'tl')).toBe(
      'Naka-advance ang sahod mo kasi Sabado ang Nov 15 at holiday ang Nov 14 (Test Holiday).',
    )
    expect(adjustmentNote([saturday, friday], 'en')).toBe(
      'Your payday was moved earlier because Nov 15 is a Saturday and Nov 14 is a holiday (Test Holiday).',
    )
    const monday: AdjustReason = { kind: 'holiday', date: '2025-12-15', name: 'Test' }
    const sat: AdjustReason = { kind: 'saturday', date: '2025-12-13' }
    expect(adjustmentNote([monday, sunday, sat], 'tl')).toBe(
      'Naka-advance ang sahod mo kasi holiday ang Dec 15 (Test), Linggo ang Dec 14 at Sabado ang Dec 13.',
    )
    expect(adjustmentNote([monday, sunday, sat], 'en')).toBe(
      'Your payday was moved earlier because Dec 15 is a holiday (Test), Dec 14 is a Sunday and Dec 13 is a Saturday.',
    )
  })
})

describe('screen copy', () => {
  it('builds the month title and empty text in both languages', () => {
    expect(copies.tl.home.holidaysTitle('December')).toBe('Mga holiday ngayong December')
    expect(copies.tl.home.noHolidays('March')).toBe('Walang holiday ngayong March.')
    expect(copies.en.home.holidaysTitle('December')).toBe('Holidays this December')
    expect(copies.en.home.noHolidays('March')).toBe('No holidays this March.')
  })

  it('has a big payday heading in both languages (shown instead of the number 0)', () => {
    expect(copies.tl.home.paydayTitle).toBe('Sahod na!')
    expect(copies.en.home.paydayTitle).toBe('Payday!')
  })

  it('has the upcoming and today tags', () => {
    for (const lang of LANGUAGES) {
      expect(copies[lang].home.tagUpcoming).toBe('Upcoming')
      expect(copies[lang].home.tagToday).toBe('Today')
    }
  })

  it('builds the countdown unit and the screen reader line', () => {
    expect(copies.en.home.unit(1)).toBe('day to go')
    expect(copies.en.home.unit(5)).toBe('days to go')
    expect(copies.tl.home.unit(1)).toBe('day na lang')
    // On payday the date is today, so it is not read out
    expect(copies.en.home.announce(0, 'On Friday, Dec 14')).toBe('It’s payday today.')
    expect(copies.tl.home.announce(0, 'Sa Biyernes, Dec 14')).toBe('Sahod na ngayon.')
    expect(copies.en.home.announce(1, 'On Friday, Dec 14')).toBe('1 day until payday. On Friday, Dec 14.')
    expect(copies.en.home.announce(4, 'On Friday, Dec 14')).toBe('4 days until payday. On Friday, Dec 14.')
    expect(copies.tl.home.announce(4, 'Sa Biyernes, Dec 14')).toBe('4 araw na lang bago sumahod. Sa Biyernes, Dec 14.')
  })

  it('builds the about lines with the author in both languages', () => {
    for (const lang of LANGUAGES) {
      const about = copies[lang].about
      expect(about.appLine('0.1.2')).toBe('Kelan Sahod v0.1.2')
      expect(about.madeBy(about.author)).toContain(about.author)
      expect(about.author.length).toBeGreaterThan(0)
    }
    expect(copies.en.about.madeBy('Ara')).toBe('Made by Ara')
    expect(copies.tl.about.madeBy('Ara')).toBe('Gawa ni Ara')
  })

  it('has labels for both layout modes in both languages', () => {
    for (const lang of LANGUAGES) {
      expect(copies[lang].settings.compactLabel).toBe('Compact')
      expect(copies[lang].settings.fullLabel).toBe('Full')
      expect(copies[lang].home.settings).toBe('Settings')
    }
  })
})

describe('switching language live', () => {
  it('effects that read `copy` re-run when the language changes', async () => {
    const seen: string[] = []
    const stop = watchEffect(() => {
      seen.push(copy.setup.save)
    })
    await nextTick()
    setLanguage('en')
    await nextTick()
    setLanguage('tl')
    await nextTick()
    stop()
    expect(seen).toEqual(['I-save', 'Save', 'I-save'])
  })

  it('computed values built with the helpers update too', () => {
    const day = computed(() => weekdayName(5))
    const note = computed(() => adjustmentNote([{ kind: 'saturday', date: '2025-11-15' }]))
    expect(day.value).toBe('Biyernes')
    expect(note.value).toBe('Naka-advance ang sahod mo kasi Sabado ang Nov 15.')
    setLanguage('en')
    expect(day.value).toBe('Friday')
    expect(note.value).toBe('Your payday was moved earlier because Nov 15 is a Saturday.')
  })
})
