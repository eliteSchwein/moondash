<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'

const emit = defineEmits<{
  (e: 'back'): void
}>()

const { t } = useI18n()
const appStore = useAppStore()
const { websocket } = storeToRefs(appStore)

type WebcamOption = {
  title: string
  value: string
  url: string
}

const webcamIndex = ref(0)
const webcams = ref<WebcamOption[]>([])
const loading = ref(false)
const refreshNonce = ref(Date.now())

const currentWebcam = computed(() => webcams.value[webcamIndex.value] ?? null)

const currentWebcamSrc = computed(() => {
  if (!currentWebcam.value?.url) return ''
  const separator = currentWebcam.value.url.includes('?') ? '&' : '?'
  return `${currentWebcam.value.url}${separator}_ts=${refreshNonce.value}`
})

function selectWebcam(index: number) {
  webcamIndex.value = index >= 0 ? index : 0
  refreshWebcam()
}

function refreshWebcam() {
  refreshNonce.value = Date.now()
}

function getMoonrakerHostName(): string {
  const rawIp = String(websocket.value?.ip ?? '').trim()

  if (!rawIp) return ''

  try {
    return new URL(rawIp).hostname
  } catch {
    return rawIp
        .replace(/^wss?:\/\//, '')
        .replace(/^https?:\/\//, '')
        .split('/')[0]
        .split(':')[0]
        .trim()
  }
}

function rewriteWebrtcToSnapshotPath(pathname: string): string {
  if (pathname.endsWith('/webrtc')) {
    return pathname.replace(/\/webrtc$/, '/')
  }

  return pathname.replace('/webrtc', '/')
}

function normalizeWebcamUrl(url: string): string {
  if (!url) return ''

  const remoteHost = getMoonrakerHostName()
  if (!remoteHost) return url

  try {
    const parsed = new URL(url)

    if (
        parsed.hostname === 'localhost' ||
        parsed.hostname === '127.0.0.1' ||
        parsed.hostname === '::1'
    ) {
      parsed.hostname = remoteHost
    }

    if (parsed.pathname.includes('/webrtc')) {
      parsed.pathname = rewriteWebrtcToSnapshotPath(parsed.pathname)
      parsed.search = '?action=snapshot'
    }

    return parsed.toString()
  } catch {
    let normalized = url.trim()

    if (normalized.includes('/webrtc')) {
      normalized = normalized.replace(/\/webrtc$/, '/').replace('/webrtc', '/')

      if (normalized.includes('?')) {
        if (!/[?&]action=/.test(normalized)) {
          normalized += '&action=snapshot'
        }
      } else {
        normalized += '?action=snapshot'
      }
    }

    if (normalized.startsWith('//')) {
      return `http:${normalized}`
    }

    if (normalized.startsWith('/')) {
      return `http://${remoteHost}${normalized}`
    }

    return `http://${remoteHost}/${normalized}`
  }
}

function pickWebcamUrl(cam: Record<string, unknown>): string {
  const candidates = [
    cam.snapshot_url,
    cam.url_snapshot,
    cam.stream_url,
    cam.url_stream,
    cam.mjpg_url,
    cam.webrtc_url,
    cam.url,
  ]

  for (const candidate of candidates) {
    const value = String(candidate ?? '').trim()
    if (value) return value
  }

  return ''
}

async function loadWebcams() {
  try {
    loading.value = true

    const result = await moonrakerClient.call<{
      webcams?: Array<Record<string, unknown>>
      result?: {
        webcams?: Array<Record<string, unknown>>
      }
    }>('server.webcams.list')

    const rawWebcams =
        result.webcams ??
        result.result?.webcams ??
        []

    const items = rawWebcams
        .map((cam, index) => {
          const title = String(cam.name ?? cam.title ?? `Webcam ${index + 1}`)
          const value = String(cam.uid ?? cam.id ?? cam.name ?? index)

          const rawUrl = pickWebcamUrl(cam)
          const url = normalizeWebcamUrl(rawUrl)

          if (!url) return null

          return {
            title,
            value,
            url,
          } satisfies WebcamOption
        })
        .filter((item): item is WebcamOption => item !== null)

    webcams.value = items

    if (webcamIndex.value >= items.length) {
      webcamIndex.value = 0
    }

    refreshWebcam()
  } catch (error) {
    console.error('Failed to load webcams', error)
    webcams.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadWebcams()
})
</script>

<template>
  <div class="tool-panel">
    <v-card class="tool-card" elevation="0" rounded="lg">
      <v-card-text class="tool-webcam__layout">
        <div class="tool-webcam__viewer">
          <template v-if="currentWebcam">
            <img
                class="tool-webcam__media"
                :src="currentWebcamSrc"
                :alt="currentWebcam.title"
            />
          </template>

          <v-alert v-else-if="loading" type="info" variant="tonal">
            Loading webcam…
          </v-alert>

          <v-alert v-else type="info" variant="tonal">
            {{ t('settings.tools.noWebcam') }}
          </v-alert>
        </div>

        <div class="tool-webcam__controls">
          <div class="tool-webcam__control-group">
            <v-select
                v-if="webcams.length > 0"
                :items="webcams"
                item-title="title"
                item-value="value"
                density="compact"
                variant="outlined"
                hide-details
                class="tool-webcam-select"
                :model-value="currentWebcam?.value"
                @update:model-value="(value) => selectWebcam(webcams.findIndex((w) => w.value === value))"
            />

            <v-alert v-else-if="!loading" type="info" variant="tonal">
              {{ t('settings.tools.webcam.no_webcam') }}
            </v-alert>
          </div>

          <div class="tool-webcam__actions">
            <v-btn
                prepend-icon="mdi-refresh"
                variant="tonal"
                :disabled="loading || !currentWebcam"
                @click="refreshWebcam"
            >
              {{ t('settings.tools.webcam.refresh') }}
            </v-btn>

            <v-btn
                prepend-icon="mdi-reload"
                variant="text"
                :loading="loading"
                @click="loadWebcams"
            >
              {{ t('settings.tools.webcam.reload_webcams') }}
            </v-btn>
          </div>
        </div>
      </v-card-text>
    </v-card>
  </div>
</template>

<style scoped>
.tool-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tool-card {
  min-height: 0;
}

.tool-webcam__layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 280px;
  gap: 16px;
  align-items: start;
  min-height: 0;
}

.tool-webcam__viewer {
  min-width: 0;
  background: #000;
  border-radius: 12px;
  overflow: hidden;
}

.tool-webcam__controls {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tool-webcam__control-group,
.tool-webcam__info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tool-webcam__actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tool-webcam__label {
  font-size: 0.85rem;
  opacity: 0.7;
}

.tool-webcam__value {
  font-weight: 600;
}

.tool-webcam-select {
  min-width: 0;
}

.tool-webcam__media {
  width: 100%;
  height: auto;
  display: block;
  background: #000;
}
</style>