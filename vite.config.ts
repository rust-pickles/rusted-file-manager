import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import {fileURLToPath} from 'url';
import {defineConfig} from 'vite';

// ESM-safe replacement for __dirname: Vite's native config loader (the default
// in a future major) cannot provide CommonJS globals.
const root = path.dirname(fileURLToPath(import.meta.url));

// The React sources live in ui/ (a git subtree mirroring rusted-file-manager-v2)
// while this config sits at the repository root, because the Tauri shell treats
// the repo root as the app root. Everything Vite needs therefore points into ui/.
export default defineConfig(() => {
  return {
    // ui/index.html is the entry document.
    root: path.resolve(root, 'ui'),
    // Assets are served relative to the document so the Tauri asset protocol can
    // load them from tauri://localhost without absolute-path lookups.
    base: './',
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        // The upstream config aliases "@" to the source root; after moving into
        // ui/ that root is ui/ itself. No upstream file uses it today, but the
        // mirror must keep working if the frontend starts to.
        '@': path.resolve(root, 'ui'),
      },
    },
    server: {
      // Dev server only; the packaged app loads from the custom protocol instead.
      port: 3000,
      // Bind all interfaces so a proxied preview (or a phone on the LAN) can
      // reach the dev server.
      host: '0.0.0.0',
      strictPort: true,
      // Vite blocks unknown Host headers by default, which rejects the proxied
      // preview domain. This only affects `vite dev`; the packaged app loads from
      // the custom protocol and never hits this list.
      allowedHosts: ['.e2b.app', 'localhost', '127.0.0.1'],
      // HMR stays disabled under DISABLE_HMR, as upstream documented.
      hmr: process.env.DISABLE_HMR !== 'true',
      watch: process.env.DISABLE_HMR === 'true' ? null : {},
    },
    build: {
      // tauri.conf.json frontendDist points here.
      outDir: path.resolve(root, 'dist'),
      emptyOutDir: true,
      // Windows WebView2 on older machines still runs Chromium < 111 targets, so
      // keep the output to a broadly supported baseline.
      target: 'chrome105',
      sourcemap: false,
    },
    clearScreen: false,
  };
});
