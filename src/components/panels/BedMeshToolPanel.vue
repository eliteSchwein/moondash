<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'

type BedMeshProfile = {
  name: string
  variance: number | null
}

const emit = defineEmits<{
  (e: 'back'): void
}>()

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const busyAction = ref<string | null>(null)

function asNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null
}

function flattenNumbers(value: unknown): number[] {
  if (!Array.isArray(value)) return []

  const result: number[] = []

  for (const item of value) {
    if (Array.isArray(item)) {
      result.push(...flattenNumbers(item))
      continue
    }

    const num = asNumber(item)
    if (num !== null) result.push(num)
  }

  return result
}

function computeRange(values: number[]): number | null {
  if (!values.length) return null

  let min = Number.POSITIVE_INFINITY
  let max = Number.NEGATIVE_INFINITY

  for (const value of values) {
    if (value < min) min = value
    if (value > max) max = value
  }

  return Number.isFinite(min) && Number.isFinite(max) ? max - min : null
}

function asMatrix(value: unknown): number[][] | null {
  if (!Array.isArray(value)) return null
  if (!value.length) return null

  const rows: number[][] = []

  for (const row of value) {
    if (!Array.isArray(row) || !row.length) continue

    const numbers = row
        .map((cell) => asNumber(cell))
        .filter((cell): cell is number => cell !== null)

    if (numbers.length === row.length) {
      rows.push(numbers)
    }
  }

  return rows.length ? rows : null
}

const bedMeshState = computed(() => {
  const rawObjects = moonraker.value.rawObjects as Record<string, unknown>

  const bedMesh =
      typeof rawObjects.bed_mesh === 'object' &&
      rawObjects.bed_mesh &&
      !Array.isArray(rawObjects.bed_mesh)
          ? (rawObjects.bed_mesh as Record<string, unknown>)
          : null

  const profilesSource =
      bedMesh && typeof bedMesh.profiles === 'object' && bedMesh.profiles && !Array.isArray(bedMesh.profiles)
          ? (bedMesh.profiles as Record<string, unknown>)
          : null

  const loadedMatrix =
      bedMesh
          ? asMatrix(
              bedMesh.mesh_matrix ??
              bedMesh.probed_matrix ??
              bedMesh.matrix ??
              bedMesh.points ??
              bedMesh.z_values,
          )
          : null

  const loadedProfileName = bedMesh
      ? String(bedMesh.profile_name ?? bedMesh.profile ?? bedMesh.mesh_name ?? '').trim()
      : ''

  const profiles: BedMeshProfile[] = []

  if (profilesSource) {
    for (const [name, value] of Object.entries(profilesSource)) {
      if (!value || typeof value !== 'object' || Array.isArray(value)) continue

      const record = value as Record<string, unknown>

      const varianceDirect =
          asNumber(record.variance) ??
          asNumber(record.saved_variance) ??
          asNumber(record.profile_variance) ??
          asNumber(record.mesh_variance)

      const varianceFromPoints = varianceDirect ?? computeRange(flattenNumbers(record.points))

      profiles.push({
        name,
        variance: varianceFromPoints,
      })
    }
  }

  profiles.sort((a, b) => a.name.localeCompare(b.name))

  return {
    loadedMatrix,
    loadedProfileName,
    profiles,
  }
})

function matrixMinMax(matrix: number[][]): { min: number; max: number } {
  let min = Number.POSITIVE_INFINITY
  let max = Number.NEGATIVE_INFINITY

  for (const row of matrix) {
    for (const value of row) {
      if (value < min) min = value
      if (value > max) max = value
    }
  }

  return { min, max }
}

function lerp(a: number, b: number, t: number): number {
  return Math.round(a + (b - a) * t)
}

function colorForValue(value: number, min: number, max: number): string {
  if (max <= min) return 'rgb(200 200 200)'

  const t = (value - min) / (max - min)

  if (t < 0.33) {
    const tt = t / 0.33
    return `rgb(${lerp(50, 80, tt)} ${lerp(100, 200, tt)} ${lerp(255, 120, tt)})`
  }

  if (t < 0.66) {
    const tt = (t - 0.33) / 0.33
    return `rgb(${lerp(80, 255, tt)} ${lerp(200, 220, tt)} ${lerp(120, 80, tt)})`
  }

  const tt = (t - 0.66) / 0.34
  return `rgb(${255} ${lerp(220, 80, tt)} ${lerp(80, 70, tt)})`
}

const bedGraph = computed(() => {
  const matrix = bedMeshState.value.loadedMatrix
  if (!matrix || !matrix.length || !matrix[0]?.length) return null

  const { min, max } = matrixMinMax(matrix)

  return {
    matrix,
    min,
    max,
    rows: matrix.length,
    cols: matrix[0].length,
  }
})

async function runBedMeshCalibration() {
  if (busyAction.value) return

  try {
    busyAction.value = 'calibrate'
    await moonrakerClient.call('printer.gcode.script', {
      script: 'BED_MESH_CALIBRATE',
    })
  } catch (error) {
    console.error('Failed to start bed mesh calibration', error)
  } finally {
    busyAction.value = null
  }
}

async function homeAll() {
  if (busyAction.value) return

  try {
    busyAction.value = 'home-all'
    await moonrakerClient.call('printer.gcode.script', {
      script: 'G28',
    })
  } catch (error) {
    console.error('Failed to home all axes', error)
  } finally {
    busyAction.value = null
  }
}

async function clearMesh() {
  if (busyAction.value) return

  try {
    busyAction.value = 'clear-mesh'
    await moonrakerClient.call('printer.gcode.script', {
      script: 'BED_MESH_CLEAR',
    })
  } catch (error) {
    console.error('Failed to clear bed mesh', error)
  } finally {
    busyAction.value = null
  }
}

async function loadProfile(profile: BedMeshProfile) {
  if (busyAction.value) return

  try {
    busyAction.value = `load:${profile.name}`
    await moonrakerClient.call('printer.gcode.script', {
      script: `BED_MESH_PROFILE LOAD=${profile.name}`,
    })
  } catch (error) {
    console.error(`Failed to load bed mesh "${profile.name}"`, error)
  } finally {
    busyAction.value = null
  }
}

async function deleteProfile(profile: BedMeshProfile) {
  if (busyAction.value) return

  try {
    busyAction.value = `delete:${profile.name}`
    await moonrakerClient.call('printer.gcode.script', {
      script: `BED_MESH_PROFILE REMOVE=${profile.name}`,
    })
  } catch (error) {
    console.error(`Failed to delete bed mesh "${profile.name}"`, error)
  } finally {
    busyAction.value = null
  }
}

function formatVariance(value: number | null): string {
  if (typeof value !== 'number' || !Number.isFinite(value)) return '--'
  return value.toFixed(3)
}
</script>

<template>
  <div class="tool-panel">
    <div class="tool-panel__grid">
      <v-card elevation="0" class="tool-card pa-0 ml-2" variant="tonal" rounded="lg">
        <v-card-text class="pa-2 tool-card__content">
          <div class="bed-graph__actions">
            <v-btn
                elevation="0"
                :loading="busyAction === 'home-all'"
                :disabled="!!busyAction"
                @click="homeAll"
            >
              <v-icon icon="mdi-home-variant" />
            </v-btn>

            <v-btn
                elevation="0"
                :loading="busyAction === 'clear-mesh'"
                :disabled="!!busyAction"
                @click="clearMesh"
            >
              {{ t('settings.tools.bed_mesh.actions.clear_mesh') }}
            </v-btn>

            <v-btn
                elevation="0"
                :loading="busyAction === 'calibrate'"
                :disabled="!!busyAction"
                @click="runBedMeshCalibration"
            >
              {{ t('settings.tools.bed_mesh.actions.calibrate') }}
            </v-btn>
          </div>

          <div class="bed-graph">
            <div class="bed-graph__svg-wrap">
              <svg
                  v-if="bedGraph"
                  class="bed-graph__svg"
                  :viewBox="`-0.5 -0.5 ${bedGraph.cols + 1} ${bedGraph.rows + 1}`"
                  preserveAspectRatio="xMidYMid meet"
              >
                <g v-for="(row, y) in bedGraph.matrix" :key="y">
                  <rect
                      v-for="(value, x) in row"
                      :key="`${x}-${y}`"
                      :x="x"
                      :y="bedGraph.rows - 1 - y"
                      width="1"
                      height="1"
                      :fill="colorForValue(value, bedGraph.min, bedGraph.max)"
                  />
                </g>
              </svg>

              <div v-else class="bed-graph__empty">
                <v-row>
                  <v-col cols="12">
                    <v-icon icon="mdi-grid-off" size="64" />
                  </v-col>
                  <v-col cols="12">
                    {{ t('settings.tools.bed_mesh.no_loaded_mesh') }}
                  </v-col>
                </v-row>
              </div>
            </div>
          </div>
        </v-card-text>
      </v-card>

      <v-card class="tool-card pa-0 mr-2" variant="tonal" rounded="lg">
        <v-card-title class="text-subtitle-1">
          {{ t('settings.tools.bed_mesh.available_meshes') }}
        </v-card-title>

        <v-card-text class="tool-card__list px-0">
          <v-list
              v-if="bedMeshState.profiles.length"
              density="compact"
              elevation="0"
              class="bed-mesh-list pa-0"
          >
            <template
                v-for="(profile, index) in bedMeshState.profiles"
                :key="profile.name"
            >
              <v-list-item
                  class="bed-mesh-list__item pa-0"
                  :class="{ 'bed-mesh-list__item--active': profile.name === bedMeshState.loadedProfileName }"
              >
                <v-list-item-title class="bed-mesh-list__title">
                  <v-btn
                      class="bed-mesh-list__load-btn mr-2"
                      variant="text"
                      :loading="busyAction === `load:${profile.name}`"
                      :disabled="!!busyAction"
                      @click="loadProfile(profile)"
                  >
                    <span class="bed-mesh-list__name">
                      {{ profile.name }}
                    </span>
                    <span class="bed-mesh-list__variance">
                      {{ formatVariance(profile.variance) }}
                    </span>
                  </v-btn>
                </v-list-item-title>

                <template #append>
                  <div class="bed-mesh-list__actions">
                    <v-btn
                        v-if="profile.name !== bedMeshState.loadedProfileName"
                        size="small"
                        variant="text"
                        color="secondary"
                        :loading="busyAction === 'calibrate'"
                        :disabled="!!busyAction"
                        @click="runBedMeshCalibration"
                    >
                      <v-icon icon="mdi-progress-upload" />
                    </v-btn>

                    <v-btn
                        size="small"
                        variant="text"
                        color="error"
                        :loading="busyAction === `delete:${profile.name}`"
                        :disabled="!!busyAction"
                        @click="deleteProfile(profile)"
                    >
                      <v-icon icon="mdi-delete" />
                    </v-btn>
                  </div>
                </template>
              </v-list-item>

              <v-divider v-if="index < bedMeshState.profiles.length - 1" />
            </template>
          </v-list>

          <v-alert v-else type="info" variant="tonal">
            {{ t('settings.tools.bed_mesh.no_meshes') }}
          </v-alert>
        </v-card-text>
      </v-card>
    </div>
  </div>
</template>

<style scoped>
.tool-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
}

.tool-panel__grid {
  display: grid;
  grid-template-columns: minmax(320px, 1.15fr) minmax(280px, 0.85fr);
  gap: 16px;
  min-height: 0;
}

.tool-card {
  height: calc(100% - 145px);
  min-height: 0;
  overflow: hidden;
}

.tool-card__content {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  box-sizing: border-box;
  overflow: hidden;
}

.tool-card__list {
  overflow: auto;
  height: calc(100vh - 30px);
  max-height: calc(100vh - 30px);
}

.bed-graph__actions {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  flex: 0 0 auto;
}

.bed-graph {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1 1 auto;
  min-height: 0;
}

.bed-graph__svg-wrap {
  flex: 1 1 auto;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.bed-graph__svg {
  width: 100%;
  max-width: 100%;
  max-height: 100%;
  display: block;
  image-rendering: pixelated;
}

.bed-graph__empty {
  width: 100%;
  height: 100%;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  opacity: 0.7;
  padding: 12px;
}

.bed-mesh-list {
  padding: 0;
}

.bed-mesh-list__item {
  align-items: center;
}

.bed-mesh-list__title {
  font-weight: 700;
}

.bed-mesh-list__load-btn {
  width: 100%;
  justify-content: space-between;
  gap: 12px;
  text-transform: none;
  font: inherit;
  font-weight: 700;
}

.bed-mesh-list__name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: 10px;
}

.bed-mesh-list__variance {
  opacity: 0.8;
  font-weight: 500;
  white-space: nowrap;
}

.bed-mesh-list__actions {
  display: flex;
  gap: 6px;
  align-items: center;
}

.bed-mesh-list__item--active {
  background: rgba(var(--v-theme-primary), 0.16);
}

.bed-mesh-list__item--active .bed-mesh-list__load-btn {
  color: rgb(var(--v-theme-primary));
}
</style>