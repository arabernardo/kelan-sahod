import { describe, expect, it } from 'vitest'
import {
  adjustmentNote,
  bucketFor,
  copy,
  describeReason,
  formatShortDate,
  messages,
  pickMessage,
  weekdayName,
} from './copy'
import type { AdjustReason } from './types'

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

describe('pickMessage', () => {
  it('uses a message from the matching bucket for every range', () => {
    const cases: Array<[number, keyof typeof messages]> = [
      [0, 'today'],
      [1, 'tomorrow'],
      [2, 'fewDays'],
      [3, 'fewDays'],
      [5, 'week'],
      [12, 'twoWeeks'],
      [20, 'long'],
    ]
    for (const [days, bucket] of cases) {
      const variants = messages[bucket].map((m) => m.replace(/\{n\}/g, String(days)))
      for (let i = 0; i < variants.length; i++) {
        // rng that lands on variant i
        const result = pickMessage(days, () => (i + 0.5) / variants.length)
        expect(result).toBe(variants[i])
      }
    }
  })

  it('fills in {n}', () => {
    const result = pickMessage(5, () => 0)
    expect(result).toContain('5')
    expect(result).not.toContain('{n}')
  })

  it('never leaves a {n} placeholder in any variant', () => {
    for (const days of [0, 1, 2, 3, 4, 7, 8, 15, 16, 40]) {
      for (let i = 0; i < 3; i++) {
        expect(pickMessage(days, () => i / 3)).not.toContain('{n}')
      }
    }
  })

  it('has 2-3 variants per bucket', () => {
    for (const variants of Object.values(messages)) {
      expect(variants.length).toBeGreaterThanOrEqual(2)
      expect(variants.length).toBeLessThanOrEqual(3)
    }
  })

  it('keeps the sample copy from the brief', () => {
    expect(messages.today).toContain('SAHOD NA TODAY, bes! 🎉 Treat yourself, but like, hindi sobra ha.')
    expect(messages.tomorrow).toContain('Bukas na ang sahod! Konting tiis na lang, literal.')
  })
})

describe('dates', () => {
  it('formats ISO dates without timezone shifts', () => {
    expect(formatShortDate('2026-12-14')).toBe('Dec 14')
    expect(formatShortDate('2026-01-01')).toBe('Jan 1')
    expect(formatShortDate('2028-02-29')).toBe('Feb 29')
  })

  it('names weekdays from 1 (Monday) to 7 (Sunday)', () => {
    expect(weekdayName(1)).toBe('Lunes')
    expect(weekdayName(5)).toBe('Biyernes')
    expect(weekdayName(7)).toBe('Linggo')
  })
})

describe('adjustment note', () => {
  const saturday: AdjustReason = { kind: 'saturday', date: '2025-11-15' }
  const sunday: AdjustReason = { kind: 'sunday', date: '2025-12-14' }
  const holiday: AdjustReason = { kind: 'holiday', date: '2025-12-30', name: 'Rizal Day' }

  it('describes each reason type', () => {
    expect(describeReason(saturday)).toBe('Sabado ang Nov 15')
    expect(describeReason(sunday)).toBe('Linggo ang Dec 14')
    expect(describeReason(holiday)).toBe('holiday ang Dec 30 (Rizal Day)')
  })

  it('builds the note for one reason', () => {
    expect(adjustmentNote([saturday])).toBe('Naka-advance ang sahod mo kasi Sabado ang Nov 15.')
    expect(adjustmentNote([holiday])).toBe(
      'Naka-advance ang sahod mo kasi holiday ang Dec 30 (Rizal Day).',
    )
  })

  it('joins several reasons with commas and "at"', () => {
    const friday: AdjustReason = { kind: 'holiday', date: '2025-11-14', name: 'Test Holiday' }
    expect(adjustmentNote([saturday, friday])).toBe(
      'Naka-advance ang sahod mo kasi Sabado ang Nov 15 at holiday ang Nov 14 (Test Holiday).',
    )
    const monday: AdjustReason = { kind: 'holiday', date: '2025-12-15', name: 'Test' }
    const sat: AdjustReason = { kind: 'saturday', date: '2025-12-13' }
    expect(adjustmentNote([monday, sunday, sat])).toBe(
      'Naka-advance ang sahod mo kasi holiday ang Dec 15 (Test), Linggo ang Dec 14 at Sabado ang Dec 13.',
    )
  })
})

describe('holiday list hints', () => {
  it('has the outdated-list hint and the "as of" line', () => {
    expect(copy.home.holidayMaybeOutdated).toBe('Baka may bagong holiday na hindi pa nakalista.')
    expect(copy.home.holidayListAsOf(formatShortDate('2026-11-26'))).toBe('Holiday list as of Nov 26')
  })
})
