<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore } from '@/stores/app'
import router from '../../router'

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

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
  filament_weights?: unknown
  filament_colors?: unknown
  extruder_colors?: unknown
  filament_type?: unknown
  filament_name?: unknown
}

type LaneOption = {
  key: string
  label: string
  color: string
}

type ToolMapEntry = {
  tool: string
  lane: string
}

type MoonrakerObjectsQueryResult = {
  status?: Record<string, unknown>
}

const props = defineProps<{
  modelValue: boolean
  file: MoonrakerFile | null
  metadata?: MoonrakerGcodeMetadata | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'start', payload: {
    file: MoonrakerFile
    timelapse: boolean
    toolMap: ToolMapEntry[]
  }): void
}>()

function showErrorToast(message: string) {
  const store = appStore as unknown as {
    showToast?: (payload: { message: string; color?: string }) => void
  }

  if (typeof store.showToast === 'function') {
    store.showToast({
      message,
      color: 'error',
    })
    return
  }

  console.error(message)
}

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => {
    if (saving.value) return
    emit('update:modelValue', value)
  },
})

const displayName = computed(() => {
  const file = props.file
  if (!file) return ''

  const raw = file.display || file.filename || file.path || t('print_files.unknown_file')
  return raw.replace(/\.gcode$/i, '')
})

const estimatedTimeLabel = computed(() => {
  const seconds = props.metadata?.estimated_time
  if (typeof seconds !== 'number' || !Number.isFinite(seconds) || seconds < 0) return '--'

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
})

const filamentWeightLabel = computed(() => {
  const weight =
      props.metadata?.filament_weight_total ??
      props.metadata?.filament_weight ??
      props.metadata?.filament?.weight_total ??
      props.metadata?.filament?.weight

  if (typeof weight !== 'number' || !Number.isFinite(weight)) return '--'
  return `${weight.toFixed(1)}g`
})

const timelapseEnabled = ref(false)
const saving = ref(false)
const syncingTimelapse = ref(false)
const selectedLaneByTool = ref<Record<string, string>>({})
const dialogThumbnails = ref<MoonrakerThumbnail[]>([])
const loadingThumbnails = ref(false)

function getRawObjects(): Record<string, unknown> {
  return moonraker.value.rawObjects as Record<string, unknown>
}

function getAfcObjects(): Record<string, unknown> {
  return moonraker.value.afc.objects as Record<string, unknown>
}

function getTimelapseSettingsState(source?: Record<string, unknown>): Record<string, unknown> | null {
  const objects = source ?? getRawObjects()
  const settings = objects.timelapse_settings

  return settings && typeof settings === 'object' && !Array.isArray(settings)
      ? (settings as Record<string, unknown>)
      : null
}

function getLegacyTimelapseMacroState(source?: Record<string, unknown>): Record<string, unknown> | null {
  const objects = source ?? getRawObjects()
  const macro =
      objects['gcode_macro TIMELAPSE_TAKE_FRAME'] ??
      objects['gcode_macro timelapse_take_frame']

  return macro && typeof macro === 'object' && !Array.isArray(macro)
      ? (macro as Record<string, unknown>)
      : null
}

function extractTimelapseEnabled(source?: Record<string, unknown>): boolean {
  const macro = getLegacyTimelapseMacroState(source)

  if (macro && typeof macro.enable !== 'undefined') {
    return Boolean(macro.enable)
  }

  if (macro && typeof macro.enabled !== 'undefined') {
    return Boolean(macro.enabled)
  }

  if (macro && typeof macro.variable_enable !== 'undefined') {
    return Boolean(macro.variable_enable)
  }

  const settings = getTimelapseSettingsState(source)
  if (settings && typeof settings.enabled !== 'undefined') {
    return Boolean(settings.enabled)
  }

  return false
}

const timelapseAvailable = computed(() => {
  const objects = getRawObjects()

  return Boolean(
      objects['gcode_macro TIMELAPSE_TAKE_FRAME'] ||
      objects['gcode_macro timelapse_take_frame'] ||
      objects.timelapse_settings ||
      objects.timelapse ||
      objects.timelapse_mode,
  )
})

function syncTimelapseStateFromStore() {
  const enabled = extractTimelapseEnabled()
  timelapseEnabled.value = enabled
  return enabled
}

function mergeRawObjectUpdate(update: Record<string, unknown>) {
  const rawObjects = getRawObjects()

  for (const [key, value] of Object.entries(update)) {
    const existing = rawObjects[key]

    if (
        existing &&
        typeof existing === 'object' &&
        !Array.isArray(existing) &&
        value &&
        typeof value === 'object' &&
        !Array.isArray(value)
    ) {
      rawObjects[key] = {
        ...(existing as Record<string, unknown>),
        ...(value as Record<string, unknown>),
      }
    } else {
      rawObjects[key] = value
    }
  }
}

function mergeAfcObjectUpdate(update: Record<string, unknown>) {
  const afcObjects = getAfcObjects()

  for (const [key, value] of Object.entries(update)) {
    const lower = key.toLowerCase()

    const isAfcKey =
        lower === 'afc' ||
        lower.startsWith('afc ') ||
        lower.startsWith('afc_') ||
        lower.includes(' afc ') ||
        key.startsWith('AFC')

    if (!isAfcKey) continue

    const existing = afcObjects[key]

    if (
        existing &&
        typeof existing === 'object' &&
        !Array.isArray(existing) &&
        value &&
        typeof value === 'object' &&
        !Array.isArray(value)
    ) {
      afcObjects[key] = {
        ...(existing as Record<string, unknown>),
        ...(value as Record<string, unknown>),
      }
    } else {
      afcObjects[key] = value
    }
  }
}

async function refreshTimelapseStateFromMoonraker() {
  syncingTimelapse.value = true

  try {
    const result = await moonrakerClient.call<MoonrakerObjectsQueryResult>('printer.objects.query', {
      objects: {
        timelapse_settings: null,
        'gcode_macro TIMELAPSE_TAKE_FRAME': null,
        'gcode_macro timelapse_take_frame': null,
      },
    })

    const status = result?.status ?? {}
    mergeRawObjectUpdate(status)

    const enabled = extractTimelapseEnabled(status)
    timelapseEnabled.value = enabled

    return enabled
  } catch {
    const fallback = extractTimelapseEnabled()
    timelapseEnabled.value = fallback
    return fallback
  } finally {
    syncingTimelapse.value = false
  }
}

function getAfcStepperObjectKeys(): string[] {
  const afcObjects = getAfcObjects()
  const afcRoot = afcObjects.AFC as Record<string, unknown> | undefined

  if (afcRoot && Array.isArray(afcRoot.lanes)) {
    return afcRoot.lanes
        .map((laneName) => String(laneName))
        .filter(Boolean)
        .map((laneName) =>
            laneName.toLowerCase().startsWith('afc_stepper ')
                ? laneName
                : `AFC_stepper ${laneName}`,
        )
  }

  return Object.keys(afcObjects).filter((key) => key.toLowerCase().startsWith('afc_stepper '))
}

async function refreshAfcMappingsFromMoonraker() {
  try {
    const objectsToQuery: Record<string, null> = {
      AFC: null,
    }

    for (const key of getAfcStepperObjectKeys()) {
      objectsToQuery[key] = null
    }

    const result = await moonrakerClient.call<MoonrakerObjectsQueryResult>('printer.objects.query', {
      objects: objectsToQuery,
    })

    const status = result?.status ?? {}
    mergeAfcObjectUpdate(status)
  } catch {
    showErrorToast(t('print.dialog.lane_update_failed'))
  }
}

function syncSelectedLanesFromAfc() {
  const next: Record<string, string> = {}

  for (const tool of requiredTools.value) {
    next[tool] = getCurrentLaneForTool(tool)
  }

  selectedLaneByTool.value = next
}

watch(
    () => [
      moonraker.value.rawObjects['gcode_macro TIMELAPSE_TAKE_FRAME'],
      moonraker.value.rawObjects['gcode_macro timelapse_take_frame'],
      moonraker.value.rawObjects.timelapse_settings,
    ],
    () => {
      if (!props.modelValue) return
      if (saving.value || syncingTimelapse.value) return

      syncTimelapseStateFromStore()
    },
    { deep: true, immediate: true },
)

async function onTimelapseSwitchUpdate() {
  if (saving.value || !timelapseAvailable.value) return

  const currentEnabled = await refreshTimelapseStateFromMoonraker()
  const nextEnabled = !currentEnabled

  try {
    saving.value = true

    await moonrakerClient.call('machine.timelapse.post_settings', {
      enabled: nextEnabled,
    })

    await refreshTimelapseStateFromMoonraker()
  } catch {
    showErrorToast(t('print.dialog.timelapse_update_failed'))
    await refreshTimelapseStateFromMoonraker()
  } finally {
    saving.value = false
  }
}

const afcEnabled = computed(() => Boolean(moonraker.value.afc.available))

function toToolListFromFilamentWeights(value: unknown): string[] {
  if (!Array.isArray(value)) return []

  return value
      .map((item, index) => {
        const weight = typeof item === 'number' ? item : Number(item)
        if (!Number.isFinite(weight) || weight <= 0) return null
        return `T${index}`
      })
      .filter((tool): tool is string => Boolean(tool))
}

const requiredTools = computed(() => {
  const tools = toToolListFromFilamentWeights(props.metadata?.filament_weights)
  return Array.from(new Set(tools))
})

const hasAfcToolSection = computed(() => afcEnabled.value && requiredTools.value.length > 0)

function normalizeColor(color: unknown): string {
  if (typeof color !== 'string') return '#434343'
  const value = color.trim()
  return value || '#434343'
}

function parseColorToRgb(color: string): { r: number; g: number; b: number } | null {
  const value = color.trim().toLowerCase()

  if (!value.startsWith('#')) return null

  const hex = value.slice(1)

  if (hex.length === 3) {
    const r = parseInt(hex[0] + hex[0], 16)
    const g = parseInt(hex[1] + hex[1], 16)
    const b = parseInt(hex[2] + hex[2], 16)
    return { r, g, b }
  }

  if (hex.length === 6) {
    const r = parseInt(hex.slice(0, 2), 16)
    const g = parseInt(hex.slice(2, 4), 16)
    const b = parseInt(hex.slice(4, 6), 16)
    return { r, g, b }
  }

  return null
}

function isBrightColor(color: string): boolean {
  const rgb = parseColorToRgb(color)
  if (!rgb) return false

  const { r, g, b } = rgb
  const luminance = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255
  return luminance >= 0.65
}

function getReadableTextColor(backgroundColor: string): string {
  return isBrightColor(backgroundColor) ? '#000000' : '#FFFFFF'
}

function getToolColor(tool: string): string {
  const index = Number(tool.slice(1))
  const colors = props.metadata?.filament_colors

  if (!Array.isArray(colors)) return '#434343'

  const value = colors[index]
  return normalizeColor(value)
}

function laneOptionsFromAfc(): LaneOption[] {
  const objects = moonraker.value.afc.objects as Record<string, any>
  const afcRoot = objects.AFC

  if (!afcRoot) return []

  const laneNames: string[] = Array.isArray(afcRoot.lanes)
      ? afcRoot.lanes
      : Object.keys(objects).filter((key) => key.toLowerCase().startsWith('afc_stepper '))

  return laneNames.map((laneName) => {
    const laneKey = laneName.toLowerCase().startsWith('afc_stepper ')
        ? laneName
        : `AFC_stepper ${laneName}`

    const laneObject = objects[laneKey] ?? {}
    const label =
        typeof laneObject.label === 'string' && laneObject.label.trim()
            ? laneObject.label.trim()
            : typeof laneObject.name === 'string' && laneObject.name.trim()
                ? laneObject.name.trim()
                : laneName.replace(/^AFC_stepper\s+/i, '')

    const color = normalizeColor(laneObject.color)

    return { key: laneKey, label, color }
  })
}

const laneOptions = computed<LaneOption[]>(() => laneOptionsFromAfc())

const laneSelectItems = computed(() => {
  return laneOptions.value.map((lane) => ({
    title: lane.label,
    value: lane.key,
  }))
})

function laneDisplayName(laneKey: string): string {
  if (!laneKey) return ''
  const lane = laneOptions.value.find((entry) => entry.key === laneKey)
  if (!lane) return laneKey.replace(/^AFC_stepper\s+/i, '')
  return lane.label
}

function laneDisplayColor(laneKey: string): string {
  if (!laneKey) return 'transparent'
  const lane = laneOptions.value.find((entry) => entry.key === laneKey)
  return lane?.color ?? '#434343'
}

function getLaneMacroName(laneKey: string): string {
  const lane = moonraker.value.afc.objects[laneKey] as Record<string, unknown> | undefined

  if (lane && typeof lane.name === 'string' && lane.name.trim()) {
    return lane.name.trim()
  }

  return laneKey.replace(/^AFC_stepper\s+/i, '').trim()
}

function getCurrentLaneForTool(tool: string): string {
  const objects = moonraker.value.afc.objects as Record<string, any>
  const afcRoot = objects.AFC

  if (!afcRoot) return ''

  const laneNames: string[] = Array.isArray(afcRoot.lanes)
      ? afcRoot.lanes
      : Object.keys(objects).filter((key) => key.toLowerCase().startsWith('afc_stepper '))

  for (const laneName of laneNames) {
    const laneKey = laneName.toLowerCase().startsWith('afc_stepper ')
        ? laneName
        : `AFC_stepper ${laneName}`

    const laneObject = objects[laneKey] ?? {}

    const laneTool =
        typeof laneObject.map === 'string'
            ? laneObject.map
            : typeof laneObject.tool === 'string'
                ? laneObject.tool
                : typeof laneObject.current_tool === 'string'
                    ? laneObject.current_tool
                    : null

    if (!laneTool) continue

    const normalized = laneTool.trim().toUpperCase()
    if (normalized === tool.toUpperCase()) return laneKey
  }

  return ''
}

function getFilePath(file: MoonrakerFile | null): string | null {
  const value = file?.path || file?.filename || file?.display
  return typeof value === 'string' && value.trim() ? value : null
}

async function loadDialogThumbnails(file: MoonrakerFile | null) {
  const filePath = getFilePath(file)

  if (!filePath) {
    dialogThumbnails.value = []
    return
  }

  try {
    loadingThumbnails.value = true

    const result = await moonrakerClient.call<unknown>('server.files.thumbnails', {
      filename: filePath,
    })

    if (Array.isArray(result)) {
      dialogThumbnails.value = result as MoonrakerThumbnail[]
      return
    }

    if (result && typeof result === 'object') {
      const record = result as Record<string, unknown>

      if (Array.isArray(record.thumbnails)) {
        dialogThumbnails.value = record.thumbnails as MoonrakerThumbnail[]
        return
      }

      if (Array.isArray(record.result)) {
        dialogThumbnails.value = record.result as MoonrakerThumbnail[]
        return
      }
    }

    dialogThumbnails.value = []
  } catch {
    dialogThumbnails.value = []
  } finally {
    loadingThumbnails.value = false
  }
}

const thumbnailUrl = computed(() => {
  const filePath = getFilePath(props.file)
  if (!filePath) return null

  const thumbnails = dialogThumbnails.value
  if (!Array.isArray(thumbnails) || !thumbnails.length) return null

  const sorted = [...thumbnails].sort((a, b) => {
    const aArea = (a.width ?? 0) * (a.height ?? 0)
    const bArea = (b.width ?? 0) * (b.height ?? 0)
    return bArea - aArea
  })

  const selectedThumb = sorted.find((item) => item?.thumbnail_path)
  const thumbnailPath = selectedThumb?.thumbnail_path
  if (!thumbnailPath) return null

  const wsUrl = moonrakerClient.getStatus().url
  if (!wsUrl) return null

  try {
    const parsed = new URL(wsUrl)
    const protocol = parsed.protocol === 'wss:' ? 'https:' : 'http:'
    const base = `${protocol}//${parsed.host}`

    const fileDir = filePath.includes('/')
        ? filePath.slice(0, filePath.lastIndexOf('/'))
        : ''

    const normalizedThumbPath = thumbnailPath.replace(/^\.?\//, '')
    const fullThumbPath = fileDir
        ? `${fileDir}/${normalizedThumbPath}`
        : normalizedThumbPath

    return `${base}/server/files/gcodes/${encodeURI(fullThumbPath)}`
  } catch {
    return null
  }
})

watch(
    () => [props.modelValue, props.file],
    async ([open, file]) => {
      if (!open || !file) return

      await refreshTimelapseStateFromMoonraker()
      await refreshAfcMappingsFromMoonraker()
      if(file !== true)
        await loadDialogThumbnails(file)
      syncSelectedLanesFromAfc()
    },
    { immediate: true },
)

watch(
    () => props.modelValue,
    (open) => {
      if (!open) {
        dialogThumbnails.value = []
      }
    },
)

watch(requiredTools, () => {
  if (!props.modelValue) return
  syncSelectedLanesFromAfc()
})

function closeDialog() {
  if (saving.value) return
  dialogOpen.value = false
}

async function setLaneForTool(tool: string, lane: string) {
  if (saving.value || !lane) return

  const previousState = { ...selectedLaneByTool.value }

  selectedLaneByTool.value = {
    ...selectedLaneByTool.value,
    [tool]: lane,
  }

  try {
    saving.value = true

    const laneName = getLaneMacroName(lane)
    await moonrakerClient.call('printer.gcode.script', {
      script: `SET_MAP LANE=${laneName} MAP=${tool}`,
    })

    await refreshAfcMappingsFromMoonraker()
    syncSelectedLanesFromAfc()
  } catch {
    selectedLaneByTool.value = previousState
    showErrorToast(t('print.dialog.lane_update_failed'))
  } finally {
    saving.value = false
  }
}

async function startPrint() {
  if (!props.file || saving.value) return

  const filePath = getFilePath(props.file)
  if (!filePath) {
    showErrorToast(t('print.dialog.print_start_failed'))
    return
  }

  const toolMap = requiredTools.value
      .map((tool) => ({
        tool,
        lane: selectedLaneByTool.value[tool],
      }))
      .filter((entry): entry is ToolMapEntry => Boolean(entry.lane))

  try {
    saving.value = true

    await moonrakerClient.call('printer.print.start', {
      filename: filePath,
    })

    emit('start', {
      file: props.file,
      timelapse: timelapseEnabled.value,
      toolMap,
    })

    dialogOpen.value = false
    await router.push('/')
  } catch {
    showErrorToast(t('print.dialog.print_start_failed'))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="1200">
    <v-card v-if="file" rounded="lg" class="pa-0">
      <v-card-text class="pa-0">
        <div class="print-dialog-layout">
          <div class="print-dialog-left">
            <div
                class="print-dialog-thumb"
                :style="{
                backgroundImage: thumbnailUrl ? `url(${thumbnailUrl})` : 'none',
              }"
            >
              <div v-if="!thumbnailUrl" class="print-dialog-thumb__placeholder">
                <v-progress-circular
                    v-if="loadingThumbnails"
                    indeterminate
                    size="40"
                />
                <v-icon
                    v-else
                    icon="mdi-printer-3d"
                    size="72"
                />
              </div>
            </div>

            <div class="print-dialog-info-bar">
              <div class="print-dialog-info-bar__name">
                {{ displayName }}
              </div>

              <div class="print-dialog-info-bar__meta">
                <span class="print-dialog-info-bar__meta-item">
                  <v-icon icon="mdi-clock-outline" size="16" />
                  <span>{{ estimatedTimeLabel }}</span>
                </span>

                <span class="print-dialog-info-bar__meta-item">
                  <v-icon icon="mdi-scale-balance" size="16" />
                  <span>{{ filamentWeightLabel }}</span>
                </span>
              </div>
            </div>
          </div>

          <div class="print-dialog-right pa-2">
            <div v-if="hasAfcToolSection" class="tool-strip-wrap">
              <div class="tool-strip">
                <div
                    v-for="tool in requiredTools"
                    :key="tool"
                    class="tool-strip__item"
                    :style="{
                    backgroundColor: getToolColor(tool),
                    color: getReadableTextColor(getToolColor(tool)),
                  }"
                >
                  <div class="tool-strip__select-wrap">
                    <v-select
                        icon-color="transparent"
                        :model-value="selectedLaneByTool[tool] || ''"
                        :items="laneSelectItems"
                        item-title="title"
                        item-value="value"
                        hide-details
                        variant="solo"
                        density="compact"
                        class="tool-strip__select"
                        :menu-props="{
                          contentClass: 'tool-strip__menu',
                          maxHeight: 320,
                          zIndex: 9999
                        }"
                        :disabled="saving || !laneSelectItems.length"
                        @update:model-value="(value) => setLaneForTool(tool, typeof value === 'string' ? value : '')"
                    >
                      <template #item="{ props: itemProps, item }">
                        <v-list-item
                            v-bind="itemProps"
                            :title="item.title"
                            class="tool-strip__dropdown-item"
                            :style="{
                              backgroundColor: laneDisplayColor(String(item.value)),
                              color: getReadableTextColor(laneDisplayColor(String(item.value))),
                            }"
                        />
                      </template>
                    </v-select>
                  </div>

                  <div
                      class="tool-strip__bottom-bar"
                      :style="{
                      backgroundColor: selectedLaneByTool[tool]
                        ? laneDisplayColor(selectedLaneByTool[tool] || '')
                        : 'transparent',
                      color: selectedLaneByTool[tool]
                        ? getReadableTextColor(laneDisplayColor(selectedLaneByTool[tool] || ''))
                        : 'inherit',
                    }"
                  >
                    <span class="tool-strip__bottom-label">
                      {{ laneDisplayName(selectedLaneByTool[tool] || '') }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="timelapseAvailable" class="timelapse-row pr-2 pl-1">
              <div class="timelapse-row__label">
                {{ t('print.dialog.timelapse') }}
              </div>

              <v-switch
                  :model-value="timelapseEnabled"
                  :color="timelapseEnabled ? 'primary' : undefined"
                  hide-details
                  inset
                  density="compact"
                  class="timelapse-row__switch"
                  :disabled="saving"
                  @update:model-value="onTimelapseSwitchUpdate"
              />
            </div>

            <div class="print-dialog-actions">
              <v-btn variant="text" :disabled="saving" @click="closeDialog">
                {{ t('print.dialog.cancel') }}
              </v-btn>
              <v-btn color="primary" variant="flat" :disabled="saving" @click="startPrint">
                {{ t('print.dialog.print') }}
              </v-btn>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.print-dialog-layout {
  display: grid;
  grid-template-columns: minmax(360px, 1.25fr) minmax(360px, 1fr);
  align-items: stretch;
}

.print-dialog-left {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}

.print-dialog-thumb {
  position: relative;
  flex: 1;
  min-height: 360px;
  height: 100%;
  border-radius: 18px;
  overflow: hidden;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  display: flex;
  align-items: center;
  justify-content: center;
}

.print-dialog-thumb__placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.55;
  width: 100%;
  height: 100%;
}

.print-dialog-info-bar {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 12px 14px;
  background: rgba(var(--v-theme-on-surface), 0.08);
  display: flex;
  flex-direction: column;
  gap: 8px;
  backdrop-filter: blur(10px);
}

.print-dialog-info-bar__name {
  font-size: 1rem;
  font-weight: 700;
  line-height: 1.2;
  word-break: break-word;
}

.print-dialog-info-bar__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  opacity: 0.85;
}

.print-dialog-info-bar__meta-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.print-dialog-right {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  height: 100%;
  background: rgba(var(--v-theme-on-surface), 0.08);
}

.tool-strip-wrap {
  width: 100%;
  max-height: 16.5rem;
  min-height: 0;
  height: 16.5rem;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1 1 auto;
}

.tool-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  width: 100%;
  max-width: 100%;
  justify-content: start;
  align-items: stretch;
  min-height: 0;
}

.tool-strip__item {
  flex: 1 1 calc(25% - 12px);
  max-width: calc(25% - 12px);
  min-width: 60px;
  border-radius: 16px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.18);
  box-sizing: border-box;
}

.tool-strip__select-wrap {
  width: 100%;
  max-height: 100%;
  min-height: 0;
  overflow: hidden;
}

.tool-strip__select {
  width: 100%;
  max-width: 100%;
  border-radius: 0;
  background: transparent;
}

.tool-strip__select :deep(.v-input),
.tool-strip__select :deep(.v-field),
.tool-strip__select :deep(.v-field__input),
.tool-strip__select :deep(.v-select__selection) {
  max-width: 100%;
  min-width: 0;
}

.tool-strip__select :deep(.v-field) {
  box-shadow: none;
  border-radius: 0;
  background: transparent;
  min-height: 84px;
  width: 100%;
  max-width: 100%;
}

.tool-strip__select :deep(.v-field__input) {
  min-height: 84px;
  align-items: center;
  justify-content: center;
  width: 100%;
  max-width: 100%;
}

.tool-strip__select :deep(.v-select__selection) {
  display: flex;
  width: 100%;
  justify-content: center;
  max-width: 100%;
}

.tool-strip__select :deep(.v-select__selection-text) {
  display: none;
}

.tool-strip__select :deep(.v-field__overlay) {
  display: none;
}

.tool-strip__select :deep(.v-overlay__content .v-list) {
  background: transparent;
}

.tool-strip__selection {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  padding: 4px 2px;
  box-sizing: border-box;
}

.tool-strip__selection-material {
  display: none;
}

.tool-strip__dropdown-item :deep(.v-list-item__overlay) {
  opacity: 0.08;
}

.tool-strip__dropdown-item :deep(.v-list-item-title) {
  font-weight: 600;
}

.tool-strip__bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px;
  font-weight: 700;
  font-size: 0.95rem;
  min-height: 40px;
}

.tool-strip__bottom-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.timelapse-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-top: 10px;
  margin-bottom: 4px;
}

.timelapse-row__label {
  font-weight: 600;
}

.timelapse-row__switch {
  width: auto;
  max-width: none;
  flex: 0 0 auto;
  min-width: 0;
}

.print-dialog-actions {
  margin-top: auto;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 18px;
}
</style>