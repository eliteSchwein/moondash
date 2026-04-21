<script setup lang="ts">
import { computed, ref, watch } from 'vue'
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
  network: WifiNetwork | null
  hidden?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'submit', payload: { ssid: string; password: string }): void
}>()

const localSsid = ref('')
const localPassword = ref('')
const revealPassword = ref(false)

const dialogOpen = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

const isSecured = computed(() => Boolean(props.hidden || props.network?.secured))

watch(
    () => [props.modelValue, props.network, props.hidden],
    ([open]) => {
      if (!open) return

      localSsid.value = props.hidden ? '' : (props.network?.ssid ?? '')
      localPassword.value = ''
      revealPassword.value = false
    },
    { immediate: true },
)

function closeDialog() {
  dialogOpen.value = false
}

function submit() {
  const ssid = localSsid.value.trim()
  if (!ssid) return

  emit('submit', {
    ssid,
    password: localPassword.value,
  })
}
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="640" persistent>
    <v-card rounded="lg">
      <v-card-title>
        {{
          hidden
              ? t('settings.network.wifi.hidden')
              : t('settings.network.wifi.connect_to', { ssid: network?.ssid ?? '' })
        }}
      </v-card-title>

      <v-card-text class="wifi-dialog__content">
        <v-text-field
            v-model="localSsid"
            :label="t('settings.network.wifi.ssid')"
            variant="outlined"
            density="comfortable"
            :readonly="!hidden"
        />

        <v-text-field
            v-if="isSecured"
            v-model="localPassword"
            :label="t('settings.network.wifi.password')"
            :type="revealPassword ? 'text' : 'password'"
            variant="outlined"
            density="comfortable"
            :append-inner-icon="revealPassword ? 'mdi-eye-off' : 'mdi-eye'"
            @click:append-inner="revealPassword = !revealPassword"
        />
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="closeDialog">
          {{ t('settings.network.cancel') }}
        </v-btn>
        <v-btn
            color="primary"
            variant="flat"
            :disabled="!localSsid.trim()"
            @click="submit"
        >
          {{ t('settings.network.connect') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.wifi-dialog__content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>