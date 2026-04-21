<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

type WifiNetwork = {
  ssid: string
  secured: boolean
  saved: boolean
  signalPercent: number | null
}

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  enabled: boolean
  connectedSsid: string | null
  visibleNetworks: WifiNetwork[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'open-network', network: WifiNetwork): void
  (e: 'changed'): void
}>()

const scanBusy = ref(false)

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

function signalLabel(percent: number | null | undefined): string {
  if (typeof percent !== 'number' || !Number.isFinite(percent)) {
    return '--'
  }
  return `${percent}%`
}

function signalColor(percent: number | null | undefined): string | undefined {
  if (typeof percent !== 'number' || !Number.isFinite(percent)) return undefined
  if (percent >= 80) return 'success'
  if (percent >= 50) return 'warning'
  return 'error'
}

async function scanWifi() {
  if (!props.enabled || scanBusy.value) return

  try {
    scanBusy.value = true
    await invoke('scan_wifi_networks')
    emit('changed')
  } finally {
    scanBusy.value = false
  }
}

function openNetwork(network: WifiNetwork) {
  emit('open-network', network)
}

watch(
    () => props.modelValue,
    (open) => {
      if (!open) return
      void scanWifi()
    },
    { immediate: false },
)
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="760" persistent>
    <v-card rounded="lg">
      <v-card-title class="wifi-scan-dialog__title">
        <span>{{ t('settings.network.wifi.scan') }}</span>

        <v-btn
            size="small"
            variant="tonal"
            prepend-icon="mdi-wifi-refresh"
            :disabled="!enabled || scanBusy"
            :loading="scanBusy"
            @click="scanWifi"
        >
          {{ t('settings.network.wifi.scan') }}
        </v-btn>
      </v-card-title>

      <v-card-text>
        <div class="wifi-scan-dialog__list-wrap">
          <v-list density="compact" bg-color="transparent">
            <v-list-item
                v-for="network in visibleNetworks"
                :key="`visible-${network.ssid}`"
                @click="openNetwork(network)"
            >
              <template #prepend>
                <v-icon :color="signalColor(network.signalPercent)">
                  {{ network.secured ? 'mdi-wifi-lock' : 'mdi-wifi' }}
                </v-icon>
              </template>

              <v-list-item-title>{{ network.ssid }}</v-list-item-title>
              <v-list-item-subtitle>
                {{ signalLabel(network.signalPercent) }}
              </v-list-item-subtitle>

              <template #append>
                <v-chip
                    v-if="connectedSsid === network.ssid"
                    size="small"
                    color="success"
                    variant="tonal"
                >
                  {{ t('settings.network.wifi.connected') }}
                </v-chip>
                <v-chip
                    v-else-if="network.saved"
                    size="small"
                    variant="tonal"
                >
                  {{ t('settings.network.wifi.saved') }}
                </v-chip>
              </template>
            </v-list-item>

            <v-list-item v-if="!visibleNetworks.length">
              <v-list-item-title>
                {{ t('settings.network.wifi.no_networks') }}
              </v-list-item-title>
            </v-list-item>
          </v-list>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="dialogOpen = false">
          {{ t('settings.network.close') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.wifi-scan-dialog__title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.wifi-scan-dialog__list-wrap {
  max-height: calc(100vh - 200px);
  height: calc(100vh - 200px);
  overflow-y: auto;
  overflow-x: hidden;
}
</style>