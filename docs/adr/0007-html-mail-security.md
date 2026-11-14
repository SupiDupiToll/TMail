# ADR 0007 — Security model for rendered mail

- **Status:** accepted
- **Date:** 2026-11-14

## Decision

HTML mail is untrusted input. The renderer applies all of the following:

1. **Sanitisation in the renderer** with DOMPurify in a restricted profile:
   no script, no event handlers, no forms, no `object`/`embed`/`iframe`, no
   `meta`, no `base`, no `javascript:`/`data:` links. Only safe presentation
   tags and inline styles are kept.
2. **Script-free sandboxed iframe** with `sandbox="allow-same-origin"` and
   **without** `allow-scripts`. The document has no script engine and its CSP
   says `script-src 'none'`; same-origin access lets the trusted parent measure
   the body and intercept links, while `allow-scripts` is deliberately absent so
   a DOMPurify bypass still cannot execute. Links are rewritten to `data-href`
   (no navigable `href`) and activated by the parent.
3. **Strict CSP** in the iframe document:
   `default-src 'none'; script-src 'none'; img-src data:;
   style-src 'unsafe-inline'; font-src data:`. Remote images are **blocked by
   default**, which also blocks tracking pixels.
4. **Per-sender opt-in** for remote content. The allowlist is stored by sender
   address, never by message, and is revocable in settings. Loading remote
   content re-renders with `img-src https:` only; scripts stay forbidden.
5. **No automatic attachment preview** for active content. HTML/SVG
   attachments are never rendered inline; the user must save them.
6. **Link safety**: links open in the system browser only after an explicit
   click, and the full destination is shown in the status line. `mailto:` links
   are routed back into our composer.

## Why not sanitise in Rust

Sanitisers must track the HTML parser and CSS quirks of the actual renderer.
Doing it in the WebView, immediately before insertion, avoids a
“sanitised-for-a-different-parser” gap. The core still strips all remote
content by policy and never injects HTML into Tauri itself.

## Consequences

- Mail looks “broken” until the user opts in to images; this is intentional
  and explained in the reader banner.
- The UI test suite asserts the iframe sandbox, the CSP meta tag, absence of
  `script` nodes, link interception in the parent, and absence of remote image
  requests by default.
