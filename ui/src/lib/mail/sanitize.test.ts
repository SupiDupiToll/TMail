import { describe, expect, it } from 'vitest';
import { sanitizeMailHtml } from './sanitize';
import { BOOTSTRAP_HASH, BOOTSTRAP_SCRIPT } from './bootstrap';
import { createHash } from 'node:crypto';

const options = { allowRemote: false, sender: 'ada@example.com' };

describe('sanitizeMailHtml', () => {
  it('removes scripts and event handlers but keeps safe presentation CSS', () => {
    const { srcdoc } = sanitizeMailHtml(
      '<p style="color:#9b2f29;background:url(https://evil.example/p)">Hi</p><script>window.x=1</script><img src="x" onerror="alert(1)">',
      options
    );
    expect(srcdoc).not.toContain('<script>window.x');
    expect(srcdoc).not.toContain('onerror');
    expect(srcdoc).toContain('color:#9b2f29');
    expect(srcdoc).not.toContain('evil.example');
    expect(srcdoc).not.toContain('url(');
    expect(srcdoc).toContain('Hi');
  });

  it('keeps layout CSS from style blocks while stripping resource loads', () => {
    const { srcdoc } = sanitizeMailHtml(
      '<style>@import url("https://evil.example/x.css"); .hero{color:#1d6b5f;background:url(https://evil.example/p.gif);position:fixed} @media (min-width:600px){.hero{padding:12px}}</style><div class="hero">Hi</div>',
      options
    );
    expect(srcdoc).toContain('.hero');
    expect(srcdoc).toContain('color:#1d6b5f');
    expect(srcdoc).toContain('@media');
    expect(srcdoc).not.toContain('@import');
    expect(srcdoc).not.toContain('evil.example');
    expect(srcdoc).not.toContain('fixed');
  });

  it('keeps the mail canvas light by dropping dark-mode overrides', () => {
    const { srcdoc } = sanitizeMailHtml(
      '<style>@media (prefers-color-scheme: dark){body{background:#1a211d;color:#f5f6f3}}</style><p>Hi</p>',
      options
    );
    expect(srcdoc).not.toContain('prefers-color-scheme: dark');
  });

  it('keeps legacy presentational markup', () => {
    const { srcdoc } = sanitizeMailHtml(
      '<center><font color="#1d6b5f" face="Arial" size="3">Hi</font></center>',
      options
    );
    expect(srcdoc).toContain('<center>');
    expect(srcdoc).toContain('color="#1d6b5f"');
    expect(srcdoc).toContain('face="Arial"');
  });

  it('blocks remote images and keeps the frame script-free', () => {
    const { srcdoc, sandbox } = sanitizeMailHtml(
      '<p><img src="https://tracker.example/pixel.gif" alt="pixel"></p>',
      options
    );
    expect(srcdoc).not.toContain('tracker.example');
    expect(srcdoc).toContain('[remote image blocked');
    expect(srcdoc).toContain("script-src 'sha256-");
    expect(srcdoc).toContain("default-src 'none'");
    expect(sandbox).toBe('allow-scripts');
  });

  it('allows remote images only after the per-sender opt-in', () => {
    const { srcdoc, sandbox } = sanitizeMailHtml(
      '<img src="https://images.example/logo.png" alt="logo">',
      { allowRemote: true, sender: 'ada@example.com' }
    );
    expect(srcdoc).toContain('https://images.example/logo.png');
    expect(srcdoc).toContain('img-src data: https:');
    expect(sandbox).toContain('allow-scripts');
    expect(sandbox).not.toContain('allow-same-origin');
  });

  it('rewrites links so the frame cannot navigate', () => {
    const { srcdoc } = sanitizeMailHtml(
      '<a href="https://example.com/x" target="_blank">go</a><a href="javascript:alert(1)">bad</a>',
      options
    );
    expect(srcdoc).toContain('data-href="https://example.com/x"');
    expect(srcdoc).toContain('role="link"');
    expect(srcdoc).not.toContain('target="_blank"');
    expect(srcdoc).not.toContain('javascript:');
  });

  it('exposes links for the parent link strip', () => {
    const { links } = sanitizeMailHtml(
      '<a href="https://example.com/x">Example</a><a href="https://example.com/x">Again</a>',
      options
    );
    expect(links).toEqual([{ href: 'https://example.com/x', label: 'Example' }]);
  });

  it('pins the bootstrap script to its CSP hash', () => {
    const digest = createHash('sha256').update(BOOTSTRAP_SCRIPT).digest('base64');
    expect(BOOTSTRAP_HASH).toBe(`sha256-${digest}`);
  });

  it('keeps data: images and table markup', () => {
    const { srcdoc } = sanitizeMailHtml(
      '<table><tr><td><img src="data:image/png;base64,AAAA" alt=""></td></tr></table>',
      options
    );
    expect(srcdoc).toContain('<table>');
    expect(srcdoc).toContain('data:image/png;base64,AAAA');
  });
});
