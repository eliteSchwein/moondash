<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useI18n } from 'vue-i18n'
import ColorPickerDialog from '@/components/dialogs/ColorPickerDialog.vue'

const { t } = useI18n()

type AfcLaneDialogData = {
  id: string
  label: string
  color: string
  material: string
  weight: number | null
}

const props = defineProps<{
  modelValue: boolean
  lane: AfcLaneDialogData | null
  laneColors?: string[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => {
    if (saving.value) return
    emit('update:modelValue', value)
  },
})

const colorPickerOpen = ref(false)
const saving = ref(false)
const localMaterial = ref('')
const localColor = ref('#434343')
const localWeight = ref<string>('')

const materialOptions = [
  'ABS',
  'ASA',
  'PETG',
  'PLA',
  'TPU',
  'PA',
  'PC',
]

function normalizeHexColor(color: unknown): string {
  if (typeof color !== 'string' || !color.trim()) {
    return '#434343'
  }

  const value = color.trim()

  if (/^#[0-9a-f]{6}$/i.test(value)) {
    return value.toUpperCase()
  }

  if (/^#[0-9a-f]{3}$/i.test(value)) {
    return value.toUpperCase()
  }

  return '#434343'
}

watch(
    () => props.lane,
    (lane) => {
      localMaterial.value = lane?.material?.trim() || ''
      localColor.value = normalizeHexColor(lane?.color)
      localWeight.value =
          typeof lane?.weight === 'number' && Number.isFinite(lane.weight)
              ? String(Math.round(lane.weight))
              : ''
      colorPickerOpen.value = false
    },
    { immediate: true },
)

function closeDialog() {
  if (saving.value) return
  dialogOpen.value = false
  colorPickerOpen.value = false
}

function openColorPicker() {
  if (saving.value) return
  colorPickerOpen.value = true
}

function parseWeight(value: string): number | null {
  const trimmed = value.trim()
  if (!trimmed) return null

  const parsed = Number(trimmed)
  if (!Number.isFinite(parsed)) return null

  return Math.round(parsed)
}

function stripHash(color: string): string {
  return color.replace(/^#/, '').toUpperCase()
}

function adjustWeight(delta: number) {
  if (saving.value) return

  const current = parseWeight(localWeight.value) ?? 0
  const next = Math.max(0, current + delta)
  localWeight.value = String(next)
}

function onColorSelected(color: string) {
  localColor.value = normalizeHexColor(color)
}

async function saveDialog() {
  if (!props.lane || saving.value) return

  const laneId = props.lane.id
  const material = localMaterial.value.trim()
  const color = stripHash(localColor.value)
  const weight = parseWeight(localWeight.value)

  let saveSucceeded = false

  try {
    saving.value = true

    if (material) {
      await moonrakerClient.call('printer.gcode.script', {
        script: `SET_MATERIAL LANE=${laneId} MATERIAL=${material}`,
      })
    }

    await moonrakerClient.call('printer.gcode.script', {
      script: `SET_COLOR LANE=${laneId} COLOR=${color}`,
    })

    if (weight !== null) {
      await moonrakerClient.call('printer.gcode.script', {
        script: `SET_WEIGHT LANE=${laneId} WEIGHT=${weight}`,
      })
    }

    saveSucceeded = true
  } catch (error) {
    console.error(`Failed to save lane ${laneId}`, error)
  } finally {
    saving.value = false
  }

  if (saveSucceeded) {
    colorPickerOpen.value = false
    emit('update:modelValue', false)
  }
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="760" persistent>
    <v-card rounded="lg">
      <v-card-title class="text-h5 pt-6 px-6">
        {{ lane?.label ?? t('afc.edit.lane') }}
      </v-card-title>

      <v-card-text class="px-6 pb-2">
        <div class="form-grid">
          <div class="form-label">
            {{ t('afc.edit.filament') }}
          </div>

          <div class="form-field">
            <v-select
                v-model="localMaterial"
                :items="materialOptions"
                variant="outlined"
                density="comfortable"
                hide-details
                :disabled="saving"
            />
          </div>

          <div class="form-label">
            {{ t('afc.edit.color') }}
          </div>

          <div class="color-row">
            <button
                class="color-swatch color-swatch--button"
                :style="{ backgroundColor: localColor }"
                type="button"
                :disabled="saving"
                @click="openColorPicker"
            />

            <div class="color-value">
              {{ localColor }}
            </div>
          </div>

          <div class="form-label">
            {{ t('afc.edit.weight') }}
          </div>

          <div class="form-field">
            <div class="control-row control-row--compact">
              <button
                  type="button"
                  class="control-btn"
                  :disabled="saving"
                  @click="adjustWeight(-50)"
              >
                -50
              </button>
              <button
                  type="button"
                  class="control-btn"
                  :disabled="saving"
                  @click="adjustWeight(-10)"
              >
                -10
              </button>
              <button
                  type="button"
                  class="control-btn"
                  :disabled="saving"
                  @click="adjustWeight(-1)"
              >
                -1
              </button>

              <div class="control-input">
                <v-text-field
                    v-model="localWeight"
                    variant="outlined"
                    density="comfortable"
                    hide-details
                    type="number"
                    min="0"
                    step="1"
                    suffix="g"
                    :disabled="saving"
                />
              </div>

              <button
                  type="button"
                  class="control-btn"
                  :disabled="saving"
                  @click="adjustWeight(1)"
              >
                +1
              </button>
              <button
                  type="button"
                  class="control-btn"
                  :disabled="saving"
                  @click="adjustWeight(10)"
              >
                +10
              </button>
              <button
                  type="button"
                  class="control-btn"
                  :disabled="saving"
                  @click="adjustWeight(50)"
              >
                +50
              </button>
            </div>
          </div>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" :disabled="saving" @click="closeDialog">
          {{ t('afc.edit.cancel') }}
        </v-btn>
        <v-btn
            color="primary"
            :loading="saving"
            :disabled="saving"
            @click="saveDialog"
        >
          {{ t('afc.edit.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <ColorPickerDialog
      v-model="colorPickerOpen"
      :selected-color="localColor"
      :additional-colors="props.laneColors ?? []"
      :additional-colors-title="t('afc.edit.other_lane_colors')"
      :vibrant-only="false"
      @select="onColorSelected"
  />
</template>

<style scoped>
</style>