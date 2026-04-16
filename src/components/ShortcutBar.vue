<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import ShortcutBarButton from './ShortcutBarButton.vue'

const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

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
  if (typeof speedFactor === 'number') {
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
            @click="{}"
        />
      </v-list-item>

      <v-divider />

      <v-list-item class="pa-0">
        <ShortcutBarButton
            icon="mdi-radiator"
            :label="heaterBedLabel"
            :color="heaterBedIsHeating ? 'error' : undefined"
            @click="{}"
        />
      </v-list-item>

      <v-divider />

      <v-list-item class="pa-0">
        <ShortcutBarButton
            :icon="printSpeedIcon"
            :label="printSpeedLabel"
            @click="{}"
        />
      </v-list-item>

      <v-divider />

      <v-list-item class="shortcut-bar-bottom-item">
        A
      </v-list-item>
    </v-list>
  </div>
</template>

<style scoped>
.shortcut-bar-container {
  height: 100%;
  min-width: 4rem;
  padding-top: 10px;
  padding-bottom: 10px;
  display: flex;
  flex-direction: column;
}

.shortcut-bar-led-btn {
  width: 100% !important;
  border-radius: 5px;
  margin-bottom: 5px;
  flex-shrink: 0;
}

.shortcut-bar-list {
  width: 100%;
  flex: 1;
  min-height: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  background-color: rgba(var(--v-theme-on-surface), 0.12);
}

.shortcut-bar-bottom-item {
  margin-top: auto;
}

.shortcut-bar-list :deep(.v-list-item) {
  min-height: 0;
  padding: 0;
}

.shortcut-bar-list :deep(.v-list-item__content) {
  padding: 0;
}
</style>