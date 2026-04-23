<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const tab = ref('network')
const { t } = useI18n()


</script>

<template>
  <v-main>
    <v-app-bar elevation="0" color="transparent" density="compact">
      <v-tabs v-model="tab" color="primary">
        <v-tab value="network">{{ t('settings.tab.network') }}</v-tab>
        <v-tab value="settings">{{ t('settings.tab.settings') }}</v-tab>
        <v-tab value="tools">{{ t('settings.tab.tools') }}</v-tab>
        <v-tab value="shortcuts">{{ t('settings.tab.shortcuts') }}</v-tab>
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