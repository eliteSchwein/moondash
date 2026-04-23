<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ShortcutButtonConfig } from '@/stores/app'
import KeyboardOverlay from '@/components/KeyboardOverlay.vue'

type ActiveType = '' | 'output_pin' | 'fan_generic' | 'fan' | 'temperature_fan'

type ShortcutButtonEditorItem = ShortcutButtonConfig & {
  id: string
  position?: number
}

const { t } = useI18n()

const props = defineProps<{
  modelValue: boolean
  item: ShortcutButtonEditorItem | null
  macroSuggestions: string[]
  activeConfigSuggestions: string[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', value: ShortcutButtonEditorItem): void
}>()

const localItem = ref<ShortcutButtonEditorItem | null>(null)

const keyboardModel = ref('')
const keyboardVisible = ref(false)
const keyboardTarget = ref<'' | 'name'>('')
const keyboardTitle = computed(() => {
  if (keyboardTarget.value === 'name') {
    return t('settings.shortcuts.dialog.fields.sectionName')
  }
  return ''
})

function createFallbackItem(): ShortcutButtonEditorItem {
  return {
    id: `${Date.now()}_${Math.random().toString(16).slice(2)}`,
    name: '',
    macro_inactive: '',
    macro_active: '',
    icon: 'mdi-lightbulb',
    active_config: '',
    active_type: '',
    active_threshould: undefined,
    position: undefined,
  }
}

function autoThresholdForType(type: ActiveType): number | undefined {
  if (type === 'output_pin') return 0.5
  if (type === 'fan_generic' || type === 'fan' || type === 'temperature_fan') return 0.2
  return undefined
}

function getActiveSelection(item: ShortcutButtonEditorItem | null): string {
  if (!item?.active_type) return ''
  return item.active_config ? `${item.active_type} ${item.active_config}` : item.active_type
}

function parseActiveSelection(value: string): { active_type: ActiveType; active_config: string } {
  const trimmed = value.trim()
  if (!trimmed) return { active_type: '', active_config: '' }

  const spacerIndex = trimmed.indexOf(' ')
  if (spacerIndex === -1) {
    return { active_type: trimmed as ActiveType, active_config: '' }
  }

  return {
    active_type: trimmed.slice(0, spacerIndex).trim() as ActiveType,
    active_config: trimmed.slice(spacerIndex + 1).trim(),
  }
}

function cloneItem(item: ShortcutButtonEditorItem | null): ShortcutButtonEditorItem {
  return {
    ...(item ?? createFallbackItem()),
    macro_active: item?.macro_active ?? '',
    active_config: item?.active_config ?? '',
    active_type: (item?.active_type ?? '') as ActiveType,
  }
}

watch(
    () => props.modelValue,
    (value) => {
      if (value) {
        localItem.value = cloneItem(props.item)
      } else {
        closeKeyboard()
      }
    },
    { immediate: true },
)

watch(
    () => props.item,
    (value) => {
      if (props.modelValue) {
        localItem.value = cloneItem(value)
      }
    },
)

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
})

const iconOptions = [
  'mdi-lightbulb',
  'mdi-fan',
  'mdi-printer-3d',
  'mdi-printer-3d-nozzle-heat',
  'mdi-printer-3d-nozzle',
  'mdi-led-strip-variant',
]

const activeSelectionOptions = computed(() => {
  return props.activeConfigSuggestions.map((suggestion) => ({
    title: suggestion,
    value: suggestion,
  }))
})

const canSave = computed(() => {
  if (!localItem.value) return false

  const name = localItem.value.name?.trim() ?? ''
  const macroInactive = localItem.value.macro_inactive?.trim() ?? ''
  const macroActive = localItem.value.macro_active?.trim() ?? ''
  const icon = localItem.value.icon?.trim() ?? ''

  if (!name) return false
  if (!macroInactive) return false
  if (!icon) return false

  if (macroInactive.startsWith('_')) return false
  if (macroActive && macroActive.startsWith('_')) return false

  return true
})

function openKeyboardForName() {
  if (!localItem.value) return
  keyboardTarget.value = 'name'
  keyboardModel.value = localItem.value.name ?? ''
  keyboardVisible.value = true
}

function handleKeyboardEnter(value: string) {
  if (!localItem.value) {
    closeKeyboard()
    return
  }

  if (keyboardTarget.value === 'name') {
    localItem.value.name = value
  }

  closeKeyboard()
}

function closeKeyboard() {
  keyboardVisible.value = false
  keyboardTarget.value = ''
  keyboardModel.value = ''
}

function onActiveSelectionChanged(value: string | null) {
  if (!localItem.value) return

  const parsed = parseActiveSelection(value ?? '')
  localItem.value.active_type = parsed.active_type
  localItem.value.active_config = parsed.active_config
  localItem.value.active_threshould = autoThresholdForType(parsed.active_type)
}

function save() {
  if (!localItem.value || !canSave.value) return

  emit('save', {
    ...localItem.value,
    name: localItem.value.name?.trim() ?? '',
    macro_inactive: localItem.value.macro_inactive?.trim() ?? '',
    macro_active: localItem.value.macro_active?.trim() ?? '',
    icon: localItem.value.icon?.trim() ?? '',
    active_config: localItem.value.active_config?.trim() ?? '',
    active_type: localItem.value.active_type?.trim() as ActiveType,
  })
}

function close() {
  closeKeyboard()
  dialogModel.value = false
}
</script>

<template>
  <v-dialog v-model="dialogModel" max-width="1100">
    <v-card rounded="lg">
      <v-card-title class="text-h5 pt-6 px-6">
        <span>{{ t('settings.shortcuts.dialog.title') }}</span>
      </v-card-title>

      <v-card-text v-if="localItem" class="px-6 pb-2">
        <v-row dense>
          <v-col cols="10">
            <v-text-field
                v-model="localItem.name"
                variant="outlined"
                :label="t('settings.shortcuts.dialog.fields.sectionName')"
                density="compact"
                @focus="openKeyboardForName"
            />
          </v-col>

          <v-col cols="2">
            <v-select
                v-model="localItem.icon"
                variant="outlined"
                :items="iconOptions"
                :label="t('settings.shortcuts.dialog.fields.icon')"
                density="compact"
            >
              <template #item="{ props: itemProps, item }">
                <v-list-item v-bind="itemProps">
                  <template #prepend>
                    <v-icon :icon="item" class="mr-2" />
                  </template>
                </v-list-item>
              </template>

              <template #selection="{ item }">
                <div class="d-flex align-center ga-2">
                  <v-icon :icon="item" />
                </div>
              </template>
            </v-select>
          </v-col>

          <v-col cols="6">
            <v-autocomplete
                v-model="localItem.macro_inactive"
                variant="outlined"
                :items="macroSuggestions"
                :label="t('settings.shortcuts.dialog.fields.macroInactive')"
                :placeholder="t('settings.shortcuts.dialog.placeholders.macroInactive')"
                density="compact"
            />
          </v-col>

          <v-col cols="6">
            <v-autocomplete
                v-model="localItem.macro_active"
                variant="outlined"
                :items="macroSuggestions"
                :label="t('settings.shortcuts.dialog.fields.macroActive')"
                :placeholder="t('settings.shortcuts.dialog.placeholders.macroActive')"
                density="compact"
            />
          </v-col>

          <v-col cols="12">
            <v-autocomplete
                :model-value="getActiveSelection(localItem)"
                variant="outlined"
                :items="activeSelectionOptions"
                :label="t('settings.shortcuts.dialog.fields.activeConfig')"
                :placeholder="t('settings.shortcuts.dialog.placeholders.activeConfig')"
                density="compact"
                clearable
                @update:model-value="onActiveSelectionChanged"
            />
          </v-col>

          <v-col cols="12">
            <v-text-field
                v-model="localItem.active_type"
                class="d-none"
                density="compact"
            />
            <v-text-field
                v-model="localItem.active_config"
                class="d-none"
                density="compact"
            />
          </v-col>

          <v-col cols="12">
            <v-alert
                v-if="localItem.active_type === 'output_pin'"
                type="info"
                variant="tonal"
                density="compact"
            >
              {{ t('settings.shortcuts.dialog.threshold.outputPin') }}
            </v-alert>

            <v-alert
                v-else-if="localItem.active_type === 'fan_generic' || localItem.active_type === 'fan' || localItem.active_type === 'temperature_fan'"
                type="info"
                variant="tonal"
                density="compact"
            >
              {{ t('settings.shortcuts.dialog.threshold.fan') }}
            </v-alert>

            <v-alert
                v-else
                type="info"
                variant="tonal"
                density="compact"
            >
              {{ t('settings.shortcuts.dialog.threshold.none') }}
            </v-alert>
          </v-col>
        </v-row>
      </v-card-text>

      <v-card-actions class="justify-end">
        <v-btn variant="text" @click="close">
          {{ t('settings.shortcuts.dialog.actions.cancel') }}
        </v-btn>
        <v-btn color="primary" :disabled="!canSave" @click="save">
          {{ t('settings.shortcuts.dialog.actions.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <KeyboardOverlay
      v-model="keyboardModel"
      :visible="keyboardVisible"
      :title="keyboardTitle"
      layout="default"
      @enter="handleKeyboardEnter"
      @close="closeKeyboard"
  />
</template>