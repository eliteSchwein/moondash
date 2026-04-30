import { createApp } from "vue";
import App from "./App.vue";
import vuetify from './plugins/vuetify'
import router from '@/router'
import {createPinia} from "pinia";
import { i18n } from './plugins/i18n'
import { invoke } from "@tauri-apps/api/core";

type ConsoleLevel = "log" | "warn" | "error";

const formatConsoleArg = (arg: unknown): string => {
    if (arg instanceof Error) {
        return `${arg.name}: ${arg.message}${arg.stack ? `\n${arg.stack}` : ""}`;
    }

    if (typeof arg === "string") {
        return arg;
    }

    try {
        return JSON.stringify(arg);
    } catch {
        return String(arg);
    }
};

const installBackendConsoleLogging = () => {
    const originalConsole = {
        log: console.log.bind(console),
        warn: console.warn.bind(console),
        error: console.error.bind(console),
    };

    const forwardToBackend = (level: ConsoleLevel, args: unknown[]) => {
        const message = args.map(formatConsoleArg).join(" ");

        invoke("frontend_log", { level, message }).catch((err) => {
            originalConsole.error("failed to forward frontend log to backend", err);
        });
    };

    console.log = (...args: unknown[]) => {
        originalConsole.log(...args);
        forwardToBackend("log", args);
    };

    console.warn = (...args: unknown[]) => {
        originalConsole.warn(...args);
        forwardToBackend("warn", args);
    };

    console.error = (...args: unknown[]) => {
        originalConsole.error(...args);
        forwardToBackend("error", args);
    };
};

installBackendConsoleLogging();

const app = createApp(App)

app
    .use(createPinia())
    .use(vuetify)
    .use(router)
    .use(i18n)

app.mount("#app");
