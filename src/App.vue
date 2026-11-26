<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import HomeScreen from './components/HomeScreen.vue'
import SettingsScreen from './components/SettingsScreen.vue'
import SetupScreen from './components/SetupScreen.vue'
import {
  applyLayout,
  getPaydayInfo,
  loadLanguage,
  loadLayout,
  loadSchedule,
  refreshHolidays,
  saveLanguage,
  saveLayout,
  saveSchedule,
} from './api'
import { copy, language, setLanguage } from './copy'
import type { Language, LayoutMode, PaydayInfo, Schedule } from './types'

type Screen = 'loading' | 'setup' | 'home' | 'settings' | 'error'

const screen = ref<Screen>('loading')
const schedule = ref<Schedule | null>(null)
const info = ref<PaydayInfo | null>(null)
const layout = ref<LayoutMode>('full')

// Load the saved layout and make sure the window matches it. Rust already sized the window
// at startup; this is the safety net. A failure here just means the full layout.
async function initLayout() {
  try {
    layout.value = await loadLayout()
    await applyLayout(layout.value)
  } catch (err) {
    console.error(err)
  }
}

// Settings apply immediately: resize the window, then remember the choice.
async function onLayoutChange(mode: LayoutMode) {
  layout.value = mode
  try {
    await applyLayout(mode)
    await saveLayout(mode)
  } catch (err) {
    console.error(err)
  }
}

// Load the saved language. A failure here just means Tagalog.
async function initLanguage() {
  try {
    setLanguage(await loadLanguage())
  } catch (err) {
    console.error(err)
  }
}

// Switching applies immediately everywhere; then remember the choice.
async function onLanguageChange(next: Language) {
  setLanguage(next)
  try {
    await saveLanguage(next)
  } catch (err) {
    console.error(err)
  }
}

// Tell the page (and screen readers) which language the text is in.
watch(
  language,
  (current) => {
    document.documentElement.lang = current === 'tl' ? 'fil' : 'en'
  },
  { immediate: true },
)

function onSettingsBack() {
  screen.value = schedule.value && info.value ? 'home' : 'setup'
}

async function init() {
  screen.value = 'loading'
  await Promise.all([initLayout(), initLanguage()])
  try {
    const saved = await loadSchedule()
    if (!saved) {
      screen.value = 'setup'
      return
    }
    schedule.value = saved
    info.value = await getPaydayInfo(saved)
    screen.value = 'home'
    void syncHolidays()
  } catch (err) {
    console.error(err)
    screen.value = 'error'
  }
}

async function onSave(next: Schedule) {
  try {
    await saveSchedule(next)
    schedule.value = next
    info.value = await getPaydayInfo(next)
    screen.value = 'home'
    void syncHolidays()
  } catch (err) {
    console.error(err)
    screen.value = 'error'
  }
}

function onCancel() {
  if (schedule.value && info.value) screen.value = 'home'
}

// If the app stays open past midnight, refresh quietly so the countdown never goes stale.
async function refresh() {
  if (screen.value !== 'home' || !schedule.value) return
  try {
    info.value = await getPaydayInfo(schedule.value)
  } catch (err) {
    console.error(err)
  }
  void syncHolidays()
}

// Quietly fetch the latest holiday list (at launch, then at most every 6 hours).
// Failing is fine: offline, the cached or built-in list is used and nothing is shown.
const SYNC_EVERY_MS = 6 * 60 * 60 * 1000
let lastSyncAttempt = 0

async function syncHolidays() {
  if (Date.now() - lastSyncAttempt < SYNC_EVERY_MS) return
  lastSyncAttempt = Date.now()
  try {
    if (await refreshHolidays()) await refresh()
  } catch (err) {
    console.debug('Holiday sync skipped:', err)
  }
}

let timer: ReturnType<typeof setInterval> | undefined

onMounted(() => {
  init()
  timer = setInterval(refresh, 60_000)
  document.addEventListener('visibilitychange', refresh)
})

onBeforeUnmount(() => {
  if (timer) clearInterval(timer)
  document.removeEventListener('visibilitychange', refresh)
})
</script>

<template>
  <main class="shell" :class="`shell--${layout}`">
    <p v-if="screen === 'loading'" class="status" role="status">{{ copy.loading }}</p>

    <SetupScreen
      v-else-if="screen === 'setup'"
      :initial="schedule"
      @save="onSave"
      @cancel="onCancel"
    />

    <HomeScreen
      v-else-if="screen === 'home' && info"
      :info="info"
      :compact="layout === 'compact'"
      @edit="screen = 'setup'"
      @settings="screen = 'settings'"
    />

    <SettingsScreen
      v-else-if="screen === 'settings'"
      :layout="layout"
      :language="language"
      @layout="onLayoutChange"
      @language="onLanguageChange"
      @back="onSettingsBack"
    />

    <div v-else class="error" role="alert">
      <p class="error__message">{{ copy.error.message }}</p>
      <button type="button" class="btn btn--primary" @click="init">{{ copy.error.retry }}</button>
    </div>
  </main>
</template>
