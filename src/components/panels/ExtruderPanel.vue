<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'

type SensorItem = {
  key: string
  name: string
  enabled: boolean | null
  detected: boolean | null
}

const { t } = useI18n()

const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const runningAction = ref<string | null>(null)

const rawObjects = computed(() => {
  return moonraker.value.rawObjects as Record<string, unknown>
})

const filamentSensors = computed<SensorItem[]>(() => {
  return Object.entries(rawObjects.value)
      .filter(([key, value]) => {
        if (!value || typeof value !== 'object' || Array.isArray(value)) return false

        const lower = key.toLowerCase()

        if (!lower.startsWith('filament_switch_sensor ')) return false
        if (lower.startsWith('_')) return false

        return true
      })
      .map(([key, value]) => {
        const record = value as Record<string, unknown>

        const enabled =
            typeof record.enabled === 'boolean'
                ? record.enabled
                : typeof record.sensor_enabled === 'boolean'
                    ? record.sensor_enabled
                    : null

        const detected =
            typeof record.filament_detected === 'boolean'
                ? record.filament_detected
                : typeof record.detected === 'boolean'
                    ? record.detected
                    : typeof record.state === 'boolean'
                        ? record.state
                        : null

        return {
          key,
          name: key
              .replace(/^filament_switch_sensor\s+/i, '')
              .replace(/_/g, ' '),
          enabled,
          detected,
        }
      })
      .sort((a, b) => a.name.localeCompare(b.name))
})

function isBusy(action: string): boolean {
  return runningAction.value === action
}

function getSensorButtonColor(sensor: SensorItem): string | undefined {
  if (sensor.enabled === false) return 'surface'
  if (sensor.enabled === true && sensor.detected === true) return 'primary'
  return 'warning'
}

function getSensorButtonVariant(sensor: SensorItem): 'flat' | 'tonal' {
  if (sensor.enabled === false) return 'tonal'
  if (sensor.enabled === true && sensor.detected === true) return 'flat'
  return 'tonal'
}

function getSensorButtonClass(sensor: SensorItem): Record<string, boolean> {
  return {
    'sensor-btn--off': sensor.enabled === false,
    'sensor-btn--idle': sensor.enabled === true && sensor.detected !== true,
  }
}

async function runGcode(
    action: string,
    script: string,
    timeoutMs: number | null = 10000,) {
  if (runningAction.value) return false

  try {
    runningAction.value = action

    await moonrakerClient.call('printer.gcode.script', {
      script,
      timeoutMs
    })

    return true
  } catch (error) {
    console.error(`Failed action ${action}`, error)
    return false
  } finally {
    runningAction.value = null
  }
}

async function loadFilament() {
  await runGcode('load-filament', 'LOAD_FILAMENT', 60_000)
}

async function unloadFilament() {
  await runGcode('unload-filament', 'UNLOAD_FILAMENT', 60_000)
}

function getSensorCommandName(key: string): string {
  return key.replace(/^filament_switch_sensor\s+/i, '').trim()
}

async function toggleSensor(sensor: SensorItem) {
  const sensorName = getSensorCommandName(sensor.key)
  const enable = sensor.enabled === false

  await runGcode(
      `toggle-sensor-${sensor.key}`,
      `SET_FILAMENT_SENSOR SENSOR=${sensorName} ENABLE=${enable ? 1 : 0}`,
  )
}
</script>

<template>
  <div class="shortcut-bar-container">
    <v-list class="shortcut-bar-list" rounded="rounded">
      <v-list-item class="pa-0">
        <v-btn
            block
            rounded="0"
            variant="text"
            class="action-btn"
            :loading="isBusy('load-filament')"
            :disabled="!!runningAction"
            @click="loadFilament"
        >
          {{ t('extruder.load') }}
        </v-btn>
      </v-list-item>

      <v-divider />

      <v-list-item class="pa-0">
        <v-btn
            block
            rounded="0"
            variant="text"
            class="action-btn"
            :loading="isBusy('unload-filament')"
            :disabled="!!runningAction"
            @click="unloadFilament"
        >
          {{ t('extruder.unload') }}
        </v-btn>
      </v-list-item>

      <v-divider />

      <template v-for="sensor in filamentSensors" :key="sensor.key">
        <v-list-item class="pa-0">
          <v-btn
              block
              rounded="0"
              class="sensor-btn"
              :class="getSensorButtonClass(sensor)"
              :variant="getSensorButtonVariant(sensor)"
              :color="getSensorButtonColor(sensor)"
              :loading="isBusy(`toggle-sensor-${sensor.key}`)"
              :disabled="!!runningAction"
              @click="toggleSensor(sensor)"
          >
            <div class="sensor-content">
              <div class="sensor-name">
                {{ sensor.name }}
              </div>
            </div>
          </v-btn>
        </v-list-item>

        <v-divider />
      </template>
    </v-list>
  </div>
</template>

<style scoped>
.shortcut-bar-container {
  width: 7rem;
  height: 100%;
}

.shortcut-bar-list {
  padding: 0;
}

.action-btn,
.sensor-btn {
  min-height: 56px;
  text-transform: none;
  width: 100%;
}

.sensor-btn--off {
  background: rgba(var(--v-theme-on-surface), 0.18) !important;
}

.sensor-btn--idle {
  background: rgba(var(--v-theme-on-surface), 0.08) !important;
}

.sensor-content {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 6px 2px;
}

.sensor-name {
  max-width: 100%;
  font-size: 0.8rem;
  line-height: 1.1;
  text-align: center;
  white-space: normal;
  word-break: break-word;
}

.sensor-state-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>