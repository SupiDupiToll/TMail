# Status

## Phase 0 — Research & decisions (2026-11-14)

Done.

- Studied upstream `tutanota` (README, BUILDING, HACKING, events, tuta-sdk
  API surface) and `tutabridge` (login, sync, event bus, blob/AEAD patches,
  session storage, send path).
- ADRs 0001–0007: protocol/crypto, shell, frontend, storage, license,
  tutabridge reuse, HTML-mail security.
- Feature matrix in `docs/FEATURE-MATRIX.md`.
- Legal review in `docs/LEGAL.md` (unofficial status, ToS clauses 7 and 12).

## Phase 1 — Core crate

Done and unit-tested (58 tests):

- Vendored, patched SDK at `tutanota-release-362.261020.0`; upstream + our
  tests pass. Patch stack documented in `vendor/tuta/PATCHES.md`.
- Sign-in with TOTP, session in Secret Service (memory fallback), BLAKE3-derived
  SQLCipher cache, FTS5 search, folder tree, mail listing, body/attachment blob
  reads, send + draft creation, move/trash/archive/delete.
- Event bus with reconnect/backoff, cursor persistence, server counter
  updates (persisted in `meta`), optimistic local counter adjustments.
- Sender aliases: `GroupInfo.mailAddress` + enabled aliases, validated before
  send, selectable in the composer.
- Recipient classification: known Tuta domains are a fast path; custom domains
  hosted by Tuta are detected through the public-key service. A lookup failure
  aborts rather than silently downgrading encryption.
- Sending: non-confidential mail, plus automatic end-to-end encryption for
  Tuta recipients (recipient key data via the public-key provider, RSA and
  TutaCrypt); sender group identity is used for TutaCrypt.
- Tuta-only network allowlist (owned domains only), bounded concurrency,
  429/5xx backoff.

Not yet: live-account verification; full incremental event replay into the
cache (events trigger a folder refresh; counters are applied directly); draft
editing in place.

## Phase 2 — Read-only mail + UI

Done:

- Tauri 2 shell: window, tray with close-to-tray fallback, single instance,
  notifications, native save dialog, `mailto:` cold start, window state, and a
  signed in-app updater for AppImage/Windows/macOS (package-managed Linux
  installs use their package manager).
- Svelte 5 UI: three-pane quiet-workspace layout, virtualised avatar list with
  no timestamps, collapsible icon rail with a visible expand control and
  resizable columns, reader with sandboxed HTML and a light mail canvas,
  per-sender remote-content choice, attachment previews, command palette,
  Gmail-style composer that expands to a large centred window, expanded local
  settings, keyboard shortcuts, light/dark/system, density.
- Text-based verification: 32 Playwright tests + 15 Vitest tests pass (axe serious/critical = 0,
  contrast computed numerically, 800/1280/1440/1920, 1.5 dpr, 200% text,
  sandbox/CSP, keyboard traversal, archive undo).
- Impeccable detector: 0 findings over `ui/src`.
- Dev resilience: component CSS is compiled into the component modules
  (`css: injected`), Vite pre-transforms components at startup and disables
  dev caching. Do not run `pnpm -C ui build`/`check` while `tauri dev` is up;
  that rewrites `.svelte-kit` and was the cause of the broken dev CSS.

Screenshots (`docs/review-shots/`): login light/dark, inbox light/dark, reader
light/dark, composer, palette, settings, mobile inbox/reader (14 images). **The owner should
review the look manually; agents must not open the images.**

## Phase 3 — Compose

Working: send, reply, reply-all, forward, attachments, plain-text mode, sender
alias selection, automatic E2E for Tuta recipients (including symmetric wrapping
for the user's own addresses), scheduled send, save draft (create).

Missing: rich-text editing, editing an existing draft, **password-protected
external mail**. The latter needs Tuta's external-group creation flow and a
Bcrypt KDF, neither of which is exposed by the Rust SDK at this tag; we refuse
instead of silently downgrading. The composer shows an explicit "Not encrypted"
state for external recipients.

## Phase 4 — Management

Working: move to folder, trash, archive, delete permanently, read/unread,
bulk selection, local FTS search, a full search view with unread/attachment/
folder filters, labels (apply existing labels), conversation strips from the
conversation-entry chain, contact names/autocomplete from Tuta contacts, and
TutaCrypt/RSA/AES trust badges from `EncryptionAuthStatus`.

Partially working: editing an existing draft (text and recipients; the old
draft is removed after save/send) but no attachment editing or autosave.

Missing: creating/renaming labels, inbox rules UI, spam classification UI,
contact photos, saved searches, full conversation loading beyond the previous
chain.

## Next

1. Obtain the dedicated test account (`docs/QUESTIONS.md`) and run
   `docs/SMOKE.md` (sender aliases, auto E2E, scheduled send, mail formatting,
   attachment preview). The send path now also assigns required `_id`s to the
   aggregate send records (`DraftData`, `DraftRecipient`,
   `SymEncInternalRecipientKeyData`), which was the cause of the reported
   `InstanceMapperError ... TypeId(1347) _id`. Re-test and report any new SDK
   detail.
2. Add a `tauri-driver` smoke job for tray/notifications/mailto.
3. Implement external confidential mail once the SDK exposes the external
   group/Bcrypt flow (or port it carefully with live tests).
4. Implement draft update + a search mode in the list pane.
5. Measure 20k-message scroll and memory; revisit SQLCipher WAL if needed.
6. Add contacts autocomplete and calendar after mail is proven.

## Risks

- Protocol patches are pinned to one upstream tag; a version bump can break
  them. Treat as routine maintenance, test with the live account first.
- Tuta may restrict third-party clients; the app is clearly unofficial.
- No telemetry means we learn about production failures only from user reports;
  error messages are designed to be actionable.

## Phase 5 — v0.2.1 release

- Published `v0.2.1` on 2026-11-15 with Linux `.deb`/`.rpm`/AppImage,
  Windows `.exe`/`.msi`, macOS universal `.dmg`, per-OS SHA-256 files,
  all review screenshots and `tmail-screenshots.zip`.
- Added the signed Tauri updater for AppImage, Windows and macOS. Package-managed
  Linux installs (`deb`/`rpm`/Arch) show a package-manager notice instead of
  updating behind the package manager. The update key pair lives in
  `~/.tauri/tmail-updater.{key,password}`; the public key is in
  `src-tauri/tauri.conf.json` and CI uses the matching repository secrets.
- `latest.json` is generated by the release matrix, signed artifacts are
  uploaded, and the public endpoint resolves to the latest published release.
  Windows update metadata prefers NSIS; checksums use asset basenames.
- The superseded `v0.2.0` draft and tag were removed; `v0.2.1` is the current
  release and the first one that can update itself.
- Verification: 58 core tests, 32 Playwright tests (1 skipped), 15 Vitest tests,
  3 desktop tests, 0 Impeccable findings; `latest.json`, installers, screenshot
  URLs and the Windows SHA-256 file were fetched from the published release.

## Phase 6 — v0.2.2 production fix

- v0.2.1 production builds were unusable: the CSP's inline-script hash did not
  match SvelteKit's generated hydration bootstrap, so the webview stayed blank
  even though its stylesheet loaded. Dev mode masked this because `devCsp`
  allows inline scripts.
- `ui/scripts/fix-csp.mjs` now runs as `postbuild` and externalizes executable
  inline scripts to hashed `/bootstrap.*.js` files after `vite build`. The
  strict production CSP stays intact (`script-src 'self'`; no `unsafe-inline`).
- The build was verified against the exact production CSP with Chromium: the
  app hydrates, stylesheets and injected component styles apply, and there are
  zero `securitypolicyviolation` events. The postbuild step fails if an
  executable inline script remains.
- `packaging/PKGBUILD` now uses the fixed source tag
  (`git+https://github.com/SupiDupiToll/TMail.git#tag=v$pkgver`) instead of a
  missing local tarball, and builds from the correct `tmail` directory. `url`
  points at the repository. It disables Arch's global LTO (which broke the
  bundled SQLCipher/ring/PQClean static libraries under lld) and enables
  `tauri/custom-protocol` so the packaged binary embeds the frontend instead of
  navigating to the dev server.
- Tests after the fix: 58 core, 32 Playwright (1 skipped), 15 Vitest, 0
  Svelte/A11y findings; release build succeeds with `--locked`.
- v0.2.1 remains published with a warning; v0.2.2 is the first working
  production release with the updater and can be installed manually once.
- Published `v0.2.2` on 2026-11-16. All three checksum files verified against
  the downloaded Linux, Windows and macOS assets. The released AppImage and an
  Arch `makepkg` package were started under an isolated Xvfb/DBus session and
  rendered the login UI (including "TMail 0.2.2"), confirming the production
  assets and CSP bootstrap. The v0.2.1 release carries a visible warning.
