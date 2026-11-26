<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import HomeScreen from './components/HomeScreen.vue'
import SetupScreen from './components/SetupScreen.vue'
import { getPaydayInfo, loadSchedule, saveSchedule } from './api'
import { copy } from './copy'
import type { PaydayInfo, Schedule } from './types'

type Screen = 'loading' | 'setup' | 'home' | 'error'

const screen = ref<Screen>('loading')
const schedule = ref<Schedule | null>(null)
const info = ref<PaydayInfo | null>(null)

async function init() {
  screen.value = 'loading'
  try {
    const saved = await loadSchedule()
    if (!saved) {
      screen.value = 'setup'
      return
    }
    schedule.value = saved
    info.value = await getPaydayInfo(saved)
    screen.value = 'home'
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
  <main class="shell">
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
      @edit="screen = 'setup'"
    />

    <div v-else class="error" role="alert">
      <p class="error__message">{{ copy.error.message }}</p>
      <button type="button" class="btn btn--primary" @click="init">{{ copy.error.retry }}</button>
    </div>
  </main>
</template>
