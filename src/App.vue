<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTheme } from 'vuetify'
import Navigation from './components/Navigation.vue'
import { useAppStore } from './stores/app'
import { moonraker } from './plugins/moonraker'
import { resolveLocale } from './plugins/i18n'
import { getCurrentWindow } from '@tauri-apps/api/window'

const appStore = useAppStore()
const { locale } = useI18n({ useScope: 'global' })
const theme = useTheme()

let cleanupConnectionLog: (() => void) | null = null
let cleanupErrors: (() => void) | null = null
let cleanupNotificationHandlers: Array<() => void> = []

let syncingAfterReconnect = false

const primaryColor = computed(() => appStore.getPrimaryColor)
const secondaryColor = computed(() => appStore.getSecondaryColor)

const defaultLightPrimary =
    theme.themes.value.light?.colors?.primary ?? '#1976D2'
const defaultLightSecondary =
    theme.themes.value.light?.colors?.secondary ?? '#424242'
const defaultDarkPrimary =
    theme.themes.value.dark?.colors?.primary ?? '#2196F3'
const defaultDarkSecondary =
    theme.themes.value.dark?.colors?.secondary ?? '#424242'

watchEffect(() => {
  const lightTheme = theme.themes.value.light
  const darkTheme = theme.themes.value.dark

  if (lightTheme?.colors) {
    lightTheme.colors.primary = primaryColor.value || defaultLightPrimary
    lightTheme.colors.secondary = secondaryColor.value || defaultLightSecondary
  }

  if (darkTheme?.colors) {
    darkTheme.colors.primary = primaryColor.value || defaultDarkPrimary
    darkTheme.colors.secondary = secondaryColor.value || defaultDarkSecondary
  }
})

async function refreshMoonrakerState() {
  if (syncingAfterReconnect) return
  if (!moonraker.getStatus().connected) return

  try {
    syncingAfterReconnect = true

    const initialObjects = await moonraker.registerAllKnownObjects()
    appStore.applyMoonrakerSubscriptionPayload(initialObjects)
  } catch (error) {
    console.warn('moonraker refresh after reconnect failed', error)
  } finally {
    syncingAfterReconnect = false
  }
}

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

    cleanupConnectionLog = moonraker.onConnectionChange(async (status) => {
      appStore.setWebsocketConnected(status.connected)
      appStore.setMoonrakerReady(status.ready)

      if (!status.connected) {
        appStore.resetMoonrakerData()
        appStore.resetFiles()
        return
      }

      await refreshMoonrakerState()
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

      moonraker.onNotification('notify_klippy_ready', async () => {
        appStore.setMoonrakerReady(true)
        await refreshMoonrakerState()
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
      await refreshMoonrakerState()
    }
  } catch (err) {
    console.error('config/moonraker init failed:', err)
  }

  await getCurrentWindow().show()
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