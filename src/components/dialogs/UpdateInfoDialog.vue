<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  modelValue: boolean
  title: string
  messages: string[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function close() {
  emit('update:modelValue', false)
}
</script>

<template>
  <v-dialog
      :model-value="modelValue"
      max-width="520"
      persistent
      @update:model-value="emit('update:modelValue', $event)"
  >
    <v-card rounded="lg">
      <v-card-title class="text-h5 pt-6 px-6">
        {{ title }}
      </v-card-title>

      <v-card-text>

        <div class="update-info-dialog__messages">
          <v-alert
              v-for="message in messages"
              :key="message"
              type="info"
              variant="tonal"
              density="compact"
              class="mb-2"
              border="start"
          >
            {{ message }}
          </v-alert>
        </div>
      </v-card-text>


      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="close">
          {{ t('settings.updates.dialog.close') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.update-info-dialog__messages {
  max-height: 50vh;
  overflow-y: auto;
}

.update-info-dialog__message + .update-info-dialog__message {
  margin-top: 10px;
}
</style>
