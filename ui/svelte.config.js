import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
export default {
  compilerOptions: {
    // Dev uses injected component styles to avoid the fragile virtual CSS
    // modules on Vite startup. Production must extract them into the global
    // stylesheet: runtime-injected <style> tags are blocked by Tauri's
    // production CSP, which leaves component layouts unstyled.
    css: process.env.NODE_ENV === 'production' ? 'external' : 'injected'
  },
  kit: {
    adapter: adapter({ fallback: 'index.html', precompress: false, strict: true }),
    alias: {
      $lib: 'src/lib'
    }
  }
};
