<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'

const { t } = useI18n()

const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const savingAction = ref<string | null>(null)

const rawObjects = computed(() => {
  return moonraker.value.rawObjects as Record<string, unknown>
})

const extruderObject = computed(() => {
  const value = rawObjects.value.extruder
  return value && typeof value === 'object' && !Array.isArray(value)
      ? (value as Record<string, unknown>)
      : {}
})

const liveFlow = computed(() => {
  const gcodeMove = rawObjects.value.gcode_move
  if (gcodeMove && typeof gcodeMove === 'object' && !Array.isArray(gcodeMove)) {
    const factor = (gcodeMove as Record<string, unknown>).extrude_factor
    if (typeof factor === 'number' && Number.isFinite(factor)) {
      return Math.round(factor * 100)
    }
  }

  return 100
})

const livePressureAdvance = computed(() => {
  const value = extruderObject.value.pressure_advance
  return typeof value === 'number' && Number.isFinite(value) ? value : 0
})

const livePressureAdvanceSmoothTime = computed(() => {
  const value =
      typeof extruderObject.value.pressure_advance_smooth_time === 'number'
          ? extruderObject.value.pressure_advance_smooth_time
          : extruderObject.value.smooth_time

  return typeof value === 'number' && Number.isFinite(value) ? value : 0
})

const flowInput = ref('100')
const paInput = ref('0.000')
const paSmoothTimeInput = ref('0.040')

watch(
    liveFlow,
    (value) => {
      if (savingAction.value !== 'flow') {
        flowInput.value = String(value)
      }
    },
    { immediate: true },
)

watch(
    livePressureAdvance,
    (value) => {
      if (savingAction.value !== 'pa') {
        paInput.value = value.toFixed(3)
      }
    },
    { immediate: true },
)

watch(
    livePressureAdvanceSmoothTime,
    (value) => {
      if (savingAction.value !== 'pa_smooth') {
        paSmoothTimeInput.value = value.toFixed(3)
      }
    },
    { immediate: true },
)

function isSaving(action: string): boolean {
  return savingAction.value === action
}

function parseNumber(value: string): number | null {
  const normalized = value.trim().replace(',', '.')
  if (!normalized) return null

  const parsed = Number(normalized)
  return Number.isFinite(parsed) ? parsed : null
}

function formatDecimal(value: number, digits = 3): string {
  return value.toFixed(digits)
}

function clampFlow(value: number): number {
  return Math.max(1, Math.min(300, Math.round(value)))
}

function clampPa(value: number): number {
  return Math.max(0, Math.min(2, value))
}

function clampPaSmooth(value: number): number {
  return Math.max(0, Math.min(1, value))
}

async function runGcode(action: string, script: string) {
  if (savingAction.value) return

  try {
    savingAction.value = action

    await moonrakerClient.call('printer.gcode.script', {
      script,
    })
  } catch (error) {
    console.error(`Failed action ${action}`, error)
  } finally {
    savingAction.value = null
  }
}

function adjustFlow(delta: number) {
  if (savingAction.value) return

  const current = parseNumber(flowInput.value) ?? 100
  flowInput.value = String(clampFlow(current + delta))
}

function adjustPa(delta: number) {
  if (savingAction.value) return

  const current = parseNumber(paInput.value) ?? 0
  paInput.value = formatDecimal(clampPa(current + delta))
}

function adjustPaSmooth(delta: number) {
  if (savingAction.value) return

  const current = parseNumber(paSmoothTimeInput.value) ?? 0
  paSmoothTimeInput.value = formatDecimal(clampPaSmooth(current + delta))
}

async function saveFlow() {
  const value = parseNumber(flowInput.value)
  if (value === null) return

  const clamped = clampFlow(value)
  flowInput.value = String(clamped)

  await runGcode('flow', `M221 S${clamped}`)
}

async function savePressureAdvance() {
  const value = parseNumber(paInput.value)
  if (value === null) return

  const clamped = clampPa(value)
  paInput.value = formatDecimal(clamped)

  await runGcode('pa', `SET_PRESSURE_ADVANCE ADVANCE=${clamped.toFixed(3)}`)
}

async function savePressureAdvanceSmoothTime() {
  const value = parseNumber(paSmoothTimeInput.value)
  if (value === null) return

  const clamped = clampPaSmooth(value)
  paSmoothTimeInput.value = formatDecimal(clamped)

  await runGcode(
      'pa_smooth',
      `SET_PRESSURE_ADVANCE SMOOTH_TIME=${clamped.toFixed(3)}`,
  )
}
</script>

<template>
  <div class="tune-panel pr-3">
    <v-card class="tune-card pa-0" variant="tonal">
      <v-card-title>{{ t('tune.flow') }}</v-card-title>

      <v-card-text>
        <div class="panel-grid">
          <div class="form-field">
            <div class="control-row control-row--wide">
              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustFlow(-10)"
              >
                -10
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustFlow(-5)"
              >
                -5
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustFlow(-1)"
              >
                -1
              </button>

              <div class="control-input control-input--tall">
                <v-text-field
                    v-model="flowInput"
                    variant="outlined"
                    density="comfortable"
                    hide-details
                    type="number"
                    min="1"
                    max="300"
                    step="1"
                    suffix="%"
                    :disabled="!!savingAction"
                />
              </div>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustFlow(1)"
              >
                +1
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustFlow(5)"
              >
                +5
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustFlow(10)"
              >
                +10
              </button>

              <v-btn
                  class="action-btn"
                  color="primary"
                  variant="flat"
                  :loading="isSaving('flow')"
                  :disabled="!!savingAction"
                  rounded="0"
                  @click="saveFlow"
              >
                <v-icon icon="mdi-check" />
              </v-btn>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>

    <v-card class="tune-card" variant="tonal">
      <v-card-title>{{ t('tune.pa') }}</v-card-title>

      <v-card-text>
        <div class="panel-grid">
          <div class="form-field">
            <div class="control-row control-row--wide">
              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPa(-0.1)"
              >
                -0.1
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPa(-0.01)"
              >
                -0.01
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPa(-0.001)"
              >
                -0.001
              </button>

              <div class="control-input control-input--tall">
                <v-text-field
                    v-model="paInput"
                    variant="outlined"
                    density="comfortable"
                    hide-details
                    type="number"
                    min="0"
                    max="2"
                    step="0.001"
                    :disabled="!!savingAction"
                />
              </div>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPa(0.001)"
              >
                +0.001
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPa(0.01)"
              >
                +0.01
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPa(0.1)"
              >
                +0.1
              </button>

              <v-btn
                  class="action-btn"
                  color="primary"
                  variant="flat"
                  :loading="isSaving('pa')"
                  :disabled="!!savingAction"
                  rounded="0"
                  @click="savePressureAdvance"
              >
                <v-icon icon="mdi-check" />
              </v-btn>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>

    <v-card class="tune-card" variant="tonal">
      <v-card-title>{{ t('tune.pa_smooth') }}</v-card-title>

      <v-card-text>
        <div class="panel-grid">
          <div class="form-field">
            <div class="control-row control-row--wide">
              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPaSmooth(-0.1)"
              >
                -0.1
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPaSmooth(-0.01)"
              >
                -0.01
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPaSmooth(-0.001)"
              >
                -0.001
              </button>

              <div class="control-input control-input--tall">
                <v-text-field
                    v-model="paSmoothTimeInput"
                    variant="outlined"
                    density="comfortable"
                    hide-details
                    type="number"
                    min="0"
                    max="1"
                    step="0.001"
                    suffix="s"
                    :disabled="!!savingAction"
                />
              </div>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPaSmooth(0.001)"
              >
                +0.001
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPaSmooth(0.01)"
              >
                +0.01
              </button>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="!!savingAction"
                  @click="adjustPaSmooth(0.1)"
              >
                +0.1
              </button>

              <v-btn
                  class="action-btn"
                  color="primary"
                  variant="flat"
                  :loading="isSaving('pa_smooth')"
                  :disabled="!!savingAction"
                  rounded="0"
                  @click="savePressureAdvanceSmoothTime"
              >
                <v-icon icon="mdi-check" />
              </v-btn>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>
  </div>
</template>

<style scoped>
.tune-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-sizing: border-box;
}

.tune-card {
  width: 100%;
}

.tune-input-slot :deep(.v-input) {
  width: 100%;
  height: 100%;
  margin: 0;
}

.tune-input-slot :deep(.v-input__control) {
  height: 100%;
}

.tune-input-slot :deep(.v-field) {
  height: 100%;
  border-radius: 0;
  box-shadow: none;
  background: transparent;
}

.tune-input-slot :deep(.v-field__field) {
  height: 100%;
}

.tune-input-slot :deep(.v-field__input) {
  min-height: 50px;
  height: 100%;
  padding-top: 0;
  padding-bottom: 0;
  justify-content: center;
  text-align: center;
}

.tune-input-slot :deep(input) {
  text-align: center;
}
</style>