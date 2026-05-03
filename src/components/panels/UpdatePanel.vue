<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { moonraker as moonrakerClient } from '@/plugins/moonraker'
import { useAppStore } from '@/stores/app'
import UpdateInfoDialog from '@/components/dialogs/UpdateInfoDialog.vue'

const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

type UpdateStatusResponse = {
  busy?: boolean
  version_info?: Record<string, MoonrakerUpdateService>
}

type MoonrakerUpdateService = {
  name?: string
  configured_type?: string
  channel?: string
  version?: string
  remote_version?: string
  full_version_string?: string
  current_hash?: string
  remote_hash?: string
  commits_behind_count?: number
  package_count?: number
  package_list?: string[]
  is_valid?: boolean
  is_dirty?: boolean
  dirty?: boolean
  warnings?: string[]
  anomalies?: string[]
  last_error?: string
}

type ServiceEntry = {
  key: string
  service: MoonrakerUpdateService
  name: string
  currentVersion: string
  remoteVersion: string
  updateAvailable: boolean
  dirty: boolean
  valid: boolean
  warnings: string[]
  anomalies: string[]
  lastError: string
}

const loading = ref(false)
const refreshing = ref(false)
const updatingAll = ref(false)
const updatingService = ref<string | null>(null)
const updateBusy = ref(false)
const versionInfo = ref<Record<string, MoonrakerUpdateService>>({})
const errorMessage = ref<string | null>(null)
const infoDialogOpen = ref(false)
const infoDialogTitle = ref('')
const infoDialogMessages = ref<string[]>([])

const services = computed<ServiceEntry[]>(() => {
  return Object.entries(versionInfo.value)
      .map(([key, service]) => {
        const name = service.name || key
        const currentVersion = service.full_version_string || service.version || '--'
        const remoteVersion = service.remote_version || service.remote_hash || '--'

        return {
          key,
          service,
          name,
          currentVersion,
          remoteVersion,
          updateAvailable: hasUpdate(service),
          dirty: isDirty(service),
          valid: service.is_valid !== false,
          warnings: Array.isArray(service.warnings) ? service.warnings : [],
          anomalies: Array.isArray(service.anomalies) ? service.anomalies : [],
          lastError: service.last_error || '',
        }
      })
      .sort((a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }))
})

const updateCount = computed(() => services.value.filter((service) => service.updateAvailable).length)
const hasUpdates = computed(() => updateCount.value > 0)
const toolbarTitle = computed(() => {
  if (!hasUpdates.value) return t('settings.updates.no_updates_aviable')
  return t('settings.updates.updates', { count: updateCount.value })
})
const busy = computed(() => loading.value || refreshing.value || updatingAll.value || Boolean(updatingService.value) || updateBusy.value)
const printIsRunning = computed(() => {
  const state = moonraker.value.printStats.state?.toLowerCase() ?? ''
  return ['printing', 'paused', 'pausing', 'resuming', 'cancelling'].includes(state)
})

function isDirty(service: MoonrakerUpdateService): boolean {
  if (service.is_dirty || service.dirty) return true

  const values = [
    service.version,
    service.remote_version,
    service.full_version_string,
  ]

  return values.some((value) => typeof value === 'string' && value.toLowerCase().includes('dirty'))
}

function hasUpdate(service: MoonrakerUpdateService): boolean {
  if (service.configured_type === 'system') {
    return Number(service.package_count ?? 0) > 0
  }

  if (typeof service.commits_behind_count === 'number' && service.commits_behind_count > 0) {
    return true
  }

  if (service.current_hash && service.remote_hash && service.current_hash !== service.remote_hash) {
    return true
  }

  if (service.version && service.remote_version && service.version !== service.remote_version) {
    return true
  }

  return false
}

function getServiceMessages(service: ServiceEntry): string[] {
  const packages = Array.isArray(service.service.package_list)
      ? service.service.package_list.map((pkg) => `${t('settings.updates.package')}: ${pkg}`)
      : []

  return [
    ...service.warnings,
    ...service.anomalies,
    service.lastError,
    ...packages,
  ].filter((message): message is string => Boolean(message))
}

function hasServiceMessages(service: ServiceEntry): boolean {
  return getServiceMessages(service).length > 0
}

function openInfoDialog(service: ServiceEntry) {
  infoDialogTitle.value = service.name
  infoDialogMessages.value = getServiceMessages(service)
  infoDialogOpen.value = true
}

async function loadUpdateStatus() {
  const result = await moonrakerClient.call<UpdateStatusResponse>('machine.update.status')
  versionInfo.value = result.version_info ?? {}
  updateBusy.value = Boolean(result.busy)
}

async function refreshUpdates() {
  try {
    errorMessage.value = null
    refreshing.value = true

    const result = await moonrakerClient.call<UpdateStatusResponse>('machine.update.refresh', undefined, 60000)
    versionInfo.value = result.version_info ?? {}
    updateBusy.value = Boolean(result.busy)
  } catch (error) {
    console.error('Failed to refresh updates', error)
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    refreshing.value = false
  }
}

async function updateService(service: ServiceEntry) {
  if (!service.updateAvailable || service.dirty || !service.valid || printIsRunning.value) return

  try {
    errorMessage.value = null
    updatingService.value = service.key

    await moonrakerClient.call('machine.update.upgrade', { name: service.name }, null)
    await loadUpdateStatus()
  } catch (error) {
    console.error(`Failed to update ${service.name}`, error)
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    updatingService.value = null
  }
}

async function updateAll() {
  if (!hasUpdates.value || printIsRunning.value) return

  try {
    errorMessage.value = null
    updatingAll.value = true

    await moonrakerClient.call('machine.update.full', undefined, null)
    await loadUpdateStatus()
  } catch (error) {
    console.error('Failed to update all services', error)
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    updatingAll.value = false
  }
}

onMounted(async () => {
  try {
    loading.value = true
    await loadUpdateStatus()
  } catch (error) {
    console.error('Failed to load update status', error)
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="update-panel">
    <v-toolbar
        class="update-panel__toolbar px-2"
        density="compact"
        rounded
        flat
    >
      <v-toolbar-title>{{ toolbarTitle }}</v-toolbar-title>

      <v-spacer />

      <v-btn
          v-if="hasUpdates"
          prepend-icon="mdi-update"
          variant="tonal"
          :loading="updatingAll"
          :disabled="(busy && !updatingAll) || printIsRunning"
          @click="updateAll"
      >
        {{ t('settings.updates.update_all') }}
      </v-btn>

      <v-btn
          icon="mdi-refresh"
          variant="text"
          :loading="refreshing"
          :disabled="busy && !refreshing"
          @click="refreshUpdates"
      />
    </v-toolbar>

    <v-alert
        v-if="errorMessage"
        type="error"
        variant="tonal"
        closable
        @click:close="errorMessage = null"
    >
      {{ errorMessage }}
    </v-alert>

    <div v-if="loading" class="update-panel__state">
      <v-progress-circular indeterminate />
    </div>

    <v-list v-else class="update-panel__list" density="comfortable" bg-color="transparent">
      <v-list-item
          v-for="service in services"
          :key="service.key"
          class="update-panel__service"
          rounded="lg"
      >
        <v-list-item-title class="update-panel__service-title">
          {{ service.name }}
        </v-list-item-title>

        <v-list-item-subtitle>
          <div class="update-panel__versions">
            <span>{{ service.currentVersion }}</span>
            <template v-if="service.updateAvailable">
              <v-icon icon="mdi-arrow-right" size="14" />
              <span>{{ service.remoteVersion }}</span>
            </template>
          </div>
        </v-list-item-subtitle>

        <div
            v-if="service.service.configured_type === 'system' && service.service.package_count"
            class="update-panel__packages"
        >
          {{ t('settings.updates.package_count', { count: service.service.package_count }) }}
        </div>

        <template #append>
          <div class="update-panel__actions">
            <v-btn
                v-if="hasServiceMessages(service)"
                class="update-panel__action update-panel__action--icon"
                icon="mdi-information-outline"
                variant="text"
                @click="openInfoDialog(service)"
            />

            <v-btn
                v-if="service.dirty"
                class="update-panel__action update-panel__action--update"
                color="warning"
                prepend-icon="mdi-alert-outline"
                variant="tonal"
                disabled
            >
              {{ t('settings.updates.dirty') }}
            </v-btn>

            <v-btn
                v-else-if="service.updateAvailable"
                class="update-panel__action update-panel__action--update"
                color="primary"
                prepend-icon="mdi-update"
                variant="tonal"
                :loading="updatingService === service.key"
                :disabled="(busy && updatingService !== service.key) || !service.valid || printIsRunning"
                @click="updateService(service)"
            >
              {{ t('settings.updates.update') }}
            </v-btn>

            <v-btn
                v-else
                class="update-panel__action update-panel__action--update"
                color="success"
                prepend-icon="mdi-check-circle-outline"
                variant="tonal"
                disabled
            >
              {{ t('settings.updates.up_to_date') }}
            </v-btn>
          </div>
        </template>
      </v-list-item>
    </v-list>

    <UpdateInfoDialog
        v-model="infoDialogOpen"
        :title="infoDialogTitle"
        :messages="infoDialogMessages"
    />
  </div>
</template>

<style scoped>
.update-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px;
  box-sizing: border-box;
}

.update-panel__toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
}

.update-panel__state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.update-panel__list {
  flex: 1;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.update-panel__service {
  background: rgba(var(--v-theme-surface), 1);
}

.update-panel__service-title {
  font-weight: 600;
}

.update-panel__versions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
}

.update-panel__packages {
  margin-top: 6px;
  font-size: 0.85em;
  opacity: 0.75;
}

.update-panel__actions {
  display: inline-flex;
  align-items: stretch;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  border-radius: 10px;
  overflow: hidden;
  height: 44px;
}

.update-panel__action {
  border-radius: 0;
  min-height: 44px;
}

.update-panel__action:not(:last-child) {
  border-right: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.update-panel__action--icon {
  width: 44px;
  min-width: 44px;
}

.update-panel__action--icon :deep(.v-icon) {
  transform: translateY(-3px);
}

.update-panel__action--update {
  min-width: 132px;
}
</style>
