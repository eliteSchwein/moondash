<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
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
  <div class="idle-overlay" v-if="appStore.isSleeping"/>
</template>

<style scoped>
.idle-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 1);
  z-index: 9999;
}
</style>