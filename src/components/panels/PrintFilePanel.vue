<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore } from '@/stores/app'
import PrintDialog from '@/components/dialogs/PrintDialog.vue'
import KeyboardOverlay from '@/components/KeyboardOverlay.vue'
import PrintFilePreview from '@/components/PrintFilePreview.vue'

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker, moonrakerReady } = storeToRefs(appStore)

const FILES_PER_REQUEST = 12

type SortMode = 'modified-desc' | 'modified-asc' | 'name-asc' | 'name-desc'

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

type FolderEntry = {
  type: 'folder'
  path: string
  name: string
  modified: number
}

type FileEntry = {
  type: 'file'
  file: MoonrakerFile
  path: string
  name: string
  modified: number
}

type BrowserEntry = FolderEntry | FileEntry

const allFiles = ref<MoonrakerFile[]>([])
const currentFolder = ref('')
const currentPage = ref(0)
const sortMode = ref<SortMode>('modified-desc')
const nameFilter = ref('')
const keyboardOpen = ref(false)

const loadingFiles = ref(false)
const loadingPage = ref(false)

const pageThumbnails = ref<Record<string, MoonrakerThumbnail[]>>({})
const fileMetadata = ref<Record<string, MoonrakerGcodeMetadata>>({})

const printDialogOpen = ref(false)
const selectedFile = ref<MoonrakerFile | null>(null)
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

const breadcrumbs = computed(() => {
  if (!currentFolder.value) return []

  const parts = currentFolder.value.split('/').filter(Boolean)
  return parts.map((part, index) => ({
    name: part,
    path: parts.slice(0, index + 1).join('/'),
  }))
})

const browserEntries = computed<BrowserEntry[]>(() => {
  const folderMap = new Map<string, FolderEntry>()
  const files: FileEntry[] = []
  const folderPrefix = currentFolder.value ? `${currentFolder.value}/` : ''
  const normalizedFilter = nameFilter.value.trim().toLowerCase()

  for (const file of allFiles.value) {
    const path = getFilePath(file)
    if (!path) continue
    if (currentFolder.value && path !== currentFolder.value && !path.startsWith(folderPrefix)) continue

    const relativePath = currentFolder.value ? path.slice(folderPrefix.length) : path
    if (!relativePath || relativePath === path && currentFolder.value && !path.startsWith(folderPrefix)) continue

    const [firstSegment, ...rest] = relativePath.split('/').filter(Boolean)
    if (!firstSegment) continue

    const modified = typeof file.modified === 'number' ? file.modified : 0

    if (rest.length) {
      if (normalizedFilter && !firstSegment.toLowerCase().includes(normalizedFilter)) continue

      const folderPath = folderPrefix ? `${folderPrefix}${firstSegment}` : firstSegment
      const existing = folderMap.get(folderPath)
      if (!existing || modified > existing.modified) {
        folderMap.set(folderPath, {
          type: 'folder',
          path: folderPath,
          name: firstSegment,
          modified,
        })
      }
      continue
    }

    const name = getFileName(file)
    if (normalizedFilter && !name.toLowerCase().includes(normalizedFilter)) continue

    files.push({
      type: 'file',
      file,
      path,
      name,
      modified,
    })
  }

  return [...folderMap.values(), ...files].sort(compareEntries)
})

const pageEntries = computed(() => {
  const start = currentPage.value * FILES_PER_REQUEST
  return browserEntries.value.slice(start, start + FILES_PER_REQUEST)
})

const pageFolders = computed(() => pageEntries.value.filter((entry): entry is FolderEntry => entry.type === 'folder'))
const pageFiles = computed(() => pageEntries.value.filter((entry): entry is FileEntry => entry.type === 'file'))
const canGoUp = computed(() => currentPage.value > 0)

const canGoDown = computed(() => {
  const nextStart = (currentPage.value + 1) * FILES_PER_REQUEST
  return nextStart < browserEntries.value.length
})

const sortButtons = computed<Array<{ mode: SortMode; icon: string; label: string }>>(() => [
  { mode: 'modified-desc', icon: 'mdi-sort-clock-descending-outline', label: 'Modified newest first' },
  { mode: 'modified-asc', icon: 'mdi-sort-clock-ascending-outline', label: 'Modified oldest first' },
  { mode: 'name-asc', icon: 'mdi-sort-alphabetical-ascending', label: 'Name A to Z' },
  { mode: 'name-desc', icon: 'mdi-sort-alphabetical-descending', label: 'Name Z to A' },
])

function compareEntries(a: BrowserEntry, b: BrowserEntry): number {
  if (a.type !== b.type) return a.type === 'folder' ? -1 : 1

  switch (sortMode.value) {
    case 'modified-asc':
      return a.modified - b.modified || a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
    case 'name-asc':
      return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
    case 'name-desc':
      return b.name.localeCompare(a.name, undefined, { sensitivity: 'base' })
    case 'modified-desc':
    default:
      return b.modified - a.modified || a.name.localeCompare(b.name, undefined, { sensitivity: 'base' })
  }
}

function getFileName(file: MoonrakerFile): string {
  const name = file.display || file.filename || file.path || t('print_files.unknown_file')
  const pathParts = name.split('/').filter(Boolean)
  const leafName = pathParts.length ? pathParts[pathParts.length - 1] : name
  return leafName.replace(/\.gcode$/i, '')
}

function getMetadata(file: MoonrakerFile): MoonrakerGcodeMetadata | null {
  const filePath = getFilePath(file)
  if (!filePath) return null
  return fileMetadata.value[filePath] ?? null
}

function getFilePath(file: MoonrakerFile): string | null {
  const value = file.path || file.filename || file.display
  return typeof value === 'string' && value.trim() ? value.replace(/^\/+/, '') : null
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
  selectedFileMetadata.value = filePath ? fileMetadata.value[filePath] ?? null : null
  printDialogOpen.value = true
}

async function openFolder(path: string) {
  currentFolder.value = path
  currentPage.value = 0
  await loadFiles()
  await loadCurrentPageThumbnails()
}

async function goBackFolder() {
  if (!currentFolder.value) return

  const parts = currentFolder.value.split('/').filter(Boolean)
  parts.pop()
  currentFolder.value = parts.join('/')
  currentPage.value = 0

  await loadFiles()
  await loadCurrentPageThumbnails()
}

async function goRootFolder() {
  currentFolder.value = ''
  currentPage.value = 0

  await loadFiles()
  await loadCurrentPageThumbnails()
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
        pageFiles.value.map(async (entry) => {
          const filePath = entry.path
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

watch([currentFolder, sortMode, nameFilter], async () => {
  currentPage.value = 0
  await loadCurrentPageThumbnails()
})

onMounted(async () => {
  currentPage.value = 0
  await loadFiles()
  await loadCurrentPageThumbnails()
})
</script>

<template>
  <div
      class="print-file-panel"
      :class="{ 'print-file-panel--subfolder': currentFolder }"
  >
    <v-toolbar
        class="print-file-panel__toolbar px-2"
        density="compact"
        rounded
        flat
    >
      <v-btn-toggle
          v-model="sortMode"
          mandatory
          density="comfortable"
          divided
      >
        <v-btn
            v-for="button in sortButtons"
            :key="button.mode"
            :value="button.mode"
            :icon="button.icon"
            :aria-label="button.label"
            :title="button.label"
            style="min-width: 50px;"
        />
      </v-btn-toggle>

      <v-text-field
          v-model="nameFilter"
          class="print-file-panel__filter"
          density="compact"
          variant="plain"
          hide-details
          readonly
          prepend-inner-icon="mdi-filter-outline"
          :placeholder="t('print_files.filter')"
          single-line
          @click="keyboardOpen = true"
          @focus="keyboardOpen = true"
      />
    </v-toolbar>

    <div v-if="currentFolder" class="print-file-panel__breadcrumbs">
      <v-btn
          icon="mdi-arrow-left"
          density="comfortable"
          variant="text"
          @click="goBackFolder"
      />
      <v-btn
          size="small"
          variant="text"
          prepend-icon="mdi-home-outline"
          @click="goRootFolder"
      >

      </v-btn>
      <template v-for="crumb in breadcrumbs" :key="crumb.path">
        <v-icon icon="mdi-chevron-right" size="16" />
        <v-btn
            size="small"
            variant="text"
            @click="openFolder(crumb.path)"
        >
          {{ crumb.name }}
        </v-btn>
      </template>
    </div>

    <div class="print-file-panel__nav">
      <v-btn
          v-if="canGoUp"
          icon="mdi-chevron-up"
          :disabled="loadingFiles || loadingPage"
          @click="previousPage"
      />

      <v-spacer />

      <v-btn
          v-if="canGoDown"
          icon="mdi-chevron-down"
          :disabled="loadingFiles || loadingPage"
          @click="nextPage"
      />
    </div>

    <div v-if="loadingFiles" class="print-file-panel__state">
      <v-progress-circular indeterminate />
    </div>

    <div v-else-if="!pageEntries.length" class="print-file-panel__state">
      <v-alert type="info" variant="tonal">
        {{ t('print_files.none_found') }}
      </v-alert>
    </div>

    <div v-else class="print-file-panel__grid">
      <v-card
          v-for="entry in pageFolders"
          :key="`folder:${entry.path}`"
          class="print-folder-card pa-0"
          variant="flat"
          @click="openFolder(entry.path)"
      >
        <v-icon icon="mdi-folder" style="font-size: 48px;" />
        <div class="print-folder-card__title">
          {{ entry.name }}
        </div>
      </v-card>

      <PrintFilePreview
          v-for="entry in pageFiles"
          :key="entry.path"
          :file="entry.file"
          :title="entry.name"
          :thumbnail-url="getBestThumbnailUrl(entry.file)"
          :metadata="getMetadata(entry.file)"
          :loading="loadingPage"
          :disabled="!printerIsReadyForSelection"
          @select="openPrintDialog"
      />
    </div>

    <KeyboardOverlay
        v-model="nameFilter"
        :visible="keyboardOpen"
        :title="t('print_files.filter_description')"
        @enter="keyboardOpen = false"
        @close="keyboardOpen = false"
    />

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

.print-file-panel__toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
}

.print-file-panel__filter {
  max-width: 320px;
  margin-left: auto;
}

.print-file-panel__breadcrumbs {
  display: flex;
  align-items: center;
  gap: 2px;
  min-height: 36px;
  overflow-x: auto;
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
  --print-card-height: 25.5vh;
  flex: 1;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  grid-auto-rows: var(--print-card-height);
  gap: 16px;
  overflow: auto;
}

.print-file-panel--subfolder .print-file-panel__grid {
  --print-card-height: 22vh;
}

.print-folder-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  cursor: pointer;
  box-shadow: none !important;
  height: 100%;
  min-height: 0;
}

.print-folder-card__title {
  max-width: 100%;
  font-weight: 600;
  line-height: 1.2;
  text-align: center;
  word-break: break-word;
}

.print-file-panel__filter :deep(.v-field) {
  align-items: center;
}

.print-file-panel__filter :deep(.v-field__field) {
  align-items: center;
}

.print-file-panel__filter :deep(.v-field__input) {
  min-height: 36px;
  padding-top: 0;
  padding-bottom: 0;
  align-items: center;
}

.print-file-panel__filter :deep(.v-field__prepend-inner) {
  transform: translateY(-4px);
}
</style>
