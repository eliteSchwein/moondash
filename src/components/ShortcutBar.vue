<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import ShortcutBarButton from './ShortcutBarButton.vue'
import ShortcutBarAFC from './afc/ShortcutBarAFC.vue'
import TempDialog from './dialogs/TempDialog.vue'
import SpeedDialog from './dialogs/SpeedDialog.vue'

const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const extruderDialogOpen = ref(false)
const heaterBedDialogOpen = ref(false)
const speedDialogOpen = ref(false)

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
    <v-btn icon="mdi-lightbulb" class="shortcut-bar-led-btn" variant="tonal" />

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

      <v-list-item class="shortcut-bar-bottom-item pa-0">
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
</style>