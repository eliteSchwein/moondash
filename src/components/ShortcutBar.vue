<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import ShortcutBarButton from './ShortcutBarButton.vue'
import ShortcutBarShortcutButton from './ShortcutBarShortcutButton.vue'
import ShortcutBarAFC from './afc/ShortcutBarAFC.vue'
import TempDialog from './dialogs/TempDialog.vue'
import SpeedDialog from './dialogs/SpeedDialog.vue'

const appStore = useAppStore()
const { moonraker, shortcutButtons } = storeToRefs(appStore)

const extruderDialogOpen = ref(false)
const heaterBedDialogOpen = ref(false)
const speedDialogOpen = ref(false)

const afcEnabled = computed(() => moonraker.value.afc.available)


const orderedShortcutButtons = computed(() => {
  return [...shortcutButtons.value].sort((a, b) => (a.position ?? 0) - (b.position ?? 0))
})

const extruderLabel = computed(() => {
  const temp = moonraker.value.extruder.temperature
  if (typeof temp !== 'number') return '--°C'
  return `${Math.round(temp)}°C`
})

const heaterBedLabel = computed(() => {
  const temp = moonraker.value.heaterBed.temperature
  if (typeof temp !== 'number') return '--°C'
  return `${Math.round(temp)}°C`
})

const printSpeedPercent = computed(() => {
  const speedFactor = moonraker.value.gcodeMove.speedFactor
  if (typeof speedFactor === 'number' && Number.isFinite(speedFactor)) {
    return Math.round(speedFactor * 100)
  }
  return null
})

const printSpeedLabel = computed(() => {
  if (typeof printSpeedPercent.value === 'number') {
    return `${printSpeedPercent.value}%`
  }

  const speed = moonraker.value.gcodeMove.speed
  if (typeof speed === 'number') {
    return `${Math.round(speed)}`
  }

  return '--'
})

const printSpeedIcon = computed(() => {
  if (typeof printSpeedPercent.value !== 'number') {
    return 'mdi-speedometer-medium'
  }
  if (printSpeedPercent.value > 100) {
    return 'mdi-speedometer'
  }
  if (printSpeedPercent.value < 100) {
    return 'mdi-speedometer-slow'
  }
  return 'mdi-speedometer-medium'
})

const extruderIsHeating = computed(() => {
  const temp = moonraker.value.extruder.temperature
  const target = moonraker.value.extruder.target

  return (
      typeof temp === 'number' &&
      typeof target === 'number' &&
      target !== 0 &&
      Math.abs(target - temp) >= 5
  )
})

const heaterBedIsHeating = computed(() => {
  const temp = moonraker.value.heaterBed.temperature
  const target = moonraker.value.heaterBed.target

  return (
      typeof temp === 'number' &&
      typeof target === 'number' &&
      target !== 0 &&
      Math.abs(target - temp) >= 5
  )
})

const extruderMaxTemp = computed(() => 300)
const heaterBedMaxTemp = computed(() => 120)
</script>

<template>
  <div class="shortcut-bar-container">
    <div class="shortcut-bar-shortcuts">
      <template v-for="item in orderedShortcutButtons" :key="item.name">
        <ShortcutBarShortcutButton :item="item" />
      </template>
    </div>

    <v-list class="shortcut-bar-list" rounded="rounded">
      <v-list-item class="pa-0">
        <ShortcutBarButton
            icon="mdi-printer-3d-nozzle-heat"
            :label="extruderLabel"
            :color="extruderIsHeating ? 'error' : undefined"
            @click="extruderDialogOpen = true"
        />
      </v-list-item>

      <v-divider />

      <v-list-item class="pa-0">
        <ShortcutBarButton
            icon="mdi-radiator"
            :label="heaterBedLabel"
            :color="heaterBedIsHeating ? 'error' : undefined"
            @click="heaterBedDialogOpen = true"
        />
      </v-list-item>

      <v-divider />

      <v-list-item class="pa-0">
        <ShortcutBarButton
            :icon="printSpeedIcon"
            :label="printSpeedLabel"
            @click="speedDialogOpen = true"
        />
      </v-list-item>

      <v-divider />

      <v-list-item class="shortcut-bar-bottom-item pa-0" v-if="afcEnabled">
        <ShortcutBarAFC />
      </v-list-item>
    </v-list>

    <TempDialog
        v-model="extruderDialogOpen"
        heater="extruder"
        icon="mdi-printer-3d-nozzle-heat"
        title="extruder"
        :current-temp="moonraker.extruder.temperature"
        :current-target="moonraker.extruder.target"
        :max-temp="extruderMaxTemp"
    />

    <TempDialog
        v-model="heaterBedDialogOpen"
        heater="heater_bed"
        icon="mdi-radiator"
        title="heater_bed"
        :current-temp="moonraker.heaterBed.temperature"
        :current-target="moonraker.heaterBed.target"
        :max-temp="heaterBedMaxTemp"
    />

    <SpeedDialog
        v-model="speedDialogOpen"
        :current-percent="printSpeedPercent"
    />
  </div>
</template>

<style scoped>
.shortcut-bar-container {
  padding-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  min-height: 0;
}

.shortcut-bar-shortcuts {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
}

.shortcut-bar-list {
  width: 100%;
  flex: 1 1 auto;
  min-height: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  background-color: rgba(var(--v-theme-on-surface), 0.12);
  max-height: 100%;
}

.shortcut-bar-bottom-item {
  margin-top: auto;
}
</style>