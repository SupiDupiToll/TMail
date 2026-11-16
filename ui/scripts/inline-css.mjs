import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const build = join(root, 'build');
const assets = join(build, '_app', 'immutable', 'assets');
const indexPath = join(build, 'index.html');
let html = readFileSync(indexPath, 'utf8');

// WebKitGTK can fail to load the extracted stylesheet from the custom asset
// protocol in production, and runtime-injected component styles are blocked by
// Tauri's CSP. Inline every extracted stylesheet into the HTML; the CSP allows
// inline styles, and all component CSS then lives in the same document.
const cssFiles = readdirSync(assets)
  .filter((name) => name.endsWith('.css'))
  .sort();

let css = '';
for (const name of cssFiles) {
  const href = `/_app/immutable/assets/${name}`;
  const source = readFileSync(join(assets, name), 'utf8');
  css += source.replace(/url\(\s*(['"]?)([^'")]+)\1\s*\)/g, (all, quote, raw) => {
    if (/^(?:data:|https?:|#|\/)/i.test(raw)) return all;
    const resolved = new URL(raw, `http://local${href}`).pathname;
    return `url("${resolved}")`;
  });
  css += '\n';
}

if (!css) {
  console.log('inline-css: no stylesheets found');
  process.exit(0);
}

html = html.replace(/<link\b[^>]*>/gi, (tag) => {
  const rel = /\brel=["']([^"']+)["']/i.exec(tag)?.[1];
  return rel?.split(/\s+/).includes('stylesheet') ? '' : tag;
});

html = html.replace('</head>', `<style data-tmail-global>${css}</style>\n</head>`);
writeFileSync(indexPath, html, 'utf8');

const remainingStylesheet = [...html.matchAll(/<link\b[^>]*>/gi)].some(([tag]) => {
  const rel = /\brel=["']([^"']+)["']/i.exec(tag)?.[1];
  return rel?.split(/\s+/).includes('stylesheet') ?? false;
});
if (remainingStylesheet) {
  throw new Error('inline-css: external stylesheet links remain');
}

console.log(`inline-css: inlined ${cssFiles.length} stylesheet(s), ${css.length} bytes`);
