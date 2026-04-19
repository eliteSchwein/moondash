<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore } from '@/stores/app'
import PrintDialog from '@/components/dialogs/PrintDialog.vue'

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker, moonrakerReady } = storeToRefs(appStore)

const FILES_PER_REQUEST = 12

type MoonrakerFile = {
  path?: string
  filename?: string
  display?: string
  modified?: number
  permissions?: string
}

type MoonrakerThumbnail = {
  width?: number
  height?: number
  size?: number
  thumbnail_path?: string
}

type MoonrakerGcodeMetadata = {
  estimated_time?: number
  filament_weight_total?: number
  filament_weight?: number
  filament?: {
    weight_total?: number
    weight?: number
  }
}

const allFiles = ref<MoonrakerFile[]>([])
const currentPage = ref(0)

const loadingFiles = ref(false)
const loadingPage = ref(false)

const pageThumbnails = ref<Record<string, MoonrakerThumbnail[]>>({})
const fileMetadata = ref<Record<string, MoonrakerGcodeMetadata>>({})

const printDialogOpen = ref(false)
const selectedFile = ref<MoonrakerFile | null>(null)
const selectedFileThumbnails = ref<MoonrakerThumbnail[]>([])
const selectedFileMetadata = ref<MoonrakerGcodeMetadata | null>(null)

const printerIsReadyForSelection = computed(() => {
  if (!moonrakerReady.value) return false

  const printState = moonraker.value.printStats.state?.toLowerCase() ?? ''
  const webhookState = moonraker.value.webhooks.state?.toLowerCase() ?? ''

  const busyPrintStates = new Set([
    'printing',
    'paused',
    'pausing',
    'resuming',
    'cancelling',
    'error',
  ])

  const readyWebhookStates = new Set(['ready'])

  if (!readyWebhookStates.has(webhookState)) return false
  if (busyPrintStates.has(printState)) return false

  return printState === '' || printState === 'standby' || printState === 'complete'
})

const moonrakerHttpBase = computed(() => {
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

const sortedFiles = computed(() => {
  return [...allFiles.value].sort((a, b) => {
    const aModified = typeof a.modified === 'number' ? a.modified : 0
    const bModified = typeof b.modified === 'number' ? b.modified : 0
    return bModified - aModified
  })
})

const pageFiles = computed(() => {
  const start = currentPage.value * FILES_PER_REQUEST
  return sortedFiles.value.slice(start, start + FILES_PER_REQUEST)
})

const canGoUp = computed(() => currentPage.value > 0)

const canGoDown = computed(() => {
  const nextStart = (currentPage.value + 1) * FILES_PER_REQUEST
  return nextStart < sortedFiles.value.length
})

function getFileName(file: MoonrakerFile): string {
  const name = file.display || file.filename || file.path || t('print_files.unknown_file')
  return name.replace(/\.gcode$/i, '')
}

function formatPrintTime(seconds: number | undefined): string {
  if (typeof seconds !== 'number' || !Number.isFinite(seconds) || seconds < 0) return ''

  let remaining = Math.floor(seconds)
  const days = Math.floor(remaining / 86400)
  remaining %= 86400
  const hours = Math.floor(remaining / 3600)
  remaining %= 3600
  const minutes = Math.floor(remaining / 60)

  const parts: string[] = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0) parts.push(`${minutes}m`)

  return parts.join('') || '0m'
}

function formatWeight(weight: number | undefined): string {
  if (typeof weight !== 'number' || !Number.isFinite(weight)) return ''
  return `${weight.toFixed(1)}g`
}

function getMetadata(file: MoonrakerFile): MoonrakerGcodeMetadata | null {
  const filePath = getFilePath(file)
  if (!filePath) return null
  return fileMetadata.value[filePath] ?? null
}

function getFilePath(file: MoonrakerFile): string | null {
  const value = file.path || file.filename || file.display
  return typeof value === 'string' && value.trim() ? value : null
}

function getBestThumbnailUrl(file: MoonrakerFile): string | null {
  const filePath = getFilePath(file)
  if (!filePath) return null

  const thumbnails = pageThumbnails.value[filePath]
  if (!Array.isArray(thumbnails) || !thumbnails.length) return null

  const sorted = [...thumbnails].sort((a, b) => {
    const aArea = (a.width ?? 0) * (a.height ?? 0)
    const bArea = (b.width ?? 0) * (b.height ?? 0)
    return bArea - aArea
  })

  const thumbnailPath = sorted[0]?.thumbnail_path
  if (!thumbnailPath || !moonrakerHttpBase.value) return null

  return `${moonrakerHttpBase.value}/server/files/gcodes/${thumbnailPath}`
}

function openPrintDialog(file: MoonrakerFile) {
  if (!printerIsReadyForSelection.value) return

  const filePath = getFilePath(file)

  selectedFile.value = file
  selectedFileThumbnails.value = filePath ? pageThumbnails.value[filePath] ?? [] : []
  selectedFileMetadata.value = filePath ? fileMetadata.value[filePath] ?? null : null
  printDialogOpen.value = true
}

async function loadFiles() {
  try {
    loadingFiles.value = true

    const result = await moonrakerClient.call<unknown>('server.files.list')

    if (Array.isArray(result)) {
      allFiles.value = result as MoonrakerFile[]
      return
    }

    if (result && typeof result === 'object') {
      const record = result as Record<string, unknown>

      if (Array.isArray(record.files)) {
        allFiles.value = record.files as MoonrakerFile[]
        return
      }

      if (Array.isArray(record.result)) {
        allFiles.value = record.result as MoonrakerFile[]
        return
      }
    }

    allFiles.value = []
  } catch (error) {
    console.error('Failed to load print files', error)
    allFiles.value = []
  } finally {
    loadingFiles.value = false
  }
}

async function loadCurrentPageThumbnails() {
  try {
    loadingPage.value = true
    pageThumbnails.value = {}
    fileMetadata.value = {}

    const entries = await Promise.all(
        pageFiles.value.map(async (file) => {
          const filePath = getFilePath(file)
          if (!filePath) return null

          const [thumbnailResult, metadataResult] = await Promise.all([
            moonrakerClient
                .call<MoonrakerThumbnail[]>('server.files.thumbnails', { filename: filePath })
                .catch(() => []),
            moonrakerClient
                .call<MoonrakerGcodeMetadata>('server.files.metadata', { filename: filePath })
                .catch(() => null),
          ])

          if (metadataResult) {
            fileMetadata.value[filePath] = metadataResult
          }

          return [filePath, Array.isArray(thumbnailResult) ? thumbnailResult : []] as const
        }),
    )

    pageThumbnails.value = Object.fromEntries(
        entries.filter(Boolean) as Array<readonly [string, MoonrakerThumbnail[]]>,
    )
  } finally {
    loadingPage.value = false
  }
}

async function nextPage() {
  if (!canGoDown.value || loadingFiles.value || loadingPage.value) return
  currentPage.value += 1
}

async function previousPage() {
  if (!canGoUp.value || loadingFiles.value || loadingPage.value) return
  currentPage.value -= 1
}

watch(currentPage, async () => {
  await loadCurrentPageThumbnails()
})

onMounted(async () => {
  currentPage.value = 0
  await loadFiles()
  await loadCurrentPageThumbnails()
})
</script>

<template>
  <div class="print-file-panel">
    <div class="print-file-panel__nav">
      <v-btn
          v-if="canGoUp"
          icon="mdi-chevron-up"
          variant="tonal"
          :disabled="loadingFiles || loadingPage"
          @click="previousPage"
      />

      <v-spacer />

      <v-btn
          v-if="canGoDown"
          icon="mdi-chevron-down"
          variant="tonal"
          :disabled="loadingFiles || loadingPage"
          @click="nextPage"
      />
    </div>

    <div v-if="loadingFiles" class="print-file-panel__state">
      <v-progress-circular indeterminate />
    </div>

    <div v-else-if="!pageFiles.length" class="print-file-panel__state">
      <v-alert type="info" variant="tonal">
        {{ t('print_files.none_found') }}
      </v-alert>
    </div>

    <div v-else class="print-file-panel__grid">
      <v-card
          v-for="file in pageFiles"
          :key="file.path || file.filename"
          class="print-file-card"
          :class="{ 'print-file-card--disabled': !printerIsReadyForSelection }"
          variant="flat"
          @click="openPrintDialog(file)"
      >
        <div class="print-file-card__title">
          {{ getFileName(file) }}
        </div>

        <div class="print-file-card__thumb-wrap">
          <img
              v-if="getBestThumbnailUrl(file)"
              :src="getBestThumbnailUrl(file)!"
              :alt="getFileName(file)"
              class="print-file-card__thumb"
          >

          <div v-else class="print-file-card__thumb-placeholder">
            <v-progress-circular
                v-if="loadingPage"
                indeterminate
                size="32"
            />
            <v-icon
                v-else
                icon="mdi-printer-3d"
                size="48"
            />
          </div>
        </div>

        <v-card-text class="print-file-card__content">
          <div class="print-file-card__meta">
            <span
                v-if="formatPrintTime(getMetadata(file)?.estimated_time)"
                class="print-file-card__meta-item"
            >
              <v-icon icon="mdi-clock-outline" size="14" />
              <span>{{ formatPrintTime(getMetadata(file)?.estimated_time) }}</span>
            </span>

            <span
                v-if="formatWeight(getMetadata(file)?.filament_weight_total ?? getMetadata(file)?.filament_weight ?? getMetadata(file)?.filament?.weight_total ?? getMetadata(file)?.filament?.weight)"
                class="print-file-card__meta-item"
            >
              <v-icon icon="mdi-scale-balance" size="14" />
              <span>{{ formatWeight(getMetadata(file)?.filament_weight_total ?? getMetadata(file)?.filament_weight ?? getMetadata(file)?.filament?.weight_total ?? getMetadata(file)?.filament?.weight) }}</span>
            </span>
          </div>
        </v-card-text>
      </v-card>
    </div>

    <PrintDialog
        v-model="printDialogOpen"
        :file="selectedFile"
        :metadata="selectedFileMetadata"
        @start="() => {}"
    />
  </div>
</template>

<style scoped>
.print-file-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px;
  box-sizing: border-box;
}

.print-file-panel__nav {
  z-index: 2;
  position: fixed;
  right: 8px;
  bottom: 8px;
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 8px;
}

.print-file-panel__state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.print-file-panel__grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 16px;
  overflow: auto;
}

.print-file-card {
  overflow: hidden;
  background: transparent !important;
  box-shadow: none !important;
  position: relative;
  display: flex;
  flex-direction: column;
  cursor: pointer;
}

.print-file-card--disabled {
  cursor: default;
  pointer-events: auto;
  opacity: 0.55;
}

.print-file-card__title,
.print-file-card__meta {
  z-index: 2;
  background: rgba(var(--v-theme-on-surface), 0.2);
  backdrop-filter: blur(10px);
}

.print-file-card__title {
  font-weight: 600;
  line-height: 1.2;
  word-break: break-word;
  padding: 8px 10px;
  border-radius: 8px 8px 0 0;
  font-size: 0.8rem;
}

.print-file-card__thumb-wrap {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  aspect-ratio: 1.3 / 1;
  background: rgba(var(--v-theme-on-surface), 0.06);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.print-file-card__thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.print-file-card__thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.print-file-card__content {
  padding: 0;
}

.print-file-card__meta {
  position: absolute;
  left: 0;
  bottom: 0;
  width: 100%;
  font-size: 0.9rem;
  opacity: 0.75;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  padding: 8px 10px;
  border-radius: 0 0 8px 8px;
}

.print-file-card__meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
</style>