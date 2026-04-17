<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import TempDialog from './dialogs/TempDialog.vue'
import FanDialog from './dialogs/FanDialog.vue'

type TempDialogDevice = {
  key: string
  heater: string
  icon: string
  title: string
  currentTemp: number | null
  currentTarget: number | null
  maxTemp: number
}

type FanDialogDevice = {
  key: string
  label: string
  speed: number | null
  temperature: number | null
  target: number | null
  isTemperatureFan: boolean
  maxValue?: number | null
}

const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const activeTempDialog = ref<TempDialogDevice | null>(null)
const activeFanDialog = ref<FanDialogDevice | null>(null)

function formatTemp(value: number | null): string {
  return typeof value === 'number' ? `${Math.round(value)}°C` : ''
}

function formatPercentFromUnit(value: number | null): string {
  return typeof value === 'number' ? `${Math.round(value * 100)}%` : ''
}

function isHeaterActive(currentTemp: number | null, currentTarget: number | null): boolean {
  return (
      typeof currentTemp === 'number' &&
      typeof currentTarget === 'number' &&
      currentTarget !== 0 &&
      Math.abs(currentTarget - currentTemp) >= 5
  )
}

function isFanControllable(key: string, isTemperatureFan: boolean): boolean {
  if (isTemperatureFan) return true

  const lower = key.toLowerCase()

  if (lower === 'fan') return true
  if (lower.startsWith('fan_generic ')) return true

  return false
}

function openTempDialog(device: TempDialogDevice) {
  activeTempDialog.value = device
}

function openFanDialog(device: FanDialogDevice) {
  activeFanDialog.value = device
}

const tempDialogModel = computed({
  get: () => activeTempDialog.value !== null,
  set: (value: boolean) => {
    if (!value) activeTempDialog.value = null
  },
})

const fanDialogModel = computed({
  get: () => activeFanDialog.value !== null,
  set: (value: boolean) => {
    if (!value) activeFanDialog.value = null
  },
})

const heaterItems = computed(() => {
  const items = [
    {
      key: 'extruder',
      heater: 'extruder',
      icon: 'mdi-printer-3d-nozzle-heat',
      title: 'extruder',
      name: 'extruder',
      currentTemp: moonraker.value.extruder.temperature,
      currentTarget: moonraker.value.extruder.target,
      power: moonraker.value.extruder.power,
      maxTemp: 300,
      clickable: true,
    },
    {
      key: 'heater_bed',
      heater: 'heater_bed',
      icon: 'mdi-radiator',
      title: 'heater_bed',
      name: 'heater_bed',
      currentTemp: moonraker.value.heaterBed.temperature,
      currentTarget: moonraker.value.heaterBed.target,
      power: moonraker.value.heaterBed.power,
      maxTemp: 120,
      clickable: true,
    },
    ...moonraker.value.dynamicHeaters.map((heater) => ({
      key: heater.key,
      heater: heater.key,
      icon: 'mdi-radiator',
      title: heater.label,
      name: heater.label,
      currentTemp: heater.temperature,
      currentTarget: heater.target,
      power: heater.power,
      maxTemp: 300,
      clickable: true,
    })),
  ]

  return items.map((item) => ({
    ...item,
    mainValue: formatTemp(item.currentTemp),
    isActive: isHeaterActive(item.currentTemp, item.currentTarget),
  }))
})

const fanItems = computed(() => {
  return moonraker.value.dynamicFans.map((fan) => {
    const isSpinning =
        (typeof fan.speed === 'number' && fan.speed > 0) ||
        (typeof fan.rpm === 'number' && fan.rpm > 0)

    const mainValue = fan.isTemperatureFan
        ? formatTemp(fan.temperature)
        : typeof fan.speed === 'number'
            ? formatPercentFromUnit(fan.speed)
            : typeof fan.rpm === 'number'
                ? (fan.rpm > 0 ? '100%' : '0%')
                : ''

    const clickable = isFanControllable(fan.key, fan.isTemperatureFan)

    return {
      key: fan.key,
      icon: 'mdi-fan',
      title: fan.label,
      name: fan.label,
      mainValue,
      isTemperatureFan: fan.isTemperatureFan,
      isActive: isSpinning,
      speed: fan.speed,
      temperature: fan.temperature,
      target: fan.target,
      clickable,
    }
  })
})

function handleHeaterClick(item: (typeof heaterItems.value)[number]) {
  if (!item.clickable) return

  openTempDialog({
    key: item.key,
    heater: item.heater,
    icon: item.icon,
    title: item.title,
    currentTemp: item.currentTemp,
    currentTarget: item.currentTarget,
    maxTemp: item.maxTemp,
  })
}

function handleFanClick(item: (typeof fanItems.value)[number]) {
  if (!item.clickable) return

  if (item.isTemperatureFan) {
    openTempDialog({
      key: item.key,
      heater: item.key,
      icon: 'mdi-fan',
      title: item.title,
      currentTemp: item.temperature,
      currentTarget: item.target,
      maxTemp: 120,
    })
    return
  }

  openFanDialog({
    key: item.key,
    label: item.title,
    speed: item.speed,
    temperature: item.temperature,
    target: item.target,
    isTemperatureFan: false,
    maxValue: 100,
  })
}
</script>

<template>
  <div class="shortcut-bar-container">
    <v-list class="shortcut-bar-list" rounded="rounded">
      <template v-for="item in heaterItems" :key="item.key">
        <v-list-item class="pa-0">
          <button
              type="button"
              class="control-card"
              @click="handleHeaterClick(item)"
          >
            <div class="control-card-name">
              {{ item.name }}
            </div>

            <div class="control-card-row">
              <v-icon
                  :icon="item.icon"
                  class="control-card-icon"
                  :class="{ 'control-card-icon--active': item.isActive }"
              />

              <div class="control-card-main">
                {{ item.mainValue }}
              </div>
            </div>
          </button>
        </v-list-item>
        <v-divider />
      </template>

      <template v-for="item in fanItems" :key="item.key">
        <v-list-item class="pa-0">
          <button
              type="button"
              class="control-card"
              :class="{ 'control-card--disabled': !item.clickable }"
              @click="handleFanClick(item)"
          >
            <div class="control-card-name">
              {{ item.name }}
            </div>

            <div class="control-card-row">
              <v-icon
                  :icon="item.icon"
                  class="control-card-icon control-card-icon--fan"
                  :class="{ 'control-card-icon--spinning': item.isActive }"
              />

              <div class="control-card-main">
                {{ item.mainValue }}
              </div>
            </div>
          </button>
        </v-list-item>
        <v-divider />
      </template>
    </v-list>

    <TempDialog
        v-if="activeTempDialog"
        v-model="tempDialogModel"
        :heater="activeTempDialog.heater"
        :icon="activeTempDialog.icon"
        :title="activeTempDialog.title"
        :current-temp="activeTempDialog.currentTemp"
        :current-target="activeTempDialog.currentTarget"
        :max-temp="activeTempDialog.maxTemp"
    />

    <FanDialog
        v-if="activeFanDialog"
        v-model="fanDialogModel"
        :fan="activeFanDialog"
    />
  </div>
</template>

<style scoped>
.shortcut-bar-container {
  width: 7rem;
}

.control-card {
  width: 100%;
  border: 0;
  padding: 8px 8px 10px;
  color: inherit;
  cursor: pointer;
  text-align: left;
  background: transparent;
}

.control-card:hover {
  background: rgba(var(--v-theme-on-surface), 0.18);
}

.control-card--disabled {
  cursor: default;
}

.control-card--disabled:hover {
  background: transparent;
}

.control-card-name {
  font-size: 0.72rem;
  line-height: 1.1;
  opacity: 0.8;
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.control-card-row {
  display: grid;
  grid-template-columns: 16px 1fr auto;
  align-items: center;
  gap: 6px;
}

.control-card-icon {
  font-size: 16px;
}

.control-card-icon--fan {
  font-size: 14px;
}

.control-card-icon--active {
  color: rgb(var(--v-theme-error));
}

.control-card-icon--spinning {
  animation: fan-spin 1.2s linear infinite;
}

.control-card-main {
  font-size: 0.92rem;
  font-weight: 600;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@keyframes fan-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>