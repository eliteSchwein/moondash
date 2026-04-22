<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore, type ShortcutButtonConfig } from '@/stores/app'

const props = defineProps<{
  item: ShortcutButtonConfig
}>()

const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

function normalizeNumber(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value

  if (typeof value === 'string') {
    const parsed = Number.parseFloat(value)
    return Number.isFinite(parsed) ? parsed : null
  }

  return null
}

function resolveActiveObject(activeConfig: string): unknown {
  const rawObjects = moonraker.value.rawObjects
  const wanted = activeConfig.trim().toLowerCase()

  const exactKeys = [
    activeConfig,
    wanted,
    `output_pin ${wanted}`,
    `gcode_macro ${wanted}`,
    `fan_generic ${wanted}`,
    `heater_fan ${wanted}`,
    `temperature_fan ${wanted}`,
    `filament_switch_sensor ${wanted}`,
  ]

  for (const key of exactKeys) {
    const match = Object.entries(rawObjects).find(([rawKey]) => rawKey.toLowerCase() === key.toLowerCase())
    if (match) return match[1]
  }

  for (const [rawKey, value] of Object.entries(rawObjects)) {
    const lower = rawKey.toLowerCase()
    if (lower === wanted) return value
    if (lower.endsWith(` ${wanted}`)) return value
  }

  return undefined
}

function getNumericValue(raw: unknown, activeType?: string): number | null {
  if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
    const record = raw as Record<string, unknown>

    if (activeType === 'output_pin') {
      return normalizeNumber(record.value ?? record.state ?? record.status)
    }

    if (activeType === 'fan_generic' || activeType === 'fan' || activeType === 'temperature_fan') {
      return normalizeNumber(record.speed ?? record.value ?? record.state ?? record.status)
    }

    if ('value' in record) return normalizeNumber(record.value)
    if ('speed' in record) return normalizeNumber(record.speed)
    if ('state' in record) return normalizeNumber(record.state)
    if ('status' in record) return normalizeNumber(record.status)
  }

  return normalizeNumber(raw)
}

const isActive = computed(() => {
  const item = props.item
  if (!item.active_config) return false

  const raw = resolveActiveObject(item.active_config)

  if (item.active_type === 'output_pin') {
    const numeric = getNumericValue(raw, item.active_type)
    const threshold = item.active_threshould ?? 0
    return numeric !== null ? numeric >= threshold : false
  }

  if (item.active_type === 'fan_generic' || item.active_type === 'fan' || item.active_type === 'temperature_fan') {
    const numeric = getNumericValue(raw, item.active_type)
    const threshold = item.active_threshould ?? 0
    return numeric !== null ? numeric >= threshold : false
  }

  if (typeof raw === 'boolean') return raw

  const numeric = getNumericValue(raw)
  if (numeric !== null) return numeric > 0

  if (typeof raw === 'string') {
    const lower = raw.trim().toLowerCase()
    return lower === 'true' || lower === 'on' || lower === 'enabled' || lower === '1'
  }

  return false
})

const buttonColor = computed(() => (isActive.value ? 'primary' : undefined))

const buttonTitle = computed(() => {
  return isActive.value && props.item.macro_active
      ? props.item.macro_active
      : props.item.macro_inactive
})

const macroToSend = computed(() => {
  if (isActive.value && props.item.macro_active) return props.item.macro_active
  return props.item.macro_inactive
})

async function handleClick() {
  const script = macroToSend.value?.trim()
  if (!script) return

  try {
    await moonrakerClient.call('printer.gcode.script', {
      script,
    })
  } catch (error) {
    console.error(`Failed to run shortcut macro "${script}"`, error)
  }
}
</script>

<template>
  <v-btn
      class="shortcut-bar-led-btn"
      variant="tonal"
      :color="buttonColor"
      :icon="item.icon"
      :title="buttonTitle"
      @click="handleClick"
  />
</template>

<style scoped>
.shortcut-bar-led-btn {
  width: 100%;
}
</style>