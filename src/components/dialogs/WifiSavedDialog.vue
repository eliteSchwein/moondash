<script setup lang="ts">
import { computed, ref } from 'vue'
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
  savedNetworks: WifiNetwork[]
  connectedSsid: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'changed'): void
  (e: 'edit-network', network: WifiNetwork): void
}>()

const forgettingSsid = ref<string | null>(null)
const connectingSsid = ref<string | null>(null)

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

const sortedSavedNetworks = computed(() => {
  const bySsid = new Map<string, WifiNetwork>()

  for (const network of props.savedNetworks) {
    const ssid = network.ssid.trim()
    if (!ssid) continue

    const existing = bySsid.get(ssid)

    if (!existing) {
      bySsid.set(ssid, {
        ...network,
        ssid,
      })
      continue
    }

    const existingSignal = existing.signalPercent ?? -1
    const nextSignal = network.signalPercent ?? -1

    if (nextSignal > existingSignal) {
      bySsid.set(ssid, {
        ...network,
        ssid,
      })
    }
  }

  return [...bySsid.values()].sort((a, b) =>
      a.ssid.localeCompare(b.ssid, undefined, { sensitivity: 'base' }),
  )
})

function signalLabel(percent: number | null | undefined): string {
  if (typeof percent !== 'number' || !Number.isFinite(percent)) {
    return '--'
  }

  return `${percent}%`
}

async function connectSavedNetwork(network: WifiNetwork) {
  if (connectingSsid.value || forgettingSsid.value) return

  try {
    connectingSsid.value = network.ssid

    await invoke('connect_to_wifi', {
      ssid: network.ssid,
      password: null,
    })

    emit('changed')
  } finally {
    connectingSsid.value = null
  }
}

function editNetwork(network: WifiNetwork) {
  emit('edit-network', network)
}

async function forgetNetwork(ssid: string) {
  if (connectingSsid.value || forgettingSsid.value) return

  try {
    forgettingSsid.value = ssid
    await invoke('forget_saved_wifi', { ssid })
    emit('changed')
  } finally {
    forgettingSsid.value = null
  }
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="760" persistent>
    <v-card rounded="lg">
      <v-card-title>
        {{ t('settings.network.wifi.saved_networks') }}
      </v-card-title>

      <v-card-text>
        <div class="wifi-saved-dialog__list-wrap">
          <v-list density="compact" bg-color="transparent">
            <v-list-item
                v-for="network in sortedSavedNetworks"
                :key="`saved-${network.ssid}`"
                :disabled="Boolean(connectingSsid || forgettingSsid)"
                @click="connectSavedNetwork(network)"
            >
              <template #prepend>
                <v-icon :color="connectedSsid === network.ssid ? 'success' : undefined">
                  {{ connectedSsid === network.ssid ? 'mdi-wifi-check' : 'mdi-content-save' }}
                </v-icon>
              </template>

              <v-list-item-title>{{ network.ssid }}</v-list-item-title>

              <v-list-item-subtitle>
                {{ signalLabel(network.signalPercent) }}
              </v-list-item-subtitle>

              <template #append>
                <div class="wifi-saved-dialog__actions">
                  <v-progress-circular
                      v-if="connectingSsid === network.ssid"
                      indeterminate
                      size="24"
                      width="3"
                      color="primary"
                  />

                  <v-chip
                      v-else-if="connectedSsid === network.ssid"
                      color="success"
                      variant="tonal"
                      size="small"
                  >
                    {{ t('settings.network.wifi.connected') }}
                  </v-chip>

                  <v-btn
                      icon="mdi-pencil"
                      variant="text"
                      :disabled="Boolean(connectingSsid || forgettingSsid)"
                      @click.stop="editNetwork(network)"
                  />

                  <v-btn
                      icon="mdi-delete"
                      variant="text"
                      color="error"
                      :loading="forgettingSsid === network.ssid"
                      :disabled="Boolean(connectingSsid || forgettingSsid)"
                      @click.stop="forgetNetwork(network.ssid)"
                  />
                </div>
              </template>
            </v-list-item>

            <v-list-item v-if="!sortedSavedNetworks.length">
              <v-list-item-title>
                {{ t('settings.network.wifi.no_saved_networks') }}
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
.wifi-saved-dialog__list-wrap {
  max-height: calc(100vh - 200px);
  height: calc(100vh - 200px);
  overflow-y: auto;
  overflow-x: hidden;
}

.wifi-saved-dialog__actions {
  display: flex;
  align-items: center;
  gap: 4px;
}
</style>