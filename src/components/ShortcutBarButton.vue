<script setup lang="ts">
defineProps<{
  icon?: string
  label?: string
  color?: string
}>()

defineEmits<{
  (e: 'click', event: MouseEvent): void
}>()
</script>

<template>
  <v-btn
      class="shortcut-bar-list-btn"
      variant="text"
      block
      rounded="0"
      :color="color"
      @click="$emit('click', $event)"
  >
    <div class="shortcut-bar-list-btn-content">
      <v-icon v-if="icon" :icon="icon" size="30px" />
      <span v-if="label">{{ label }}</span>
      <slot />
    </div>
  </v-btn>
</template>

<style scoped>
.shortcut-bar-list-btn {
  width: 100%;
  min-width: 0;
  min-height: 60px;
  height: auto;
  justify-content: center;
  padding: 0;
}

.shortcut-bar-list-btn-content {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  line-height: 1.1;
  gap: 4px;
  padding-top: 8px;
  padding-bottom: 8px;
}

.shortcut-bar-list-btn :deep(.v-btn__content) {
  width: 100%;
  display: flex;
  justify-content: center;
}
</style>