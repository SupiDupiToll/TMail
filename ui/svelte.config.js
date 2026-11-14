import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
export default {
  kit: {
    adapter: adapter({ fallback: 'index.html', precompress: false, strict: true }),
    alias: {
      $lib: 'src/lib'
    }
  }
};
