import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";

// The web app offers the User's Manual for download (from its welcome screen), straight from docs/
const userManual = () => ({
  name: "vault-user-manual",
  apply: "build",
  generateBundle() {
    this.emitFile({ type: "asset", fileName: "The-Vault-User-Manual.pdf", source: readFileSync(new URL("./docs/Vault-User-Manual.pdf", import.meta.url)) });
  },
});

// Set by `tauri android dev` / `tauri ios dev` so a real phone on your Wi-Fi can load the dev server
const host = process.env.TAURI_DEV_HOST;

// `--mode web` builds the web app (PWA): the same screens, with vault-core running in the browser as WebAssembly
// instead of in the Rust backend of the desktop and Android apps
export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    mode === "web" && userManual(),
    // Makes the web app installable (home-screen icon, full screen) and lets it open offline. Only in the web build:
    // the desktop and Android apps never get a service worker.
    VitePWA({
      disable: mode !== "web",
      registerType: "prompt", // a new version waits until the user taps Reload (src/UpdateBanner.jsx)
      injectRegister: false,
      manifest: {
        name: "The Vault — by RCbreezy",
        short_name: "The Vault",
        description: "Plan every bill against the paycheck that pays it, synced across your family's devices.",
        start_url: "/",
        scope: "/",
        display: "standalone",
        background_color: "#f3f4f6",
        theme_color: "#1e3a8a",
        icons: [
          { src: "/icons/icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "/icons/icon-512.png", sizes: "512x512", type: "image/png" },
          { src: "/icons/maskable-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
        ],
      },
      workbox: {
        // Everything the app needs to open with no connection, including vault-core's WebAssembly (about 1.4 MB) and
        // the User's Manual (about 1 MB), so the manual downloads even offline
        globPatterns: ["**/*.{js,css,html,wasm,svg,png,pdf}"],
        maximumFileSizeToCacheInBytes: 5 * 1024 * 1024,
        navigateFallback: "/index.html",
        navigateFallbackDenylist: [/^\/__\//],
        cleanupOutdatedCaches: true,
      },
    }),
  ],
  resolve: {
    alias: {
      "@device-replica": fileURLToPath(new URL(mode === "web" ? "./src/sync/webReplica.js" : "./src/sync/tauriReplica.js", import.meta.url)),
    },
  },
  server: {
    // 🚨 NEW: We are telling it exactly which channel to broadcast on!
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/*.db", "**/*.db-journal", "**/src-tauri/**", "**/server/**", "**/vault-core/**"],
    },
  },
  clearScreen: false,
  // TAURI_ENV_PLATFORM tells the app whether it is the desktop or the phone build
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    // The web app supports iPhones from iOS 15 (2021) on
    target: mode === 'web' ? 'safari15' : process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // Firebase makes the bundle large, but it ships inside the app rather than being downloaded, so that's fine
    chunkSizeWarningLimit: 1500,
  },
}));
