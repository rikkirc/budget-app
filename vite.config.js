import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Set by `tauri android dev` / `tauri ios dev` so a real phone on your Wi-Fi can load the dev server
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
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
    target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // Firebase makes the bundle large, but it ships inside the app rather than being downloaded, so that's fine
    chunkSizeWarningLimit: 1500,
  },
});
