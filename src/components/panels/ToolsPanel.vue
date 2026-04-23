<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BedMeshToolPanel from '@/components/panels/BedMeshToolPanel.vue'
import PidToolPanel from '@/components/panels/PidToolPanel.vue'
import WebcamToolPanel from '@/components/panels/WebcamToolPanel.vue'

type ToolView = 'home' | 'bed-mesh' | 'pid' | 'resonance' | 'webcam'

const { t } = useI18n()
const currentView = ref<ToolView>('home')

const toolItems = [
  {
    view: 'bed-mesh' as const,
    title: 'settings.tools.bed_mesh.title',
    icon: 'mdi-grid',
  },
  {
    view: 'pid' as const,
    title: 'settings.tools.pid.title',
    icon: 'mdi-tune-variant',
  },
  {
    view: 'webcam' as const,
    title: 'settings.tools.webcam.title',
    icon: 'mdi-webcam',
  },
]

function openTool(view: ToolView) {
  currentView.value = view
}

function goHome() {
  currentView.value = 'home'
}
</script>

<template>
  <v-card rounded="lg" variant="flat" class="tools-panel pa-0">
    <v-card-text class="tools-panel__content pa-0">
      <div v-if="currentView === 'home'" class="tools-home">
        <v-list class="tools-list pa-0" rounded="lg" density="compact">
          <template v-for="item in toolItems" :key="item.view">
            <v-list-item
                class="tools-list__item"
                rounded="lg"
                @click="openTool(item.view)"
            >
              <template #prepend>
                <v-icon :icon="item.icon" class="tools-list__icon" />
              </template>

              <v-list-item-title class="tools-list__title">
                {{ t(item.title) }}
              </v-list-item-title>
            </v-list-item>

            <v-divider />
          </template>
        </v-list>
      </div>

      <div v-else class="tools-subpanel">
        <div class="tools-subpanel__header">
          <v-btn icon="mdi-arrow-left" variant="text" @click="goHome" />
          <div class="tools-subpanel__title">
            <template v-if="currentView === 'bed-mesh'">
              {{ t('settings.tools.bed_mesh.title') }}
            </template>
            <template v-else-if="currentView === 'pid'">
              {{ t('settings.tools.pid.title') }}
            </template>
            <template v-else-if="currentView === 'webcam'">
              {{ t('settings.tools.webcam.title') }}
            </template>
          </div>
        </div>

        <BedMeshToolPanel v-if="currentView === 'bed-mesh'" @back="goHome" />
        <PidToolPanel v-else-if="currentView === 'pid'" @back="goHome" />
        <WebcamToolPanel v-else-if="currentView === 'webcam'" @back="goHome" />
      </div>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.tools-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tools-panel__content {
  flex: 1 1 auto;
  min-height: 0;
}

.tools-home,
.tools-subpanel {
  padding: 0px;
}

.tools-list {
  padding: 0;
  overflow: hidden;
}

.tools-list__item {
  min-height: 64px;
  cursor: pointer;
}

.tools-list__icon {
  font-size: 20px;
}

.tools-list__title {
  font-size: 1rem;
  font-weight: 600;
}

.tools-subpanel__header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.tools-subpanel__title {
  font-size: 1.2rem;
  font-weight: 700;
}
</style>