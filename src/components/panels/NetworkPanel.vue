<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import WifiNetworkDialog from '@/components/dialogs/WifiNetworkDialog.vue'
import WifiSavedDialog from '@/components/dialogs/WifiSavedDialog.vue'
import WifiScanDialog from '@/components/dialogs/WifiScanDialog.vue'

type WifiNetwork = {
  ssid: string
  secured: boolean
  saved: boolean
  signalPercent: number | null
}

type WifiSettings = {
  enabled: boolean
  connectedSsid: string | null
  connectedIp: string | null
  savedNetworks: WifiNetwork[]
  scannedNetworks: WifiNetwork[]
}

type WiredInterface = {
  interfaceName: string
  connected: boolean
  ip: string | null
}

type WiredSettings = {
  interfaces: WiredInterface[]
}

const { t } = useI18n()

const loading = ref(false)
const wifiBusy = ref(false)
const wiredBusy = ref<string | null>(null)

const wifiSettings = ref<WifiSettings | null>(null)
const wiredSettings = ref<WiredSettings | null>(null)

const wifiDialogOpen = ref(false)
const wifiSavedDialogOpen = ref(false)
const wifiScanDialogOpen = ref(false)

const dialogNetwork = ref<WifiNetwork | null>(null)
const hiddenMode = ref(false)

const savedNetworks = computed(() => wifiSettings.value?.savedNetworks ?? [])
const visibleNetworks = computed(() => wifiSettings.value?.scannedNetworks ?? [])
const wiredInterfaces = computed(() => wiredSettings.value?.interfaces ?? [])

async function loadWifiSettings() {
  wifiSettings.value = await invoke<WifiSettings>('get_wifi_settings')
}

async function loadWiredSettings() {
  wiredSettings.value = await invoke<WiredSettings>('get_wired_settings')
}

async function refreshAll() {
  try {
    loading.value = true
    await Promise.all([
      loadWifiSettings(),
      loadWiredSettings(),
    ])
  } finally {
    loading.value = false
  }
}

async function toggleWifi(enabled: boolean) {
  try {
    wifiBusy.value = true
    await invoke('set_wifi_enabled', { enabled })
    await loadWifiSettings()
  } finally {
    wifiBusy.value = false
  }
}

async function toggleWired(interfaceName: string, enabled: boolean) {
  try {
    wiredBusy.value = interfaceName
    await invoke('set_wired_interface_enabled', {
      interfaceName,
      enabled,
    })
    await loadWiredSettings()
  } finally {
    wiredBusy.value = null
  }
}

function openHiddenDialog() {
  dialogNetwork.value = null
  hiddenMode.value = true
  wifiDialogOpen.value = true
}

function openVisibleNetworkDialog(network: WifiNetwork) {
  dialogNetwork.value = network
  hiddenMode.value = false
  wifiDialogOpen.value = true
}

async function handleWifiDialogSubmit(payload: { ssid: string; password: string }) {
  if (hiddenMode.value) {
    await invoke('connect_hidden_wifi', {
      ssid: payload.ssid,
      password: payload.password || null,
    })
  } else {
    await invoke('connect_to_wifi', {
      ssid: payload.ssid,
      password: payload.password || null,
    })
  }

  wifiDialogOpen.value = false
  await loadWifiSettings()
}

async function handleNetworksChanged() {
  await loadWifiSettings()
}

onMounted(async () => {
  await refreshAll()
})
</script>

<template>
  <div class="network-panel">
    <div class="network-panel__grid">
      <v-card rounded="lg" variant="flat">
        <v-card-title class="network-panel__title-row">
          <span>{{ t('settings.network.wifi.title') }}</span>

          <div class="network-panel__wifi-controls">
            <v-btn
                icon="mdi-wifi-plus"
                size="small"
                variant="text"
                :disabled="!(wifiSettings?.enabled) || wifiBusy || loading"
                @click="openHiddenDialog"
            />

            <v-switch
                :model-value="wifiSettings?.enabled ?? false"
                color="primary"
                hide-details
                density="compact"
                inset
                :disabled="wifiBusy || loading"
                @update:model-value="(value) => toggleWifi(Boolean(value))"
            />
          </div>
        </v-card-title>

        <v-card-text class="network-panel__card-content">
          <div class="network-panel__row">
            <span class="network-panel__label">{{ t('settings.network.wifi.connected_ssid') }}</span>
            <span class="network-panel__value">
              {{ wifiSettings?.connectedSsid || t('settings.network.none') }}
            </span>
          </div>

          <div class="network-panel__row">
            <span class="network-panel__label">{{ t('settings.network.wifi.ip') }}</span>
            <span class="network-panel__value">
              {{ wifiSettings?.connectedIp || '--' }}
            </span>
          </div>

          <div class="network-panel__actions">
            <v-btn
                size="small"
                variant="tonal"
                prepend-icon="mdi-wifi-refresh"
                :disabled="!(wifiSettings?.enabled)"
                @click="wifiScanDialogOpen = true"
            >
              {{ t('settings.network.wifi.scan') }}
            </v-btn>

            <v-btn
                size="small"
                variant="tonal"
                prepend-icon="mdi-content-save"
                :disabled="!savedNetworks.length"
                @click="wifiSavedDialogOpen = true"
            >
              {{ t('settings.network.wifi.saved_networks') }}
            </v-btn>
          </div>
        </v-card-text>
      </v-card>

      <v-card rounded="lg" variant="flat">
        <v-card-title>
          {{ t('settings.network.wired.title') }}
        </v-card-title>

        <v-card-text class="network-panel__card-content">
          <v-list density="compact" bg-color="transparent">
            <v-list-item
                v-for="iface in wiredInterfaces"
                :key="iface.interfaceName"
            >
              <template #prepend>
                <v-icon :color="iface.connected ? 'success' : undefined">
                  mdi-ethernet
                </v-icon>
              </template>

              <v-list-item-title>{{ iface.interfaceName }}</v-list-item-title>
              <v-list-item-subtitle>{{ iface.ip || '--' }}</v-list-item-subtitle>

              <template #append>
                <v-switch
                    :model-value="iface.connected"
                    color="primary"
                    hide-details
                    density="compact"
                    inset
                    :disabled="wiredBusy === iface.interfaceName"
                    @update:model-value="(value) => toggleWired(iface.interfaceName, Boolean(value))"
                />
              </template>
            </v-list-item>

            <v-list-item v-if="!wiredInterfaces.length">
              <v-list-item-title>
                {{ t('settings.network.wired.no_interfaces') }}
              </v-list-item-title>
            </v-list-item>
          </v-list>
        </v-card-text>
      </v-card>
    </div>

    <WifiNetworkDialog
        v-model="wifiDialogOpen"
        :network="dialogNetwork"
        :hidden="hiddenMode"
        @submit="handleWifiDialogSubmit"
    />

    <WifiSavedDialog
        v-model="wifiSavedDialogOpen"
        :saved-networks="savedNetworks"
        @changed="handleNetworksChanged"
    />

    <WifiScanDialog
        v-model="wifiScanDialogOpen"
        :enabled="wifiSettings?.enabled ?? false"
        :connected-ssid="wifiSettings?.connectedSsid ?? null"
        :visible-networks="visibleNetworks"
        @open-network="openVisibleNetworkDialog"
        @changed="handleNetworksChanged"
    />
  </div>
</template>

<style scoped>
.network-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-height: 100%;
}

.network-panel__grid {
  display: grid;
  grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr);
  gap: 16px;
}

.network-panel__title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.network-panel__wifi-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.network-panel__card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.network-panel__row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.network-panel__label {
  opacity: 0.75;
}

.network-panel__value {
  text-align: right;
  word-break: break-word;
}

.network-panel__actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
</style>