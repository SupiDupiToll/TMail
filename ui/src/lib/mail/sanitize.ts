import DOMPurify from 'dompurify';
import * as csstree from 'css-tree';
import { BOOTSTRAP_HASH, BOOTSTRAP_SCRIPT } from './bootstrap';

/**
 * Mail HTML sanitisation and the sandboxed document wrapper.
 *
 * The iframe is sandboxed `allow-same-origin` but **without**
 * `allow-scripts`: the document has no script engine, and its CSP says
 * `script-src 'none'`. The trusted parent can still measure the body and
 * intercept link clicks because the document is same-origin. DOMPurify strips
 * every active element first; links are rewritten to `data-href`, so even a
 * parent failure cannot navigate the frame.
 */

export interface SanitizeOptions {
  allowRemote: boolean;
  sender: string;
}

export interface SanitizedMail {
  srcdoc: string;
  /** Always the same; kept here so the component cannot drift. */
  sandbox: string;
  /** Links extracted from the message, usable even if frame access is denied. */
  links: { href: string; label: string }[];
}

const ALLOWED_TAGS = [
  'a', 'abbr', 'address', 'article', 'aside', 'b', 'bdi', 'bdo', 'big',
  'blockquote', 'br', 'caption', 'center', 'cite', 'code', 'col', 'colgroup',
  'dd', 'del', 'details', 'dfn', 'div', 'dl', 'dt', 'em', 'figcaption',
  'figure', 'font', 'footer', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header',
  'hr', 'i', 'img', 'ins', 'kbd', 'li', 'main', 'mark', 'nav', 'ol', 'p',
  'pre', 'q', 'rp', 'rt', 'ruby', 's', 'samp', 'section', 'small', 'span',
  'strike', 'strong', 'sub', 'summary', 'sup', 'table', 'tbody', 'td',
  'tfoot', 'th', 'thead', 'time', 'tr', 'u', 'ul', 'var', 'wbr'
];

const ALLOWED_ATTR = [
  'href', 'title', 'alt', 'src', 'width', 'height', 'colspan', 'rowspan',
  'align', 'valign', 'dir', 'lang', 'datetime', 'cite', 'style', 'class',
  'id', 'color', 'face', 'size', 'bgcolor', 'border', 'cellpadding',
  'cellspacing', 'nowrap'
];

const FORBID_TAGS = [
  'style', 'script', 'iframe', 'frame', 'frameset', 'object', 'embed', 'applet',
  'form', 'input', 'button', 'select', 'option', 'textarea', 'meta', 'link',
  'base', 'svg', 'math', 'video', 'audio', 'source', 'track', 'canvas',
  'template', 'portal', 'dialog', 'marquee', 'blink'
];

const FORBID_ATTR = [
  'srcset', 'background', 'poster', 'formaction', 'xlink:href',
  'action', 'method', 'ping', 'onerror', 'onload'
];

const URL_PATTERN = /^(?:https?:|mailto:|data:image\/(?:png|jpe?g|gif|webp);base64,)/i;

let configured = false;

function configure(): void {
  if (configured) return;
  configured = true;
  DOMPurify.addHook('uponSanitizeAttribute', (_node, data) => {
    if (data.attrName === 'href' && !URL_PATTERN.test(data.attrValue.trim())) {
      data.keepAttr = false;
    }
  });
}

export function sanitizeMailHtml(html: string, options: SanitizeOptions): SanitizedMail {
  configure();
  const parser = new DOMParser();

  // CSS is extracted before DOMPurify so we can keep the sender's layout while
  // removing resource loads, fixed positioning and other active CSS. DOMParser
  // documents are inert: no scripts run and no resources load.
  const rawDoc = parser.parseFromString(html || '', 'text/html');
  const emailCss = Array.from(rawDoc.querySelectorAll('style'))
    .map((style) => sanitizeCss(style.textContent ?? ''))
    .filter(Boolean)
    .join('\n');

  const clean = DOMPurify.sanitize(html || '<p></p>', {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    FORBID_TAGS,
    FORBID_ATTR,
    ALLOW_DATA_ATTR: false,
    ALLOW_ARIA_ATTR: false,
    KEEP_CONTENT: true,
    RETURN_TRUSTED_TYPE: false
  });

  const doc = parser.parseFromString(`<body>${clean}</body>`, 'text/html');
  const root = doc.body;
  if (!root) return wrap('', options, emailCss);

  // CSS in style attributes: keep presentation, drop resource loads/exfiltration.
  for (const element of root.querySelectorAll<HTMLElement>('[style]')) {
    const css = sanitizeCss(element.getAttribute('style') ?? '', true);
    if (css) element.setAttribute('style', css);
    else element.removeAttribute('style');
  }

  // Legacy color attributes: accept only plain colour values.
  for (const element of root.querySelectorAll<HTMLElement>('[color], [bgcolor]')) {
    for (const name of ['color', 'bgcolor']) {
      const value = element.getAttribute(name);
      if (value && !/^(#[0-9a-f]{3,8}|[a-z]{3,20}|rgb\([0-9 ,.%]+\))$/i.test(value.trim())) {
        element.removeAttribute(name);
      }
    }
  }

  const links = new Map<string, string>();
  // Links: keep the destination for our click handler, never let the frame
  // navigate. The trusted parent handles activation.
  for (const anchor of root.querySelectorAll('a')) {
    const href = anchor.getAttribute('href')?.trim() ?? '';
    if (!/^(https?:|mailto:)/i.test(href)) {
      anchor.removeAttribute('href');
      anchor.removeAttribute('role');
      anchor.removeAttribute('tabindex');
      continue;
    }
    anchor.setAttribute('data-href', href);
    anchor.setAttribute('role', 'link');
    anchor.setAttribute('tabindex', '0');
    // Keep a harmless href so WebKit treats it as a real link; the injected
    // bootstrap prevents the fragment navigation and forwards the URL.
    anchor.setAttribute('href', '#');
    anchor.removeAttribute('target');
    const label = (anchor.textContent ?? '').trim() || href;
    if (!links.has(href)) links.set(href, label.slice(0, 120));
  }

  // Images: remote pixels are removed unless the user allowed this sender.
  for (const img of root.querySelectorAll('img')) {
    const src = img.getAttribute('src')?.trim() ?? '';
    const isData = /^data:image\/(png|jpe?g|gif|webp);base64,/i.test(src);
    const isRemote = /^https:/i.test(src);
    if (isData) {
      img.setAttribute('loading', 'lazy');
      img.setAttribute('decoding', 'async');
      img.setAttribute('alt', img.getAttribute('alt') || '');
      continue;
    }
    if (isRemote && options.allowRemote) {
      img.setAttribute('loading', 'lazy');
      img.setAttribute('decoding', 'async');
      img.setAttribute('referrerpolicy', 'no-referrer');
      img.setAttribute('alt', img.getAttribute('alt') || '');
      continue;
    }
    // Replace blocked images with their alt text so the layout still reads.
    const alt = img.getAttribute('alt')?.trim();
    const replacement = doc.createElement('span');
    replacement.className = 'blocked-image';
    replacement.textContent = alt ? `[remote image blocked: ${alt}]` : '[remote image blocked]';
    img.replaceWith(replacement);
  }

  return wrap(root.innerHTML, options, emailCss, Array.from(links, ([href, label]) => ({ href, label })));
}

const BLOCKED_PROPERTIES = new Set([
  'behavior',
  '-moz-binding',
  'animation',
  'animation-name',
  'animation-duration',
  'animation-delay',
  'animation-iteration-count',
  'animation-direction',
  'animation-fill-mode',
  'animation-play-state',
  'transition',
  'transition-property',
  'transition-duration',
  'transition-delay',
  'transition-timing-function'
]);

const BLOCKED_AT_RULES = new Set([
  'import',
  'charset',
  'namespace',
  'font-face',
  'page',
  'document',
  'keyframes',
  '-webkit-keyframes',
  'counter-style',
  'font-feature-values'
]);

function sanitizeCss(css: string, declarationList = false): string {
  if (!css.trim() || css.length > 200_000) return '';
  try {
    const ast = csstree.parse(css, {
      context: declarationList ? 'declarationList' : 'stylesheet',
      positions: false,
      parseValue: false,
      parseCustomProperty: false
    });
    csstree.walk(ast, {
      visit: 'Atrule',
      enter(node, item, list) {
        const name = node.name.toLowerCase();
        if (list && BLOCKED_AT_RULES.has(name)) {
          list.remove(item);
          return;
        }
        if (name === 'media') {
          const prelude = node.prelude ? csstree.generate(node.prelude).toLowerCase() : '';
          if (prelude.includes('prefers-color-scheme: dark')) {
            if (list) list.remove(item);
            return;
          }
          return;
        }
        if (name === 'supports') return;
        if (list) list.remove(item);
      }
    });
    csstree.walk(ast, {
      visit: 'Declaration',
      enter(node, item, list) {
        const property = node.property.toLowerCase();
        if (BLOCKED_PROPERTIES.has(property)) {
          if (list) list.remove(item);
          return;
        }
        if (property === 'position') {
          const value = csstree.generate(node.value).toLowerCase();
          if (value.includes('fixed') || value.includes('sticky')) {
            node.value = csstree.parse('static', { context: 'value' }) as unknown as typeof node.value;
          }
        }
        const lowered = csstree.generate(node.value).toLowerCase();
        const urls = Array.from(lowered.matchAll(/url\(([^)]*)\)/g)).map((match) =>
          match[1].replace(/["']/g, '').trim()
        );
        const onlySafeDataUrls = urls.every((url) =>
          /^data:image\/(png|jpe?g|gif|webp);base64,/i.test(url)
        );
        if (
          !onlySafeDataUrls ||
          lowered.includes('expression(') ||
          lowered.includes('javascript:') ||
          lowered.includes('@import') ||
          lowered.includes('behavior:')
        ) {
          if (list) list.remove(item);
        }
      }
    });
    return csstree.generate(ast).replace(/<\/style/gi, '');
  } catch {
    return '';
  }
}

function wrap(
  body: string,
  options: SanitizeOptions,
  emailCss = '',
  links: { href: string; label: string }[] = []
): SanitizedMail {
  const imgSrc = options.allowRemote ? 'data: https:' : 'data:';
  const csp = [
    "default-src 'none'",
    `img-src ${imgSrc}`,
    "style-src 'unsafe-inline'",
    `script-src '${BOOTSTRAP_HASH}'`,
    "font-src data:",
    "connect-src 'none'",
    "media-src 'none'",
    "frame-src 'none'",
    "object-src 'none'",
    "form-action 'none'",
    "base-uri 'none'"
  ].join('; ');

  const sender = escapeHtml(options.sender);
  const srcdoc = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="referrer" content="no-referrer">
<title>Message from ${sender}</title>
<style>
  :root { color-scheme: light; }
  html, body { margin: 0; padding: 0; }
  body {
    font-family: 'Source Serif 4', Georgia, 'Times New Roman', serif;
    font-size: 16px;
    line-height: 1.62;
    color: #1a211d;
    background: #ffffff;
    padding: 4px 2px 24px;
    overflow-wrap: anywhere;
    font-variation-settings: 'opsz' 20, 'wght' 430;
  }
  p { margin: 0 0 1em; }
  h1, h2, h3, h4 { line-height: 1.25; margin: 1.4em 0 0.5em; font-family: 'Atkinson Hyperlegible Next', sans-serif; }
  h1 { font-size: 1.5rem; } h2 { font-size: 1.25rem; } h3 { font-size: 1.08rem; }
  a { color: #1d6b5f; text-decoration: underline; text-underline-offset: 2px; cursor: pointer; }
  blockquote {
    margin: 1em 0; padding: 0 0 0 1em;
    border-left: 2px solid #d5dbd4; color: #56625b;
  }
  pre, code, kbd { font-family: ui-monospace, monospace; font-size: 0.9em; }
  pre { overflow-x: auto; padding: 10px 12px; background: rgba(127,127,127,0.08); border-radius: 5px; }
  table { border-collapse: collapse; max-width: 100%; }
  td, th { padding: 4px 8px; vertical-align: top; }
  th { text-align: left; }
  img { max-width: 100%; height: auto; }
  hr { border: 0; border-top: 1px solid #d5dbd4; margin: 1.5em 0; }
  .blocked-image {
    display: inline-block; padding: 2px 8px;
    border: 1px dashed #b8c2b9; border-radius: 4px;
    font-family: 'Atkinson Hyperlegible Next', sans-serif;
    font-size: 12px; color: #56625b;
  }
</style>
${emailCss ? `<style>${emailCss}</style>` : ''}
</head>
<body>${body}
<script>${BOOTSTRAP_SCRIPT}</script>
</body>
</html>`;

  return {
    srcdoc,
    // The only script is the nonce-less, hash-pinned bootstrap above. The
    // frame has an opaque origin, so it cannot reach the app DOM or IPC.
    sandbox: 'allow-scripts',
    links
  };
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
