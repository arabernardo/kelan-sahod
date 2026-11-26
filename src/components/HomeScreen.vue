<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Confetti from './Confetti.vue'
import { adjustmentNote, copy, formatShortDate, language, monthName, pickMessage, weekdayName } from '../copy'
import type { PaydayInfo } from '../types'

const props = defineProps<{ info: PaydayInfo; compact?: boolean }>()
defineEmits<{ edit: []; settings: [] }>()

// Pick a new random message only when the number of days or the language changes,
// so the text doesn't flicker on every background refresh.
const message = ref('')
watch(
  () => [props.info.daysLeft, language.value] as const,
  ([days]) => {
    message.value = pickMessage(days)
  },
  { immediate: true },
)

const next = computed(() => props.info.nextPayday)

const dateLine = computed(() =>
  copy.home.dateLine(weekdayName(next.value.actualWeekday), formatShortDate(next.value.actualDate)),
)
const note = computed(() => (next.value.wasAdjusted ? adjustmentNote(next.value.reasons) : ''))
const percent = computed(() => Math.round(props.info.progress * 100))
const isPayday = computed(() => props.info.daysLeft === 0)

// Compact mode keeps the warnings but drops the "as of" line (and the month list, below).
const hasHints = computed(
  () =>
    props.info.holidayDataMissingForYear ||
    props.info.holidayListMayBeOutdated ||
    (!props.compact && !!props.info.holidayListUpdated),
)
const month = computed(() => monthName(props.info.today))
</script>

<template>
  <section class="home" :class="{ 'home--compact': compact }">
    <!-- Left column on wide windows: the countdown -->
    <div class="home__main">
      <Confetti v-if="isPayday" />

      <!-- Screen readers get one clear announcement; the big number is decorative. -->
      <p class="sr-only" aria-live="polite" aria-atomic="true">
        {{ copy.home.announce(info.daysLeft, dateLine) }}
      </p>

      <div class="hero" :class="{ 'hero--payday': isPayday }">
        <!-- On payday there is nothing to count, so no "0": just the celebration heading. -->
        <p v-if="isPayday" class="hero__payday" aria-hidden="true">{{ copy.home.paydayTitle }}</p>
        <template v-else>
          <p class="hero__number" aria-hidden="true">{{ info.daysLeft }}</p>
          <p class="hero__unit" aria-hidden="true">{{ copy.home.unit(info.daysLeft) }}</p>
        </template>
        <!-- On payday the date is today, so there is nothing to say about it. -->
        <p v-if="!isPayday" class="hero__date" aria-hidden="true">{{ dateLine }}</p>
      </div>

      <p v-if="note" class="note">{{ note }}</p>

      <p class="message">{{ message }}</p>
    </div>

    <!-- Right column on wide windows: progress, holidays, settings -->
    <div class="home__side">
    <div class="progress-block">
      <div
        class="progress"
        role="progressbar"
        :aria-label="copy.home.progressLabel"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="percent"
      >
        <div class="progress__fill" :style="{ width: percent + '%' }"></div>
      </div>
      <p class="progress__legend">
        <span>{{ copy.home.lastPayday(formatShortDate(info.previousPayday.actualDate)) }}</span>
        <span>{{ formatShortDate(next.actualDate) }}</span>
      </p>
    </div>

    <section v-if="!compact" class="month" aria-labelledby="month-title">
      <h2 id="month-title" class="month__title">{{ copy.home.holidaysTitle(month) }}</h2>

      <ul v-if="info.monthHolidays.length" class="month__list">
        <li
          v-for="holiday in info.monthHolidays"
          :key="holiday.date"
          class="holiday"
          :class="`holiday--${holiday.state}`"
        >
          <span class="holiday__date">{{ formatShortDate(holiday.date) }}</span>
          <span class="holiday__body">
            <span class="holiday__name">{{ holiday.name }}</span>
            <span class="holiday__day">{{ weekdayName(holiday.weekday) }}</span>
          </span>
          <span v-if="holiday.state === 'upcoming'" class="tag">{{ copy.home.tagUpcoming }}</span>
          <span v-else-if="holiday.state === 'today'" class="tag">{{ copy.home.tagToday }}</span>
          <span v-else class="sr-only">{{ copy.home.pastLabel }}</span>
        </li>
      </ul>
      <p v-else class="hint">{{ copy.home.noHolidays(month) }}</p>
    </section>

    <div v-if="hasHints" class="hints">
      <p v-if="info.holidayDataMissingForYear" class="hint">{{ copy.home.holidayHint }}</p>
      <p v-else-if="info.holidayListMayBeOutdated" class="hint">{{ copy.home.holidayMaybeOutdated }}</p>
      <p v-if="info.holidayListUpdated && !compact" class="hint hint--quiet">
        {{ copy.home.holidayListAsOf(formatShortDate(info.holidayListUpdated)) }}
      </p>
    </div>

    <div class="home__actions">
      <button type="button" class="btn btn--quiet" @click="$emit('edit')">
        {{ copy.home.editSchedule }}
      </button>
      <button type="button" class="btn btn--quiet" @click="$emit('settings')">
        {{ copy.home.settings }}
      </button>
    </div>
    </div>
  </section>
</template>
