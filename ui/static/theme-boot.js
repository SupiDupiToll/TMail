// Apply the saved/system theme before first paint. Kept as an external file so
// the Tauri CSP can stay `script-src 'self'` with no inline-script exception.
(() => {
  try {
    const saved = localStorage.getItem('tmail.theme');
    const system = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    const theme = saved === 'light' || saved === 'dark' ? saved : system;
    document.documentElement.dataset.theme = theme;
    document.documentElement.dataset.themePreference = saved || 'system';
    const density = localStorage.getItem('tmail.density');
    if (density) document.documentElement.dataset.density = density;
  } catch (_) {
    /* localStorage can be disabled; defaults still work */
  }
})();
