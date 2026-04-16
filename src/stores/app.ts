import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

type StylingConfig = {
  zoom?: number
  darkmode?: boolean
}

type DevConfig = {
  debug?: boolean
}

type AppConfig = {
  websocket?: {
    ip?: string
  }
  styling?: StylingConfig
  dev?: DevConfig
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

type MoonrakerTimelapseState = {
  enabled: boolean | null
  active: boolean | null
  payload: unknown
}

type MoonrakerPowerDevice = {
  name: string
  status: string | null
  lockedWhilePrinting?: boolean
  type?: string
  payload?: unknown
}

type MoonrakerGcodeMove = {
  speed: number | null
  speedFactor: number | null
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

export const useAppStore = defineStore('app', {
  state: () => ({
    darkmode: true as boolean,
    zoom: 1.0 as number,
    debug: false as boolean,

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

      timelapse: {
        enabled: null,
        active: null,
        payload: null,
      } as MoonrakerTimelapseState,

      powerDevices: {} as Record<string, MoonrakerPowerDevice>,
    },

    configListener: null as UnlistenFn | null,
  }),

  getters: {
    isDarkmode: (state) => state.darkmode,
    getZoom: (state) => state.zoom,
    isDebugEnabled: (state) => state.debug,

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
    getSystemCpuUsage: (state) => state.moonraker.procStats.systemCpuUsage,

    getPowerDevices: (state) => state.moonraker.powerDevices,
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

    setWebsocketIp(ip: string) {
      this.websocket.ip = ip
    },

    setWebsocketConnected(value: boolean) {
      this.websocket.connected = value
    },

    setMoonrakerReady(value: boolean) {
      this.moonrakerReady = value
    },

    applyConfig(config: AppConfig) {
      if (config.styling) {
        if (typeof config.styling.darkmode === 'boolean') {
          this.setDarkmode(config.styling.darkmode)
        }

        if (typeof config.styling.zoom === 'number') {
          this.setZoom(config.styling.zoom)
        }
      }

      if (config.websocket?.ip) {
        this.setWebsocketIp(config.websocket.ip)
      }

      if (typeof config.dev?.debug === 'boolean') {
        this.setDebug(config.dev.debug)
      }
    },

    async loadConfig() {
      if (!isTauriRuntime()) {
        console.warn('Tauri runtime not detected, skipping loadConfig')
        return null
      }

      const config = await invoke<AppConfig>('get_config')
      this.applyConfig(config)
      return config
    },

    async reloadConfigFromFile(path: string) {
      if (!isTauriRuntime()) {
        console.warn('Tauri runtime not detected, skipping reloadConfigFromFile')
        return null
      }

      const config = await invoke<AppConfig>('load_config_file', {
        configPath: path,
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

    applyMoonrakerStatusUpdate(status: Record<string, any>) {
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

      if (status.heater_bed) {
        if ('temperature' in status.heater_bed) {
          this.moonraker.heaterBed.temperature = asNumber(status.heater_bed.temperature)
        }
        if ('target' in status.heater_bed) {
          this.moonraker.heaterBed.target = asNumber(status.heater_bed.target)
        }
        if ('power' in status.heater_bed) {
          this.moonraker.heaterBed.power = asNumber(status.heater_bed.power)
        }
      }

      if (status.toolhead?.position && Array.isArray(status.toolhead.position)) {
        this.moonraker.toolhead.position = status.toolhead.position
      }

      if (status.gcode_move) {
        if ('speed' in status.gcode_move) {
          this.moonraker.gcodeMove.speed = asNumber(status.gcode_move.speed)
        }
        if ('speed_factor' in status.gcode_move) {
          this.moonraker.gcodeMove.speedFactor = asNumber(status.gcode_move.speed_factor)
        }
      }

      if (status.print_stats) {
        if ('state' in status.print_stats) {
          this.moonraker.printStats.state = asString(status.print_stats.state)
        }
        if ('filename' in status.print_stats) {
          this.moonraker.printStats.filename = asString(status.print_stats.filename)
        }
        if ('message' in status.print_stats) {
          this.moonraker.printStats.message = asString(status.print_stats.message)
        }
        if ('print_duration' in status.print_stats) {
          this.moonraker.printStats.printDuration = asNumber(status.print_stats.print_duration)
        }
        if ('total_duration' in status.print_stats) {
          this.moonraker.printStats.totalDuration = asNumber(status.print_stats.total_duration)
        }
        if ('filament_used' in status.print_stats) {
          this.moonraker.printStats.filamentUsed = asNumber(status.print_stats.filament_used)
        }
        if (
            'info' in status.print_stats &&
            status.print_stats.info &&
            typeof status.print_stats.info === 'object'
        ) {
          this.moonraker.printStats.info = status.print_stats.info
        }
      }

      if (status.virtual_sdcard) {
        if ('progress' in status.virtual_sdcard) {
          this.moonraker.virtualSdcard.progress = asNumber(status.virtual_sdcard.progress)
        }
        if ('file_position' in status.virtual_sdcard) {
          this.moonraker.virtualSdcard.filePosition = asNumber(status.virtual_sdcard.file_position)
        }
        if ('is_active' in status.virtual_sdcard) {
          this.moonraker.virtualSdcard.isActive = asBoolean(status.virtual_sdcard.is_active)
        }
      }

      if (status.display_status) {
        if ('message' in status.display_status) {
          this.moonraker.displayStatus.message = asString(status.display_status.message)
        }
        if ('progress' in status.display_status) {
          this.moonraker.displayStatus.progress = asNumber(status.display_status.progress)
        }
      }
    },

    applyMoonrakerSubscriptionPayload(payload: any) {
      const status = payload?.status ?? payload
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

    applyMoonrakerTimelapse(payload: any) {
      this.moonraker.timelapse.payload = payload

      if (payload && typeof payload === 'object') {
        if ('enabled' in payload) {
          this.moonraker.timelapse.enabled = asBoolean(payload.enabled)
        }
        if ('is_running' in payload) {
          this.moonraker.timelapse.active = asBoolean(payload.is_running)
        }
      }
    },

    setPowerDevice(device: MoonrakerPowerDevice) {
      this.moonraker.powerDevices[device.name] = device
    },

    applyMoonrakerPowerDevices(payload: any) {
      if (!payload) return

      if (Array.isArray(payload)) {
        for (const entry of payload) {
          if (!entry || typeof entry !== 'object' || typeof entry.device !== 'string') continue

          this.setPowerDevice({
            name: entry.device,
            status: asString(entry.status),
            lockedWhilePrinting:
                typeof entry.locked_while_printing === 'boolean'
                    ? entry.locked_while_printing
                    : undefined,
            type: asString(entry.type) ?? undefined,
            payload: entry,
          })
        }
        return
      }

      if (typeof payload === 'object') {
        for (const [name, entry] of Object.entries(payload)) {
          if (!entry || typeof entry !== 'object') continue

          this.setPowerDevice({
            name,
            status: asString((entry as any).status),
            lockedWhilePrinting:
                typeof (entry as any).locked_while_printing === 'boolean'
                    ? (entry as any).locked_while_printing
                    : undefined,
            type: asString((entry as any).type) ?? undefined,
            payload: entry,
          })
        }
      }
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

      this.moonraker.timelapse = {
        enabled: null,
        active: null,
        payload: null,
      }

      this.moonraker.powerDevices = {}
    },

    resetToDefaults() {
      this.setDarkmode(true)
      this.setZoom(1.0)
      this.setDebug(false)
      this.setWebsocketIp('127.0.0.1:7125')
      this.resetConnectionState()
      this.resetMoonrakerData()
    },
  },
})