<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  updateCount: number
}>()

const { t } = useI18n()
const router = useRouter()

async function openUpdatesPanel() {
  if (typeof window !== 'undefined') {
    window.sessionStorage.setItem('settings.tab', 'updates')
    window.dispatchEvent(new CustomEvent('settings:open-tab', { detail: { tab: 'updates' } }))
  }

  await router.push({ path: '/settings', query: { tab: 'updates' } })
}
</script>

<template>
  <v-alert
      color="primary"
      variant="tonal"
      border="start"
      density="compact"
      icon="mdi-update"
  >
    <div class="alert-body">
      <div class="alert-body__content">
        <div class="alert-body__title">
          {{ t('settings.updates.title') }}
        </div>
        <div class="alert-body__message">
          {{ t('settings.updates.updates', { count: props.updateCount }) }}
        </div>
      </div>

      <div class="alert-body__actions">
        <v-btn
            color="primary"
            variant="flat"
            prepend-icon="mdi-cog-outline"
            @click.stop="openUpdatesPanel"
        >
          {{ t('settings.updates.open') }}
        </v-btn>
      </div>
    </div>
  </v-alert>
</template>

<style scoped>
:deep(.v-alert__content) {
  display: flex;
  align-items: flex-start;
  padding-top: 2px;
}

:deep(.v-alert__prepend) {
  align-self: flex-start;
  padding-top: 2px;
}

:deep(.v-alert__close) {
  align-self: flex-start;
}

.alert-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
  width: 100%;
}

.alert-body__content {
  min-width: 0;
  width: 100%;
}

.alert-body__title {
  font-weight: 700;
  margin-bottom: 2px;
  line-height: 1.15;
  word-break: break-word;
}

.alert-body__message {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.3;
}

.alert-body__actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 2px;
}
</style>
