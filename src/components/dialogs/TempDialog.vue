<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  heater: string
  icon: string
  title: string
  currentTemp: number | null
  currentTarget: number | null
  maxTemp: number
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const saving = ref(false)
const localValue = ref('')

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => {
    if (saving.value) return
    emit('update:modelValue', value)
  },
})

const displayValue = computed(() => {
  if (localValue.value !== '') return localValue.value

  if (typeof props.currentTarget === 'number' && Number.isFinite(props.currentTarget)) {
    return String(Math.max(0, Math.round(props.currentTarget)))
  }

  return '0'
})

watch(
    () => props.modelValue,
    (open) => {
      if (!open) return
      localValue.value = ''
    },
    { immediate: true },
)

function closeDialog() {
  if (saving.value) return
  dialogOpen.value = false
}

function appendDigit(digit: string) {
  if (saving.value) return

  const rawNext = `${localValue.value}${digit}`.replace(/^0+(\d)/, '$1')
  const parsed = Number(rawNext)

  if (!Number.isFinite(parsed)) {
    localValue.value = ''
    return
  }

  localValue.value = String(clampTarget(parsed))
}

function clearValue() {
  if (saving.value) return
  localValue.value = ''
}

function backspace() {
  if (saving.value) return
  localValue.value = localValue.value.slice(0, -1)
}

function clampTarget(value: number): number {
  return Math.max(0, Math.min(props.maxTemp, Math.round(value)))
}

function getSetTemperatureScript(target: number): string {
  if (props.heater.startsWith('temperature_fan ')) {
    const fanName = props.heater.replace(/^temperature_fan\s+/, '')
    return `SET_TEMPERATURE_FAN_TARGET TEMPERATURE_FAN=${fanName} TARGET=${target}`
  }

  return `SET_HEATER_TEMPERATURE HEATER=${props.heater} TARGET=${target}`
}

async function saveTemp() {
  if (saving.value) return

  const fallbackTarget =
      typeof props.currentTarget === 'number' && Number.isFinite(props.currentTarget)
          ? props.currentTarget
          : 0
  const parsed = Number(localValue.value === '' ? fallbackTarget : localValue.value)
  const target = clampTarget(Number.isFinite(parsed) ? parsed : fallbackTarget)

  try {
    saving.value = true

    await moonrakerClient.call('printer.gcode.script', {
      script: getSetTemperatureScript(target),
    })

    emit('update:modelValue', false)
  } catch (error) {
    console.error(`Failed to set ${props.heater} temperature`, error)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="760" persistent>
    <v-card rounded="lg">
      <v-card-title class="text-h5 pt-6 px-6 temp-dialog-title">
        <v-icon :icon="icon" class="mr-3" />
        {{ title }}
      </v-card-title>

      <v-card-text class="px-6 pb-2">
        <div class="temp-panel-layout">
          <div class="temp-panel-left">
            <div class="temp-panel-card temp-panel-card--setpoint">
              <div class="temp-panel-label">
                {{ t('temp.dialog.target') }}
              </div>
              <div class="temp-panel-setpoint-value">
                {{ displayValue }}°C
              </div>
            </div>

            <div class="temp-panel-card">
              <div class="temp-panel-label">
                {{ t('temp.dialog.current') }}
              </div>
              <div class="temp-panel-value">
                {{ typeof currentTemp === 'number' ? `${Math.round(currentTemp)}°C` : '--°C' }}
              </div>
            </div>

            <div class="temp-panel-card">
              <div class="temp-panel-label">
                {{ t('temp.dialog.max') }}
              </div>
              <div class="temp-panel-value">
                {{ maxTemp }}°C
              </div>
            </div>
          </div>

          <div class="temp-panel-right">
            <div class="temp-panel-keypad">
              <v-btn
                  v-for="key in ['1', '2', '3', '4', '5', '6', '7', '8', '9']"
                  :key="key"
                  size="x-large"
                  :disabled="saving"
                  @click="appendDigit(key)"
                  variant="tonal"
              >
                {{ key }}
              </v-btn>

              <v-btn size="x-large" :disabled="saving" @click="clearValue" variant="tonal">
                <v-icon icon="mdi-trash-can-outline" />
              </v-btn>

              <v-btn size="x-large" :disabled="saving" @click="appendDigit('0')" variant="tonal">
                0
              </v-btn>

              <v-btn size="x-large" :disabled="saving" @click="backspace" variant="tonal">
                <v-icon icon="mdi-backspace-outline" />
              </v-btn>
            </div>
          </div>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" :disabled="saving" @click="closeDialog">
          {{ t('temp.dialog.cancel') }}
        </v-btn>
        <v-btn
            color="primary"
            :loading="saving"
            :disabled="saving"
            @click="saveTemp"
        >
          {{ t('temp.dialog.set') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.temp-dialog-title {
  display: flex;
  align-items: center;
}
</style>