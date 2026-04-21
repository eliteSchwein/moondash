import { defineConfig } from "vite";
import Layouts from 'vite-plugin-vue-layouts-next'
import Vuetify, { transformAssetUrls } from 'vite-plugin-vuetify'
import Vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import VueRouter from 'vue-router/vite'
import { fileURLToPath, URL } from 'node:url'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
      VueRouter({
          dts: 'src/typed-router.d.ts',
      }),
      Layouts(),
      AutoImport({
          imports: [
              'vue',
              {
                  'vue-router/auto': ['useRoute', 'useRouter'],
              }
          ],
          dts: 'src/auto-imports.d.ts',
          eslintrc: {
              enabled: true,
          },
          vueTemplate: true,
      }),
      Components({
          dts: 'src/components.d.ts',
      }),
      Vue({
          template: { transformAssetUrls },
      }),
      Vuetify({
          autoImport: true,
          styles: {
              configFile: 'src/styles/settings.scss',
          },
      }),
  ],
    resolve: {
        alias: {
            '@': fileURLToPath(new URL('./src', import.meta.url)),
        },
    },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
