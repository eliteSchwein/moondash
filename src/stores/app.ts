import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

type StylingConfig = {
  zoom?: number
  darkmode?: boolean
  primary?: string | null
  secondary?: string | null
}

type DevConfig = {
  debug?: boolean
}

type SystemConfig = {
  language?: string | null
}

export type ShortcutButtonConfig = {
  name: string
  macro_inactive: string
  macro_active?: string
  icon: string
  active_config?: string
  active_type?: string
  active_threshould?: number
}

type AppConfig = {
  websocket?: {
    ip?: string
  }
  styling?: StylingConfig
  dev?: DevConfig
  system?: SystemConfig
  shortcutbuttons?: ShortcutButtonConfig[]
}

type MoonrakerHeater = {
  temperature: number | null
  target: number | null
  power: number | null
}

type MoonrakerToolhead = {
  position: number[]
}

type MoonrakerWebhooks = {
  state: string | null
  stateMessage: string | null
}

type MoonrakerPrintStats = {
  state: string | null
  filename: string | null
  message: string | null
  printDuration: number | null
  totalDuration: number | null
  filamentUsed: number | null
  info: Record<string, unknown>
}

type MoonrakerVirtualSdcard = {
  progress: number | null
  filePosition: number | null
  isActive: boolean | null
}

type MoonrakerProcStats = {
  moonrakerCpuUsage: number | null
  systemCpuUsage: number | null
  cpuTemp: number | null
  memory: number | null
  network: Record<string, unknown>
  systemUptime: number | null
}

type MoonrakerThrottleState = {
  bits: number | null
  flags: string[]
}

type MoonrakerDisplayStatus = {
  message: string | null
  progress: number | null
}

type MoonrakerHistoryState = {
  lastAction: string | null
  payload: unknown
}

type MoonrakerGcodeMove = {
  speed: number | null
  speedFactor: number | null
}

type MoonrakerAfcState = {
  available: boolean
  objects: Record<string, unknown>
}

type MoonrakerDynamicHeater = {
  key: string
  label: string
  temperature: number | null
  target: number | null
  power: number | null
}

type MoonrakerDynamicFan = {
  key: string
  label: string
  speed: number | null
  rpm: number | null
  temperature: number | null
  target: number | null
  isTemperatureFan: boolean
}

type FilesState = {
  items: unknown[]
  lastUpdated: number | null
}

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function asNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null
}

function asString(value: unknown): string | null {
  return typeof value === 'string' ? value : null
}

function asBoolean(value: unknown): boolean | null {
  return typeof value === 'boolean' ? value : null
}

function prettifyMoonrakerObjectName(key: string): string {
  const lower = key.toLowerCase()

  if (lower === 'extruder') return 'extruder'
  if (lower === 'heater_bed') return 'heater_bed'
  if (lower === 'fan') return 'fan'

  return key
      .replace(/^heater_generic\s+/i, '')
      .replace(/^fan_generic\s+/i, '')
      .replace(/^controller_fan\s+/i, '')
      .replace(/^heater_fan\s+/i, '')
      .replace(/^temperature_fan\s+/i, '')
      .replace(/^temperature_sensor\s+/i, '')
      .replace(/^_+|_+$/g, '')
      .replace(/_/g, ' ')
}

function parseShortcutButtonsFromConfig(config: Record<string, unknown>): ShortcutButtonConfig[] {
  const result: ShortcutButtonConfig[] = []

  for (const [key, value] of Object.entries(config)) {
    if (!key.toLowerCase().startsWith('shortcutbutton ')) continue
    if (!value || typeof value !== 'object' || Array.isArray(value)) continue

    const record = value as Record<string, unknown>
    const name = key.substring('shortcutbutton '.length).trim()
    if (!name) continue

    const macroInactive = asString(record.macro_inactive)
    const icon = asString(record.icon)

    if (!macroInactive || !icon) continue

    const button: ShortcutButtonConfig = {
      name,
      macro_inactive: macroInactive,
      icon,
    }

    const macroActive = asString(record.macro_active)
    if (macroActive) button.macro_active = macroActive

    const activeConfig = asString(record.active_config)
    if (activeConfig) button.active_config = activeConfig

    const activeType = asString(record.active_type)
    if (activeType) button.active_type = activeType

    const threshold = asNumber(record.active_threshould)
    if (typeof threshold === 'number') button.active_threshould = threshold

    result.push(button)
  }

  return result.sort((a, b) => a.name.localeCompare(b.name))
}

export const useAppStore = defineStore('app', {
  state: () => ({
    darkmode: true as boolean,
    zoom: 1.0 as number,
    debug: false as boolean,
    language: null as string | null,
    primaryColor: null as string | null,
    secondaryColor: null as string | null,
    shortcutButtons: [] as ShortcutButtonConfig[],

    websocket: {
      ip: '127.0.0.1:7125',
      connected: false,
    },

    moonrakerReady: false as boolean,

    moonraker: {
      webhooks: {
        state: null,
        stateMessage: null,
      } as MoonrakerWebhooks,

      extruder: {
        temperature: null,
        target: null,
        power: null,
      } as MoonrakerHeater,

      heaterBed: {
        temperature: null,
        target: null,
        power: null,
      } as MoonrakerHeater,

      toolhead: {
        position: [],
      } as MoonrakerToolhead,

      gcodeMove: {
        speed: null,
        speedFactor: null,
      } as MoonrakerGcodeMove,

      printStats: {
        state: null,
        filename: null,
        message: null,
        printDuration: null,
        totalDuration: null,
        filamentUsed: null,
        info: {},
      } as MoonrakerPrintStats,

      virtualSdcard: {
        progress: null,
        filePosition: null,
        isActive: null,
      } as MoonrakerVirtualSdcard,

      procStats: {
        moonrakerCpuUsage: null,
        systemCpuUsage: null,
        cpuTemp: null,
        memory: null,
        network: {},
        systemUptime: null,
      } as MoonrakerProcStats,

      throttle: {
        bits: null,
        flags: [],
      } as MoonrakerThrottleState,

      displayStatus: {
        message: null,
        progress: null,
      } as MoonrakerDisplayStatus,

      history: {
        lastAction: null,
        payload: null,
      } as MoonrakerHistoryState,

      afc: {
        available: false,
        objects: {},
      } as MoonrakerAfcState,

      dynamicHeaters: [] as MoonrakerDynamicHeater[],
      dynamicFans: [] as MoonrakerDynamicFan[],
      rawObjects: {} as Record<string, unknown>,
    },

    files: {
      items: [],
      lastUpdated: null,
    } as FilesState,

    configListener: null as UnlistenFn | null,
  }),

  getters: {
    isDarkmode: (state) => state.darkmode,
    getZoom: (state) => state.zoom,
    isDebugEnabled: (state) => state.debug,
    getLanguage: (state) => state.language,
    getPrimaryColor: (state) => state.primaryColor,
    getSecondaryColor: (state) => state.secondaryColor,
    getShortcutButtons: (state) => state.shortcutButtons,

    getWebsocket: (state) => state.websocket,
    getWebsocketIp: (state) => state.websocket.ip,
    isWebsocketConnected: (state) => state.websocket.connected,
    isMoonrakerReady: (state) => state.moonrakerReady,

    getMoonrakerState: (state) => state.moonraker.webhooks.state,
    getMoonrakerStateMessage: (state) => state.moonraker.webhooks.stateMessage,

    getExtruder: (state) => state.moonraker.extruder,
    getHeaterBed: (state) => state.moonraker.heaterBed,
    getToolheadPosition: (state) => state.moonraker.toolhead.position,
    getGcodeMove: (state) => state.moonraker.gcodeMove,
    getPrintSpeed: (state) => state.moonraker.gcodeMove.speed,
    getPrintSpeedFactor: (state) => state.moonraker.gcodeMove.speedFactor,

    getPrintStats: (state) => state.moonraker.printStats,
    getPrintFilename: (state) => state.moonraker.printStats.filename,
    getPrintState: (state) => state.moonraker.printStats.state,
    getPrintProgress: (state) => state.moonraker.virtualSdcard.progress,

    getProcStats: (state) => state.moonraker.procStats,
    getCpuTemp: (state) => state.moonraker.procStats.cpuTemp,

    getAfc: (state) => state.moonraker.afc,
    isAfcAvailable: (state) => state.moonraker.afc.available,

    getDynamicHeaters: (state) => state.moonraker.dynamicHeaters,
    getDynamicFans: (state) => state.moonraker.dynamicFans,
    getRawMoonrakerObjects: (state) => state.moonraker.rawObjects,

    getFiles: (state) => state.files.items,
    getFilesState: (state) => state.files,
    getHistory: (state) => state.moonraker.history,
  },

  actions: {
    setDarkmode(value: boolean) {
      this.darkmode = value
    },

    setZoom(value: number) {
      this.zoom = value
    },

    setDebug(value: boolean) {
      this.debug = value
    },

    setLanguage(value: string | null) {
      this.language = value
    },

    setWebsocketIp(ip: string) {
      this.websocket.ip = ip
    },

    setWebsocketConnected(value: boolean) {
      this.websocket.connected = value
    },

    setMoonrakerReady(value: boolean) {
      this.moonrakerReady = value
    },

    setShortcutButtons(value: ShortcutButtonConfig[]) {
      this.shortcutButtons = Array.isArray(value) ? value : []
    },

    applyConfig(config: AppConfig) {
      if (config.styling) {
        if (typeof config.styling.darkmode === 'boolean') {
          this.setDarkmode(config.styling.darkmode)
        }

        if (typeof config.styling.zoom === 'number') {
          this.setZoom(config.styling.zoom)
        }

        if (typeof config.styling.primary === 'string' && config.styling.primary.trim()) {
          this.primaryColor = config.styling.primary.trim()
        } else {
          this.primaryColor = null
        }

        if (typeof config.styling.secondary === 'string' && config.styling.secondary.trim()) {
          this.secondaryColor = config.styling.secondary.trim()
        } else {
          this.secondaryColor = null
        }
      }

      if (config.websocket?.ip) {
        this.setWebsocketIp(config.websocket.ip)
      }

      if (typeof config.dev?.debug === 'boolean') {
        this.setDebug(config.dev.debug)
      }

      if (typeof config.system?.language === 'string') {
        this.setLanguage(config.system.language)
      } else if (config.system?.language === null) {
        this.setLanguage(null)
      }

      if (config.shortcutbuttons && Array.isArray(config.shortcutbuttons)) {
        this.setShortcutButtons(
            config.shortcutbuttons.filter(
                (item): item is ShortcutButtonConfig =>
                    Boolean(item && typeof item.name === 'string' && item.name.trim() && typeof item.macro_inactive === 'string' && typeof item.icon === 'string'),
            ),
        )
      } else if (typeof config === 'object' && config !== null) {
        this.setShortcutButtons(parseShortcutButtonsFromConfig(config as Record<string, unknown>))
      } else {
        this.setShortcutButtons([])
      }
    },

    async loadConfig() {
      if (!isTauriRuntime()) return null
      const config = await invoke<AppConfig>('get_config')
      this.applyConfig(config)
      return config
    },

    async reloadConfigFromFile(path: string) {
      if (!isTauriRuntime()) return null
      const config = await invoke<AppConfig>('load_config_file', {
        configPath: path,
      })
      this.applyConfig(config)
      return config
    },

    async saveEditableConfig(payload: {
      styling?: {
        darkmode?: boolean
        primary?: string | null
        secondary?: string | null
      }
      system?: {
        language?: string | null
      }
    }) {
      if (!isTauriRuntime()) return null

      const config = await invoke<AppConfig>('save_editable_config', {
        editableConfig: payload,
      })

      this.applyConfig(config)
      return config
    },

    async startConfigListener() {
      if (this.configListener || !isTauriRuntime()) return

      this.configListener = await listen<AppConfig>('config-loaded', (event) => {
        this.applyConfig(event.payload)
      })
    },

    stopConfigListener() {
      if (this.configListener) {
        this.configListener()
        this.configListener = null
      }
    },

    applyMoonrakerAfcUpdate(status: Record<string, any>) {
      let foundAfc = false

      for (const [key, value] of Object.entries(status)) {
        const lower = key.toLowerCase()

        if (
            lower === 'afc' ||
            lower.startsWith('afc ') ||
            lower.startsWith('afc_') ||
            lower.includes(' afc ') ||
            lower.includes('filament_switch_sensor afc')
        ) {
          const existing = this.moonraker.afc.objects[key]

          if (
              existing &&
              typeof existing === 'object' &&
              !Array.isArray(existing) &&
              value &&
              typeof value === 'object' &&
              !Array.isArray(value)
          ) {
            this.moonraker.afc.objects[key] = {
              ...(existing as Record<string, unknown>),
              ...(value as Record<string, unknown>),
            }
          } else {
            this.moonraker.afc.objects[key] = value
          }

          foundAfc = true
        }
      }

      if (foundAfc) {
        this.moonraker.afc.available = true
      }
    },

    applyRawMoonrakerObjects(status: Record<string, any>) {
      for (const [key, value] of Object.entries(status)) {
        const existing = this.moonraker.rawObjects[key]

        if (
            existing &&
            typeof existing === 'object' &&
            !Array.isArray(existing) &&
            value &&
            typeof value === 'object' &&
            !Array.isArray(value)
        ) {
          this.moonraker.rawObjects[key] = {
            ...(existing as Record<string, unknown>),
            ...(value as Record<string, unknown>),
          }
        } else {
          this.moonraker.rawObjects[key] = value
        }
      }
    },

    applyDynamicMoonrakerDevices() {
      const heaterMap = new Map<string, MoonrakerDynamicHeater>()
      const fanMap = new Map<string, MoonrakerDynamicFan>()

      for (const [key, value] of Object.entries(this.moonraker.rawObjects)) {
        if (!value || typeof value !== 'object' || Array.isArray(value)) continue

        const lower = key.toLowerCase()
        const record = value as Record<string, unknown>

        if (lower === 'extruder' || lower === 'heater_bed') continue
        if (lower.startsWith('afc')) continue

        const isTemperatureFan = lower.startsWith('temperature_fan ')
        const isFan =
            lower === 'fan' ||
            lower.startsWith('fan_generic ') ||
            lower.startsWith('controller_fan ') ||
            lower.startsWith('heater_fan ') ||
            isTemperatureFan

        if (isFan) {
          fanMap.set(key, {
            key,
            label: prettifyMoonrakerObjectName(key),
            speed: asNumber(record.speed),
            rpm: asNumber(record.rpm),
            temperature: asNumber(record.temperature),
            target: asNumber(record.target),
            isTemperatureFan,
          })
          continue
        }

        const isHeater =
            lower.startsWith('heater_generic ') ||
            ('temperature' in record && 'target' in record)

        if (isHeater) {
          heaterMap.set(key, {
            key,
            label: prettifyMoonrakerObjectName(key),
            temperature: asNumber(record.temperature),
            target: asNumber(record.target),
            power: asNumber(record.power),
          })
        }
      }

      this.moonraker.dynamicHeaters = Array.from(heaterMap.values())
      this.moonraker.dynamicFans = Array.from(fanMap.values())
    },

    applyMoonrakerStatusUpdate(status: Record<string, any>) {
      this.applyRawMoonrakerObjects(status)

      if (status.webhooks) {
        if ('state' in status.webhooks) {
          this.moonraker.webhooks.state = asString(status.webhooks.state)
        }

        if ('state_message' in status.webhooks) {
          this.moonraker.webhooks.stateMessage = asString(status.webhooks.state_message)
        }
      }

      if (status.extruder) {
        if ('temperature' in status.extruder) {
          this.moonraker.extruder.temperature = asNumber(status.extruder.temperature)
        }
        if ('target' in status.extruder) {
          this.moonraker.extruder.target = asNumber(status.extruder.target)
        }
        if ('power' in status.extruder) {
          this.moonraker.extruder.power = asNumber(status.extruder.power)
        }
      }

      if (status.heater_bed || status.heaterBed) {
        const bed = status.heater_bed ?? status.heaterBed

        if ('temperature' in bed) {
          this.moonraker.heaterBed.temperature = asNumber(bed.temperature)
        }
        if ('target' in bed) {
          this.moonraker.heaterBed.target = asNumber(bed.target)
        }
        if ('power' in bed) {
          this.moonraker.heaterBed.power = asNumber(bed.power)
        }
      }

      if (status.toolhead?.position && Array.isArray(status.toolhead.position)) {
        this.moonraker.toolhead.position = status.toolhead.position
      }

      if (status.gcode_move || status.gcodeMove) {
        const gcodeMove = status.gcode_move ?? status.gcodeMove

        if ('speed' in gcodeMove) {
          this.moonraker.gcodeMove.speed = asNumber(gcodeMove.speed)
        }
        if ('speed_factor' in gcodeMove) {
          this.moonraker.gcodeMove.speedFactor = asNumber(gcodeMove.speed_factor)
        } else if ('speedFactor' in gcodeMove) {
          this.moonraker.gcodeMove.speedFactor = asNumber(gcodeMove.speedFactor)
        }
      }

      if (status.print_stats || status.printStats) {
        const printStats = status.print_stats ?? status.printStats

        if ('state' in printStats) {
          this.moonraker.printStats.state = asString(printStats.state)
        }
        if ('filename' in printStats) {
          this.moonraker.printStats.filename = asString(printStats.filename)
        }
        if ('message' in printStats) {
          this.moonraker.printStats.message = asString(printStats.message)
        }
        if ('print_duration' in printStats) {
          this.moonraker.printStats.printDuration = asNumber(printStats.print_duration)
        } else if ('printDuration' in printStats) {
          this.moonraker.printStats.printDuration = asNumber(printStats.printDuration)
        }
        if ('total_duration' in printStats) {
          this.moonraker.printStats.totalDuration = asNumber(printStats.total_duration)
        } else if ('totalDuration' in printStats) {
          this.moonraker.printStats.totalDuration = asNumber(printStats.totalDuration)
        }
        if ('filament_used' in printStats) {
          this.moonraker.printStats.filamentUsed = asNumber(printStats.filament_used)
        } else if ('filamentUsed' in printStats) {
          this.moonraker.printStats.filamentUsed = asNumber(printStats.filamentUsed)
        }
        if ('info' in printStats && printStats.info && typeof printStats.info === 'object') {
          this.moonraker.printStats.info = printStats.info
        }
      }

      if (status.virtual_sdcard || status.virtualSdcard) {
        const virtualSdcard = status.virtual_sdcard ?? status.virtualSdcard

        if ('progress' in virtualSdcard) {
          this.moonraker.virtualSdcard.progress = asNumber(virtualSdcard.progress)
        }
        if ('file_position' in virtualSdcard) {
          this.moonraker.virtualSdcard.filePosition = asNumber(virtualSdcard.file_position)
        } else if ('filePosition' in virtualSdcard) {
          this.moonraker.virtualSdcard.filePosition = asNumber(virtualSdcard.filePosition)
        }
        if ('is_active' in virtualSdcard) {
          this.moonraker.virtualSdcard.isActive = asBoolean(virtualSdcard.is_active)
        } else if ('isActive' in virtualSdcard) {
          this.moonraker.virtualSdcard.isActive = asBoolean(virtualSdcard.isActive)
        }
      }

      if (status.display_status || status.displayStatus) {
        const displayStatus = status.display_status ?? status.displayStatus

        if ('message' in displayStatus) {
          this.moonraker.displayStatus.message = asString(displayStatus.message)
        }
        if ('progress' in displayStatus) {
          this.moonraker.displayStatus.progress = asNumber(displayStatus.progress)
        }
      }

      this.applyMoonrakerAfcUpdate(status)
      this.applyDynamicMoonrakerDevices()
    },

    applyMoonrakerSubscriptionPayload(payload: any) {
      const status = payload?.status ?? payload?.result?.status ?? payload
      if (status && typeof status === 'object') {
        this.applyMoonrakerStatusUpdate(status)
      }
    },

    applyMoonrakerProcStats(payload: any) {
      if (!payload || typeof payload !== 'object') return

      if (
          'moonraker_stats' in payload &&
          payload.moonraker_stats &&
          typeof payload.moonraker_stats === 'object'
      ) {
        const stats = payload.moonraker_stats as Record<string, unknown>

        if ('cpu_usage' in stats) {
          this.moonraker.procStats.moonrakerCpuUsage = asNumber(stats.cpu_usage)
        }
        if ('memory' in stats) {
          this.moonraker.procStats.memory = asNumber(stats.memory)
        }
      }

      if ('cpu_temp' in payload) {
        this.moonraker.procStats.cpuTemp = asNumber(payload.cpu_temp)
      }

      if ('system_cpu_usage' in payload) {
        this.moonraker.procStats.systemCpuUsage = asNumber(payload.system_cpu_usage)
      }

      if ('system_uptime' in payload) {
        this.moonraker.procStats.systemUptime = asNumber(payload.system_uptime)
      }

      if ('network' in payload && payload.network && typeof payload.network === 'object') {
        this.moonraker.procStats.network = payload.network
      }
    },

    applyMoonrakerHistoryUpdate(payload: unknown) {
      this.moonraker.history = {
        lastAction: 'history_update',
        payload,
      }
    },

    applyMoonrakerThrottle(payload: any) {
      if (!payload || typeof payload !== 'object') return

      if ('bits' in payload) {
        this.moonraker.throttle.bits = asNumber(payload.bits)
      }

      if ('flags' in payload && Array.isArray(payload.flags)) {
        this.moonraker.throttle.flags = payload.flags.filter(
            (x: unknown): x is string => typeof x === 'string',
        )
      }
    },

    setFiles(payload: unknown) {
      if (Array.isArray(payload)) {
        this.files.items = payload
      } else if (payload && typeof payload === 'object' && Array.isArray((payload as any).files)) {
        this.files.items = (payload as any).files
      } else {
        this.files.items = []
      }

      this.files.lastUpdated = Date.now()
    },

    resetConnectionState() {
      this.setWebsocketConnected(false)
      this.setMoonrakerReady(false)
    },

    resetMoonrakerData() {
      this.moonraker.webhooks.state = null
      this.moonraker.webhooks.stateMessage = null

      this.moonraker.extruder = {
        temperature: null,
        target: null,
        power: null,
      }

      this.moonraker.heaterBed = {
        temperature: null,
        target: null,
        power: null,
      }

      this.moonraker.toolhead.position = []

      this.moonraker.gcodeMove = {
        speed: null,
        speedFactor: null,
      }

      this.moonraker.printStats = {
        state: null,
        filename: null,
        message: null,
        printDuration: null,
        totalDuration: null,
        filamentUsed: null,
        info: {},
      }

      this.moonraker.virtualSdcard = {
        progress: null,
        filePosition: null,
        isActive: null,
      }

      this.moonraker.procStats = {
        moonrakerCpuUsage: null,
        systemCpuUsage: null,
        cpuTemp: null,
        memory: null,
        network: {},
        systemUptime: null,
      }

      this.moonraker.throttle = {
        bits: null,
        flags: [],
      }

      this.moonraker.displayStatus = {
        message: null,
        progress: null,
      }

      this.moonraker.history = {
        lastAction: null,
        payload: null,
      }

      this.moonraker.afc = {
        available: false,
        objects: {},
      }

      this.moonraker.dynamicHeaters = []
      this.moonraker.dynamicFans = []
      this.moonraker.rawObjects = {}
    },

    resetFiles() {
      this.files = {
        items: [],
        lastUpdated: null,
      }
    },

    resetToDefaults() {
      this.setDarkmode(true)
      this.setZoom(1.0)
      this.setDebug(false)
      this.setLanguage(null)
      this.primaryColor = null
      this.secondaryColor = null
      this.setShortcutButtons([])
      this.setWebsocketIp('127.0.0.1:7125')
      this.resetConnectionState()
      this.resetMoonrakerData()
      this.resetFiles()
    },
  },
})