<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore } from '@/stores/app'
import SkipObjectDialog from '@/components/dialogs/SkipObjectDialog.vue'

type MoonrakerThumbnail = {
  width?: number
  height?: number
  size?: number
  thumbnail_path?: string
}

type MoonrakerFileMetadata = {
  estimated_time?: number
  layer_count?: number
  object_height?: number
  layer_height?: number
  first_layer_height?: number
}

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const loading = ref(false)
const skipDialogOpen = ref(false)
const currentFileThumbnails = ref<MoonrakerThumbnail[]>([])
const currentFileMetadata = ref<MoonrakerFileMetadata | null>(null)

const printState = computed(() => moonraker.value.printStats.state?.toLowerCase() ?? '')
const printFilename = computed(() => moonraker.value.printStats.filename ?? '')
const fileDisplayName = computed(() => printFilename.value.replace(/\.gcode$/i, ''))

const hasLoadedFile = computed(() => Boolean(printFilename.value))
const isPrinting = computed(() => printState.value === 'printing')
const isPaused = computed(() => printState.value === 'paused')
const isFinished = computed(() => printState.value === 'complete')
const isActivePrint = computed(() => isPrinting.value || isPaused.value || isFinished.value)

const progressRatio = computed(() => {
  const displayProgress = moonraker.value.displayStatus.progress
  const fileProgress = moonraker.value.virtualSdcard.progress

  if (typeof displayProgress === 'number' && Number.isFinite(displayProgress)) {
    return Math.max(0, Math.min(1, displayProgress))
  }

  if (typeof fileProgress === 'number' && Number.isFinite(fileProgress)) {
    return Math.max(0, Math.min(1, fileProgress))
  }

  return 0
})

const progressPercent = computed(() => Math.round(progressRatio.value * 100))

function parseFiniteNumber(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : null
}

const currentLayer = computed(() => {
  const info = moonraker.value.printStats.info as Record<string, unknown> | undefined

  const current =
      parseFiniteNumber(info?.current_layer) ??
      parseFiniteNumber(info?.currentLayer) ??
      parseFiniteNumber(info?.layer) ??
      parseFiniteNumber(info?.current_layer_number)

  let total =
      parseFiniteNumber(currentFileMetadata.value?.layer_count)

  if (total === null) {
    const objectHeight = parseFiniteNumber(currentFileMetadata.value?.object_height)
    const layerHeight = parseFiniteNumber(currentFileMetadata.value?.layer_height)
    const firstLayerHeight = parseFiniteNumber(currentFileMetadata.value?.first_layer_height)

    if (
        objectHeight !== null &&
        layerHeight !== null &&
        layerHeight > 0
    ) {
      if (firstLayerHeight !== null && firstLayerHeight > 0) {
        total = Math.max(1, Math.ceil(1 + (objectHeight - firstLayerHeight) / layerHeight))
      } else {
        total = Math.max(1, Math.ceil(objectHeight / layerHeight))
      }
    }
  }

  if (current !== null && total !== null && total > 0) {
    return `${Math.max(0, Math.floor(current))}/${Math.max(1, Math.floor(total))}`
  }

  if (current !== null) {
    return `${Math.max(0, Math.floor(current))}/--`
  }

  return '--/--'
})

function formatDuration(seconds: number | null | undefined): string {
  if (typeof seconds !== 'number' || !Number.isFinite(seconds) || seconds < 0) return '--'

  let remaining = Math.floor(seconds)
  const hours = Math.floor(remaining / 3600)
  remaining %= 3600
  const minutes = Math.floor(remaining / 60)

  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

const remainingTime = computed(() => {
  const total = currentFileMetadata.value?.estimated_time
  if (typeof total !== 'number' || !Number.isFinite(total) || total <= 0) return '--'

  const remaining = total * (1 - progressRatio.value)
  return formatDuration(remaining)
})

const excludeObjectState = computed(() => {
  const raw = moonraker.value.rawObjects['exclude_object']
  return raw && typeof raw === 'object' && !Array.isArray(raw)
      ? (raw as Record<string, unknown>)
      : {}
})

const canSkipObject = computed(() => {
  const objects = excludeObjectState.value.objects
  return Array.isArray(objects) && objects.length > 1 && (isPrinting.value || isPaused.value)
})

const httpBase = computed(() => {
  const wsUrl = moonrakerClient.getStatus().url
  if (!wsUrl) return ''

  try {
    const parsed = new URL(wsUrl)
    const protocol = parsed.protocol === 'wss:' ? 'https:' : 'http:'
    return `${protocol}//${parsed.host}`
  } catch {
    return ''
  }
})

async function loadCurrentFileAssets() {
  if (!printFilename.value) {
    currentFileThumbnails.value = []
    currentFileMetadata.value = null
    return
  }

  try {
    const [thumbs, metadata] = await Promise.all([
      moonrakerClient.call<unknown>('server.files.thumbnails', {
        filename: printFilename.value,
      }).catch(() => []),
      moonrakerClient.call<MoonrakerFileMetadata>('server.files.metadata', {
        filename: printFilename.value,
      }).catch(() => null),
    ])

    if (Array.isArray(thumbs)) {
      currentFileThumbnails.value = thumbs as MoonrakerThumbnail[]
    } else if (
        thumbs &&
        typeof thumbs === 'object' &&
        Array.isArray((thumbs as Record<string, unknown>).thumbnails)
    ) {
      currentFileThumbnails.value = (thumbs as Record<string, unknown>).thumbnails as MoonrakerThumbnail[]
    } else if (
        thumbs &&
        typeof thumbs === 'object' &&
        Array.isArray((thumbs as Record<string, unknown>).result)
    ) {
      currentFileThumbnails.value = (thumbs as Record<string, unknown>).result as MoonrakerThumbnail[]
    } else {
      currentFileThumbnails.value = []
    }

    currentFileMetadata.value = metadata
  } catch {
    currentFileThumbnails.value = []
    currentFileMetadata.value = null
  }
}

const previewUrl = computed(() => {
  if (!printFilename.value || !httpBase.value || !currentFileThumbnails.value.length) return null

  const sorted = [...currentFileThumbnails.value].sort((a, b) => {
    const aArea = (a.width ?? 0) * (a.height ?? 0)
    const bArea = (b.width ?? 0) * (b.height ?? 0)
    return bArea - aArea
  })

  const thumb = sorted.find((item) => item.thumbnail_path)
  if (!thumb?.thumbnail_path) return null

  const fileDir = printFilename.value.includes('/')
      ? printFilename.value.slice(0, printFilename.value.lastIndexOf('/'))
      : ''

  const normalizedThumbPath = thumb.thumbnail_path.replace(/^\.?\//, '')
  const fullThumbPath = fileDir
      ? `${fileDir}/${normalizedThumbPath}`
      : normalizedThumbPath

  return `${httpBase.value}/server/files/gcodes/${encodeURI(fullThumbPath)}`
})

async function pausePrint() {
  if (loading.value) return
  try {
    loading.value = true
    await moonrakerClient.call('printer.print.pause')
  } finally {
    loading.value = false
  }
}

async function resumePrint() {
  if (loading.value) return
  try {
    loading.value = true
    await moonrakerClient.call('printer.print.resume')
  } finally {
    loading.value = false
  }
}

async function cancelPrint() {
  if (loading.value) return
  try {
    loading.value = true
    await moonrakerClient.call('printer.print.cancel')
  } finally {
    loading.value = false
  }
}

async function reprint() {
  if (loading.value || !printFilename.value) return
  try {
    loading.value = true
    await moonrakerClient.call('printer.print.start', {
      filename: printFilename.value,
    })
  } finally {
    loading.value = false
  }
}

async function clearFile() {
  if (loading.value) return
  try {
    loading.value = true
    await moonrakerClient.call('printer.gcode.script', {
      script: 'SDCARD_RESET_FILE',
    })
  } finally {
    loading.value = false
  }
}

watch(
    printFilename,
    async () => {
      await loadCurrentFileAssets()
    },
    { immediate: true },
)
</script>

<template>
  <v-card class="current-print-panel" rounded="lg" variant="flat">
    <template v-if="isActivePrint && hasLoadedFile">
      <div
          class="current-print-panel__hero"
          :style="{
          backgroundImage: previewUrl ? `url(${previewUrl})` : 'none',
        }"
      >
        <div v-if="!previewUrl" class="current-print-panel__placeholder">
          <v-icon icon="mdi-printer-3d" size="96" />
        </div>

        <div class="current-print-panel__topbar">
          {{ fileDisplayName }}
        </div>

        <div class="current-print-panel__bottombar">
          <div class="current-print-panel__stats">
            <div class="current-print-panel__stat">
              <div class="current-print-panel__stat-label">{{ t('print.current.progress') }}</div>
              <div class="current-print-panel__stat-value">{{ progressPercent }}%</div>
            </div>

            <div class="current-print-panel__stat current-print-panel__stat--center">
              <div class="current-print-panel__stat-label">{{ t('print.current.layer') }}</div>
              <div class="current-print-panel__stat-value">{{ currentLayer }}</div>
            </div>

            <div class="current-print-panel__stat current-print-panel__stat--right">
              <div class="current-print-panel__stat-label">{{ t('print.current.remaining') }}</div>
              <div class="current-print-panel__stat-value">{{ remainingTime }}</div>
            </div>
          </div>

          <v-progress-linear
              :model-value="progressPercent"
              color="primary"
              bg-color="rgba(255,255,255,0.25)"
              height="10"
              rounded
          />

          <div class="current-print-panel__actions">
            <v-btn
                v-if="canSkipObject"
                class="current-print-panel__action current-print-panel__action--first pr-1"
                variant="tonal"
                :disabled="loading"
                @click="skipDialogOpen = true"
            >
              <v-icon icon="mdi-debug-step-over" size="18" start />
            </v-btn>

            <template v-if="!isFinished">
              <v-btn
                  class="current-print-panel__action current-print-panel__action--middle"
                  :class="{ 'current-print-panel__action--primary': !isPaused }"
                  :color="isPaused ? 'success' : 'warning'"
                  variant="flat"
                  :disabled="loading"
                  @click="isPaused ? resumePrint() : pausePrint()"
                  :prepend-icon="isPaused ? 'mdi-play' : 'mdi-pause'"
              >
                {{ isPaused ? t('print.current.resume') : t('print.current.pause') }}
              </v-btn>

              <v-btn
                  class="current-print-panel__action current-print-panel__action--last"
                  color="error"
                  variant="flat"
                  :disabled="loading"
                  @click="cancelPrint"
                  prepend-icon="mdi-stop"
              >
                {{ t('print.current.stop') }}
              </v-btn>
            </template>

            <template v-else>
              <v-btn
                  class="current-print-panel__action current-print-panel__action--middle"
                  variant="tonal"
                  :disabled="loading"
                  @click="reprint"
                  prepend-icon="mdi-printer"
              >
                {{ t('print.current.reprint') }}
              </v-btn>

              <v-btn
                  class="current-print-panel__action current-print-panel__action--last"
                  color="primary"
                  variant="flat"
                  :disabled="loading"
                  @click="clearFile"
                  prepend-icon="mdi-broom"
              >
                {{ t('print.current.clear_file') }}
              </v-btn>
            </template>
          </div>
        </div>
      </div>

      <SkipObjectDialog v-model="skipDialogOpen" />
    </template>

    <template v-else>
      <div class="current-print-panel__idle">
        <v-icon icon="mdi-cube-outline" size="96" />
      </div>
    </template>
  </v-card>
</template>

<style scoped>
.current-print-panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

.current-print-panel__hero {
  position: relative;
  height: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  display: flex;
  align-items: stretch;
  justify-content: center;
}

.current-print-panel__placeholder,
.current-print-panel__idle {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.6;
}

.current-print-panel__topbar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 12px 14px;
  font-weight: 700;
  backdrop-filter: blur(10px);
  background: rgba(var(--v-theme-on-surface), 0.12);
}

.current-print-panel__bottombar {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  backdrop-filter: blur(10px);
  background: rgba(var(--v-theme-on-surface), 0.12);
}

.current-print-panel__stats {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 12px;
  align-items: end;
}

.current-print-panel__stat--center {
  text-align: center;
}

.current-print-panel__stat--right {
  text-align: right;
}

.current-print-panel__stat-label {
  font-size: 0.8rem;
  opacity: 0.8;
}

.current-print-panel__stat-value {
  font-size: 1rem;
  font-weight: 700;
}

.current-print-panel__actions {
  display: inline-flex;
  margin-top: 5px;
  align-items: stretch;
  align-self: flex-start;
  flex-wrap: nowrap;
  gap: 0;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.22);
  border-radius: 14px;
  overflow: hidden;
  backdrop-filter: blur(10px);
  background: rgba(var(--v-theme-surface), 0.16);
}

.current-print-panel__action {
  border-radius: 0 !important;
  min-width: 0;
  box-shadow: none !important;
  border-right: 1px solid rgba(var(--v-theme-on-surface), 0.18);
}

.current-print-panel__action:last-child {
  border-right: none;
}

.current-print-panel__action--first {
  border-top-left-radius: 14px !important;
  border-bottom-left-radius: 14px !important;
}

.current-print-panel__action--last {
  border-top-right-radius: 14px !important;
  border-bottom-right-radius: 14px !important;
}

.current-print-panel__action :deep(.v-btn__content) {
  white-space: nowrap;
}

.current-print-panel__action :deep(.v-btn__overlay) {
  opacity: 0.06;
}
</style>