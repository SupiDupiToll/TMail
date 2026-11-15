# Design critique & technical audit (internal)

_Date: 2026-11-14. Method: source review, Playwright DOM/accessibility snapshots,
computed-style and bounding-box measurements, WCAG contrast computed numerically,
keyboard traversal, axe-core, Impeccable detector. No images were opened._

## Audit health

| # | Dimension | Score | Key finding |
|---|---|---|---|
| 1 | Accessibility | 4/4 | axe: 0 serious/critical on login, inbox, reader, composer, settings; every sampled text ≥4.5:1; focus ring on every tab stop |
| 2 | Performance | 3/4 | Virtualised list and no remote images; measured on 240 synthetic rows, not yet on 20k real rows |
| 3 | Theming | 4/4 | Tokens for all colours/sizes/radii; detector 0; dark mode verified |
| 4 | Responsive design | 4/4 | 800/1280/1440/1920 + 1.5 dpr + 200% text: no horizontal overflow; drawer and overlay reader |
| 5 | Implementation integrity | 4/4 | Product-specific quiet-workspace vocabulary, no placeholder cards, no stock component library |
| **Total** | | **19/20** | Excellent with integration risk |

## Verdict

The interface expresses a coherent, product-specific system: the quiet
workspace, with deterministic sender avatars as its signature. A user who knows
mail clients can work immediately; a keyboard user never needs the mouse. The
weaknesses are product completeness, not visual drift: no live-account
verification, plain-text-only compose, no draft editing, and no password-
protected external mail.

## Priority findings

### [P1] Live protocol paths are unverified
- **Location:** `core/src/mail_ext.rs`, `core/src/send.rs`, `core/src/auth.rs`
- **Impact:** AEAD/body/attachment/send paths compile and are unit-shaped, but no
  dedicated Tuta account has been used. A wrong assumption here blocks the whole
  product.
- **Recovery:** run the smoke checklist in `docs/SMOKE.md` against the test
  account as soon as credentials exist.

### [P1] Desktop-only surfaces are not automated
- **Location:** `src-tauri/`
- **Impact:** tray, close-to-tray, notifications, native save dialog and
  `mailto:` cold start are compile-checked and start under Xvfb, but not
  exercised end to end.
- **Recovery:** add a `tauri-driver`/WebDriver job once the test account exists;
  keep the browser suite as the fast gate.

### [P2] Password-protected external mail is not available
- **Location:** `ui/src/lib/components/Composer.svelte`, `core/src/send.rs`
- **Impact:** external recipients always receive non-confidential mail; the
  composer says so plainly. The requested password mode needs Tuta's
  external-group creation flow and Bcrypt KDF, which the SDK does not expose.
- **Recovery:** implement only with a live test account; never fake the E2E
  state or silently downgrade.

### [P2] Sender logos are deliberately absent
- **Impact:** avatars are deterministic initials, not company logos.
- **Reason:** fetching third-party logos would make requests outside Tuta and
  create a phishing signal. Tuta contact photos are the safe source later.

### [P2] Server-side settings are still in Tuta web
- **Impact:** rules, vacation replies and templates are not editable in TMail.
  Local sending, notification, appearance and privacy settings are present.
- **Recovery:** add the server facades with live-account verification.

### [P2] Compose is plain text only
- **Location:** `ui/src/lib/components/Composer.svelte`
- **Impact:** no bold/italic/lists; users writing formatted mail must use the
  official client. This is a deliberate safety/scope trade, not an accident.
- **Recovery:** rich-text editor with sanitisation on both ends, or an explicit
  "formatting not supported" hint in the composer (currently the UI is silent).

### [P2] Draft editing is create-only
- **Impact:** clicking a draft cannot resume it; saving twice creates two
  drafts.
- **Recovery:** `DraftUpdateData` flow (model exists), then a "Continue editing"
  action in the reader.

### [P2] Search is palette-only
- **Impact:** 20 results, no paging, no filters. Fine for a quick jump, weak for
  archive research.
- **Recovery:** a search mode in the list pane backed by the same FTS query.

### [P3] Cold-start counters are a best effort
- **Impact:** counters are now persisted and updated by the event bus, but a
  brand-new account with no cache shows zeros until the first counter message.
- **Recovery:** request a counter snapshot on connect if the server exposes one.

### [P3] Notification click does not open the message
- **Impact:** the notification opens/focuses the app, not the mail.
- **Recovery:** carry the mail id in the event and route the reader.

## Strengths

- The avatar rows are recognisable without timestamps, and sender weight does
  the scanning work; columns resize and the rail collapses to a clear,
  re-expandable icon strip.
- Safe mail CSS and legacy markup are preserved, so real mail looks like mail
  rather than a flattened text export; the mail canvas stays light for contrast.
- Remote content is blocked with a concrete, per-sender decision; the mail frame
  is script-free, rounded and its CSS is filtered for resource loads and fixed
  positioning.
- New-mail detection has an event-bus fast path and an Inbox poll as a safety
  net; notifications respect focus/hidden state, appear once, carry the TMail
  icon and an "Open email" action. Mail links are forwarded from the frame
  bootstrap, so they work inline even when WebKit denies direct document access.
- Keyboard flow (`j/k`, `Enter`, `u/e/#`, `r/a/f`, `Ctrl+K`, `g` sequences) is
  coherent and discoverable through the palette and help sheet.
- The encrypted-cache/session/keyring story is visible in settings and errors,
  not marketing copy.

## Evidence

- `pnpm -C ui check`: 0 errors, 0 warnings.
- `pnpm -C ui exec playwright test`: 28 passed, 1 skipped (screenshots).
- `impeccable detect --json ui/src`: 0 findings.
- `pnpm -C ui test`: 15 passed (sanitiser, hash pin and formatting).
- `cargo test -p tmail-core`: 58 passed.
- `cargo test -p tmail-desktop`: 3 passed (mailto parser).
- Screenshots in `docs/review-shots/` (14: light/dark inbox, reader, composer,
  palette, settings, login, mobile). **Agents must not open them; the owner
  reviews the look.**
