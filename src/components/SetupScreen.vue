<script setup lang="ts">
import { computed, ref } from 'vue'
import { copy } from '../copy'
import type { Schedule } from '../types'

const props = defineProps<{ initial: Schedule | null }>()
const emit = defineEmits<{ save: [schedule: Schedule]; cancel: [] }>()

const CUSTOM = 'custom'

function sameDays(a: Schedule, b: Schedule): boolean {
  return (
    (a.firstDay === b.firstDay && a.secondDay === b.secondDay) ||
    (a.firstDay === b.secondDay && a.secondDay === b.firstDay)
  )
}

// Start from what's saved: a matching preset, or Custom with the saved days.
const matchedPreset = props.initial ? copy.presets.find((p) => sameDays(p.schedule, props.initial!)) : undefined
const choice = ref<string>(matchedPreset?.id ?? (props.initial ? CUSTOM : copy.presets[0].id))
const customFirst = ref<number | ''>(props.initial && !matchedPreset ? props.initial.firstDay : '')
const customSecond = ref<number | ''>(props.initial && !matchedPreset ? props.initial.secondDay : '')

function validDay(value: number | ''): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 1 && value <= 31
}

const isCustom = computed(() => choice.value === CUSTOM)

// Stay quiet until both fields are filled, then explain what's wrong.
const customError = computed(() => {
  if (!isCustom.value) return ''
  const a = customFirst.value
  const b = customSecond.value
  if (a === '' || b === '') return ''
  if (!validDay(a) || !validDay(b)) return copy.setup.errorRange
  if (a === b) return copy.setup.errorSame
  return ''
})

const canSave = computed(() => {
  if (!isCustom.value) return true
  const a = customFirst.value
  const b = customSecond.value
  return validDay(a) && validDay(b) && a !== b
})

function submit() {
  if (!canSave.value) return
  if (isCustom.value) {
    emit('save', { firstDay: customFirst.value as number, secondDay: customSecond.value as number })
    return
  }
  const preset = copy.presets.find((p) => p.id === choice.value) ?? copy.presets[0]
  emit('save', { ...preset.schedule })
}
</script>

<template>
  <form class="setup" @submit.prevent="submit" novalidate>
    <header class="setup__head">
      <h1 class="setup__title">{{ copy.setup.title }}</h1>
      <p class="lede">{{ copy.setup.subtitle }}</p>
    </header>

    <div class="setup__form">
    <fieldset class="choices">
      <legend class="choices__legend">{{ copy.setup.question }}</legend>

      <label
        v-for="preset in copy.presets"
        :key="preset.id"
        class="choice"
        :class="{ 'choice--on': choice === preset.id }"
      >
        <input v-model="choice" type="radio" name="schedule" :value="preset.id" />
        <span class="choice__label">{{ preset.label }}</span>
        <span class="choice__hint">{{ preset.hint }}</span>
      </label>

      <label class="choice" :class="{ 'choice--on': isCustom }">
        <input v-model="choice" type="radio" name="schedule" :value="CUSTOM" />
        <span class="choice__label">{{ copy.setup.customLabel }}</span>
        <span class="choice__hint">{{ copy.setup.customHint }}</span>
      </label>
    </fieldset>

    <div v-if="isCustom" class="custom">
      <div class="custom__fields">
        <div class="field">
          <label for="first-day">{{ copy.setup.firstDay }}</label>
          <input
            id="first-day"
            v-model.number="customFirst"
            type="number"
            inputmode="numeric"
            min="1"
            max="31"
            :aria-invalid="customError ? 'true' : 'false'"
            aria-describedby="custom-note custom-error"
          />
        </div>
        <div class="field">
          <label for="second-day">{{ copy.setup.secondDay }}</label>
          <input
            id="second-day"
            v-model.number="customSecond"
            type="number"
            inputmode="numeric"
            min="1"
            max="31"
            :aria-invalid="customError ? 'true' : 'false'"
            aria-describedby="custom-note custom-error"
          />
        </div>
      </div>
      <p id="custom-note" class="custom__note">{{ copy.setup.customNote }}</p>
      <p id="custom-error" class="custom__error" role="alert">{{ customError }}</p>
    </div>

    <div class="setup__actions">
      <button type="submit" class="btn btn--primary" :disabled="!canSave">{{ copy.setup.save }}</button>
      <button v-if="initial" type="button" class="btn btn--quiet" @click="emit('cancel')">
        {{ copy.setup.cancel }}
      </button>
    </div>
    </div>
  </form>
</template>
