<script setup lang="ts">
import { computed } from 'vue'

type KeyboardKey = {
  label: string
  value?: string
  action?: 'backspace' | 'shift' | 'space' | 'enter' | 'close' | 'clear'
  wide?: boolean
  extraWide?: boolean
}

const props = withDefaults(defineProps<{
  modelValue: string
  visible: boolean
  title?: string
  layout?: 'default' | 'numeric'
}>(), {
  title: '',
  layout: 'default',
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'enter'): void
  (e: 'close'): void
}>()

const shifted = defineModel<boolean>('shifted', { default: false })
const previewValue = computed(() => props.modelValue)

const alphaRows = computed<KeyboardKey[][]>(() => {
  const letters1 = ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p']
  const letters2 = ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l']
  const letters3 = ['z', 'x', 'c', 'v', 'b', 'n', 'm']

  const mapKey = (key: string): KeyboardKey => ({
    label: shifted.value ? key.toUpperCase() : key,
    value: shifted.value ? key.toUpperCase() : key,
  })

  return [
    letters1.map(mapKey),
    letters2.map(mapKey),
    [
      { label: '⇧', action: 'shift', wide: true },
      ...letters3.map(mapKey),
      { label: '⌫', action: 'backspace', wide: true },
    ],
    [
      { label: 'Clear', action: 'clear', wide: true },
      { label: 'Space', action: 'space', extraWide: true },
      { label: 'Enter', action: 'enter', wide: true },
      { label: 'Close', action: 'close', wide: true },
    ],
  ]
})

const numericRows = computed<KeyboardKey[][]>(() => [
  ['1', '2', '3'].map((k) => ({ label: k, value: k })),
  ['4', '5', '6'].map((k) => ({ label: k, value: k })),
  ['7', '8', '9'].map((k) => ({ label: k, value: k })),
  [
    { label: '.', value: '.' },
    { label: '0', value: '0' },
    { label: '⌫', action: 'backspace' },
  ],
  [
    { label: 'Clear', action: 'clear', wide: true },
    { label: 'Enter', action: 'enter', wide: true },
    { label: 'Close', action: 'close', wide: true },
  ],
])

const rows = computed(() => {
  return props.layout === 'numeric' ? numericRows.value : alphaRows.value
})

function pressKey(key: KeyboardKey) {
  if (key.value !== undefined) {
    emit('update:modelValue', props.modelValue + key.value)

    if (props.layout === 'default' && shifted.value) {
      shifted.value = false
    }
    return
  }

  switch (key.action) {
    case 'backspace':
      emit('update:modelValue', props.modelValue.slice(0, -1))
      break
    case 'shift':
      shifted.value = !shifted.value
      break
    case 'space':
      emit('update:modelValue', props.modelValue + ' ')
      break
    case 'clear':
      emit('update:modelValue', '')
      break
    case 'enter':
      emit('enter')
      break
    case 'close':
      emit('close')
      break
  }
}
</script>

<template>
  <Teleport to=".v-application">
    <div v-if="visible" class="keyboard-overlay">
      <div class="keyboard-overlay__backdrop" @click="$emit('close')" />

      <div class="keyboard-overlay__panel">
        <div v-if="title" class="keyboard-overlay__title">
          {{ title }}
        </div>

        <div class="keyboard-overlay__preview">
          <span v-if="previewValue">{{ previewValue }}</span>
          <span v-else class="keyboard-overlay__preview-placeholder">…</span>
        </div>

        <div class="keyboard-overlay__rows">
          <div
              v-for="(row, rowIndex) in rows"
              :key="rowIndex"
              class="keyboard-overlay__row"
          >
            <button
                v-for="(key, keyIndex) in row"
                :key="`${rowIndex}-${keyIndex}-${key.label}`"
                type="button"
                class="keyboard-overlay__key"
                :class="{
                'keyboard-overlay__key--wide': key.wide,
                'keyboard-overlay__key--extra-wide': key.extraWide,
                'keyboard-overlay__key--active': key.action === 'shift' && shifted,
                'keyboard-overlay__key--action': !!key.action,
              }"
                @click="pressKey(key)"
            >
              {{ key.label }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.keyboard-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  pointer-events: auto;
}

.keyboard-overlay__backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.18);
}

.keyboard-overlay__panel {
  position: absolute;
  left: 0;
  bottom: 0;
  width: 100%;
  padding: 12px;
  background: rgb(var(--v-theme-surface));
  color: rgb(var(--v-theme-on-surface));
  border: 1px solid rgba(var(--v-theme-on-surface), 0.1);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.22);
}

.keyboard-overlay__title {
  margin-bottom: 10px;
  font-size: 0.95rem;
  font-weight: 700;
  opacity: 0.9;
}

.keyboard-overlay__preview {
  min-height: 44px;
  margin-bottom: 10px;
  padding: 10px 12px;
  border-radius: 12px;
  background: rgba(var(--v-theme-on-surface), 0.05);
  border: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  font-size: 1rem;
  line-height: 1.2;
  word-break: break-word;
}

.keyboard-overlay__preview-placeholder {
  opacity: 0.45;
}

.keyboard-overlay__rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.keyboard-overlay__row {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.keyboard-overlay__key {
  appearance: none;
  border: none;
  min-width: 52px;
  height: 52px;
  padding: 0 14px;
  border-radius: 14px;
  background: rgba(var(--v-theme-on-surface), 0.06);
  color: inherit;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.08s ease, background 0.12s ease;
}

.keyboard-overlay__key:hover {
  background: rgba(var(--v-theme-on-surface), 0.1);
}

.keyboard-overlay__key:active {
  transform: scale(0.97);
}

.keyboard-overlay__key--action {
  background: rgba(var(--v-theme-primary), 0.14);
}

.keyboard-overlay__key--active {
  background: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
}

.keyboard-overlay__key--wide {
  min-width: 84px;
}

.keyboard-overlay__key--extra-wide {
  min-width: 180px;
}
</style>