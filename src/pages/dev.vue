<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'

const appStore = useAppStore()

const {
  darkmode,
  zoom,
  websocket,
  moonrakerReady,
  moonraker,
} = storeToRefs(appStore)

const paused = ref(false)
const frozenStore = ref('')

const liveStore = computed(() =>
    JSON.stringify(
        {
          darkmode: darkmode.value,
          zoom: zoom.value,
          websocket: websocket.value,
          moonrakerReady: moonrakerReady.value,
          moonraker: moonraker.value,
        },
        null,
        2,
    ),
)

watch(
    liveStore,
    (value) => {
      if (!paused.value) {
        frozenStore.value = value
      }
    },
    { immediate: true },
)

function togglePause() {
  paused.value = !paused.value

  if (!paused.value) {
    frozenStore.value = liveStore.value
  }
}
</script>

<template>
  <div class="store-page">
    <v-card class="store-card" variant="tonal">
      <div class="store-header">
        <v-card-title>App Store</v-card-title>
        <v-btn
            :color="paused ? 'warning' : undefined"
            :variant="paused ? 'flat' : 'tonal'"
            size="small"
            @click="togglePause"
        >
          {{ paused ? 'Resume' : 'Pause' }}
        </v-btn>
      </div>

      <v-card-text class="store-card-text">
        <pre class="store-pre">{{ frozenStore }}</pre>
      </v-card-text>
    </v-card>
  </div>
</template>

<style scoped>
.store-page {
  width: 100%;
  height: 100vh;
  max-height: 100vh;
  padding: 16px 16px 16px 60px;
  box-sizing: border-box;
  overflow: hidden;
}

.store-card {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.store-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 16px;
}

.store-card-text {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.store-pre {
  margin: 0;
  width: 100%;
  height: 100%;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: monospace;
  font-size: 0.9rem;
  line-height: 1.4;
}
</style>