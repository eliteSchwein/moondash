<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useAppStore, type ShortcutButtonConfig } from '@/stores/app'
import ShortcutButtonEditDialog from '@/components/dialogs/ShortcutButtonEditDialog.vue'

type ActiveType = '' | 'output_pin' | 'fan_generic' | 'fan' | 'temperature_fan'

type ShortcutButtonEditorItem = ShortcutButtonConfig & {
  id: string
  position: number
}

type ShortcutButtonDialogItem = ShortcutButtonConfig & {
  id: string
  position?: number
}

const { t } = useI18n()

const appStore = useAppStore()
const { moonraker, shortcutButtons } = storeToRefs(appStore)

const saving = ref(false)
const errorMessage = ref<string | null>(null)

const editDialogOpen = ref(false)
const editingItem = ref<ShortcutButtonEditorItem | null>(null)

const editorItems = ref<ShortcutButtonEditorItem[]>([])
const initializedFromStore = ref(false)

function createId(): string {
  return `${Date.now()}_${Math.random().toString(16).slice(2)}`
}

function autoThresholdForType(type: ActiveType): number | undefined {
  if (type === 'output_pin') return 0.5
  if (type === 'fan_generic' || type === 'fan' || type === 'temperature_fan') return 0.2
  return undefined
}

function sortByPosition<T extends { position?: number }>(items: T[]): T[] {
  return [...items].sort((a, b) => (a.position ?? 0) - (b.position ?? 0))
}

function reindexPositions(items: ShortcutButtonEditorItem[]): ShortcutButtonEditorItem[] {
  return items.map((item, index) => ({
    ...item,
    position: index,
  }))
}

function makeEmptyButton(): ShortcutButtonEditorItem {
  return {
    id: createId(),
    name: '',
    macro_inactive: '',
    macro_active: '',
    icon: 'mdi-lightbulb',
    active_config: '',
    active_type: '',
    active_threshould: undefined,
    position: editorItems.value.length,
  }
}

function toEditorItems(items: ShortcutButtonConfig[]): ShortcutButtonEditorItem[] {
  return sortByPosition(items).map((item, index) => ({
    id: createId(),
    name: item.name,
    macro_inactive: item.macro_inactive,
    macro_active: item.macro_active ?? '',
    icon: item.icon,
    active_config: item.active_config ?? '',
    active_type: (item.active_type as ActiveType | undefined) ?? '',
    active_threshould: item.active_threshould,
    position: typeof item.position === 'number' ? item.position : index,
  }))
}

watch(
    shortcutButtons,
    (value) => {
      if (initializedFromStore.value) return
      editorItems.value = toEditorItems(value ?? [])
      initializedFromStore.value = true
    },
    { immediate: true, deep: true },
)

const rawObjects = computed(() => {
  return (moonraker.value?.rawObjects ?? {}) as Record<string, unknown>
})

const macroSuggestions = computed(() => {
  const keys = Object.keys(rawObjects.value)

  return Array.from(
      new Set(
          keys
              .filter((key) => key.toLowerCase().startsWith('gcode_macro '))
              .map((key) => key.substring('gcode_macro '.length).trim())
              .filter((name) => name.length > 0 && !name.startsWith('_')),
      ),
  ).sort((a, b) => a.localeCompare(b))
})

const activeConfigSuggestions = computed(() => {
  const keys = Object.keys(rawObjects.value)

  return Array.from(new Set(keys))
      .filter((key) => {
        const trimmed = key.trim()
        const lower = trimmed.toLowerCase()

        if (lower === 'fan') return false

        const prefixes = ['output_pin ', 'fan_generic ', 'temperature_fan ']

        const matchedPrefix = prefixes.find((prefix) => lower.startsWith(prefix))
        if (!matchedPrefix) return false

        const name = trimmed.slice(matchedPrefix.length).trim()
        if (!name) return false
        if (name.startsWith('_')) return false

        return true
      })
      .sort((a, b) => a.localeCompare(b))
})

function openEditDialog(item: ShortcutButtonEditorItem) {
  editingItem.value = { ...item }
  editDialogOpen.value = true
}

function closeEditDialog() {
  editDialogOpen.value = false
  editingItem.value = null
}

function normalizeButton(item: ShortcutButtonEditorItem): ShortcutButtonConfig | null {
  const name = item.name.trim()
  const macroInactive = item.macro_inactive.trim()
  const icon = item.icon.trim()

  if (!name || !macroInactive || !icon) return null

  const normalized: ShortcutButtonConfig = {
    name,
    macro_inactive: macroInactive,
    icon,
    position: item.position,
  }

  const macroActive = item.macro_active?.trim()
  if (macroActive) normalized.macro_active = macroActive

  const activeConfig = item.active_config?.trim()
  if (activeConfig) normalized.active_config = activeConfig

  const activeType = item.active_type?.trim() as ActiveType
  if (activeType) normalized.active_type = activeType

  if (typeof item.active_threshould === 'number' && Number.isFinite(item.active_threshould)) {
    normalized.active_threshould = item.active_threshould
  }

  return normalized
}

async function persistEditorItems() {
  errorMessage.value = null
  saving.value = true

  try {
    editorItems.value = reindexPositions(sortByPosition(editorItems.value))

    const normalized = editorItems.value
        .map(normalizeButton)
        .filter((item): item is ShortcutButtonConfig => item !== null)

    await appStore.saveShortcutButtons(normalized)
  } catch (error) {
    console.error('Failed to save shortcut buttons', error)
    errorMessage.value =
        error instanceof Error ? error.message : t('settings.shortcuts.errors.saveFailed')
  } finally {
    saving.value = false
  }
}

async function onEditDialogSave(updated: ShortcutButtonDialogItem) {
  const normalizedUpdated: ShortcutButtonEditorItem = {
    ...updated,
    name: updated.name?.trim() ?? '',
    macro_inactive: updated.macro_inactive?.trim() ?? '',
    macro_active: updated.macro_active?.trim() ?? '',
    icon: updated.icon?.trim() ?? '',
    active_config: updated.active_config?.trim() ?? '',
    active_type: (updated.active_type?.trim() ?? '') as ActiveType,
    active_threshould:
        autoThresholdForType((updated.active_type?.trim() ?? '') as ActiveType) ?? updated.active_threshould,
    position: typeof updated.position === 'number' ? updated.position : editorItems.value.length,
  }

  const index = editorItems.value.findIndex((item) => item.id === normalizedUpdated.id)

  if (index === -1) {
    editorItems.value = reindexPositions([...editorItems.value, normalizedUpdated])
  } else {
    const items = [...editorItems.value]
    items[index] = normalizedUpdated
    editorItems.value = reindexPositions(sortByPosition(items))
  }

  closeEditDialog()
  await persistEditorItems()
}

function addButton() {
  if (editorItems.value.length >= 3) return
  openEditDialog(makeEmptyButton())
}

async function removeButton(id: string) {
  editorItems.value = reindexPositions(editorItems.value.filter((item) => item.id !== id))
  await persistEditorItems()
}

async function moveItemUp(index: number) {
  if (index <= 0 || saving.value) return

  const items = [...editorItems.value]
  const current = items[index]
  const previous = items[index - 1]

  if (!current || !previous) return

  const currentPosition = current.position
  current.position = previous.position
  previous.position = currentPosition

  editorItems.value = sortByPosition(items)
  await persistEditorItems()
}

async function moveItemDown(index: number) {
  if (index >= editorItems.value.length - 1 || saving.value) return

  const items = [...editorItems.value]
  const current = items[index]
  const next = items[index + 1]

  if (!current || !next) return

  const currentPosition = current.position
  current.position = next.position
  next.position = currentPosition

  editorItems.value = sortByPosition(items)
  await persistEditorItems()
}
</script>

<template>
  <v-card rounded="lg" variant="flat" class="config-editor-panel">
    <v-card-text class="config-editor-panel__content pa-0">
      <div class="shortcut-buttons-panel">
        <v-alert
            v-if="errorMessage"
            type="error"
            variant="tonal"
            class="mb-4"
        >
          {{ errorMessage }}
        </v-alert>

        <v-list
            class="shortcut-buttons-list pa-0"
            rounded="lg"
            density="compact"
        >
          <template
              v-for="(item, index) in editorItems"
              :key="item.id"
          >
            <v-list-item class="shortcut-buttons-list__item">
              <template #prepend>
                <v-icon :icon="item.icon" />
              </template>

              <v-list-item-title class="shortcut-buttons-list__name">
                {{ item.name }}
              </v-list-item-title>

              <template #append>
                <div class="shortcut-buttons-list__actions">
                  <v-btn
                      v-if="index > 0"
                      icon="mdi-chevron-up"
                      variant="text"
                      :disabled="saving"
                      @click="moveItemUp(index)"
                  />

                  <v-btn
                      v-if="index < editorItems.length - 1"
                      icon="mdi-chevron-down"
                      variant="text"
                      :disabled="saving"
                      @click="moveItemDown(index)"
                  />

                  <v-btn
                      icon="mdi-pencil"
                      variant="text"
                      :disabled="saving"
                      @click="openEditDialog(item)"
                  />

                  <v-btn
                      icon="mdi-trash-can-outline"
                      variant="text"
                      color="error"
                      :disabled="saving"
                      @click="removeButton(item.id)"
                  />
                </div>
              </template>
            </v-list-item>

            <v-divider v-if="index < editorItems.length - 1" />
          </template>
        </v-list>

        <ShortcutButtonEditDialog
            v-model="editDialogOpen"
            :item="editingItem"
            :macro-suggestions="macroSuggestions"
            :active-config-suggestions="activeConfigSuggestions"
            @save="onEditDialogSave"
        />
      </div>
    </v-card-text>

    <v-card-actions class="config-editor-panel__actions">
      <v-spacer />
      <v-btn
          color="primary"
          v-if="editorItems.length < 3"
          :disabled="saving"
          @click="addButton"
          prepend-icon="mdi-plus"
      >
        {{ t('settings.shortcuts.add') }}
      </v-btn>
    </v-card-actions>
  </v-card>
</template>

<style scoped>
.config-editor-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.config-editor-panel__content {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.config-editor-panel__actions {
  flex: 0 0 auto;
}

.shortcut-buttons-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1 1 auto;
  min-height: 0;
}

.shortcut-buttons-list {
  flex: 1 1 auto;
  min-height: 0;
}

.shortcut-buttons-list__item {
  min-height: 64px;
}

.shortcut-buttons-list__name {
  font-weight: 700;
}

.shortcut-buttons-list__actions {
  display: flex;
  align-items: center;
  gap: 4px;
}
</style>