import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri drives this dev server, so the port is fixed and failures must be
// loud rather than silently moving to another port.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    // Rust sources have their own watcher; Vite reloading on them is noise.
    watch: { ignored: ['**/src-tauri/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    // macOS Tauri renders in WKWebView; this is the target Tauri documents.
    target: 'safari15',
    minify: process.env.TAURI_ENV_DEBUG ? false : 'esbuild',
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
  },
});
