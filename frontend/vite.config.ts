import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import path from 'path'

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: path.resolve('./src/lib'),
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8421',
        changeOrigin: true
      }
    }
  },
  build: {
    outDir: '../crates/tsspd/assets/web-v2',
    emptyOutDir: true,
    sourcemap: false
  }
})
