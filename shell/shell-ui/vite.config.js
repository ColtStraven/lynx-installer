import { defineConfig } from "vite";

export default defineConfig({
  // Vite dev server port must match tauri.conf.json devUrl
  server: {
    port: 1420,
    strictPort: true,
  },
  // Output to dist/ which tauri.conf.json points frontendDist at
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
  // Allow imports from @tauri-apps/* in dev
  optimizeDeps: {
    exclude: ["@tauri-apps/api"],
  },
});