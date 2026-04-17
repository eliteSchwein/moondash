<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  currentPercent: number | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const saving = ref(false)
const localValue = ref('100')

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => {
    if (saving.value) return
    emit('update:modelValue', value)
  },
})

watch(
    () => props.modelValue,
    (open) => {
      if (!open) return

      if (typeof props.currentPercent === 'number' && Number.isFinite(props.currentPercent)) {
        localValue.value = String(Math.max(1, Math.round(props.currentPercent)))
      } else {
        localValue.value = '100'
      }
    },
    { immediate: true },
)

function closeDialog() {
  if (saving.value) return
  dialogOpen.value = false
}

function parseValue(value: string): number | null {
  const trimmed = value.trim()
  if (!trimmed) return null

  const parsed = Number(trimmed)
  if (!Number.isFinite(parsed)) return null

  return Math.round(parsed)
}

function clampValue(value: number): number {
  return Math.max(1, Math.min(999, Math.round(value)))
}

function adjustValue(delta: number) {
  if (saving.value) return

  const current = parseValue(localValue.value) ?? 100
  localValue.value = String(clampValue(current + delta))
}

async function saveDialog() {
  if (saving.value) return

  const value = clampValue(parseValue(localValue.value) ?? 100)

  try {
    saving.value = true
    localValue.value = String(value)

    await moonrakerClient.call('printer.gcode.script', {
      script: `M220 S${value}`,
    })

    emit('update:modelValue', false)
  } catch (error) {
    console.error('Failed to set print speed', error)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="760" persistent>
    <v-card rounded="lg">
      <v-card-title class="text-h5 pt-6 px-6">
        {{ t('speed.dialog.title') }}
      </v-card-title>

      <v-card-text class="px-6 pb-2">
        <div class="speed-dialog-grid">
          <div class="speed-dialog-label">
            {{ t('speed.dialog.target') }}
          </div>

          <div class="speed-dialog-inline-wrap">
            <div class="speed-dialog-inline-row">
              <button
                  type="button"
                  class="speed-dialog-adjust"
                  :disabled="saving"
                  @click="adjustValue(-25)"
              >
                -25
              </button>

              <button
                  type="button"
                  class="speed-dialog-adjust"
                  :disabled="saving"
                  @click="adjustValue(-10)"
              >
                -10
              </button>

              <button
                  type="button"
                  class="speed-dialog-adjust"
                  :disabled="saving"
                  @click="adjustValue(-1)"
              >
                -1
              </button>

              <div class="speed-dialog-input-slot">
                <v-text-field
                    v-model="localValue"
                    class="speed-dialog-inline-input"
                    variant="outlined"
                    density="comfortable"
                    hide-details
                    type="number"
                    min="1"
                    max="999"
                    step="1"
                    suffix="%"
                    :placeholder="t('speed.dialog.placeholder')"
                    :disabled="saving"
                />
              </div>

              <button
                  type="button"
                  class="speed-dialog-adjust"
                  :disabled="saving"
                  @click="adjustValue(1)"
              >
                +1
              </button>

              <button
                  type="button"
                  class="speed-dialog-adjust"
                  :disabled="saving"
                  @click="adjustValue(10)"
              >
                +10
              </button>

              <button
                  type="button"
                  class="speed-dialog-adjust"
                  :disabled="saving"
                  @click="adjustValue(25)"
              >
                +25
              </button>
            </div>
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="px-6 pb-6">
        <v-spacer />
        <v-btn variant="text" :disabled="saving" @click="closeDialog">
          {{ t('speed.dialog.cancel') }}
        </v-btn>
        <v-btn
            color="primary"
            variant="flat"
            :loading="saving"
            :disabled="saving"
            @click="saveDialog"
        >
          {{ t('speed.dialog.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.speed-dialog-grid {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr);
  gap: 24px;
  align-items: center;
}

.speed-dialog-label {
  font-size: 1.25rem;
}

.speed-dialog-inline-wrap {
  min-width: 0;
}

.speed-dialog-inline-row {
  display: grid;
  grid-template-columns: repeat(3, 70px) minmax(120px, 1fr) repeat(3, 70px);
  align-items: stretch;
  border: 1px solid rgba(255, 255, 255, 0.14);
  border-radius: 12px;
  overflow: hidden;
}

.speed-dialog-adjust {
  appearance: none;
  border: 0;
  border-right: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.02);
  color: inherit;
  font: inherit;
  padding: 0 8px;
  min-height: 50px;
  cursor: pointer;
}

.speed-dialog-adjust:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.06);
}

.speed-dialog-adjust:disabled {
  opacity: 0.5;
  cursor: default;
}

.speed-dialog-input-slot {
  min-width: 0;
  min-height: 50px;
  border-right: 1px solid rgba(255, 255, 255, 0.14);
  display: flex;
  align-items: stretch;
  justify-content: stretch;
  background: rgba(255, 255, 255, 0.02);
}

.speed-dialog-inline-input {
  width: 100%;
}

.speed-dialog-input-slot :deep(.v-input) {
  width: 100%;
  height: 100%;
  margin: 0;
}

.speed-dialog-input-slot :deep(.v-input__control) {
  height: 100%;
}

.speed-dialog-input-slot :deep(.v-field) {
  height: 100%;
  border-radius: 0;
  box-shadow: none;
  background: transparent;
}

.speed-dialog-input-slot :deep(.v-field__field) {
  height: 100%;
}

.speed-dialog-input-slot :deep(.v-field__input) {
  min-height: 50px;
  height: 100%;
  padding-top: 0;
  padding-bottom: 0;
  justify-content: center;
  text-align: center;
}

.speed-dialog-input-slot :deep(input) {
  text-align: center;
}

.speed-dialog-save-btn {
  min-width: 56px !important;
  width: 56px;
  height: 50px;
  border-radius: 0 !important;
  box-shadow: none !important;
}
</style>