<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

const {
  darkmode,
  zoom,
  debug,
  websocket,
  moonrakerReady,
  moonraker,
  files,
} = storeToRefs(appStore)

const paused = ref(false)
const activeTab = ref('other')

const frozenOther = ref('')
const frozenMoonraker = ref('')
const frozenAfc = ref('')
const frozenFiles = ref('')
const frozenHistory = ref('')
const frozenFull = ref('')

const afcAvailable = computed(() => moonraker.value.afc.available)

const liveOther = computed(() =>
    JSON.stringify(
        {
          darkmode: darkmode.value,
          zoom: zoom.value,
          debug: debug.value,
          websocket: websocket.value,
          moonrakerReady: moonrakerReady.value,
        },
        null,
        2,
    ),
)

const liveMoonraker = computed(() =>
    JSON.stringify(
        {
          webhooks: moonraker.value.webhooks,
          extruder: moonraker.value.extruder,
          heaterBed: moonraker.value.heaterBed,
          toolhead: moonraker.value.toolhead,
          gcodeMove: moonraker.value.gcodeMove,
          printStats: moonraker.value.printStats,
          virtualSdcard: moonraker.value.virtualSdcard,
          procStats: moonraker.value.procStats,
          throttle: moonraker.value.throttle,
          displayStatus: moonraker.value.displayStatus,
          dynamicHeaters: moonraker.value.dynamicHeaters,
          dynamicFans: moonraker.value.dynamicFans,
        },
        null,
        2,
    ),
)

const liveAfc = computed(() =>
    JSON.stringify(
        {
          available: moonraker.value.afc.available,
          objects: moonraker.value.afc.objects,
        },
        null,
        2,
    ),
)

const liveFiles = computed(() =>
    JSON.stringify(
        {
          lastUpdated: files.value.lastUpdated,
          items: files.value.items,
        },
        null,
        2,
    ),
)

const liveHistory = computed(() =>
    JSON.stringify(
        {
          history: moonraker.value.history,
        },
        null,
        2,
    ),
)

const liveFull = computed(() =>
    JSON.stringify(
        {
          darkmode: darkmode.value,
          zoom: zoom.value,
          debug: debug.value,
          websocket: websocket.value,
          moonrakerReady: moonrakerReady.value,
          moonraker: moonraker.value,
          files: files.value,
        },
        null,
        2,
    ),
)

watch(
    [liveOther, liveMoonraker, liveAfc, liveFiles, liveHistory, liveFull],
    ([otherValue, moonrakerValue, afcValue, filesValue, historyValue, fullValue]) => {
      if (!paused.value) {
        frozenOther.value = otherValue
        frozenMoonraker.value = moonrakerValue
        frozenAfc.value = afcValue
        frozenFiles.value = filesValue
        frozenHistory.value = historyValue
        frozenFull.value = fullValue
      }
    },
    { immediate: true },
)

function togglePause() {
  paused.value = !paused.value

  if (!paused.value) {
    frozenOther.value = liveOther.value
    frozenMoonraker.value = liveMoonraker.value
    frozenAfc.value = liveAfc.value
    frozenFiles.value = liveFiles.value
    frozenHistory.value = liveHistory.value
    frozenFull.value = liveFull.value
  }
}
</script>

<template>
  <div class="store-page">
    <v-card class="store-card" variant="tonal">
      <div class="store-header">
        <div class="store-header-left">
          <v-card-title>App Store</v-card-title>
          <v-chip
              v-if="afcAvailable"
              size="small"
              color="success"
              variant="flat"
          >
            AFC available
          </v-chip>
        </div>

        <v-btn
            :color="paused ? 'warning' : undefined"
            :variant="paused ? 'flat' : 'tonal'"
            size="small"
            @click="togglePause"
        >
          {{ paused ? 'Resume' : 'Pause' }}
        </v-btn>
      </div>

      <div class="store-tabs-wrap">
        <v-tabs v-model="activeTab" class="store-tabs" density="compact">
          <v-tab value="other">Other</v-tab>
          <v-tab value="moonraker">Moonraker</v-tab>
          <v-tab value="files">Files</v-tab>
          <v-tab value="history">History</v-tab>
          <v-tab value="afc">AFC</v-tab>
          <v-tab value="full">Full</v-tab>
        </v-tabs>
      </div>

      <div class="store-content">
        <div v-show="activeTab === 'other'" class="tab-panel">
          <pre class="store-pre">{{ frozenOther }}</pre>
        </div>

        <div v-show="activeTab === 'moonraker'" class="tab-panel">
          <pre class="store-pre">{{ frozenMoonraker }}</pre>
        </div>

        <div v-show="activeTab === 'files'" class="tab-panel">
          <pre class="store-pre">{{ frozenFiles }}</pre>
        </div>

        <div v-show="activeTab === 'history'" class="tab-panel">
          <pre class="store-pre">{{ frozenHistory }}</pre>
        </div>

        <div v-show="activeTab === 'afc'" class="tab-panel">
          <pre class="store-pre">{{ frozenAfc }}</pre>
        </div>

        <div v-show="activeTab === 'full'" class="tab-panel">
          <pre class="store-pre">{{ frozenFull }}</pre>
        </div>
      </div>
    </v-card>
  </div>
</template>

<style scoped>
.store-page {
  width: 100%;
  height: 100%;
  min-height: 0;
  padding: 16px 16px 16px 60px;
  box-sizing: border-box;
  overflow: hidden;
  max-height: 100vh;
}

.store-card {
  width: 100%;
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.store-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-right: 16px;
  flex-shrink: 0;
}

.store-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.store-tabs-wrap {
  flex-shrink: 0;
  overflow: hidden;
}

.store-tabs {
  flex-shrink: 0;
}

.store-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 8px 16px 16px 16px;
  box-sizing: border-box;
}

.tab-panel {
  width: 100%;
  height: 100%;
  max-height: 100%;
  min-height: 0;
  overflow: hidden;
}

.store-pre {
  margin: 0;
  width: 100%;
  height: 100%;
  max-height: 100%;
  overflow: scroll;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: monospace;
  font-size: 0.9rem;
  line-height: 1.4;
}
</style>