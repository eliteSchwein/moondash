import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

type JsonRpcId = number

type JsonRpcRequest = {
    jsonrpc: '2.0'
    method: string
    params?: Record<string, unknown>
    id?: JsonRpcId
}

type JsonRpcResponse<T = unknown> =
    | {
    jsonrpc: '2.0'
    id: JsonRpcId
    result: T
}
    | {
    jsonrpc: '2.0'
    id?: JsonRpcId
    error: {
        code: number
        message: string
    }
}

type JsonRpcNotification = {
    jsonrpc?: '2.0'
    method: string
    params?: unknown[]
}

type MoonrakerConfig = {
    ip: string
}

type MoonrakerStatus = {
    connected: boolean
    ready: boolean
    url: string | null
}

type NotificationHandler = (params?: unknown[]) => void
type ConnectionHandler = (status: MoonrakerStatus) => void
type ErrorHandler = (error: unknown) => void

type PendingRequest = {
    resolve: (value: unknown) => void
    reject: (reason?: unknown) => void
    timeout: ReturnType<typeof setTimeout>
}

class MoonrakerConnection {
    private ws: WebSocket | null = null
    private currentConfig: MoonrakerConfig | null = null
    private currentUrl: string | null = null
    private requestId = 1
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null
    private manuallyDisconnected = false
    private ready = false

    private pending = new Map<JsonRpcId, PendingRequest>()
    private notificationHandlers = new Map<string, Set<NotificationHandler>>()
    private connectionHandlers = new Set<ConnectionHandler>()
    private errorHandlers = new Set<ErrorHandler>()
    private configListener: UnlistenFn | null = null

    private subscriptionObjects: Record<string, string[] | null> = {
        webhooks: ['state', 'state_message'],
        print_stats: null,
        virtual_sdcard: null,
        toolhead: ['position'],
        gcode_move: ['speed', 'speed_factor'],
        extruder: ['temperature', 'target', 'power'],
        heater_bed: ['temperature', 'target', 'power'],
    }

    getStatus(): MoonrakerStatus {
        return {
            connected: this.ws?.readyState === WebSocket.OPEN,
            ready: this.ready,
            url: this.currentUrl,
        }
    }

    onConnectionChange(handler: ConnectionHandler) {
        this.connectionHandlers.add(handler)
        handler(this.getStatus())
        return () => this.connectionHandlers.delete(handler)
    }

    onError(handler: ErrorHandler) {
        this.errorHandlers.add(handler)
        return () => this.errorHandlers.delete(handler)
    }

    onNotification(method: string, handler: NotificationHandler) {
        if (!this.notificationHandlers.has(method)) {
            this.notificationHandlers.set(method, new Set())
        }
        this.notificationHandlers.get(method)!.add(handler)
        return () => {
            this.notificationHandlers.get(method)?.delete(handler)
        }
    }

    registerDefaultNotifications() {
        return [
            this.onNotification('notify_status_update', (params) => {
                console.debug('[moonraker] notify_status_update', params)
            }),
            this.onNotification('notify_klippy_ready', (params) => {
                console.info('[moonraker] notify_klippy_ready', params)
                this.ready = true
                this.emitConnection()
            }),
            this.onNotification('notify_klippy_disconnected', (params) => {
                console.warn('[moonraker] notify_klippy_disconnected', params)
                this.ready = false
                this.emitConnection()
            }),
            this.onNotification('notify_klippy_shutdown', (params) => {
                console.warn('[moonraker] notify_klippy_shutdown', params)
                this.ready = false
                this.emitConnection()
            }),
            this.onNotification('notify_history_changed', (params) => {
                console.debug('[moonraker] notify_history_changed', params)
            }),
            this.onNotification('notify_webcams_changed', (params) => {
                console.debug('[moonraker] notify_webcams_changed', params)
            }),
            this.onNotification('notify_proc_stat_update', (params) => {
                console.debug('[moonraker] notify_proc_stat_update', params)
            }),
        ]
    }

    async connect(config: MoonrakerConfig): Promise<void> {
        this.currentConfig = config
        this.manuallyDisconnected = false
        this.clearReconnectTimer()

        const url = this.makeWebSocketUrl(config.ip)
        this.currentUrl = url

        if (this.ws) {
            this.ws.close()
            this.ws = null
        }

        await new Promise<void>((resolve, reject) => {
            const ws = new WebSocket(url)
            this.ws = ws

            ws.onopen = async () => {
                try {
                    this.ready = false
                    this.emitConnection()

                    await this.initializeConnection()

                    this.emitConnection()
                    resolve()
                } catch (error) {
                    this.handleError(error)
                    reject(error)
                }
            }

            ws.onmessage = (event) => {
                this.handleMessage(event.data)
            }

            ws.onerror = (event) => {
                this.handleError(event)
            }

            ws.onclose = () => {
                this.ready = false
                this.emitConnection()
                this.rejectAllPending(new Error('Moonraker websocket closed'))

                if (!this.manuallyDisconnected) {
                    this.scheduleReconnect()
                }
            }
        })
    }

    async reconnect(): Promise<void> {
        if (!this.currentConfig) {
            const config = await this.loadConfigFromBackend()
            if (!config?.websocket?.ip) {
                throw new Error('No Moonraker websocket config available')
            }
            await this.connect({ ip: config.websocket.ip })
            return
        }

        await this.connect(this.currentConfig)
    }

    disconnect() {
        this.manuallyDisconnected = true
        this.ready = false
        this.clearReconnectTimer()

        if (this.ws) {
            this.ws.close()
            this.ws = null
        }

        this.emitConnection()
    }

    async startAutoConnectFromConfig() {
        const config = await this.loadConfigFromBackend()
        if (config?.websocket?.ip) {
            await this.connect({ ip: config.websocket.ip })
        }

        if (!this.configListener) {
            this.configListener = await listen('config-loaded', async (event) => {
                const payload = event.payload as { websocket?: { ip?: string } } | null
                const newIp = payload?.websocket?.ip

                if (!newIp) return
                if (this.currentConfig?.ip === newIp && this.getStatus().connected) return

                try {
                    await this.connect({ ip: newIp })
                } catch (error) {
                    this.handleError(error)
                }
            })
        }
    }

    stopAutoConnectFromConfig() {
        if (this.configListener) {
            this.configListener()
            this.configListener = null
        }
    }

    async call<T = unknown>(
        method: string,
        params?: Record<string, unknown>,
        timeoutMs = 10000,
    ): Promise<T> {
        if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
            throw new Error('Moonraker websocket is not connected')
        }

        const id = this.requestId++
        const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            id,
            ...(params ? { params } : {}),
        }

        return new Promise<T>((resolve, reject) => {
            const timeout = setTimeout(() => {
                this.pending.delete(id)
                reject(new Error(`Moonraker request timeout: ${method}`))
            }, timeoutMs)

            this.pending.set(id, {
                resolve,
                reject,
                timeout,
            })

            this.ws!.send(JSON.stringify(request))
        })
    }

    async fetchAvailablePrinterObjects(): Promise<string[]> {
        const result = await this.call<{ objects: string[] }>('printer.objects.list')
        return result.objects ?? []
    }

    async subscribeToPrinterObjects(
        objects?: Record<string, string[] | null>,
    ): Promise<unknown> {
        return this.call('printer.objects.subscribe', {
            objects: objects ?? this.subscriptionObjects,
        })
    }

    async serverInfo() {
        return this.call('server.info')
    }

    async printerInfo() {
        return this.call('printer.info')
    }

    async registerAllKnownObjects() {
        const result = await this.call<{ objects: string[] }>('printer.objects.list')

        const objects: Record<string, string[] | null> = {
            webhooks: ['state', 'state_message'],
            configfile: ['config', 'settings', 'warnings'],
        }

        for (const name of result.objects ?? []) {
            if (!(name in objects)) {
                objects[name] = null
            }
        }

        delete objects.webhooks
        return this.subscribeToPrinterObjects(objects)
    }

    private async initializeConnection() {
        const info = await this.serverInfo() as {
            klippy_state?: string
        }

        if (info.klippy_state === 'startup') {
            await this.waitForKlippyReady()
        } else if (info.klippy_state !== 'ready') {
            console.warn('[moonraker] klippy_state is not ready:', info.klippy_state)
        }

        await this.registerAllKnownObjects()
        this.ready = true
    }

    private async waitForKlippyReady(maxAttempts = 30, delayMs = 2000) {
        for (let i = 0; i < maxAttempts; i++) {
            const info = await this.serverInfo() as {
                klippy_state?: string
            }

            if (info.klippy_state === 'ready') {
                return
            }

            await new Promise((resolve) => setTimeout(resolve, delayMs))
        }

        throw new Error('Moonraker/Klippy did not become ready in time')
    }

    private handleMessage(raw: string) {
        let message: JsonRpcResponse | JsonRpcNotification
        try {
            message = JSON.parse(raw)
        } catch (error) {
            this.handleError(error)
            return
        }

        if ('id' in message && typeof message.id === 'number') {
            const pending = this.pending.get(message.id)
            if (!pending) return

            clearTimeout(pending.timeout)
            this.pending.delete(message.id)

            if ('error' in message) {
                pending.reject(new Error(message.error.message))
            } else {
                pending.resolve(message.result)
            }
            return
        }

        if ('method' in message && typeof message.method === 'string') {
            const handlers = this.notificationHandlers.get(message.method)
            if (!handlers?.size) return

            for (const handler of handlers) {
                try {
                    handler(message.params)
                } catch (error) {
                    this.handleError(error)
                }
            }
        }
    }

    private makeWebSocketUrl(ip: string): string {
        const normalized = ip.trim()

        if (normalized.startsWith('ws://') || normalized.startsWith('wss://')) {
            return normalized.endsWith('/websocket') ? normalized : `${normalized}/websocket`
        }

        if (normalized.startsWith('http://')) {
            return normalized.replace(/^http:\/\//, 'ws://').replace(/\/?$/, '/websocket')
        }

        if (normalized.startsWith('https://')) {
            return normalized.replace(/^https:\/\//, 'wss://').replace(/\/?$/, '/websocket')
        }

        return `ws://${normalized}/websocket`
    }

    private scheduleReconnect() {
        if (this.reconnectTimer || !this.currentConfig) return

        this.reconnectTimer = setTimeout(async () => {
            this.reconnectTimer = null

            try {
                await this.reconnect()
            } catch (error) {
                this.handleError(error)
                this.scheduleReconnect()
            }
        }, 3000)
    }

    private clearReconnectTimer() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer)
            this.reconnectTimer = null
        }
    }

    private rejectAllPending(error: Error) {
        for (const [id, pending] of this.pending.entries()) {
            clearTimeout(pending.timeout)
            pending.reject(error)
            this.pending.delete(id)
        }
    }

    private emitConnection() {
        const status = this.getStatus()
        for (const handler of this.connectionHandlers) {
            handler(status)
        }
    }

    private handleError(error: unknown) {
        console.error('[moonraker]', error)
        for (const handler of this.errorHandlers) {
            handler(error)
        }
    }

    private async loadConfigFromBackend(): Promise<{ websocket?: { ip?: string } } | null> {
        try {
            return await invoke<{ websocket?: { ip?: string } }>('get_config')
        } catch (error) {
            this.handleError(error)
            return null
        }
    }
}

export const moonraker = new MoonrakerConnection()
export type { MoonrakerConfig, MoonrakerStatus }