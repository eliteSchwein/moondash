<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import Navigation from './components/Navigation.vue'
import { useAppStore } from './stores/app'
import { moonraker } from './plugins/moonraker'
import { resolveLocale } from './plugins/i18n'

const appStore = useAppStore()
const { locale } = useI18n({ useScope: 'global' })

let cleanupConnectionLog: (() => void) | null = null
let cleanupErrors: (() => void) | null = null
let cleanupNotificationHandlers: Array<() => void> = []

watch(
    () => appStore.getLanguage,
    (value) => {
      locale.value = resolveLocale(value)
    },
    { immediate: true },
)

onMounted(async () => {
  try {
    await appStore.startConfigListener()
    await appStore.loadConfig()

    locale.value = resolveLocale(appStore.getLanguage)

    cleanupConnectionLog = moonraker.onConnectionChange((status) => {
      appStore.setWebsocketConnected(status.connected)
      appStore.setMoonrakerReady(status.ready)

      if (!status.connected) {
        appStore.resetMoonrakerData()
        appStore.resetFiles()
      }
    })

    cleanupErrors = moonraker.onError((error) => {
      console.error('moonraker error:', error)
    })

    cleanupNotificationHandlers = [
      ...moonraker.registerDefaultNotifications(),

      moonraker.onNotification('notify_status_update', (params) => {
        const payload = Array.isArray(params) ? params[0] : params
        appStore.applyMoonrakerSubscriptionPayload(payload)
      }),

      moonraker.onNotification('notify_proc_stat_update', (params) => {
        const payload = Array.isArray(params) ? params[0] : params
        appStore.applyMoonrakerProcStats(payload)
      }),

      moonraker.onNotification('notify_history_changed', (params) => {
        const payload = Array.isArray(params) ? params[0] : params
        appStore.applyMoonrakerHistoryUpdate(payload)
      }),

      moonraker.onNotification('notify_klippy_ready', () => {
        appStore.setMoonrakerReady(true)
      }),

      moonraker.onNotification('notify_klippy_disconnected', () => {
        appStore.setMoonrakerReady(false)
      }),

      moonraker.onNotification('notify_klippy_shutdown', () => {
        appStore.setMoonrakerReady(false)
      }),
    ]

    await moonraker.startAutoConnectFromConfig()

    if (moonraker.getStatus().connected) {
      try {
        const initialObjects = await moonraker.registerAllKnownObjects()
        appStore.applyMoonrakerSubscriptionPayload(initialObjects)
      } catch (error) {
        console.warn('initial moonraker subscription payload failed', error)
      }

      try {
        const files = await moonraker.listFiles()
        appStore.setFiles(files)
      } catch (error) {
        console.warn('initial moonraker file list failed', error)
      }
    }
  } catch (err) {
    console.error('config/moonraker init failed:', err)
  }
})

onBeforeUnmount(() => {
  appStore.stopConfigListener()
  appStore.resetConnectionState()
  moonraker.stopAutoConnectFromConfig()
  moonraker.disconnect()

  cleanupConnectionLog?.()
  cleanupErrors?.()
  cleanupNotificationHandlers.forEach((fn) => fn())
})
</script>

<template>
  <v-app
      :theme="appStore.isDarkmode ? 'dark' : 'light'"
      :style="{ zoom: String(appStore.getZoom) }"
  >
    <v-layout>
      <Navigation />
      <router-view />
    </v-layout>
  </v-app>
</template>