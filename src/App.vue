<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch, watchEffect } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTheme } from 'vuetify'
import Navigation from './components/Navigation.vue'
import { useAppStore } from './stores/app'
import { moonraker } from './plugins/moonraker'
import { resolveLocale } from './plugins/i18n'
import { getCurrentWindow } from '@tauri-apps/api/window'
import IdleOverlay from "@/components/IdleOverlay.vue";

const appStore = useAppStore()
const { locale } = useI18n({ useScope: 'global' })
const theme = useTheme()

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

    await moonraker.startAutoConnectFromConfig()
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
})
</script>

<template>
  <v-app
      :theme="appStore.isDarkmode ? 'dark' : 'light'"
      :style="{ zoom: String(appStore.getZoom) }"
  >
    <v-layout>
      <IdleOverlay/>
      <Navigation />
      <router-view />
    </v-layout>
  </v-app>
</template>