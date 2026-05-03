<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const validTabs = new Set(['network', 'settings', 'tools', 'shortcuts', 'updates'])
const tab = ref(getInitialTab())

function getInitialTab(): string {
  const queryTab = typeof route.query.tab === 'string' ? route.query.tab : ''
  if (validTabs.has(queryTab)) return queryTab

  if (typeof window !== 'undefined') {
    const storedTab = window.sessionStorage.getItem('settings.tab') ?? ''
    window.sessionStorage.removeItem('settings.tab')
    if (validTabs.has(storedTab)) return storedTab
  }

  return 'network'
}

function openTab(value: unknown) {
  if (typeof value !== 'string' || !validTabs.has(value)) return
  tab.value = value
}

function handleOpenSettingsTab(event: Event) {
  const detail = (event as CustomEvent<{ tab?: string }>).detail
  openTab(detail?.tab)
}

watch(
    () => route.query.tab,
    (value) => openTab(value),
)

watch(tab, async (value) => {
  if (route.query.tab === value) return
  await router.replace({ query: { ...route.query, tab: value } })
})

onMounted(() => {
  window.addEventListener('settings:open-tab', handleOpenSettingsTab)
})

onBeforeUnmount(() => {
  window.removeEventListener('settings:open-tab', handleOpenSettingsTab)
})

</script>

<template>
  <v-main>
    <v-app-bar elevation="0" color="transparent" density="compact">
      <v-tabs v-model="tab" color="primary">
        <v-tab value="network">{{ t('settings.tab.network') }}</v-tab>
        <v-tab value="settings">{{ t('settings.tab.settings') }}</v-tab>
        <v-tab value="tools">{{ t('settings.tab.tools') }}</v-tab>
        <v-tab value="shortcuts">{{ t('settings.tab.shortcuts') }}</v-tab>
        <v-tab value="updates">{{ t('settings.tab.updates') }}</v-tab>
      </v-tabs>
    </v-app-bar>

    <v-tabs-window v-model="tab" class="panel-container">
      <v-tabs-window-item value="network">
        <v-sheet class="pa-0 pr-3 pb-3 pt-2" color="transparent">
          <NetworkPanel/>
        </v-sheet>
      </v-tabs-window-item>
      <v-tabs-window-item value="settings">
        <v-sheet class="pa-0 pr-3 pb-3 pt-2" color="transparent">
          <ConfigEditorPanel/>
        </v-sheet>
      </v-tabs-window-item>
      <v-tabs-window-item value="tools">
        <v-sheet class="pa-0 pr-3 pb-3 pt-2" color="transparent">
          <ToolsPanel/>
        </v-sheet>
      </v-tabs-window-item>
      <v-tabs-window-item value="shortcuts">
        <v-sheet class="pa-0 pr-3 pb-3 pt-2" color="transparent">
          <ShortcutButtonsPanel/>
        </v-sheet>
      </v-tabs-window-item>
      <v-tabs-window-item value="updates">
        <v-sheet class="pa-0 pr-3 pb-3 pt-2" color="transparent">
          <UpdatePanel/>
        </v-sheet>
      </v-tabs-window-item>

    </v-tabs-window>
  </v-main>
</template>

<style scoped>
.panel-container {
  height: 100%;
  min-height: 0;
  max-height: 100%;
}
</style>