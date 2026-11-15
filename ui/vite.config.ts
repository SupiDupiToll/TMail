import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    headers: { 'Cache-Control': 'no-store' },
    port: 5173,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
    warmup: {
      clientFiles: [
        './src/routes/+page.svelte',
        './src/routes/+layout.svelte',
        './src/lib/components/*.svelte'
      ]
    }
  },
  build: { target: 'es2022', minify: 'esbuild', sourcemap: false }
});
