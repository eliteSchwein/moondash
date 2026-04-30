<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useAppStore } from '../stores/app'
import {useI18n} from "vue-i18n";

const { t } = useI18n()

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
  <div
      class="idle-overlay d-flex flex-column align-center justify-center text-center"
      v-if="appStore.isSleeping"
  >
    <v-icon
        icon="mdi-lock-clock"
        size="64"
        class="mb-4"
    />

    <div class="text-h6 font-weight-bold">
      {{ t('lock.locked') }}
    </div>
  </div>
</template>

<style scoped>
.idle-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(var(--v-theme-background), 0.8);
  z-index: 9999;
}
</style>