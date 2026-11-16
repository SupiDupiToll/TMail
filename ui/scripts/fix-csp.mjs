import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const indexPath = join(root, 'build', 'index.html');
let html = readFileSync(indexPath, 'utf8');

// SvelteKit's static fallback inlines the hydration bootstrap. The production
// CSP intentionally has no `unsafe-inline` for scripts, and its hash changes
// with every build. Move executable inline scripts to real files so
// `script-src 'self'` can load them on every build.
const pattern = /<script(?![^>]*\bsrc=)([^>]*)>([\s\S]*?)<\/script>/gi;
const inlineScripts = [...html.matchAll(pattern)];
let externalized = 0;

for (const match of inlineScripts) {
  const [whole, attributes, body] = match;
  if (!body.trim()) continue;
  if (/type=["'](?:application\/json|application\/ld\+json)["']/i.test(attributes)) continue;
  if (/type=["']module["']/i.test(attributes) && body.includes('document.currentScript')) {
    throw new Error(
      'Cannot externalize an inline module script that uses document.currentScript'
    );
  }

  const digest = createHash('sha256').update(body).digest('hex').slice(0, 12);
  const filename = `bootstrap.${digest}.js`;
  writeFileSync(join(root, 'build', filename), body, 'utf8');
  html = html.replace(whole, `<script${attributes} src="/${filename}"></script>`);
  externalized += 1;
}

if (externalized === 0) {
  console.log('fix-csp: no inline scripts found');
  process.exit(0);
}

writeFileSync(indexPath, html, 'utf8');

const remaining = [...html.matchAll(pattern)].filter(
  ([, attributes, body]) =>
    body.trim() &&
    !/type=["'](?:application\/json|application\/ld\+json)["']/i.test(attributes)
);
if (remaining.length > 0) {
  throw new Error(`fix-csp: ${remaining.length} executable inline script(s) remain`);
}

console.log(`fix-csp: externalized ${externalized} inline script(s)`);
