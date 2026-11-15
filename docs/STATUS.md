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
  notifications, native save dialog, `mailto:` cold start, window state.
- Svelte 5 UI: three-pane quiet-workspace layout, virtualised avatar list with
  no timestamps, collapsible icon rail with a visible expand control and
  resizable columns, reader with sandboxed HTML and a light mail canvas,
  per-sender remote-content choice, attachment previews, command palette,
  Gmail-style composer that expands to a large centred window, expanded local
  settings, keyboard shortcuts, light/dark/system, density.
- Text-based verification: 28 Playwright tests + 14 Vitest tests pass (axe serious/critical = 0,
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
bulk selection, local search.

Missing: label management, inbox rules UI, spam classification UI.

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
