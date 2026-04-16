<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import Navigation from './components/Navigation.vue'
import { useAppStore } from './stores/app'
import { moonraker } from './plugins/moonraker'

const appStore = useAppStore()

let cleanupConnectionLog: (() => void) | null = null
let cleanupErrors: (() => void) | null = null
let cleanupNotificationHandlers: Array<() => void> = []

onMounted(async () => {
  try {
    await appStore.startConfigListener()
    await appStore.loadConfig()

    cleanupConnectionLog = moonraker.onConnectionChange((status) => {
      appStore.setWebsocketConnected(status.connected)
      appStore.setMoonrakerReady(status.ready)

      if (!status.connected) {
        appStore.resetMoonrakerData()
      }
    })

    cleanupErrors = moonraker.onError((error) => {
      console.error('moonraker error:', error)
    })

    cleanupNotificationHandlers = [
      ...moonraker.registerDefaultNotifications(),

      moonraker.onNotification('notify_status_update', (params) => {
        appStore.applyMoonrakerStatusUpdate(params?.[0])
      }),

      moonraker.onNotification('notify_proc_stat_update', (params) => {
        appStore.applyMoonrakerProcStats(params?.[0])
      }),

      moonraker.onNotification('notify_history_changed', (params) => {
        appStore.applyMoonrakerHistoryUpdate(params)
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

      moonraker.onNotification('notify_power_changed', (params) => {
        appStore.applyMoonrakerPowerDevices(params?.[0])
      }),

      moonraker.onNotification('notify_timelapse_changed', (params) => {
        appStore.applyMoonrakerTimelapse(params?.[0])
      }),
    ]

    await moonraker.startAutoConnectFromConfig()

    try {
      const initialObjects = await moonraker.registerAllKnownObjects()
      appStore.applyMoonrakerSubscriptionPayload(initialObjects)
    } catch (error) {
      console.warn('initial moonraker subscription payload failed', error)
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

<style>
</style>