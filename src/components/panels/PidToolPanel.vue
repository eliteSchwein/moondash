<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'

defineEmits<{
  (e: 'back'): void
}>()

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const pidHeater = ref('')
const pidTarget = ref('')
const pidSaving = ref(false)
const saveConfigSaving = ref(false)
const hasCompletedPidRun = ref(false)

const heaterOptions = computed(() => {
  const items: Array<{ title: string; value: string }> = []

  if (moonraker.value.extruder) {
    items.push({ title: 'extruder', value: 'extruder' })
  }

  if (moonraker.value.heaterBed) {
    items.push({ title: 'heater_bed', value: 'heater_bed' })
  }

  for (const heater of moonraker.value.dynamicHeaters ?? []) {
    items.push({ title: heater.key, value: heater.key })
  }

  return items
})

const displayValue = computed(() => pidTarget.value || '0')

const currentHeaterTemp = computed(() => {
  const heater = pidHeater.value

  if (!heater) return null

  if (heater === 'extruder') {
    const value = Number(moonraker.value.extruder?.temperature)
    return Number.isFinite(value) ? value : null
  }

  if (heater === 'heater_bed') {
    const value = Number(moonraker.value.heaterBed?.temperature)
    return Number.isFinite(value) ? value : null
  }

  const dynamicHeater = (moonraker.value.dynamicHeaters ?? []).find((item) => item.key === heater)
  const value = Number(dynamicHeater?.temperature)

  return Number.isFinite(value) ? value : null
})

const displayCurrentTemp = computed(() => {
  if (currentHeaterTemp.value === null) return '--'
  return String(Math.round(currentHeaterTemp.value))
})

function clampTarget(value: number): number {
  return Math.max(0, Math.min(999, Math.round(value)))
}

function appendDigit(digit: string) {
  if (pidSaving.value || saveConfigSaving.value) return

  const rawNext = `${pidTarget.value}${digit}`.replace(/^0+(\d)/, '$1')
  const parsed = Number(rawNext)

  if (!Number.isFinite(parsed)) {
    pidTarget.value = ''
    return
  }

  pidTarget.value = String(clampTarget(parsed))
}

function clearValue() {
  if (pidSaving.value || saveConfigSaving.value) return
  pidTarget.value = ''
}

function backspace() {
  if (pidSaving.value || saveConfigSaving.value) return
  pidTarget.value = pidTarget.value.slice(0, -1)
}

async function runPidTune() {
  if (pidSaving.value || saveConfigSaving.value) return

  const heater = pidHeater.value.trim()
  const target = Number(pidTarget.value)

  if (!heater || !Number.isFinite(target)) return

  try {
    pidSaving.value = true
    hasCompletedPidRun.value = false

    await moonrakerClient.call(
        'printer.gcode.script',
        {
          script: `PID_CALIBRATE HEATER=${heater} TARGET=${Math.round(target)}`,
        },
        3_600_000,
    )
  } catch (error) {
    console.error('Failed to start PID tuning', error)
  } finally {
    pidSaving.value = false
    hasCompletedPidRun.value = true
  }
}

async function saveConfig() {
  if (saveConfigSaving.value || pidSaving.value) return

  try {
    saveConfigSaving.value = true

    await moonrakerClient.call('printer.gcode.script', {
      script: 'SAVE_CONFIG',
    })
  } catch (error) {
    console.error('Failed to save config', error)
  } finally {
    saveConfigSaving.value = false
  }
}
</script>

<template>
  <div class="tool-panel">
    <v-card class="tool-card" elevation="0" rounded="lg">
      <v-card-text class="tool-pid-layout">
        <div class="temp-panel-left">
          <div class="temp-panel-card temp-panel-card--setpoint">
            <div class="temp-panel-label">
              {{ t('settings.tools.pid.target_temp') }}
            </div>
            <div class="temp-panel-setpoint-value">
              {{ displayValue }}°C
            </div>
          </div>

          <div class="temp-panel-card temp-panel-card--current">
            <div class="temp-panel-label">
              {{ t('settings.tools.pid.current_temp') }}
            </div>
            <div class="temp-panel-setpoint-value">
              {{ displayCurrentTemp }}°C
            </div>
          </div>

          <v-select
              :disabled="saveConfigSaving || pidSaving"
              v-model="pidHeater"
              rounded="lg"
              class="no-elevation"
              :items="heaterOptions"
              item-title="title"
              item-value="value"
              density="compact"
              variant="solo-filled"
              :label="t('settings.tools.pid.heater')"
              hide-details
          />
        </div>

        <div class="temp-panel-right">
          <div class="temp-panel-keypad">
            <v-btn
                v-for="key in ['1', '2', '3', '4', '5', '6', '7', '8', '9']"
                :key="key"
                size="x-large"
                :disabled="pidSaving || saveConfigSaving"
                variant="tonal"
                @click="appendDigit(key)"
            >
              {{ key }}
            </v-btn>

            <v-btn
                size="x-large"
                :disabled="pidSaving || saveConfigSaving"
                variant="tonal"
                @click="clearValue"
            >
              <v-icon icon="mdi-trash-can-outline" />
            </v-btn>

            <v-btn
                size="x-large"
                :disabled="pidSaving || saveConfigSaving"
                variant="tonal"
                @click="appendDigit('0')"
            >
              0
            </v-btn>

            <v-btn
                size="x-large"
                :disabled="pidSaving || saveConfigSaving"
                variant="tonal"
                @click="backspace"
            >
              <v-icon icon="mdi-backspace-outline" />
            </v-btn>
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="tool-card__actions">
        <v-spacer />

        <v-btn
            v-if="hasCompletedPidRun"
            color="secondary"
            :loading="saveConfigSaving"
            :disabled="saveConfigSaving || pidSaving"
            @click="saveConfig"
        >
          {{ t('settings.tools.pid.save_config') }}
        </v-btn>

        <v-btn
            color="primary"
            :loading="pidSaving"
            :disabled="pidSaving || saveConfigSaving || !pidHeater || !pidTarget"
            @click="runPidTune"
        >
          {{ t('settings.tools.pid.run') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </div>
</template>

<style scoped>
.tool-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: calc(100vh - 125px);
  min-height: 0;
}

.tool-card {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.tool-pid-layout {
  flex: 1 1 auto;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(240px, 320px) minmax(0, 1fr);
  gap: 24px;
  align-items: start;
  overflow: auto;
}

.tool-card__actions {
  flex: 0 0 auto;
  margin-top: auto;
  gap: 8px;
}

.no-elevation :deep(.v-field) {
  box-shadow: none !important;
}
</style>