<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import BedMeshToolPanel from '@/components/panels/BedMeshToolPanel.vue'
import PidToolPanel from '@/components/panels/PidToolPanel.vue'
import WebcamToolPanel from '@/components/panels/WebcamToolPanel.vue'

type ToolView = 'home' | 'bed-mesh' | 'pid' | 'webcam'

const { t } = useI18n()
const appStore = useAppStore()
const currentView = ref<ToolView>('home')
const webcamCount = ref(0)

let webcamRefreshTimer: ReturnType<typeof window.setInterval> | null = null

const hasBedMesh = computed(() => {
  const configfile = appStore.moonraker.rawObjects.configfile as
      | { config?: Record<string, unknown> }
      | undefined

  return 'bed_mesh' in (configfile?.config ?? {})
})

const hasWebcam = computed(() => webcamCount.value > 0)

const toolItems = computed(() => [
  ...(hasBedMesh.value
      ? [
        {
          view: 'bed-mesh' as const,
          title: 'settings.tools.bed_mesh.title',
          icon: 'mdi-grid',
        },
      ]
      : []),
  {
    view: 'pid' as const,
    title: 'settings.tools.pid.title',
    icon: 'mdi-tune-variant',
  },
  ...(hasWebcam.value
      ? [
        {
          view: 'webcam' as const,
          title: 'settings.tools.webcam.title',
          icon: 'mdi-webcam',
        },
      ]
      : []),
])

async function loadWebcams() {
  if (!appStore.websocket.ip || !appStore.websocket.connected) {
    webcamCount.value = 0
    return
  }

  try {
    const response = await fetch(`http://${appStore.websocket.ip}/server/webcams/list`)
    const data = await response.json()
    const webcams = data?.result?.webcams ?? data?.webcams ?? []

    webcamCount.value = Array.isArray(webcams) ? webcams.length : 0
  } catch {
    webcamCount.value = 0
  }
}

function startWebcamRefresh() {
  stopWebcamRefresh()
  loadWebcams()

  webcamRefreshTimer = window.setInterval(() => {
    loadWebcams()
  }, 5000)
}

function stopWebcamRefresh() {
  if (webcamRefreshTimer) {
    window.clearInterval(webcamRefreshTimer)
    webcamRefreshTimer = null
  }
}

watch(
    () => [appStore.websocket.ip, appStore.websocket.connected],
    () => {
      startWebcamRefresh()
    },
    { immediate: true },
)

watch([hasBedMesh, hasWebcam], () => {
  if (!hasBedMesh.value && currentView.value === 'bed-mesh') {
    currentView.value = 'home'
  }

  if (!hasWebcam.value && currentView.value === 'webcam') {
    currentView.value = 'home'
  }
})

onMounted(() => {
  startWebcamRefresh()
})

onUnmounted(() => {
  stopWebcamRefresh()
})

function openTool(view: ToolView) {
  if (view === 'bed-mesh' && !hasBedMesh.value) return
  if (view === 'webcam' && !hasWebcam.value) return

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
          <v-btn  variant="text" @click="goHome">
            <v-icon icon="mdi-arrow-left" />

            <div class="tools-subpanel__title pl-4 pr-2">
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
          </v-btn>
        </div>

        <BedMeshToolPanel
            v-if="currentView === 'bed-mesh' && hasBedMesh"
            @back="goHome"
        />

        <PidToolPanel
            v-else-if="currentView === 'pid'"
            @back="goHome"
        />

        <WebcamToolPanel
            v-else-if="currentView === 'webcam' && hasWebcam"
            @back="goHome"
        />
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
  padding: 0;
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