<script setup lang="ts">
import { computed } from 'vue'
import { copy } from '../copy'
import type { Language, LayoutMode } from '../types'

defineProps<{ layout: LayoutMode; language: Language }>()
const emit = defineEmits<{ layout: [mode: LayoutMode]; language: [language: Language]; back: [] }>()

// Computed, so the labels switch language as soon as the language changes.
const layoutOptions = computed<{ id: LayoutMode; label: string; hint: string }[]>(() => [
  { id: 'compact', label: copy.settings.compactLabel, hint: copy.settings.compactHint },
  { id: 'full', label: copy.settings.fullLabel, hint: copy.settings.fullHint },
])

const languageOptions = computed<{ id: Language; label: string; hint: string }[]>(() => [
  { id: 'tl', label: copy.settings.tagalogLabel, hint: copy.settings.tagalogHint },
  { id: 'en', label: copy.settings.englishLabel, hint: copy.settings.englishHint },
])

const version = __APP_VERSION__
</script>

<template>
  <section class="settings">
    <h1 class="settings__title">{{ copy.settings.title }}</h1>

    <div class="settings__groups">
      <fieldset class="choices">
        <legend class="choices__legend">{{ copy.settings.layoutLegend }}</legend>

        <label
          v-for="option in layoutOptions"
          :key="option.id"
          class="choice"
          :class="{ 'choice--on': layout === option.id }"
        >
          <input
            type="radio"
            name="layout"
            :value="option.id"
            :checked="layout === option.id"
            @change="emit('layout', option.id)"
          />
          <span class="choice__label">{{ option.label }}</span>
          <span class="choice__hint">{{ option.hint }}</span>
        </label>
      </fieldset>

      <fieldset class="choices">
        <legend class="choices__legend">{{ copy.settings.languageLegend }}</legend>

        <label
          v-for="option in languageOptions"
          :key="option.id"
          class="choice"
          :class="{ 'choice--on': language === option.id }"
        >
          <input
            type="radio"
            name="language"
            :value="option.id"
            :checked="language === option.id"
            @change="emit('language', option.id)"
          />
          <span class="choice__label">{{ option.label }}</span>
          <span class="choice__hint">{{ option.hint }}</span>
        </label>
      </fieldset>
    </div>

    <section class="about" aria-labelledby="about-title">
      <h2 id="about-title" class="about__title">{{ copy.about.title }}</h2>
      <p>{{ copy.about.appLine(version) }}</p>
      <p>{{ copy.about.madeBy(copy.about.author) }}</p>
    </section>

    <button type="button" class="btn btn--primary" @click="emit('back')">{{ copy.settings.back }}</button>
  </section>
</template>
