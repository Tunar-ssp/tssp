import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default {
  preprocess: vitePreprocess(),
  vitePlugin: {
    experimental: {
      inspector: {
        holdMode: true,
      },
    },
  },
}
