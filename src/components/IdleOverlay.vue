<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useAppStore } from '../stores/app'

type DisplaySleepStatePayload = {
  sleeping: boolean
}

const appStore = useAppStore()

let unlisten: UnlistenFn | null = null

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

watch(
    () => appStore.isSleeping,
    (isSleeping) => {
      console.log(`Idle overlay ${isSleeping ? 'enabled' : 'disabled'}`)
    }
)

onMounted(async () => {
  if (!isTauriRuntime()) return

  unlisten = await listen<DisplaySleepStatePayload>('display-sleep-state', (event) => {
    appStore.setIsSleeping(Boolean(event.payload.sleeping))
  })
})

onBeforeUnmount(() => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
})
</script>

<template>
  <div class="idle-overlay" v-if="appStore.isSleeping" />
</template>