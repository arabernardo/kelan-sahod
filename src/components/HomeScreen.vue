<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Confetti from './Confetti.vue'
import { adjustmentNote, copy, formatShortDate, pickMessage, weekdayName } from '../copy'
import type { PaydayInfo } from '../types'

const props = defineProps<{ info: PaydayInfo }>()
defineEmits<{ edit: [] }>()

// Pick a new random message only when the number of days changes,
// so the text doesn't flicker on every background refresh.
const message = ref('')
watch(
  () => props.info.daysLeft,
  (days) => {
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
</script>

<template>
  <section class="home">
    <Confetti v-if="isPayday" />

    <!-- Screen readers get one clear announcement; the big number is decorative. -->
    <p class="sr-only" aria-live="polite" aria-atomic="true">
      {{ copy.home.announce(info.daysLeft, dateLine) }}
    </p>

    <div class="hero" :class="{ 'hero--payday': isPayday }">
      <p class="hero__number" aria-hidden="true">{{ info.daysLeft }}</p>
      <p class="hero__unit" aria-hidden="true">{{ copy.home.unit(info.daysLeft) }}</p>
      <p class="hero__date" aria-hidden="true">{{ dateLine }}</p>
    </div>

    <p v-if="note" class="note">{{ note }}</p>

    <p class="message">{{ message }}</p>

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

    <p v-if="info.holidayDataMissingForYear" class="hint">{{ copy.home.holidayHint }}</p>

    <button type="button" class="btn btn--quiet home__edit" @click="$emit('edit')">
      {{ copy.home.editSchedule }}
    </button>
  </section>
</template>
