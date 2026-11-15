import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
export default {
  compilerOptions: {
    // Inject component styles into the component modules. This avoids the
    // fragile `?svelte&type=style` virtual CSS modules that occasionally fail
    // on Vite dev startup in the Tauri WebView.
    css: 'injected'
  },
  kit: {
    adapter: adapter({ fallback: 'index.html', precompress: false, strict: true }),
    alias: {
      $lib: 'src/lib'
    }
  }
};
