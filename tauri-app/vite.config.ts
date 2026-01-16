import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  // Prevent vite from obscuring rust errors
  clearScreen: false,
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // To access Tauri APIs, build output should be placed in `dist`
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
