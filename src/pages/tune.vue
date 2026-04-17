<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'
import PanelAFC from '@/components/afc/PanelAFC.vue'

const route = useRoute()
const router = useRouter()

const tab = ref('controls')
const { t } = useI18n()
const appStore = useAppStore()
const { moonraker } = storeToRefs(appStore)

const selectedAfcUnit = ref<string | null>(null)

function displayAfterFirstSpace(value: string): string {
  const parts = value.split(' ')
  return parts.length > 1 ? parts.slice(1).join(' ') : value
}

const afcUnits = computed(() => {
  const objects = moonraker.value.afc.objects as Record<string, any>
  const afcRoot = objects['AFC']

  if (!afcRoot || !Array.isArray(afcRoot.units)) {
    return []
  }

  return afcRoot.units.map((unitName: string) => {
    const unitKey = `AFC_BoxTurtle ${unitName}`

    return {
      id: unitName,
      key: unitKey,
      label: unitName,
      displayLabel: displayAfterFirstSpace(unitName),
    }
  })
})

watch(
    () => route.query.tab,
    (queryTab) => {
      if (typeof queryTab === 'string' && queryTab) {
        tab.value = queryTab
      }
    },
    { immediate: true },
)

watch(
    afcUnits,
    (units) => {
      if (!units.length) {
        selectedAfcUnit.value = null
        return
      }

      const queryUnit =
          typeof route.query.afcUnit === 'string' ? route.query.afcUnit : null

      if (queryUnit && units.some((u) => u.id === queryUnit)) {
        selectedAfcUnit.value = queryUnit
        return
      }

      if (!selectedAfcUnit.value || !units.some((u) => u.id === selectedAfcUnit.value)) {
        selectedAfcUnit.value = units[0].id
      }
    },
    { immediate: true },
)

watch(
    () => route.query.afcUnit,
    (queryUnit) => {
      if (typeof queryUnit !== 'string') return
      if (!afcUnits.value.some((u) => u.id === queryUnit)) return

      selectedAfcUnit.value = queryUnit
      tab.value = 'filament'
    },
    { immediate: true },
)

watch(tab, (value) => {
  if (route.query.tab === value) return

  router.replace({
    query: {
      ...route.query,
      tab: value,
    },
  })
})

watch(selectedAfcUnit, (value) => {
  if (!value) return
  if (route.query.afcUnit === value) return

  router.replace({
    query: {
      ...route.query,
      afcUnit: value,
    },
  })
})
</script>

<template>
  <v-main>
    <v-app-bar elevation="0" color="transparent" density="compact">
      <v-tabs v-model="tab" color="primary">
        <v-tab value="controls">{{ t('tune.tab.controls') }}</v-tab>
        <v-tab value="filament">{{ t('tune.tab.filament') }}</v-tab>
        <v-tab value="printoptions">{{ t('tune.tab.printoptions') }}</v-tab>
      </v-tabs>
    </v-app-bar>

    <v-tabs-window v-model="tab" class="panel-container">
      <v-tabs-window-item value="controls">
        <v-sheet class="" color="transparent">
          <ShortcutBarControls/>
        </v-sheet>
      </v-tabs-window-item>

      <v-tabs-window-item value="filament">
        <v-sheet class="pa-5 filament-sheet" color="transparent">
          <v-row class="fill-height" no-gutters>
            <v-col cols="9" md="10" class="pr-3 mb-0">
              <PanelAFC :selected-unit="selectedAfcUnit" />
            </v-col>

            <v-col cols="3" md="2">
              <v-card rounded="lg" class="unit-card pa-0">
                <v-list
                    v-if="afcUnits.length"
                    class="unit-list"
                    density="compact"
                    nav
                    variant="tonal"
                    color="primary"
                >
                  <v-list-item
                      v-for="unit in afcUnits"
                      :key="unit.id"
                      :active="selectedAfcUnit === unit.id"
                      rounded="lg"
                      @click="selectedAfcUnit = unit.id"
                  >
                    <v-list-item-title>
                      {{ unit.displayLabel }}
                    </v-list-item-title>
                  </v-list-item>
                </v-list>

                <v-card-text v-else>
                  <v-alert type="info" variant="tonal">
                    {{ t('afc.no_units') }}
                  </v-alert>
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>
        </v-sheet>
      </v-tabs-window-item>

      <v-tabs-window-item value="printoptions">
        <v-sheet class="pa-5" color="brown">Three</v-sheet>
      </v-tabs-window-item>
    </v-tabs-window>
  </v-main>
</template>

<style scoped>
.panel-container {
  height: 100%;
  min-height: 0;
}

.filament-sheet {
  height: 100%;
  min-height: 0;
}

.unit-card {
  max-height: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.unit-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0;
  max-height: calc(100vh - 89px);
  background-color: rgba(var(--v-theme-on-surface), 0.12);
}
</style>