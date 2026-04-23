<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'

const emit = defineEmits<{
  (e: 'back'): void
}>()

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const pidHeater = ref('')
const pidTarget = ref('')
const pidSaving = ref(false)
const saveConfigSaving = ref(false)

function asObject(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
      ? (value as Record<string, unknown>)
      : null
}

function normalizeValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(normalizeValue)
  }

  if (value && typeof value === 'object') {
    return Object.fromEntries(
        Object.entries(value as Record<string, unknown>).map(([k, v]) => [k, normalizeValue(v)]),
    )
  }

  if (typeof value === 'string') {
    const trimmed = value.trim()

    if (/^(true|false)$/i.test(trimmed)) {
      return trimmed.toLowerCase() === 'true'
    }

    if (trimmed !== '' && !Number.isNaN(Number(trimmed))) {
      return Number(trimmed)
    }

    return trimmed
  }

  return value
}

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

const hasPendingConfigSaveDebug = computed(() => {
  const rawObjects = moonraker.value.rawObjects as Record<string, unknown> | undefined

  if (!rawObjects) {
    return {
      result: false,
      reason: 'rawObjects missing',
      directTruePath: null as string | null,
      diffSections: [] as string[],
      diffCount: 0,
    }
  }

  const configfile = asObject(rawObjects.configfile)
  const webhooks = asObject(rawObjects.webhooks)
  const config = asObject(configfile?.config)
  const settings = asObject(configfile?.settings)

  const directCandidates = [
    { path: 'configfile.save_config_pending', value: configfile?.save_config_pending },
    { path: 'configfile.pending_save', value: configfile?.pending_save },
    { path: 'configfile.needs_save', value: configfile?.needs_save },
    { path: 'configfile.dirty', value: configfile?.dirty },

    { path: 'webhooks.save_config_pending', value: webhooks?.save_config_pending },
    { path: 'webhooks.pending_save', value: webhooks?.pending_save },
    { path: 'webhooks.needs_save', value: webhooks?.needs_save },
    { path: 'webhooks.dirty', value: webhooks?.dirty },

    { path: 'configfile.settings.save_config_pending', value: settings?.save_config_pending },
    { path: 'configfile.settings.pending_save', value: settings?.pending_save },
    { path: 'configfile.settings.needs_save', value: settings?.needs_save },
    { path: 'configfile.settings.dirty', value: settings?.dirty },
  ]

  const directMatch = directCandidates.find((candidate) => candidate.value === true)

  const diffSections: string[] = []

  if (config && settings) {
    for (const [sectionName, rawSettingValue] of Object.entries(settings)) {
      const rawConfigValue = config[sectionName]

      if (rawConfigValue === undefined) {
        diffSections.push(sectionName)
        continue
      }

      const normalizedSettingValue = normalizeValue(rawSettingValue)
      const normalizedConfigValue = normalizeValue(rawConfigValue)

      if (JSON.stringify(normalizedSettingValue) !== JSON.stringify(normalizedConfigValue)) {
        diffSections.push(sectionName)
      }
    }
  }

  return {
    result: Boolean(directMatch) || diffSections.length > 0,
    reason: directMatch
        ? `direct flag true at ${directMatch.path}`
        : diffSections.length > 0
            ? 'settings/config differences found'
            : 'no pending-save signal found',
    directTruePath: directMatch?.path ?? null,
    diffSections,
    diffCount: diffSections.length,
  }
})

const hasPendingConfigSave = computed(() => hasPendingConfigSaveDebug.value.result)

watch(
    hasPendingConfigSaveDebug,
    (debug, prev) => {
      const changed =
          !prev ||
          debug.result !== prev.result ||
          debug.reason !== prev.reason ||
          debug.directTruePath !== prev.directTruePath ||
          debug.diffCount !== prev.diffCount

      if (!changed) return
    },
    { immediate: true, deep: false },
)

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

          <v-select
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
            color="secondary"
            :loading="saveConfigSaving"
            :disabled="saveConfigSaving || pidSaving || !hasPendingConfigSave"
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
  padding: 12px 16px 16px;
  gap: 8px;
}

.no-elevation :deep(.v-field) {
  box-shadow: none !important;
}
</style>