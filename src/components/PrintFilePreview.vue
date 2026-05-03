<script setup lang="ts">
type MoonrakerFile = {
  path?: string
  filename?: string
  display?: string
  modified?: number
  permissions?: string
}

type MoonrakerGcodeMetadata = {
  estimated_time?: number
  filament_weight_total?: number
  filament_weight?: number
  filament?: {
    weight_total?: number
    weight?: number
  }
}

const props = defineProps<{
  file: MoonrakerFile
  title: string
  thumbnailUrl?: string | null
  metadata?: MoonrakerGcodeMetadata | null
  loading?: boolean
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'select', file: MoonrakerFile): void
}>()

function formatPrintTime(seconds: number | undefined): string {
  if (typeof seconds !== 'number' || !Number.isFinite(seconds) || seconds < 0) return ''

  let remaining = Math.floor(seconds)
  const days = Math.floor(remaining / 86400)
  remaining %= 86400
  const hours = Math.floor(remaining / 3600)
  remaining %= 3600
  const minutes = Math.floor(remaining / 60)

  const parts: string[] = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0) parts.push(`${minutes}m`)

  return parts.join('') || '0m'
}

function formatWeight(weight: number | undefined): string {
  if (typeof weight !== 'number' || !Number.isFinite(weight)) return ''
  return `${weight.toFixed(1)}g`
}

function selectFile() {
  if (props.disabled) return
  emit('select', props.file)
}
</script>

<template>
  <v-card
      class="print-file-card"
      :class="{ 'print-file-card--disabled': disabled }"
      variant="flat"
      @click="selectFile"
  >
    <div class="print-file-card__title">
      {{ title }}
    </div>

    <div class="print-file-card__thumb-wrap">
      <img
          v-if="thumbnailUrl"
          :src="thumbnailUrl"
          :alt="title"
          class="print-file-card__thumb"
      >

      <div v-else class="print-file-card__thumb-placeholder">
        <v-progress-circular
            v-if="loading"
            indeterminate
            size="32"
        />
        <v-icon
            v-else
            icon="mdi-printer-3d"
            size="48"
        />
      </div>
    </div>

    <v-card-text class="print-file-card__content">
      <div class="print-file-card__meta">
        <span
            v-if="formatPrintTime(metadata?.estimated_time)"
            class="print-file-card__meta-item"
        >
          <v-icon icon="mdi-clock-outline" size="14" />
          <span>{{ formatPrintTime(metadata?.estimated_time) }}</span>
        </span>

        <span
            v-if="formatWeight(metadata?.filament_weight_total ?? metadata?.filament_weight ?? metadata?.filament?.weight_total ?? metadata?.filament?.weight)"
            class="print-file-card__meta-item"
        >
          <v-icon icon="mdi-scale-balance" size="14" />
          <span>{{ formatWeight(metadata?.filament_weight_total ?? metadata?.filament_weight ?? metadata?.filament?.weight_total ?? metadata?.filament?.weight) }}</span>
        </span>
      </div>
    </v-card-text>
  </v-card>
</template>

<style scoped>
.print-file-card {
  overflow: hidden;
  background: transparent !important;
  box-shadow: none !important;
  position: relative;
  display: flex;
  flex-direction: column;
  cursor: pointer;
}

.print-file-card--disabled {
  cursor: default;
  pointer-events: auto;
  opacity: 0.55;
}

.print-file-card__title,
.print-file-card__meta {
  z-index: 2;
  background: rgba(var(--v-theme-background), 0.8);
}

.print-file-card__title {
  font-weight: 600;
  line-height: 1.2;
  word-break: break-word;
  padding: 8px 10px;
  border-radius: 8px 8px 0 0;
  font-size: 0.8rem;
}

.print-file-card__thumb-wrap {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  aspect-ratio: 1.3 / 1;
  background: rgba(var(--v-theme-on-surface), 0.06);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.print-file-card__thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.print-file-card__thumb-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.print-file-card__content {
  padding: 0;
}

.print-file-card__meta {
  position: absolute;
  left: 0;
  bottom: 0;
  width: 100%;
  font-size: 0.9rem;
  opacity: 0.75;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  padding: 8px 10px;
  border-radius: 0 0 8px 8px;
}

.print-file-card__meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
</style>
