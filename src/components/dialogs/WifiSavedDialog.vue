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
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'changed'): void
}>()

const forgettingSsid = ref<string | null>(null)

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

async function forgetNetwork(ssid: string) {
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
                v-for="network in savedNetworks"
                :key="`saved-${network.ssid}`"
            >
              <template #prepend>
                <v-icon>mdi-content-save</v-icon>
              </template>

              <v-list-item-title>{{ network.ssid }}</v-list-item-title>
              <v-list-item-subtitle>
                {{ signalLabel(network.signalPercent) }}
              </v-list-item-subtitle>

              <template #append>
                <v-btn
                    icon="mdi-delete"
                    size="small"
                    variant="text"
                    color="error"
                    :loading="forgettingSsid === network.ssid"
                    :disabled="forgettingSsid === network.ssid"
                    @click.stop="forgetNetwork(network.ssid)"
                />
              </template>
            </v-list-item>

            <v-list-item v-if="!savedNetworks.length">
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
</style>