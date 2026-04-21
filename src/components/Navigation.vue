<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import PowerDialog from '@/components/dialogs/PowerDialog.vue'

const appStore = useAppStore()
const { isDebugEnabled, moonraker } = storeToRefs(appStore)

const powerDialogOpen = ref(false)
const emergencyStopping = ref(false)

const printerState = computed(() => moonraker.value.printStats.state?.toLowerCase() ?? '')

const isPrinterRunning = computed(() => {
  return ['printing', 'paused', 'pausing', 'resuming'].includes(printerState.value)
})

async function emergencyStop() {
  if (emergencyStopping.value) return

  try {
    emergencyStopping.value = true
    await moonrakerClient.call('printer.emergency_stop')
  } finally {
    emergencyStopping.value = false
  }
}

function onPowerClick() {
  if (isPrinterRunning.value) {
    void emergencyStop()
  } else {
    powerDialogOpen.value = true
  }
}
</script>

<template>
  <v-navigation-drawer rail permanent class="sidenav" rail-width="58">
    <v-list class="nav-list" density="compact" nav>
      <v-list-item color="primary" prepend-icon="mdi-home-variant" to="/" />
      <v-list-item color="primary" prepend-icon="mdi-tune-vertical-variant" to="/tune" />
      <v-list-item color="primary" prepend-icon="mdi-printer-3d" to="/files" />
      <v-list-item color="primary" prepend-icon="mdi-cog" to="/settings" />

      <v-list-item
          v-if="isDebugEnabled"
          color="primary"
          prepend-icon="mdi-code-json"
          to="/dev"
      />

      <v-list-item
          :disabled="emergencyStopping"
          @click="onPowerClick"
      >
        <template #prepend>
          <v-icon :color="isPrinterRunning ? 'error' : 'primary'">
            {{ isPrinterRunning ? 'mdi-alert-octagon' : 'mdi-power' }}
          </v-icon>
        </template>

        <template #append>
          <v-progress-circular
              v-if="emergencyStopping"
              indeterminate
              size="16"
              width="2"
              color="error"
          />
        </template>
      </v-list-item>
    </v-list>
  </v-navigation-drawer>

  <PowerDialog v-model="powerDialogOpen" />
</template>

<style scoped>
.nav-list {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: space-evenly;
  margin: 0;
}

.sidenav {
  background: rgb(var(--v-theme-background));
  border: none;
}
</style>