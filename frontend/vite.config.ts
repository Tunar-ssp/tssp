import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  base: "/app-v2/",
  server: {
    host: "127.0.0.1",
    port: 5173,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8421",
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: "../crates/tsspd/assets/web-v2",
    emptyOutDir: true,
    sourcemap: true,
  },
});
