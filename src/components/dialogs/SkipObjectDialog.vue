<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore } from '@/stores/app'

type PointLike =
    | [number, number]
    | { x?: number; y?: number }

type ExcludeObjectDef = {
  name?: string
  polygon?: PointLike[]
}

type PreviewObject = {
  name: string
  excluded: boolean
  current: boolean
  selected: boolean
  points: Array<{ x: number; y: number }>
  hasPreview: boolean
}

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const loading = ref(false)
const selectedObjectName = ref('')

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

const excludeState = computed(() => {
  const raw = moonraker.value.rawObjects['exclude_object']
  return raw && typeof raw === 'object' && !Array.isArray(raw)
      ? (raw as Record<string, unknown>)
      : {}
})

const excludedNames = computed(() => {
  const excluded = excludeState.value.excluded_objects
  return Array.isArray(excluded) ? excluded.map(String) : []
})

const currentObject = computed(() => {
  const value = excludeState.value.current_object
  return typeof value === 'string' ? value : ''
})

function normalizePoint(point: PointLike): { x: number; y: number } | null {
  if (Array.isArray(point) && point.length >= 2) {
    const x = Number(point[0])
    const y = Number(point[1])
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y }
    return null
  }

  if (point && typeof point === 'object') {
    const x = Number(point.x)
    const y = Number(point.y)
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y }
  }

  return null
}

const objectItems = computed(() => {
  const objects = excludeState.value.objects
  if (!Array.isArray(objects)) return []

  return (objects as ExcludeObjectDef[])
      .map((item) => {
        const name = String(item?.name ?? '').trim()
        if (!name) return null

        const polygon = Array.isArray(item?.polygon)
            ? item.polygon.map(normalizePoint).filter((p): p is { x: number; y: number } => Boolean(p))
            : []

        return {
          name,
          excluded: excludedNames.value.includes(name),
          current: currentObject.value === name,
          polygon,
        }
      })
      .filter((item): item is NonNullable<typeof item> => Boolean(item))
})

watch(
    () => [props.modelValue, objectItems.value.map((item) => item.name).join('|'), currentObject.value],
    ([open]) => {
      if (!open) return

      const names = objectItems.value.map((item) => item.name)
      if (selectedObjectName.value && names.includes(selectedObjectName.value)) return

      const preferred =
          currentObject.value && names.includes(currentObject.value)
              ? currentObject.value
              : names[0] ?? ''

      selectedObjectName.value = preferred
    },
    { immediate: true },
)

const selectedObject = computed(() => {
  return objectItems.value.find((item) => item.name === selectedObjectName.value) ?? null
})

const previewObjects = computed<PreviewObject[]>(() => {
  return objectItems.value.map((item) => ({
    name: item.name,
    excluded: item.excluded,
    current: item.current,
    selected: selectedObjectName.value === item.name,
    points: item.polygon,
    hasPreview: item.polygon.length >= 3,
  }))
})

const previewBounds = computed(() => {
  const allPoints = previewObjects.value.flatMap((item) => item.points)
  if (!allPoints.length) {
    return {
      minX: 0,
      minY: 0,
      width: 100,
      height: 100,
    }
  }

  const minX = Math.min(...allPoints.map((p) => p.x))
  const maxX = Math.max(...allPoints.map((p) => p.x))
  const minY = Math.min(...allPoints.map((p) => p.y))
  const maxY = Math.max(...allPoints.map((p) => p.y))

  const width = Math.max(1, maxX - minX)
  const height = Math.max(1, maxY - minY)
  const padding = Math.max(width, height) * 0.06

  return {
    minX: minX - padding,
    minY: minY - padding,
    width: width + padding * 2,
    height: height + padding * 2,
  }
})

const hasAnyPreview = computed(() => previewObjects.value.some((item) => item.hasPreview))

function pointsToSvg(points: Array<{ x: number; y: number }>): string {
  return points.map((p) => `${p.x},${p.y}`).join(' ')
}

function selectObject(name: string) {
  selectedObjectName.value = name
}

async function skipSelectedObject() {
  const name = selectedObject.value?.name
  if (!name || loading.value || selectedObject.value?.excluded) return

  try {
    loading.value = true
    await moonrakerClient.call('printer.gcode.script', {
      script: `EXCLUDE_OBJECT NAME=${JSON.stringify(name)}`,
    })
    dialogOpen.value = false
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="1200" persistent>
    <v-card rounded="lg" class="pa-0">
      <v-card-text class="pa-0">
        <div v-if="!objectItems.length" class="skip-object-empty">
          {{ t('print.current.no_objects') }}
        </div>

        <div v-else class="skip-object-layout">
          <div class="skip-object-left">
            <div class="skip-object-preview">
              <svg
                  v-if="hasAnyPreview"
                  class="skip-object-preview__svg"
                  :viewBox="`${previewBounds.minX} ${previewBounds.minY} ${previewBounds.width} ${previewBounds.height}`"
                  preserveAspectRatio="xMidYMid meet"
              >
                <g v-for="item in previewObjects" :key="item.name">
                  <polygon
                      v-if="item.hasPreview"
                      :points="pointsToSvg(item.points)"
                      class="skip-object-preview__polygon"
                      :class="{
                      'skip-object-preview__polygon--selected': item.selected && !item.current && !item.excluded,
                      'skip-object-preview__polygon--current': item.current && !item.excluded,
                      'skip-object-preview__polygon--excluded': item.excluded,
                    }"
                      @click="selectObject(item.name)"
                  />
                </g>
              </svg>

              <div v-else class="skip-object-preview__empty">
                {{ t('print.current.no_preview') }}
              </div>
            </div>
          </div>

          <div class="skip-object-right">
            <div class="skip-object-list-wrap">
              <v-list class="skip-object-list" density="compact" bg-color="transparent">
                <v-list-item
                    v-for="item in previewObjects"
                    :key="item.name"
                    class="skip-object-item"
                    :class="{
                    'skip-object-item--selected': item.selected && !item.current,
                    'skip-object-item--current': item.current,
                    'skip-object-item--excluded': item.excluded,
                  }"
                    :active="false"
                    @click="selectObject(item.name)"
                >
                  <v-list-item-title class="skip-object-item__name">
                    {{ item.name }}
                  </v-list-item-title>

                  <v-list-item-subtitle class="skip-object-item__meta">
                    <span v-if="item.current">{{ t('print.current.current_object') }}</span>
                    <span v-else-if="item.excluded">{{ t('print.current.already_skipped') }}</span>
                    <span v-else>{{ t('print.current.select_object') }}</span>
                  </v-list-item-subtitle>
                </v-list-item>
              </v-list>
            </div>

            <div class="skip-object-actions">
              <v-btn variant="text" @click="dialogOpen = false">
                {{ t('print.current.close') }}
              </v-btn>
              <v-btn
                  color="secondary"
                  variant="flat"
                  :disabled="loading || !selectedObject || selectedObject.excluded"
                  @click="skipSelectedObject"
              >
                {{ t('print.current.skip') }}
              </v-btn>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.skip-object-empty {
  padding: 24px;
}

.skip-object-layout {
  display: grid;
  grid-template-columns: minmax(360px, 1.15fr) minmax(320px, 1fr);
  align-items: stretch;
}

.skip-object-left {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}

.skip-object-preview {
  flex: 1;
  height: calc(100vh - 50px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
  background: rgba(var(--v-theme-on-surface), 0.04);
}

.skip-object-preview__svg {
  width: 100%;
  height: 100%;
  max-width: 420px;
  max-height: 420px;
}

.skip-object-preview__empty {
  opacity: 0.7;
}

.skip-object-preview__polygon {
  fill: rgba(var(--v-theme-primary), 0.2);
  stroke: rgba(var(--v-theme-primary), 0.32);
  stroke-width: 1.2;
  cursor: pointer;
  transition: fill 0.16s ease, stroke 0.16s ease, opacity 0.16s ease;
}

.skip-object-preview__polygon:hover {
  fill: rgba(var(--v-theme-primary), 0.28);
}

.skip-object-preview__polygon--selected {
  stroke: rgba(var(--v-theme-primary), 1);
  stroke-width: 2.4;
}

.skip-object-preview__polygon--current {
  stroke: rgba(var(--v-theme-secondary), 1);
  stroke-width: 2.4;
}

.skip-object-preview__polygon--excluded {
  fill: rgba(var(--v-theme-on-surface), 0.14);
  stroke: rgba(var(--v-theme-on-surface), 0.28);
  opacity: 0.42;
}

.skip-object-right {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  height: 100%;
  background: rgba(var(--v-theme-on-surface), 0.06);
  padding: 12px;
}

.skip-object-list-wrap {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  max-height: calc(100vh - 140px);
  height: calc(100vh - 140px);
}

.skip-object-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0;
  background: transparent !important;
}

.skip-object-item {
  border-radius: 14px;
  margin-bottom: 0;
  background: rgba(var(--v-theme-on-surface), 0.05);
  transition: background 0.16s ease, opacity 0.16s ease, border-color 0.16s ease;
  border-left: 3px solid transparent;
  cursor: pointer;
}

.skip-object-item:hover {
  background: rgba(var(--v-theme-on-surface), 0.08);
}

.skip-object-item--selected {
  border-left-color: rgba(var(--v-theme-primary), 1);
  background: rgba(var(--v-theme-primary), 0.1);
}

.skip-object-item--current {
  border-left-color: rgba(var(--v-theme-secondary), 1);
}

.skip-object-item--excluded {
  background: rgba(var(--v-theme-on-surface), 0.1);
  opacity: 0.55;
}

.skip-object-item__name {
  font-weight: 700;
  word-break: break-word;
}

.skip-object-item__meta {
  font-size: 0.85rem;
  opacity: 0.78;
}

.skip-object-actions {
  margin-top: auto;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 18px;
}

.skip-object-list :deep(.v-list-item) {
  border-bottom: none !important;
}

.skip-object-list :deep(.v-list-item__append),
.skip-object-list :deep(.v-list-item__prepend) {
  align-self: center;
}

.skip-object-list :deep(.v-list-item__overlay) {
  display: none;
}
</style>